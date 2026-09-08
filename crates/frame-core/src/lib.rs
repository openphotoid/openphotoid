//! Core image types and IO for OpenPhotoId.
//!
//! Everything downstream (matting, compliance, batch) works on [`Frame`]:
//! an sRGB RGBA8 buffer plus the metadata the pipeline needs (DPI, source
//! EXIF orientation already applied).

pub mod io;
pub mod resize;

use std::path::PathBuf;

pub use image::RgbaImage;

#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    #[error("io error on {path:?}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("decode error: {0}")]
    Decode(#[from] image::ImageError),
    #[error("png encode error: {0}")]
    PngEncode(#[from] png::EncodingError),
    #[error("resize error: {0}")]
    Resize(String),
    #[error("cannot hit target size {min_kb}-{max_kb} KB (best effort {got} bytes)")]
    JpegTargetSize {
        min_kb: u32,
        max_kb: u32,
        got: usize,
    },
}

pub type Result<T> = std::result::Result<T, FrameError>;

/// Dots per inch, carried through the pipeline so print exports are mm-exact.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Dpi(pub f32);

impl Default for Dpi {
    fn default() -> Self {
        Dpi(300.0)
    }
}

/// An in-memory image in the working color space (sRGB, 8-bit RGBA),
/// EXIF orientation already applied.
#[derive(Debug, Clone)]
pub struct Frame {
    pub pixels: RgbaImage,
    pub dpi: Dpi,
}

impl Frame {
    pub fn new(pixels: RgbaImage) -> Self {
        Frame {
            pixels,
            dpi: Dpi::default(),
        }
    }

    pub fn width(&self) -> u32 {
        self.pixels.width()
    }

    pub fn height(&self) -> u32 {
        self.pixels.height()
    }
}
