//! Portrait matting: model inference -> full-resolution alpha -> composite.
//!
//! M0 spike scope: MODNet at 512x512, bilinear alpha upsample, solid-color
//! composite. The refinement chain (guided-filter upsample, closed-form
//! band refinement, foreground estimation) lands in M2 (PLAN.md §6).
//!
//! Everything except the ONNX forward pass itself is compiled
//! unconditionally — [`Matte`], the refinement chain, the composites, and
//! `modnet`'s pre/post-processing. The runners that own an `frame_engine`
//! session ([`modnet::Modnet`], [`birefnet::BiRefNetLite`]) sit behind the
//! `inference` feature, which the browser build turns off: there,
//! onnxruntime-web runs the graph and `frame-wasm` calls the same
//! `modnet::preprocess`/`postprocess` on either side of it.

pub mod birefnet;
pub mod eval;
pub mod modnet;
pub mod refine;

use frame_core::{Frame, RgbaImage};

#[derive(Debug, thiserror::Error)]
pub enum MattingError {
    #[cfg(feature = "inference")]
    #[error(transparent)]
    Engine(#[from] frame_engine::EngineError),
    #[error(transparent)]
    Core(#[from] frame_core::FrameError),
    #[error("unexpected model output: {0}")]
    BadOutput(String),
}

pub type Result<T> = std::result::Result<T, MattingError>;

/// A full-resolution alpha matte (0.0 = background, 1.0 = foreground),
/// same dimensions as the source frame.
pub struct Matte {
    pub width: u32,
    pub height: u32,
    /// Row-major, len = width*height.
    pub alpha: Vec<f32>,
}

impl Matte {
    /// Refine the alpha against the source image: guided filter snaps the
    /// upsampled matte to real edges. Radius scales with image size.
    /// Snap the (bilinearly upsampled) alpha to real image edges.
    ///
    /// Bilinear upsampling from model resolution is already smooth, so the
    /// misalignment this corrects is a fine, few-pixel effect — NOT
    /// proportional to source resolution. A small fixed radius (1-2px)
    /// sharpens hair/edge detail; anything larger starts treating hair
    /// strands themselves as texture to smooth over, producing a visible
    /// halo around the head (regression caught by eyeballing real photos —
    /// synthetic small-fixture unit tests did not surface it because a 64px
    /// fixture makes a 4-8px radius look proportionally tiny). Radius is
    /// capped at 2 regardless of image size; see refine::tests and
    /// docs/research/05-m0-results.md for the sweep that produced this.
    pub fn refine(&mut self, src: &Frame) {
        self.refine_with(src, refine::RefineParams::default());
    }

    pub fn refine_with(&mut self, src: &Frame, params: refine::RefineParams) {
        refine::guided_filter(&src.pixels, &mut self.alpha, params.radius, params.eps);
    }

    /// Composite over a solid color using estimated foreground colors
    /// (de-fringed) — the production path for background replacement.
    pub fn composite_solid_defringed(&self, src: &Frame, rgb: [u8; 3]) -> Frame {
        let fg = refine::estimate_foreground(&src.pixels, &self.alpha);
        let mut out = RgbaImage::new(src.width(), src.height());
        for (i, dst) in out.pixels_mut().enumerate() {
            let a = self.alpha[i].clamp(0.0, 1.0);
            for c in 0..3 {
                dst[c] = (fg[i * 3 + c] * a + rgb[c] as f32 * (1.0 - a)).round() as u8;
            }
            dst[3] = 255;
        }
        Frame {
            pixels: out,
            dpi: src.dpi,
        }
    }

    /// Apply the matte to the source as an alpha channel (transparent cutout).
    pub fn cutout(&self, src: &Frame) -> Frame {
        let mut out = src.pixels.clone();
        for (i, p) in out.pixels_mut().enumerate() {
            p[3] = (self.alpha[i].clamp(0.0, 1.0) * 255.0).round() as u8;
        }
        Frame {
            pixels: out,
            dpi: src.dpi,
        }
    }

    /// Composite the subject over a solid background color.
    pub fn composite_solid(&self, src: &Frame, rgb: [u8; 3]) -> Frame {
        let mut out = RgbaImage::new(src.width(), src.height());
        for (i, (dst, s)) in out.pixels_mut().zip(src.pixels.pixels()).enumerate() {
            let a = self.alpha[i].clamp(0.0, 1.0);
            for c in 0..3 {
                dst[c] = (s[c] as f32 * a + rgb[c] as f32 * (1.0 - a)).round() as u8;
            }
            dst[3] = 255;
        }
        Frame {
            pixels: out,
            dpi: src.dpi,
        }
    }

    /// Blend a matte computed over a sub-rectangle of the same source back
    /// into this one, feathering across `feather` pixels at the seam.
    ///
    /// This is the "detail pass": the matting model has a fixed input
    /// resolution, so on a half-body source the head occupies a small part
    /// of it and hair is resolved at a fraction of the model's real
    /// capability. Running the same model a second time over a crop around
    /// the head spends that whole input budget on the region where the
    /// alpha is hard, and this merges the two — full-frame alpha
    /// everywhere, high-detail alpha over the head.
    ///
    /// It is a genuine quality gain for the cost of one extra inference at
    /// the same input size, and no extra weights to download; that is why
    /// this is the quality tier rather than a heavier second model.
    ///
    /// `rect` is `[x, y, w, h]` in this matte's pixel coordinates, and
    /// `detail` must be `w` x `h`. Out-of-range rectangles are clipped.
    pub fn blend_region(&mut self, detail: &Matte, rect: [u32; 4], feather: u32) {
        let [rx, ry, rw, rh] = rect;
        if rw == 0 || rh == 0 || detail.width != rw || detail.height != rh {
            return;
        }
        let feather = feather.min(rw / 2).min(rh / 2);
        for dy in 0..rh {
            let y = ry + dy;
            if y >= self.height {
                break;
            }
            for dx in 0..rw {
                let x = rx + dx;
                if x >= self.width {
                    break;
                }
                // Distance to the nearest edge of the rectangle, so the
                // detail alpha fades in rather than leaving a visible
                // rectangular seam in the matte.
                let w = if feather == 0 {
                    1.0
                } else {
                    let d = dx.min(rw - 1 - dx).min(dy).min(rh - 1 - dy);
                    (d as f32 / feather as f32).clamp(0.0, 1.0)
                };
                let i = (y * self.width + x) as usize;
                let j = (dy * rw + dx) as usize;
                self.alpha[i] = self.alpha[i] * (1.0 - w) + detail.alpha[j] * w;
            }
        }
    }

    /// Composite over a vertical linear gradient (top color -> bottom color).
    pub fn composite_gradient(&self, src: &Frame, top: [u8; 3], bottom: [u8; 3]) -> Frame {
        let h = src.height().max(1) as f32;
        let mut out = RgbaImage::new(src.width(), src.height());
        for (x, y, dst) in out.enumerate_pixels_mut() {
            let t = y as f32 / (h - 1.0).max(1.0);
            let i = (y * src.width() + x) as usize;
            let s = src.pixels.get_pixel(x, y);
            let a = self.alpha[i].clamp(0.0, 1.0);
            for c in 0..3 {
                let bg = top[c] as f32 * (1.0 - t) + bottom[c] as f32 * t;
                dst[c] = (s[c] as f32 * a + bg * (1.0 - a)).round() as u8;
            }
            dst[3] = 255;
        }
        Frame {
            pixels: out,
            dpi: src.dpi,
        }
    }
}

/// Bilinear upsample of a model-resolution matte to the source resolution.
pub fn upsample_alpha(small: &[f32], sw: u32, sh: u32, dw: u32, dh: u32) -> Vec<f32> {
    let mut out = vec![0.0f32; (dw * dh) as usize];
    let (sw_f, sh_f) = (sw as f32, sh as f32);
    for y in 0..dh {
        let sy = (y as f32 + 0.5) * sh_f / dh as f32 - 0.5;
        let y0 = sy.floor().clamp(0.0, sh_f - 1.0) as u32;
        let y1 = (y0 + 1).min(sh - 1);
        let fy = (sy - y0 as f32).clamp(0.0, 1.0);
        for x in 0..dw {
            let sx = (x as f32 + 0.5) * sw_f / dw as f32 - 0.5;
            let x0 = sx.floor().clamp(0.0, sw_f - 1.0) as u32;
            let x1 = (x0 + 1).min(sw - 1);
            let fx = (sx - x0 as f32).clamp(0.0, 1.0);
            let idx = |xx: u32, yy: u32| small[(yy * sw + xx) as usize];
            let v = idx(x0, y0) * (1.0 - fx) * (1.0 - fy)
                + idx(x1, y0) * fx * (1.0 - fy)
                + idx(x0, y1) * (1.0 - fx) * fy
                + idx(x1, y1) * fx * fy;
            out[(y * dw + x) as usize] = v;
        }
    }
    out
}

#[cfg(test)]
mod matte_tests {
    use super::*;

    fn matte(w: u32, h: u32, v: f32) -> Matte {
        Matte {
            width: w,
            height: h,
            alpha: vec![v; (w * h) as usize],
        }
    }

    #[test]
    fn blend_region_replaces_the_interior() {
        let mut base = matte(20, 20, 0.0);
        base.blend_region(&matte(8, 8, 1.0), [6, 6, 8, 8], 2);
        // Dead centre of the region is fully the detail pass.
        assert_eq!(base.alpha[10 * 20 + 10], 1.0);
        // Outside it, untouched.
        assert_eq!(base.alpha[20 + 1], 0.0);
    }

    #[test]
    fn blend_region_feathers_the_seam() {
        let mut base = matte(20, 20, 0.0);
        base.blend_region(&matte(8, 8, 1.0), [6, 6, 8, 8], 2);
        // The rectangle's own border pixel is fully the base, and values
        // climb monotonically inward — that gradient is the whole point,
        // since a hard edge here shows up as a rectangle in the cutout.
        let at = |x: usize, y: usize| base.alpha[y * 20 + x];
        assert_eq!(at(6, 10), 0.0);
        assert!(at(7, 10) > at(6, 10));
        assert!(at(8, 10) > at(7, 10));
        assert_eq!(at(8, 10), 1.0);
    }

    #[test]
    fn blend_region_ignores_a_mismatched_detail_size() {
        let mut base = matte(20, 20, 0.25);
        base.blend_region(&matte(4, 4, 1.0), [6, 6, 8, 8], 2);
        assert!(base.alpha.iter().all(|a| *a == 0.25));
    }

    #[test]
    fn blend_region_clips_at_the_frame_edge() {
        // A head crop can legitimately run past the frame edge; clipping
        // rather than panicking is the required behaviour.
        let mut base = matte(10, 10, 0.0);
        base.blend_region(&matte(8, 8, 1.0), [6, 6, 8, 8], 0);
        assert_eq!(base.alpha[9 * 10 + 9], 1.0);
        assert_eq!(base.alpha.len(), 100);
    }
}
