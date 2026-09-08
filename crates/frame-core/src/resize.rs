//! SIMD resize via fast_image_resize (Lanczos3).
//!
//! NOTE: linear-light (sRGB-mapped) resize is a quality follow-up; the
//! spike resizes in gamma space like most competitors do.

use fast_image_resize::{images::Image, FilterType, PixelType, ResizeAlg, ResizeOptions, Resizer};
use image::RgbaImage;

use crate::{Frame, FrameError, Result};

/// A conservative cap on the longest side of a decoded photo before it
/// enters the matting/compliance pipeline. The ONNX models themselves
/// always downsample to a fixed input size, so this doesn't bound their
/// memory — it bounds `frame-matting`'s guided-filter refinement and
/// foreground estimation, which run at full SOURCE resolution and
/// allocate several f32-per-pixel buffers. 4000px (16MP) comfortably
/// covers real-world phone/DSLR photos and every compliance output
/// (600-1200px) while capping worst-case memory for those O(N) passes —
/// relevant after the BiRefNet-lite OOM finding on a constrained VM (see
/// docs/research/05-m0-results.md): large SOURCE resolution multiplies
/// that same risk for pure-Rust processing, not just model inference.
pub const DEFAULT_MAX_DIMENSION: u32 = 4000;

/// Resize to exactly `w x h` (aspect handled by the caller).
pub fn resize_exact(frame: &Frame, w: u32, h: u32) -> Result<Frame> {
    let src = Image::from_vec_u8(
        frame.width(),
        frame.height(),
        frame.pixels.as_raw().clone(),
        PixelType::U8x4,
    )
    .map_err(|e| FrameError::Resize(e.to_string()))?;
    let mut dst = Image::new(w, h, PixelType::U8x4);
    let mut resizer = Resizer::new();
    resizer
        .resize(
            &src,
            &mut dst,
            &ResizeOptions::new().resize_alg(ResizeAlg::Convolution(FilterType::Lanczos3)),
        )
        .map_err(|e| FrameError::Resize(e.to_string()))?;
    let pixels = RgbaImage::from_raw(w, h, dst.into_vec())
        .ok_or_else(|| FrameError::Resize("buffer size mismatch".into()))?;
    Ok(Frame {
        pixels,
        dpi: frame.dpi,
    })
}

/// Resize so the longest side equals `max_side`, preserving aspect.
pub fn resize_max_side(frame: &Frame, max_side: u32) -> Result<Frame> {
    let (w, h) = (frame.width(), frame.height());
    let scale = max_side as f32 / w.max(h) as f32;
    if scale >= 1.0 {
        return Ok(frame.clone());
    }
    let nw = ((w as f32 * scale).round() as u32).max(1);
    let nh = ((h as f32 * scale).round() as u32).max(1);
    resize_exact(frame, nw, nh)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbaImage;

    #[test]
    fn exact_resize_dimensions() {
        let frame = Frame::new(RgbaImage::new(100, 50));
        let out = resize_exact(&frame, 40, 20).unwrap();
        assert_eq!((out.width(), out.height()), (40, 20));
    }

    #[test]
    fn max_side_preserves_aspect() {
        let frame = Frame::new(RgbaImage::new(1000, 500));
        let out = resize_max_side(&frame, 512).unwrap();
        assert_eq!((out.width(), out.height()), (512, 256));
    }

    #[test]
    fn default_max_dimension_caps_huge_photos() {
        // A 24MP-class source (e.g. a modern phone photo) should be
        // downsized to the cap; anything already under the cap (like the
        // repo's 2687x3356 test portrait) should pass through unchanged —
        // this is what protects the full-resolution refinement passes in
        // frame-matting from unbounded memory use on huge inputs.
        let huge = Frame::new(RgbaImage::new(6000, 4000));
        let capped = resize_max_side(&huge, DEFAULT_MAX_DIMENSION).unwrap();
        assert_eq!(capped.width(), DEFAULT_MAX_DIMENSION);
        assert_eq!(
            capped.height(),
            (4000.0 * (DEFAULT_MAX_DIMENSION as f64 / 6000.0)).round() as u32
        );

        let normal = Frame::new(RgbaImage::new(2687, 3356));
        let unchanged = resize_max_side(&normal, DEFAULT_MAX_DIMENSION).unwrap();
        assert_eq!((unchanged.width(), unchanged.height()), (2687, 3356));
    }
}
