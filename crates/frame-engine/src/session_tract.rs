//! Thin session wrapper around `tract` — the pure-Rust, CPU-only backend
//! used where `ort` has no prebuilt onnxruntime binary (Android/iOS;
//! PLAN.md §M9). Mirrors session_ort.rs's public API exactly so callers
//! don't need to know which backend is active.
//!
//! Models must have concrete input shapes baked into the graph before
//! loading — tract's shape inference can't resolve symbolic dims through
//! a fractional-scale Resize. See scripts/patch-modnet-for-tract.py for
//! the transform, and MODELS.md for which registry models are ready.

use std::path::Path;

use ndarray::ArrayD;
use tract_onnx::prelude::*;

use crate::Result;

/// tract has no GPU execution providers — this exists only for API parity
/// with session_ort's `Ep`, so callers can stay backend-agnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ep {
    Cpu,
}

impl std::fmt::Display for Ep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ep::Cpu => write!(f, "cpu"),
        }
    }
}

type RunnableModel = TypedRunnableModel<TypedModel>;

pub struct EngineSession {
    model: RunnableModel,
    output_names: Vec<String>,
    pub ep: Ep,
}

impl EngineSession {
    pub fn load(model_path: &Path, ep: Ep) -> Result<Self> {
        Self::load_with_threads(model_path, ep, 0)
    }

    /// `_intra_threads` is accepted for API parity with session_ort but
    /// ignored — tract has no per-session thread-count knob analogous to
    /// ort's `with_intra_threads` (its kernels parallelize, if at all, via
    /// a process-wide pool). The ort-side comment about trimming threads
    /// to fit memory doesn't apply here: BiRefNet (the model that needed
    /// it) is deliberately excluded from the tract/mobile path.
    pub fn load_with_threads(model_path: &Path, ep: Ep, _intra_threads: usize) -> Result<Self> {
        let raw = tract_onnx::onnx().model_for_path(model_path)?;
        let output_names: Vec<String> = raw
            .output_outlets()?
            .iter()
            .map(|o| raw.outlet_label(*o).unwrap_or("output").to_string())
            .collect();
        let model = raw.into_optimized()?.into_runnable()?;
        Ok(EngineSession {
            model,
            output_names,
            ep,
        })
    }

    /// tract doesn't expose per-input names the way ort does. Every
    /// current caller passes a single positional input, so this is kept
    /// only for API parity and isn't relied on.
    pub fn input_names(&self) -> Vec<String> {
        vec!["input".to_string()]
    }

    pub fn output_names(&self) -> Vec<String> {
        self.output_names.clone()
    }

    pub fn run_f32(&mut self, input: ArrayD<f32>) -> Result<Vec<ArrayD<f32>>> {
        let tensor: Tensor = input.into_tensor();
        let outputs = self.model.run(tvec!(tensor.into()))?;
        let mut result = Vec::with_capacity(outputs.len());
        for out in outputs.iter() {
            let arr = out.to_array_view::<f32>()?.to_owned().into_dyn();
            result.push(arr);
        }
        Ok(result)
    }
}
