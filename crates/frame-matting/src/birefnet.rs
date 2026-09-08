//! BiRefNet-lite runner — the "quality" tier (MIT weights).
//!
//! As with `modnet`, [`preprocess`]/[`postprocess`] are always compiled and
//! only [`BiRefNetLite`] needs the `inference` feature.
//!
//! Preprocess: resize to 1024x1024, ImageNet mean/std normalization, NCHW.
//! Output: [1, 1, 1024, 1024]; logits or probabilities depending on the
//! export — sigmoid is applied when values fall outside [0, 1].

use frame_core::{resize::resize_exact, Frame};
#[cfg(feature = "inference")]
use ndarray::{ArrayD, IxDyn};

use crate::{upsample_alpha, Matte, MattingError, Result};

/// BiRefNet-lite's square input edge.
pub const INPUT_SIZE: u32 = 1024;
const MEAN: [f32; 3] = [0.485, 0.456, 0.406];
const STD: [f32; 3] = [0.229, 0.224, 0.225];

/// Resize to 1024x1024 and fill an NCHW RGB tensor with ImageNet
/// mean/std normalization. Returns the flat tensor (`1*3*1024*1024`).
pub fn preprocess(frame: &Frame) -> Result<Vec<f32>> {
    let resized = resize_exact(frame, INPUT_SIZE, INPUT_SIZE)?;
    let plane = (INPUT_SIZE * INPUT_SIZE) as usize;
    let mut input = vec![0.0f32; 3 * plane];
    for (x, y, p) in resized.pixels.enumerate_pixels() {
        let i = (y * INPUT_SIZE + x) as usize;
        for c in 0..3 {
            input[c * plane + i] = (p[c] as f32 / 255.0 - MEAN[c]) / STD[c];
        }
    }
    Ok(input)
}

/// Turn BiRefNet's 1024x1024 prediction into a full-resolution [`Matte`].
///
/// Exports differ in whether the final map is logits or probabilities;
/// sigmoid is applied only when values fall outside [0, 1].
pub fn postprocess(map: &[f32], width: u32, height: u32) -> Result<Matte> {
    let n = (INPUT_SIZE * INPUT_SIZE) as usize;
    if map.len() != n {
        return Err(MattingError::BadOutput(format!(
            "expected {n} values, got {}",
            map.len()
        )));
    }
    let mut flat = map.to_vec();
    let needs_sigmoid = flat.iter().any(|v| *v < -0.01 || *v > 1.01);
    if needs_sigmoid {
        for v in &mut flat {
            *v = 1.0 / (1.0 + (-*v).exp());
        }
    }
    Ok(Matte {
        width,
        height,
        alpha: upsample_alpha(&flat, INPUT_SIZE, INPUT_SIZE, width, height),
    })
}

#[cfg(feature = "inference")]
pub struct BiRefNetLite {
    session: frame_engine::EngineSession,
}

#[cfg(feature = "inference")]
impl BiRefNetLite {
    pub fn load(ep: frame_engine::Ep) -> Result<Self> {
        let path = frame_engine::ensure_model(&frame_engine::registry::BIREFNET_LITE)?;
        let session = frame_engine::EngineSession::load(&path, ep)?;
        Ok(BiRefNetLite { session })
    }

    /// See [`EngineSession::load_with_threads`] — lower thread counts trade
    /// speed for peak memory on constrained hosts.
    pub fn load_with_threads(ep: frame_engine::Ep, intra_threads: usize) -> Result<Self> {
        let path = frame_engine::ensure_model(&frame_engine::registry::BIREFNET_LITE)?;
        let session = frame_engine::EngineSession::load_with_threads(&path, ep, intra_threads)?;
        Ok(BiRefNetLite { session })
    }

    pub fn infer(&mut self, frame: &Frame) -> Result<Matte> {
        let flat = preprocess(frame)?;
        let input = ArrayD::from_shape_vec(
            IxDyn(&[1, 3, INPUT_SIZE as usize, INPUT_SIZE as usize]),
            flat,
        )
        .map_err(|e| MattingError::BadOutput(format!("input shape: {e}")))?;
        let outputs = self.session.run_f32(input)?;
        // Exports differ in output ordering; take the last output shaped
        // like a single-channel map (BiRefNet's final prediction).
        let n = (INPUT_SIZE * INPUT_SIZE) as usize;
        let map = outputs.iter().rev().find(|o| o.len() == n).ok_or_else(|| {
            MattingError::BadOutput(format!(
                "no {n}-element output; shapes: {:?}",
                outputs
                    .iter()
                    .map(|o| o.shape().to_vec())
                    .collect::<Vec<_>>()
            ))
        })?;
        let flat: Vec<f32> = map.iter().copied().collect();
        postprocess(&flat, frame.width(), frame.height())
    }
}
