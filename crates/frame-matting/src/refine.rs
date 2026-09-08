//! Alpha refinement: guided-filter upsampling and approximate foreground
//! color estimation.
//!
//! - Guided filter (He et al., gray guide): snaps the model-resolution
//!   alpha to real image edges at full resolution. O(N) via running-sum
//!   box filters.
//! - Foreground estimation: blur-ratio approximation of pymatting's
//!   multi-level estimator — F ≈ box(α·I) / box(α) in the uncertain band —
//!   which removes background color fringing before compositing.

#[derive(Debug, Clone, Copy)]
pub struct RefineParams {
    pub radius: usize,
    pub eps: f32,
}

impl Default for RefineParams {
    fn default() -> Self {
        // See Matte::refine doc comment: radius must stay small regardless
        // of source resolution, or fine hair detail gets smoothed into a
        // halo. eps=1e-4 sits below typical local luminance variance
        // (0-1 scale) so the filter still follows real edges rather than
        // degenerating into a plain box blur.
        RefineParams {
            radius: 2,
            eps: 1e-4,
        }
    }
}

/// A grayscale f32 plane with running-sum box filtering.
struct Plane {
    w: usize,
    h: usize,
    data: Vec<f32>,
}

impl Plane {
    fn new(w: usize, h: usize) -> Self {
        Plane {
            w,
            h,
            data: vec![0.0; w * h],
        }
    }

    /// Mean box filter with radius `r` (clamped borders), O(N).
    fn box_filtered(&self, r: usize) -> Plane {
        let (w, h) = (self.w, self.h);
        let mut tmp = vec![0.0f32; w * h];
        // horizontal pass
        for y in 0..h {
            let row = &self.data[y * w..(y + 1) * w];
            let mut acc = 0.0;
            let win = |i: isize| row[i.clamp(0, w as isize - 1) as usize];
            for i in -(r as isize)..=(r as isize) {
                acc += win(i);
            }
            let n = (2 * r + 1) as f32;
            for x in 0..w {
                tmp[y * w + x] = acc / n;
                acc += win(x as isize + r as isize + 1) - win(x as isize - r as isize);
            }
        }
        // vertical pass
        let mut out = Plane::new(w, h);
        for x in 0..w {
            let col = |i: isize| tmp[(i.clamp(0, h as isize - 1) as usize) * w + x];
            let mut acc = 0.0;
            for i in -(r as isize)..=(r as isize) {
                acc += col(i);
            }
            let n = (2 * r + 1) as f32;
            for y in 0..h {
                out.data[y * w + x] = acc / n;
                acc += col(y as isize + r as isize + 1) - col(y as isize - r as isize);
            }
        }
        out
    }
}

fn luminance_plane(rgba: &image::RgbaImage) -> Plane {
    let (w, h) = (rgba.width() as usize, rgba.height() as usize);
    let mut p = Plane::new(w, h);
    for (i, px) in rgba.pixels().enumerate() {
        p.data[i] = (0.2126 * px[0] as f32 + 0.7152 * px[1] as f32 + 0.0722 * px[2] as f32) / 255.0;
    }
    p
}

/// Guided filter with a grayscale guide: refines `alpha` (full-res, in
/// [0,1]) against the image so the matte follows real edges.
pub fn guided_filter(rgba: &image::RgbaImage, alpha: &mut [f32], radius: usize, eps: f32) {
    let guide = luminance_plane(rgba);
    let (w, h) = (guide.w, guide.h);
    assert_eq!(alpha.len(), w * h);

    let p = Plane {
        w,
        h,
        data: alpha.to_vec(),
    };
    let mean_i = guide.box_filtered(radius);
    let mean_p = p.box_filtered(radius);

    let mut ii = Plane::new(w, h);
    let mut ip = Plane::new(w, h);
    for i in 0..w * h {
        ii.data[i] = guide.data[i] * guide.data[i];
        ip.data[i] = guide.data[i] * p.data[i];
    }
    let mean_ii = ii.box_filtered(radius);
    let mean_ip = ip.box_filtered(radius);

    let mut a = Plane::new(w, h);
    let mut b = Plane::new(w, h);
    for i in 0..w * h {
        let var_i = mean_ii.data[i] - mean_i.data[i] * mean_i.data[i];
        let cov_ip = mean_ip.data[i] - mean_i.data[i] * mean_p.data[i];
        a.data[i] = cov_ip / (var_i + eps);
        b.data[i] = mean_p.data[i] - a.data[i] * mean_i.data[i];
    }
    let mean_a = a.box_filtered(radius);
    let mean_b = b.box_filtered(radius);
    for (((out, ga), gb), gi) in alpha
        .iter_mut()
        .zip(&mean_a.data)
        .zip(&mean_b.data)
        .zip(&guide.data)
    {
        *out = (ga * gi + gb).clamp(0.0, 1.0);
    }
}

/// Approximate foreground colors so semi-transparent edge pixels composite
/// without background fringe. Returns an RGB f32 buffer (len = 3*N).
///
/// F ≈ box(α·I) / box(α), computed coarse-to-fine at three radii; solid
/// foreground pixels keep their own color.
pub fn estimate_foreground(rgba: &image::RgbaImage, alpha: &[f32]) -> Vec<f32> {
    let (w, h) = (rgba.width() as usize, rgba.height() as usize);
    let mut fg = vec![0.0f32; w * h * 3];
    for (i, px) in rgba.pixels().enumerate() {
        fg[i * 3] = px[0] as f32;
        fg[i * 3 + 1] = px[1] as f32;
        fg[i * 3 + 2] = px[2] as f32;
    }

    // Weighted blur of α·I and α at decreasing radii; larger radii reach
    // color from deep inside the subject.
    for radius in [16usize, 6, 2] {
        let mut wa = Plane::new(w, h);
        let mut planes = [Plane::new(w, h), Plane::new(w, h), Plane::new(w, h)];
        for i in 0..w * h {
            let a = alpha[i];
            wa.data[i] = a;
            for c in 0..3 {
                planes[c].data[i] = fg[i * 3 + c] * a;
            }
        }
        let wa_b = wa.box_filtered(radius);
        let blurred: Vec<Plane> = planes.iter().map(|p| p.box_filtered(radius)).collect();
        for i in 0..w * h {
            // Only rewrite colors where the pixel is not solidly foreground:
            // fringe lives in the uncertain band.
            if alpha[i] < 0.95 && wa_b.data[i] > 1e-4 {
                for c in 0..3 {
                    fg[i * 3 + c] = (blurred[c].data[i] / wa_b.data[i]).clamp(0.0, 255.0);
                }
            }
        }
    }
    fg
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    /// Left half dark foreground, right half bright background; the soft
    /// 12-px alpha ramp is centered 4 px INTO the background — the
    /// misaligned-upsampled-matte scenario the guided filter corrects.
    fn fixture() -> (RgbaImage, Vec<f32>) {
        let (w, h) = (64u32, 32u32);
        let edge = w / 2;
        let mut img = RgbaImage::new(w, h);
        for (x, _y, p) in img.enumerate_pixels_mut() {
            *p = if x < edge {
                Rgba([40, 10, 10, 255])
            } else {
                Rgba([90, 230, 90, 255])
            };
        }
        let mut alpha = vec![0.0f32; (w * h) as usize];
        for y in 0..h {
            for x in 0..w {
                let t = (edge as f32 + 4.0 + 6.0 - x as f32) / 12.0;
                alpha[(y * w + x) as usize] = t.clamp(0.0, 1.0);
            }
        }
        (img, alpha)
    }

    fn half_crossing(alpha: &[f32], w: usize, row: usize) -> usize {
        alpha[row * w..(row + 1) * w]
            .iter()
            .position(|a| *a < 0.5)
            .unwrap_or(w)
    }

    #[test]
    fn guided_filter_realigns_soft_edge_to_guide() {
        let (img, mut alpha) = fixture();
        let w = img.width() as usize;
        let edge = w / 2; // color edge at x = 32
        let before = half_crossing(&alpha, w, 16);
        assert!(
            before >= edge + 3,
            "fixture misaligned as intended: {before}"
        );
        // Three passes with a smaller eps: strong edge-snapping needs eps
        // well below the guide's local variance, and enough passes for the
        // correction to propagate across the full ramp width.
        guided_filter(&img, &mut alpha, 6, 1e-6);
        guided_filter(&img, &mut alpha, 4, 1e-6);
        guided_filter(&img, &mut alpha, 2, 1e-6);
        let after = half_crossing(&alpha, w, 16);
        // Cores stay correct…
        let mid = 16 * w;
        assert!(alpha[mid + 10] > 0.85, "fg core {}", alpha[mid + 10]);
        assert!(alpha[mid + 54] < 0.15, "bg core {}", alpha[mid + 54]);
        // …and the 0.5-crossing moves toward the true color edge.
        assert!(
            after < before && after.abs_diff(edge) <= 2,
            "crossing should land near color edge {edge}: {before} -> {after}"
        );
    }

    #[test]
    fn foreground_estimation_removes_fringe() {
        // Red subject over green backdrop; edge pixel colors are polluted
        // 50/50 — estimation should return them to (nearly) pure red.
        let (w, h) = (32u32, 32u32);
        let mut img = RgbaImage::new(w, h);
        let mut alpha = vec![0.0f32; (w * h) as usize];
        for (x, y, p) in img.enumerate_pixels_mut() {
            let i = (y * w + x) as usize;
            if x < 14 {
                *p = Rgba([200, 20, 20, 255]);
                alpha[i] = 1.0;
            } else if x < 16 {
                *p = Rgba([110, 110, 20, 255]); // polluted edge mix
                alpha[i] = 0.5;
            } else {
                *p = Rgba([20, 200, 20, 255]);
                alpha[i] = 0.0;
            }
        }
        let fg = estimate_foreground(&img, &alpha);
        let i = (16 * w + 14) as usize; // an edge pixel
        assert!(
            fg[i * 3] > 150.0 && fg[i * 3 + 1] < 80.0,
            "edge fg should be pulled toward red, got ({}, {}, {})",
            fg[i * 3],
            fg[i * 3 + 1],
            fg[i * 3 + 2]
        );
    }
}
