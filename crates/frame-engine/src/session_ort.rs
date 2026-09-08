//! Thin session wrapper around ort with execution-provider selection.

use std::path::Path;

use ndarray::ArrayD;
use ort::session::{builder::GraphOptimizationLevel, Session};

use crate::Result;

/// Which execution provider to request. CPU always works; the accelerators
/// silently fall back per-op, which is exactly the shipping behavior we
/// want (research: CoreML/DirectML flakiness).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ep {
    Cpu,
    #[cfg(feature = "coreml")]
    CoreMl,
    #[cfg(feature = "directml")]
    DirectMl,
}

impl std::fmt::Display for Ep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ep::Cpu => write!(f, "cpu"),
            #[cfg(feature = "coreml")]
            Ep::CoreMl => write!(f, "coreml"),
            #[cfg(feature = "directml")]
            Ep::DirectMl => write!(f, "directml"),
        }
    }
}

pub struct EngineSession {
    session: Session,
    pub ep: Ep,
}

impl EngineSession {
    pub fn load(model_path: &Path, ep: Ep) -> Result<Self> {
        Self::load_with_threads(
            model_path,
            ep,
            std::thread::available_parallelism().map_or(4, |n| n.get()),
        )
    }

    /// Like [`load`](Self::load) but with an explicit intra-op thread count.
    /// Lower thread counts reduce ORT's per-thread arena memory — useful on
    /// memory-constrained hosts (see docs/research/05-m0-results.md, the
    /// BiRefNet-lite OOM on the shared dev VM).
    pub fn load_with_threads(model_path: &Path, ep: Ep, intra_threads: usize) -> Result<Self> {
        #[allow(unused_mut)]
        let mut builder = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(intra_threads.max(1))?;

        match ep {
            Ep::Cpu => {}
            #[cfg(feature = "coreml")]
            Ep::CoreMl => {
                use ort::execution_providers::CoreMLExecutionProvider;
                builder = builder
                    .with_execution_providers([CoreMLExecutionProvider::default().build()])?;
            }
            #[cfg(feature = "directml")]
            Ep::DirectMl => {
                use ort::execution_providers::DirectMLExecutionProvider;
                builder = builder
                    .with_execution_providers([DirectMLExecutionProvider::default().build()])?;
            }
        }

        let session = builder.commit_from_file(model_path)?;
        Ok(EngineSession { session, ep })
    }

    pub fn input_names(&self) -> Vec<String> {
        self.session.inputs.iter().map(|i| i.name.clone()).collect()
    }

    pub fn output_names(&self) -> Vec<String> {
        self.session
            .outputs
            .iter()
            .map(|o| o.name.clone())
            .collect()
    }

    /// Run with a single f32 tensor input (the common case for our models);
    /// returns all outputs as owned f32 arrays in declaration order.
    pub fn run_f32(&mut self, input: ArrayD<f32>) -> Result<Vec<ArrayD<f32>>> {
        let input_name = self.session.inputs[0].name.clone();
        let output_names = self.output_names();
        let value = ort::value::Tensor::from_array(input)?;
        let outputs = self.session.run(ort::inputs![input_name => value])?;
        let mut result = Vec::with_capacity(output_names.len());
        for name in &output_names {
            let (shape, data) = outputs[name.as_str()].try_extract_tensor::<f32>()?;
            let dims: Vec<usize> = shape.iter().map(|d| *d as usize).collect();
            let arr = ArrayD::from_shape_vec(ndarray::IxDyn(&dims), data.to_vec())
                .map_err(|e| crate::EngineError::Shape(e.to_string()))?;
            result.push(arr);
        }
        Ok(result)
    }
}
