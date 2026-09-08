//! Edge-preserving skin smoothing, confined to the face and capped low.
//!
//! Design constraints come straight from the market research: the
//! vertical ID-photo apps that force beautification on collect one-star
//! reviews for exactly that ("强制开启美颜、无法完全关闭"), and an ID photo
//! that no longer resembles the holder is rejected at the counter. So:
//! default off, a hard ceiling well below "smooth plastic", and the
//! effect is masked to the face region so hair, collar and background
//! keep their real texture.

use frame_core::Frame;
use frame_face::Face;

/// Upper bound on `strength`. Above this the result stops looking like a
/// photograph of the person, which for this product is a defect and not a
/// feature — see the module docs.
pub const MAX_STRENGTH: f32 = 0.6;

/// Smooth skin inside the face region.
///
/// `strength` is 0.0 (identity) to [`MAX_STRENGTH`]; anything higher is
/// clamped. The filter is a surface blur — a box average that ignores
/// neighbours whose luma differs by more than a threshold — so pores and
/// blotches average away while the eye, nostril, lip and hairline edges,
/// which are exactly the high-contrast boundaries, are left alone.
///
/// Returns the input unchanged when `strength <= 0.0` or when no face was
/// detected: there is no sensible region to confine the effect to, and a
/// whole-image blur is never what this is for.
pub fn smooth(frame: &Frame, faces: &[Face], strength: f32) -> Frame {
    let strength = strength.clamp(0.0, MAX_STRENGTH);
    let Some(face) = faces.first() else {
        return frame.clone();
    };
    if strength <= 0.0 {
        return frame.clone();
    }

    let (w, h) = (frame.width(), frame.height());
    // Radius scales with the face, not the image: the same strength has to
    // mean the same visual softness whether the source is 600px or 4000px.
    let radius = ((face.bbox[2] * 0.035 * strength / MAX_STRENGTH).round() as i32).clamp(1, 24);
    // Luma tolerance: what counts as "same surface" rather than an edge.
    let tolerance = 18.0 + 22.0 * (strength / MAX_STRENGTH);

    let mask = face_mask(face, w, h);
    let src = &frame.pixels;
    let mut out = src.clone();

    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let mi = (y as u32 * w + x as u32) as usize;
            let m = mask[mi] * strength;
            if m <= 0.001 {
                continue;
            }
            let center = src.get_pixel(x as u32, y as u32);
            let center_luma = luma(center.0);
            let (mut acc, mut n) = ([0f32; 3], 0f32);
            for dy in -radius..=radius {
                let sy = y + dy;
                if sy < 0 || sy >= h as i32 {
                    continue;
                }
                for dx in -radius..=radius {
                    let sx = x + dx;
                    if sx < 0 || sx >= w as i32 {
                        continue;
                    }
                    let p = src.get_pixel(sx as u32, sy as u32);
                    if (luma(p.0) - center_luma).abs() > tolerance {
                        continue;
                    }
                    for c in 0..3 {
                        acc[c] += p[c] as f32;
                    }
                    n += 1.0;
                }
            }
            if n <= 0.0 {
                continue;
            }
            let dst = out.get_pixel_mut(x as u32, y as u32);
            for c in 0..3 {
                let blurred = acc[c] / n;
                dst[c] = (center[c] as f32 * (1.0 - m) + blurred * m).round() as u8;
            }
        }
    }

    Frame {
        pixels: out,
        dpi: frame.dpi,
    }
}

fn luma(p: [u8; 4]) -> f32 {
    0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32
}

/// A soft elliptical mask over the face, with the eyes and mouth cut back
/// out of it. Those are the features that carry identity and expression;
/// smoothing them is what makes retouched ID photos look wrong.
fn face_mask(face: &Face, w: u32, h: u32) -> Vec<f32> {
    let [fx, fy, fw, fh] = face.bbox;
    let (cx, cy) = (fx + fw / 2.0, fy + fh * 0.55);
    let (rx, ry) = (fw * 0.52, fh * 0.62);
    let mut mask = vec![0f32; (w * h) as usize];

    // Feature keep-out radii, derived from the inter-eye distance so they
    // track face size rather than image size.
    let eye_gap = (face.landmarks[1][0] - face.landmarks[0][0]).abs().max(1.0);
    let keep_out: [([f32; 2], f32); 3] = [
        (face.landmarks[0], eye_gap * 0.42),
        (face.landmarks[1], eye_gap * 0.42),
        (face.mouth_midpoint(), eye_gap * 0.48),
    ];

    for y in 0..h {
        for x in 0..w {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let d = ((px - cx) / rx).powi(2) + ((py - cy) / ry).powi(2);
            // Feather the last 25% of the ellipse so the treated region has
            // no visible boundary against untouched skin.
            let mut v = if d >= 1.0 {
                0.0
            } else if d <= 0.75 {
                1.0
            } else {
                (1.0 - d) / 0.25
            };
            if v > 0.0 {
                for (c, r) in &keep_out {
                    let dd = ((px - c[0]).powi(2) + (py - c[1]).powi(2)).sqrt();
                    if dd < *r {
                        v = v.min((dd / r).powi(2));
                    }
                }
            }
            mask[(y * w + x) as usize] = v;
        }
    }
    mask
}

#[cfg(test)]
mod tests {
    use super::*;
    use frame_core::RgbaImage;

    fn face_at(cx: f32, cy: f32, size: f32) -> Face {
        Face {
            score: 0.99,
            bbox: [cx - size / 2.0, cy - size / 2.0, size, size],
            landmarks: [
                [cx - size * 0.18, cy - size * 0.1],
                [cx + size * 0.18, cy - size * 0.1],
                [cx, cy + size * 0.05],
                [cx - size * 0.12, cy + size * 0.22],
                [cx + size * 0.12, cy + size * 0.22],
            ],
        }
    }

    /// Deterministic high-frequency speckle over a mid-grey base, at the
    /// amplitude real sensor/JPEG noise on skin actually has. It matters
    /// that this stays small: the filter treats a jump larger than its
    /// tolerance as an *edge* and refuses to average across it, which is
    /// the whole point of using a surface blur rather than a plain one.
    /// A ±24 fixture is above that tolerance and is left untouched — the
    /// filter behaving correctly, on a fixture that was wrong.
    fn noisy(w: u32, h: u32) -> Frame {
        let mut img = RgbaImage::new(w, h);
        for (x, y, p) in img.enumerate_pixels_mut() {
            let n = if (x + y) % 2 == 0 { 8i32 } else { -8i32 };
            let v = (150 + n).clamp(0, 255) as u8;
            *p = image::Rgba([v, v, v, 255]);
        }
        Frame::new(img)
    }

    fn variance(frame: &Frame, x0: u32, y0: u32, size: u32) -> f32 {
        let mut vals = Vec::new();
        for y in y0..y0 + size {
            for x in x0..x0 + size {
                vals.push(luma(frame.pixels.get_pixel(x, y).0));
            }
        }
        let mean = vals.iter().sum::<f32>() / vals.len() as f32;
        vals.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / vals.len() as f32
    }

    #[test]
    fn zero_strength_is_identity() {
        let f = noisy(64, 64);
        let out = smooth(&f, &[face_at(32.0, 32.0, 40.0)], 0.0);
        assert_eq!(out.pixels.as_raw(), f.pixels.as_raw());
    }

    #[test]
    fn no_face_is_identity() {
        let f = noisy(64, 64);
        let out = smooth(&f, &[], 0.5);
        assert_eq!(out.pixels.as_raw(), f.pixels.as_raw());
    }

    #[test]
    fn smooths_inside_the_face_and_leaves_the_corner_alone() {
        let f = noisy(120, 120);
        let out = smooth(&f, &[face_at(60.0, 60.0, 70.0)], MAX_STRENGTH);

        // Centre of the face: speckle should be substantially reduced.
        assert!(
            variance(&out, 50, 40, 12) < variance(&f, 50, 40, 12) * 0.5,
            "expected the face region to be smoothed"
        );
        // Far corner is outside the mask entirely: untouched, bit for bit.
        for y in 0..8 {
            for x in 0..8 {
                assert_eq!(out.pixels.get_pixel(x, y), f.pixels.get_pixel(x, y));
            }
        }
    }

    #[test]
    fn strength_is_capped() {
        let f = noisy(120, 120);
        let face = face_at(60.0, 60.0, 70.0);
        let at_max = smooth(&f, std::slice::from_ref(&face), MAX_STRENGTH);
        let way_over = smooth(&f, &[face], 10.0);
        assert_eq!(at_max.pixels.as_raw(), way_over.pixels.as_raw());
    }
}
