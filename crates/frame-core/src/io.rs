//! Decode with EXIF orientation, encode with DPI metadata and
//! file-size targeting (compliance portals impose KB windows).

use std::io::Cursor;
use std::path::Path;

use image::{DynamicImage, ImageFormat, RgbaImage};

use crate::{Dpi, Frame, FrameError, Result};

/// Load an image file, apply EXIF orientation, convert to RGBA8.
pub fn load(path: &Path) -> Result<Frame> {
    let bytes = std::fs::read(path).map_err(|source| FrameError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    load_from_bytes(&bytes)
}

/// Decode an in-memory image (e.g. bytes from a browser file picker with
/// no real filesystem path), apply EXIF orientation, convert to RGBA8.
pub fn load_from_bytes(bytes: &[u8]) -> Result<Frame> {
    let img = image::load_from_memory(bytes)?;
    let img = apply_exif_orientation(img, exif_orientation(bytes));
    Ok(Frame::new(img.into_rgba8()))
}

/// EXIF orientation value (1-8), defaulting to 1 (upright).
fn exif_orientation(bytes: &[u8]) -> u32 {
    let mut cursor = Cursor::new(bytes);
    exif::Reader::new()
        .read_from_container(&mut cursor)
        .ok()
        .and_then(|e| {
            e.get_field(exif::Tag::Orientation, exif::In::PRIMARY)
                .and_then(|f| f.value.get_uint(0))
        })
        .unwrap_or(1)
}

fn apply_exif_orientation(img: DynamicImage, orientation: u32) -> DynamicImage {
    match orientation {
        2 => img.fliph(),
        3 => img.rotate180(),
        4 => img.flipv(),
        5 => img.rotate90().fliph(),
        6 => img.rotate90(),
        7 => img.rotate270().fliph(),
        8 => img.rotate270(),
        _ => img,
    }
}

/// Encode a frame as PNG with a pHYs chunk carrying its DPI.
pub fn encode_png(frame: &Frame) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, frame.width(), frame.height());
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let ppu = (frame.dpi.0 * 39.370_08).round() as u32; // dots/inch -> px/metre
        encoder.set_pixel_dims(Some(png::PixelDimensions {
            xppu: ppu,
            yppu: ppu,
            unit: png::Unit::Meter,
        }));
        let mut writer = encoder.write_header()?;
        writer.write_image_data(frame.pixels.as_raw())?;
    }
    Ok(out)
}

pub fn save_png(frame: &Frame, path: &Path) -> Result<()> {
    let bytes = encode_png(frame)?;
    std::fs::write(path, bytes).map_err(|source| FrameError::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// Encode as JPEG at an explicit quality, flattening alpha over white and
/// patching the JFIF density fields with the frame's DPI.
pub fn encode_jpeg(frame: &Frame, quality: u8) -> Result<Vec<u8>> {
    let rgb = flatten_over_white(&frame.pixels);
    let mut out = Vec::new();
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, quality);
    DynamicImage::ImageRgb8(rgb).write_with_encoder(encoder)?;
    set_jfif_dpi(&mut out, frame.dpi);
    Ok(out)
}

pub fn save_jpeg(frame: &Frame, path: &Path, quality: u8) -> Result<()> {
    let bytes = encode_jpeg(frame, quality)?;
    std::fs::write(path, bytes).map_err(|source| FrameError::Io {
        path: path.to_path_buf(),
        source,
    })
}

/// Binary-search JPEG quality so the encoded size lands inside
/// `[min_kb, max_kb]` — several government portals enforce both bounds
/// (DS-160 <= 240 KB, India PAN 20-50 KB, China COVA 40-120 KB).
pub fn encode_jpeg_within(frame: &Frame, min_kb: u32, max_kb: u32) -> Result<Vec<u8>> {
    let min = (min_kb as usize) * 1024;
    let max = (max_kb as usize) * 1024;
    let (mut lo, mut hi) = (1u8, 100u8);
    let mut best: Option<Vec<u8>> = None;
    let mut best_size = 0usize;
    while lo <= hi {
        let q = lo + (hi - lo) / 2;
        let bytes = encode_jpeg(frame, q)?;
        let len = bytes.len();
        if len <= max {
            // Under the cap: remember the largest (= highest quality) fit.
            if len >= best_size {
                best_size = len;
                best = Some(bytes);
            }
            lo = q + 1;
        } else {
            if q == 1 {
                break;
            }
            hi = q - 1;
        }
    }
    match best {
        Some(bytes) if bytes.len() >= min => Ok(bytes),
        Some(bytes) => Err(FrameError::JpegTargetSize {
            min_kb,
            max_kb,
            got: bytes.len(),
        }),
        None => Err(FrameError::JpegTargetSize {
            min_kb,
            max_kb,
            got: 0,
        }),
    }
}

fn flatten_over_white(rgba: &RgbaImage) -> image::RgbImage {
    let mut out = image::RgbImage::new(rgba.width(), rgba.height());
    for (dst, src) in out.pixels_mut().zip(rgba.pixels()) {
        let a = src[3] as u32;
        for c in 0..3 {
            dst[c] = (((src[c] as u32) * a + 255 * (255 - a)) / 255) as u8;
        }
    }
    out
}

/// Patch the JFIF APP0 density fields in place (units=dpi).
/// JPEG layout: FFD8 | FFE0 len "JFIF\0" ver units xdensity ydensity ...
fn set_jfif_dpi(jpeg: &mut [u8], dpi: Dpi) {
    if jpeg.len() < 18 || jpeg[0..2] != [0xFF, 0xD8] || jpeg[2..4] != [0xFF, 0xE0] {
        return;
    }
    if &jpeg[6..11] != b"JFIF\0" {
        return;
    }
    let d = dpi.0.round() as u16;
    jpeg[13] = 1; // units: dots per inch
    jpeg[14..16].copy_from_slice(&d.to_be_bytes());
    jpeg[16..18].copy_from_slice(&d.to_be_bytes());
}

#[allow(dead_code)]
fn format_of(path: &Path) -> Option<ImageFormat> {
    ImageFormat::from_path(path).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn test_frame(w: u32, h: u32) -> Frame {
        let mut img = RgbaImage::new(w, h);
        for (x, y, p) in img.enumerate_pixels_mut() {
            *p = Rgba([(x % 256) as u8, (y % 256) as u8, ((x + y) % 256) as u8, 255]);
        }
        Frame::new(img)
    }

    #[test]
    fn png_roundtrip_preserves_pixels() {
        let frame = test_frame(64, 48);
        let bytes = encode_png(&frame).unwrap();
        let decoded = image::load_from_memory(&bytes).unwrap().into_rgba8();
        assert_eq!(decoded, frame.pixels);
    }

    #[test]
    fn load_from_bytes_matches_load_from_path() {
        let frame = test_frame(48, 32);
        let png_bytes = encode_png(&frame).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.png");
        std::fs::write(&path, &png_bytes).unwrap();

        let via_path = load(&path).unwrap();
        let via_bytes = load_from_bytes(&png_bytes).unwrap();
        assert_eq!(via_path.pixels, via_bytes.pixels);
    }

    #[test]
    fn jpeg_dpi_patched() {
        let mut frame = test_frame(32, 32);
        frame.dpi = Dpi(600.0);
        let bytes = encode_jpeg(&frame, 90).unwrap();
        assert_eq!(&bytes[6..11], b"JFIF\0");
        assert_eq!(bytes[13], 1);
        assert_eq!(u16::from_be_bytes([bytes[14], bytes[15]]), 600);
    }

    #[test]
    fn jpeg_within_hits_window() {
        let frame = test_frame(600, 600);
        let bytes = encode_jpeg_within(&frame, 1, 240).unwrap();
        assert!(bytes.len() <= 240 * 1024);
        assert!(bytes.len() >= 1024);
    }
}
