//! MODNet portrait matting runner.
//!
//! Preprocess: resize to 512x512, normalize to [-1, 1], NCHW RGB.
//! Output: [1, 1, 512, 512] matte in [0, 1].
//!
//! [`preprocess`] and [`postprocess`] are always compiled; only [`Modnet`],
//! which owns an `frame_engine` session, needs the `inference` feature.
//! The browser build runs the graph through onnxruntime-web and calls
//! these same two functions around it — see the crate docs.

#[cfg(feature = "inference")]
use std::path::Path;

use frame_core::{resize::resize_exact, Frame};
#[cfg(feature = "inference")]
use ndarray::{ArrayD, IxDyn};

use crate::{upsample_alpha, Matte, MattingError, Result};

/// MODNet's square input edge.
pub const INPUT_SIZE: u32 = 512;

/// Resize to 512x512 and fill an NCHW RGB tensor normalized to [-1, 1].
/// Returns the flat tensor (`1*3*512*512`, C-order).
pub fn preprocess(frame: &Frame) -> Result<Vec<f32>> {
    let resized = resize_exact(frame, INPUT_SIZE, INPUT_SIZE)?;
    let plane = (INPUT_SIZE * INPUT_SIZE) as usize;
    let mut input = vec![0.0f32; 3 * plane];
    for (x, y, p) in resized.pixels.enumerate_pixels() {
        let i = (y * INPUT_SIZE + x) as usize;
        for c in 0..3 {
            input[c * plane + i] = (p[c] as f32 - 127.5) / 127.5;
        }
    }
    Ok(input)
}

/// Turn the model's 512x512 alpha map into a full-resolution [`Matte`] for
/// a `width` x `height` source, by bilinear upsample.
pub fn postprocess(matte_512: &[f32], width: u32, height: u32) -> Result<Matte> {
    let n = (INPUT_SIZE * INPUT_SIZE) as usize;
    if matte_512.len() != n {
        return Err(MattingError::BadOutput(format!(
            "expected {n} alpha values, got {}",
            matte_512.len()
        )));
    }
    let alpha = upsample_alpha(matte_512, INPUT_SIZE, INPUT_SIZE, width, height);
    Ok(Matte {
        width,
        height,
        alpha,
    })
}

#[cfg(feature = "inference")]
pub struct Modnet {
    session: frame_engine::EngineSession,
}

#[cfg(feature = "inference")]
impl Modnet {
    /// Loads the registry's stock MODNet weights. Under `ort-backend` this
    /// is the only path you need. Under `tract-backend` it will fail —
    /// the stock export isn't tract-compatible (PLAN.md §M9); use
    /// [`Modnet::load_from_path`] with a model prepared by
    /// `scripts/patch-modnet-for-tract.py` instead.
    pub fn load(ep: frame_engine::Ep) -> Result<Self> {
        // Under tract the stock export does not load; the registry carries
        // the patched variant, which the phone apps ship in their bundle.
        #[cfg(feature = "tract-backend")]
        let spec = &frame_engine::registry::MODNET_TRACT;
        #[cfg(not(feature = "tract-backend"))]
        let spec = &frame_engine::registry::MODNET_PHOTOGRAPHIC;
        let path = frame_engine::ensure_model(spec)?;
        Self::load_from_path(&path, ep)
    }

    /// Loads MODNet weights from an explicit path rather than the model
    /// registry — for a locally-prepared variant (e.g. the tract-compatible
    /// patch) that isn't a registry-managed download.
    pub fn load_from_path(path: &Path, ep: frame_engine::Ep) -> Result<Self> {
        let session = frame_engine::EngineSession::load(path, ep)?;
        Ok(Modnet { session })
    }

    pub fn ep(&self) -> frame_engine::Ep {
        self.session.ep
    }

    /// The raw forward pass on an already-prepared 512×512 input, returning
    /// the first output flat — for a caller driving `frame-session`.
    pub fn run_raw(&mut self, input: ArrayD<f32>) -> Result<ArrayD<f32>> {
        let mut outputs = self.session.run_f32(input)?;
        if outputs.is_empty() {
            return Err(MattingError::BadOutput("no outputs".into()));
        }
        Ok(outputs.swap_remove(0))
    }

    pub fn infer(&mut self, frame: &Frame) -> Result<Matte> {
        let flat = preprocess(frame)?;
        let input = ArrayD::from_shape_vec(
            IxDyn(&[1, 3, INPUT_SIZE as usize, INPUT_SIZE as usize]),
            flat,
        )
        .map_err(|e| MattingError::BadOutput(format!("input shape: {e}")))?;
        let outputs = self.session.run_f32(input)?;
        let matte = outputs
            .first()
            .ok_or_else(|| MattingError::BadOutput("no outputs".into()))?;
        let flat: Vec<f32> = matte.iter().copied().collect();
        postprocess(&flat, frame.width(), frame.height())
    }
}
