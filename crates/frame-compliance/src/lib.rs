//! ID-photo compliance engine: country/document specs, the auto-crop
//! solver, and (M4) the validation checklist.
//!
//! Specs are data, not code: JSON files under `data/specs/` following the
//! schema in `docs/research/04-compliance-specs.md`, mm-first with px
//! derived via DPI.

pub mod crop;
pub mod spec;
pub mod validate;

pub use crop::{solve_crop, CropAdjustment, CropSolution};
pub use spec::PhotoSpec;
pub use validate::{validate, ValidationReport};

#[derive(Debug, thiserror::Error)]
pub enum ComplianceError {
    #[error("spec parse error: {0}")]
    Spec(#[from] serde_json::Error),
    #[error(transparent)]
    Core(#[from] frame_core::FrameError),
    #[error("no face detected")]
    NoFace,
    #[error("crop out of bounds: {0}")]
    OutOfBounds(String),
}

pub type Result<T> = std::result::Result<T, ComplianceError>;
