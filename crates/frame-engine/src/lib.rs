//! ONNX inference for OpenPhotoId. Two backends, selected by feature —
//! everything downstream talks to [`EngineSession`], and neither backend's
//! types leak past this crate, so callers (frame-matting, frame-face)
//! don't need to know or care which one is active:
//!
//! - `ort-backend` (default): `ort` + a prebuilt onnxruntime binary.
//!   Desktop only — Windows/macOS/Linux (no Android/iOS binary exists).
//! - `tract-backend`: pure-Rust `tract`, no native build step. CPU only;
//!   used where `ort` has no prebuilt binary (mobile — PLAN.md §M9).
//!   Models need concrete input shapes and tract-supported ops; not every
//!   model in the registry is ready (see MODELS.md).

#[cfg(all(feature = "ort-backend", feature = "tract-backend"))]
compile_error!(
    "frame-engine: enable exactly one of `ort-backend` / `tract-backend`, not both — \
     they both provide the EngineSession/Ep types and would conflict."
);
#[cfg(not(any(feature = "ort-backend", feature = "tract-backend")))]
compile_error!(
    "frame-engine: enable exactly one of `ort-backend` (desktop, default) or \
     `tract-backend` (mobile) — see this crate's docs/PLAN.md §M9."
);

pub mod registry;

#[cfg(feature = "ort-backend")]
mod session_ort;
#[cfg(feature = "ort-backend")]
pub use session_ort::{EngineSession, Ep};

#[cfg(feature = "tract-backend")]
mod session_tract;
#[cfg(feature = "tract-backend")]
pub use session_tract::{EngineSession, Ep};

pub use registry::{ensure_model, models_dir, ModelSpec};

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("model download failed for {id}: {msg}")]
    Download { id: String, msg: String },
    #[error("checksum mismatch for {id}: expected {expected}, got {got}")]
    Checksum {
        id: String,
        expected: String,
        got: String,
    },
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[cfg(feature = "ort-backend")]
    #[error("ort error: {0}")]
    Ort(#[from] ort::Error),
    #[cfg(feature = "tract-backend")]
    #[error("tract error: {0}")]
    Tract(#[from] tract_onnx::prelude::TractError),
    #[error("output shape error: {0}")]
    Shape(String),
}

pub type Result<T> = std::result::Result<T, EngineError>;
