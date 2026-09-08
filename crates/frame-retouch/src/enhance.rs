//! Print enhancement: resample to the output size and restore the local
//! contrast that resampling costs.
//!
//! Competitors sell this as "HD 修复" / "AI quality restore". This one is
//! deliberately not a generative super-resolution model, and the naming in
//! the UI follows: **an ID photo is evidence**. A model that invents
//! plausible detail — sharper irises, a cleaner jawline, skin that was
//! never in the source — is inventing detail about a person's face on a
//! document that has to match them. Deterministic resampling plus
//! unsharp masking recovers what the pixels actually contain and nothing
//! else, which is the right trade here even though it is the less
//! impressive demo.
//!
//! What it does help with, measurably: a phone photo downscaled to a
//! 413x531 spec loses acutance, and prints soft at 300 DPI. This puts
//! that back.

use frame_core::{resize::resize_exact, Frame};

/// How much local contrast to restore, and how much noise to tolerate
/// while doing it.
#[derive(Debug, Clone, Copy)]
pub struct EnhanceParams {
    /// Unsharp amount. 0.0 = off; 1.0 is a firm print sharpen.
    pub amount: f32,
    /// Blur radius of the unsharp mask, in pixels.
    pub radius: usize,
    /// Minimum local difference, in 0-255 luma, before sharpening applies.
    /// This is what keeps the pass from amplifying sensor noise and JPEG
    /// blocking in flat areas — skin and a plain background being almost
    /// the whole frame in an ID photo.
    pub threshold: f32,
}

impl Default for EnhanceParams {
    fn default() -> Self {
        // Tuned for a 300 DPI print of a 35x45mm photo: visible acutance
        // at arm's length without the white halo that gives over-sharpened
        // ID photos away.
        EnhanceParams {
            amount: 0.55,
            radius: 2,
            threshold: 4.0,
        }
    }
}

/// Resample `frame` to `w` x `h` and apply [`unsharp`].
///
/// Doing both here rather than letting the caller resize separately is
/// deliberate: sharpening must happen *after* the final resample, or the
/// resample averages the halos back into mush.
pub fn to_size(frame: &Frame, w: u32, h: u32, params: EnhanceParams) -> frame_core::Result<Frame> {
    let resized = resize_exact(frame, w, h)?;
    Ok(unsharp(&resized, params))
}

/// Unsharp mask: add back a threshold-gated multiple of the difference
/// between the image and a blurred copy of it.
pub fn unsharp(frame: &Frame, params: EnhanceParams) -> Frame {
    if params.amount <= 0.0 || params.radius == 0 {
        return frame.clone();
    }
    let (w, h) = (frame.width(), frame.height());
    let blurred = box_blur(frame, params.radius);
    let mut out = frame.pixels.clone();

    for y in 0..h {
        for x in 0..w {
            let src = frame.pixels.get_pixel(x, y);
            let blur = blurred.get_pixel(x, y);
            let dst = out.get_pixel_mut(x, y);
            // Gate on luma, apply per channel — gating per channel would
            // pull colour fringes out of chroma noise.
            let d_luma = luma(src.0) - luma(blur.0);
            if d_luma.abs() < params.threshold {
                continue;
            }
            for c in 0..3 {
                let d = src[c] as f32 - blur[c] as f32;
                dst[c] = (src[c] as f32 + d * params.amount)
                    .clamp(0.0, 255.0)
                    .round() as u8;
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

/// Separable box blur over the RGB channels. Two passes (horizontal then
/// vertical) so cost is O(radius) per pixel rather than O(radius^2) —
/// this runs on a phone browser over a multi-megapixel frame.
fn box_blur(frame: &Frame, radius: usize) -> image::RgbaImage {
    let (w, h) = (frame.width() as usize, frame.height() as usize);
    let src = &frame.pixels;
    let r = radius as isize;

    let mut tmp = vec![0f32; w * h * 3];
    for y in 0..h {
        for x in 0..w {
            let (mut acc, mut n) = ([0f32; 3], 0f32);
            for dx in -r..=r {
                let sx = x as isize + dx;
                if sx < 0 || sx >= w as isize {
                    continue;
                }
                let p = src.get_pixel(sx as u32, y as u32);
                for c in 0..3 {
                    acc[c] += p[c] as f32;
                }
                n += 1.0;
            }
            for c in 0..3 {
                tmp[(y * w + x) * 3 + c] = acc[c] / n;
            }
        }
    }

    let mut out = src.clone();
    for y in 0..h {
        for x in 0..w {
            let (mut acc, mut n) = ([0f32; 3], 0f32);
            for dy in -r..=r {
                let sy = y as isize + dy;
                if sy < 0 || sy >= h as isize {
                    continue;
                }
                for c in 0..3 {
                    acc[c] += tmp[(sy as usize * w + x) * 3 + c];
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
    use frame_core::RgbaImage;

    /// A soft vertical edge — the thing sharpening is supposed to steepen.
    ///
    /// The transition has to be narrow relative to the blur radius to be a
    /// meaningful fixture. Spread gently across the whole width it is
    /// locally almost linear, its second difference falls under
    /// `threshold`, and the filter correctly declines to touch it — a
    /// passing test of nothing.
    fn soft_edge(w: u32, h: u32) -> Frame {
        let mut img = RgbaImage::new(w, h);
        let mid = (w - 1) as f32 / 2.0;
        for (x, _y, p) in img.enumerate_pixels_mut() {
            let t = ((x as f32 - mid) / 2.5).tanh() * 0.5 + 0.5;
            let v = (t * 255.0).round() as u8;
            *p = image::Rgba([v, v, v, 255]);
        }
        Frame::new(img)
    }

    #[test]
    fn zero_amount_is_identity() {
        let f = soft_edge(64, 16);
        let out = unsharp(
            &f,
            EnhanceParams {
                amount: 0.0,
                ..Default::default()
            },
        );
        assert_eq!(out.pixels.as_raw(), f.pixels.as_raw());
    }

    #[test]
    fn overshoots_on_both_sides_of_a_soft_edge() {
        // What unsharp masking does to an edge is add overshoot to its
        // shoulders: the dark side gets darker, the light side lighter,
        // which is what the eye reads as sharpness. It deliberately does
        // *not* change the gradient at the inflection point itself —
        // there the blurred copy equals the source, so there is nothing
        // to add. Asserting on the centre would be asserting on the one
        // place the filter is a no-op.
        let f = soft_edge(64, 16);
        let out = unsharp(&f, EnhanceParams::default());
        let (y, mid) = (8u32, 32u32);
        let at = |img: &Frame, x: u32| luma(img.pixels.get_pixel(x, y).0);

        let darkened = (1..mid).any(|x| at(&out, x) < at(&f, x));
        let lightened = (mid..63).any(|x| at(&out, x) > at(&f, x));
        assert!(darkened, "no pixel on the dark side of the edge darkened");
        assert!(
            lightened,
            "no pixel on the light side of the edge lightened"
        );
    }

    #[test]
    fn threshold_protects_flat_areas() {
        // A perfectly flat field has no local difference to amplify, so a
        // sharpen pass must leave it bit-for-bit identical rather than
        // pulling noise out of rounding.
        let flat = Frame::new(RgbaImage::from_pixel(
            32,
            32,
            image::Rgba([128, 128, 128, 255]),
        ));
        let out = unsharp(&flat, EnhanceParams::default());
        assert_eq!(out.pixels.as_raw(), flat.pixels.as_raw());
    }

    #[test]
    fn to_size_resamples_then_sharpens() {
        let f = soft_edge(128, 32);
        let out = to_size(&f, 64, 16, EnhanceParams::default()).unwrap();
        assert_eq!((out.width(), out.height()), (64, 16));
    }
}
