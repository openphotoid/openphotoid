//! Backgrounds beyond a flat fill: gradients, studio vignettes, and an
//! arbitrary image behind the subject.
//!
//! The competitor matrix has "AI 背景生成" as a paid, generative feature
//! everywhere it exists. What people actually reach for it for — in the
//! review evidence — is a studio-looking backdrop rather than a novel
//! invented scene, and that is a compositing problem, not a diffusion
//! problem. So this renders backdrops deterministically, instantly, and
//! offline. See `docs/premium-features.md` for what a real generative
//! backdrop would require and why it is not in the local build.
//!
//! For official documents the correct background is nearly always the
//! flat colour the spec names — [`Backdrop::Solid`], which is what the
//! compliance path selects on its own. The rest of this module is for the
//! other half of the job the research describes: the same photo reused as
//! a profile or CV portrait, where a flat government white looks wrong.

use frame_core::{Frame, RgbaImage};
use frame_matting::{refine::estimate_foreground, Matte};
use image::Rgba;

/// What to paint behind the subject.
#[derive(Debug, Clone)]
pub enum Backdrop {
    /// A flat fill — the only kind a document spec ever asks for.
    Solid([u8; 3]),
    /// Vertical linear gradient, top colour to bottom colour.
    Gradient { top: [u8; 3], bottom: [u8; 3] },
    /// Radial falloff from a bright centre behind the head to a darker
    /// edge — the classic portrait-studio sweep.
    Vignette {
        center: [u8; 3],
        edge: [u8; 3],
        /// Fraction of image height the bright centre sits at, from the
        /// top. Around 0.35 puts it behind the head.
        center_y: f32,
    },
    /// An arbitrary image, cover-fitted to the frame, optionally blurred
    /// so it reads as depth of field rather than a sticker.
    Image { rgba: RgbaImage, blur_radius: usize },
}

impl Backdrop {
    /// Render the backdrop at `w` x `h`.
    pub fn render(&self, w: u32, h: u32) -> RgbaImage {
        match self {
            Backdrop::Solid(rgb) => {
                RgbaImage::from_pixel(w, h, Rgba([rgb[0], rgb[1], rgb[2], 255]))
            }
            Backdrop::Gradient { top, bottom } => {
                let mut out = RgbaImage::new(w, h);
                let denom = (h.saturating_sub(1)).max(1) as f32;
                for (_x, y, p) in out.enumerate_pixels_mut() {
                    let t = y as f32 / denom;
                    *p = Rgba([
                        lerp(top[0], bottom[0], t),
                        lerp(top[1], bottom[1], t),
                        lerp(top[2], bottom[2], t),
                        255,
                    ]);
                }
                out
            }
            Backdrop::Vignette {
                center,
                edge,
                center_y,
            } => {
                let mut out = RgbaImage::new(w, h);
                let (cx, cy) = (w as f32 / 2.0, h as f32 * center_y.clamp(0.0, 1.0));
                // Normalise by the distance to the farthest corner, so the
                // falloff reaches `edge` exactly at the frame's extreme.
                let max_d = [
                    (0.0, 0.0),
                    (w as f32, 0.0),
                    (0.0, h as f32),
                    (w as f32, h as f32),
                ]
                .iter()
                .map(|(x, y)| ((x - cx).powi(2) + (y - cy).powi(2)).sqrt())
                .fold(1.0f32, f32::max);
                for (x, y, p) in out.enumerate_pixels_mut() {
                    // Pixel centres, not corners — otherwise the brightest
                    // point lands half a pixel off and the exact centre
                    // never reaches the centre colour.
                    let d = ((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2)).sqrt()
                        / max_d;
                    // Smoothstep, so the sweep has no visible banding ring.
                    let t = (d * d * (3.0 - 2.0 * d)).clamp(0.0, 1.0);
                    *p = Rgba([
                        lerp(center[0], edge[0], t),
                        lerp(center[1], edge[1], t),
                        lerp(center[2], edge[2], t),
                        255,
                    ]);
                }
                out
            }
            Backdrop::Image { rgba, blur_radius } => {
                let fitted = cover_fit(rgba, w, h);
                if *blur_radius == 0 {
                    fitted
                } else {
                    blur(&fitted, *blur_radius)
                }
            }
        }
    }

    /// Composite `src` over this backdrop using `matte`, with the same
    /// foreground-colour estimation the flat-fill path uses.
    ///
    /// That de-fringing step is why this doesn't just call
    /// [`Matte::composite_solid`] with a per-pixel colour: without it,
    /// semi-transparent hair pixels carry the *original* background's
    /// colour into the new one, and a subject shot against a blue wall
    /// keeps a blue halo over any backdrop you put behind them.
    pub fn composite(&self, src: &Frame, matte: &Matte) -> Frame {
        let (w, h) = (src.width(), src.height());
        let bg = self.render(w, h);
        let fg = estimate_foreground(&src.pixels, &matte.alpha);
        let mut out = RgbaImage::new(w, h);
        for (i, dst) in out.pixels_mut().enumerate() {
            let a = matte.alpha[i].clamp(0.0, 1.0);
            let b = bg.get_pixel((i as u32) % w, (i as u32) / w);
            for c in 0..3 {
                dst[c] = (fg[i * 3 + c] * a + b[c] as f32 * (1.0 - a)).round() as u8;
            }
            dst[3] = 255;
        }
        Frame {
            pixels: out,
            dpi: src.dpi,
        }
    }
}

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 * (1.0 - t) + b as f32 * t)
        .round()
        .clamp(0.0, 255.0) as u8
}

/// Scale to cover `w` x `h` preserving aspect, then centre-crop.
fn cover_fit(src: &RgbaImage, w: u32, h: u32) -> RgbaImage {
    let (sw, sh) = (src.width().max(1), src.height().max(1));
    let scale = (w as f32 / sw as f32).max(h as f32 / sh as f32);
    let (nw, nh) = (
        ((sw as f32 * scale).ceil() as u32).max(w),
        ((sh as f32 * scale).ceil() as u32).max(h),
    );
    let scaled = image::imageops::resize(src, nw, nh, image::imageops::FilterType::CatmullRom);
    let (ox, oy) = ((nw - w) / 2, (nh - h) / 2);
    image::imageops::crop_imm(&scaled, ox, oy, w, h).to_image()
}

/// Separable box blur, run three times — three box passes approximate a
/// Gaussian closely enough that the result reads as lens blur rather than
/// the boxy smear a single pass gives.
fn blur(src: &RgbaImage, radius: usize) -> RgbaImage {
    let mut cur = src.clone();
    for _ in 0..3 {
        cur = box_pass(&cur, radius);
    }
    cur
}

fn box_pass(src: &RgbaImage, radius: usize) -> RgbaImage {
    let (w, h) = (src.width() as isize, src.height() as isize);
    let r = radius as isize;
    let mut tmp = src.clone();
    for y in 0..h {
        for x in 0..w {
            let (mut acc, mut n) = ([0f32; 3], 0f32);
            for d in -r..=r {
                let sx = x + d;
                if sx < 0 || sx >= w {
                    continue;
                }
                let p = src.get_pixel(sx as u32, y as u32);
                for c in 0..3 {
                    acc[c] += p[c] as f32;
                }
                n += 1.0;
            }
            let p = tmp.get_pixel_mut(x as u32, y as u32);
            for c in 0..3 {
                p[c] = (acc[c] / n).round() as u8;
            }
        }
    }
    let mut out = tmp.clone();
    for y in 0..h {
        for x in 0..w {
            let (mut acc, mut n) = ([0f32; 3], 0f32);
            for d in -r..=r {
                let sy = y + d;
                if sy < 0 || sy >= h {
                    continue;
                }
                let p = tmp.get_pixel(x as u32, sy as u32);
                for c in 0..3 {
                    acc[c] += p[c] as f32;
                }
                n += 1.0;
            }
            let p = out.get_pixel_mut(x as u32, y as u32);
            for c in 0..3 {
                p[c] = (acc[c] / n).round() as u8;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matte(w: u32, h: u32, alpha: f32) -> Matte {
        Matte {
            width: w,
            height: h,
            alpha: vec![alpha; (w * h) as usize],
        }
    }

    #[test]
    fn solid_is_flat() {
        let img = Backdrop::Solid([12, 34, 56]).render(4, 4);
        for p in img.pixels() {
            assert_eq!(p.0, [12, 34, 56, 255]);
        }
    }

    #[test]
    fn gradient_runs_top_to_bottom() {
        let img = Backdrop::Gradient {
            top: [0, 0, 0],
            bottom: [255, 255, 255],
        }
        .render(2, 5);
        assert_eq!(img.get_pixel(0, 0).0[0], 0);
        assert_eq!(img.get_pixel(0, 4).0[0], 255);
        // Monotonic in between — no banding reversal.
        for y in 1..5 {
            assert!(img.get_pixel(0, y).0[0] >= img.get_pixel(0, y - 1).0[0]);
        }
    }

    #[test]
    fn vignette_is_brightest_at_its_center() {
        let img = Backdrop::Vignette {
            center: [255, 255, 255],
            edge: [0, 0, 0],
            center_y: 0.5,
        }
        .render(21, 21);
        let mid = img.get_pixel(10, 10).0[0];
        let corner = img.get_pixel(0, 0).0[0];
        assert!(mid > corner, "centre {mid} should outshine corner {corner}");
        assert_eq!(mid, 255);
    }

    #[test]
    fn image_backdrop_covers_without_letterboxing() {
        let wide = RgbaImage::from_pixel(40, 10, Rgba([9, 9, 9, 255]));
        let img = Backdrop::Image {
            rgba: wide,
            blur_radius: 0,
        }
        .render(10, 10);
        assert_eq!((img.width(), img.height()), (10, 10));
        // Every pixel came from the source, none from an empty margin.
        for p in img.pixels() {
            assert_eq!(p.0[3], 255);
            assert_eq!(p.0[0], 9);
        }
    }

    #[test]
    fn fully_transparent_matte_shows_only_the_backdrop() {
        let src = Frame::new(RgbaImage::from_pixel(4, 4, Rgba([200, 0, 0, 255])));
        let out = Backdrop::Solid([0, 0, 255]).composite(&src, &matte(4, 4, 0.0));
        for p in out.pixels.pixels() {
            assert_eq!(p.0, [0, 0, 255, 255]);
        }
    }

    #[test]
    fn opaque_matte_keeps_the_subject() {
        let src = Frame::new(RgbaImage::from_pixel(4, 4, Rgba([200, 10, 10, 255])));
        let out = Backdrop::Solid([0, 0, 255]).composite(&src, &matte(4, 4, 1.0));
        for p in out.pixels.pixels() {
            assert_eq!(p.0, [200, 10, 10, 255]);
        }
    }
}
