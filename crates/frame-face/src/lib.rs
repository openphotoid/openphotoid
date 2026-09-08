//! Face detection with 5-point landmarks via YuNet (OpenCV Zoo, MIT).
//!
//! The decode mirrors OpenCV's FaceDetectorYN: three anchor-free strided
//! heads (8/16/32); score = sqrt(cls * obj); boxes/landmarks are offsets
//! from the grid cell scaled by the stride; greedy NMS.
//!
//! The model-independent halves of that — [`preprocess`] and [`decode`] —
//! are always compiled. Only [`YuNet`], which owns an `frame_engine`
//! session, sits behind the `inference` feature: the browser build
//! (`frame-wasm`) runs the ONNX graph itself via onnxruntime-web and calls
//! those two functions around it, so the letterboxing and the head decode
//! are *the same code* here and there rather than a Rust original and a
//! JavaScript reimplementation that can drift apart.

use frame_core::{resize::resize_exact, Frame};

#[cfg(feature = "inference")]
use ndarray::{ArrayD, IxDyn};

#[derive(Debug, thiserror::Error)]
pub enum FaceError {
    #[cfg(feature = "inference")]
    #[error(transparent)]
    Engine(#[from] frame_engine::EngineError),
    #[error(transparent)]
    Core(#[from] frame_core::FrameError),
    #[error("unexpected model output: {0}")]
    BadOutput(String),
}

pub type Result<T> = std::result::Result<T, FaceError>;

/// One detected face in source-image pixel coordinates.
#[derive(Debug, Clone)]
pub struct Face {
    pub score: f32,
    /// x, y, w, h
    pub bbox: [f32; 4],
    /// right eye, left eye, nose tip, right mouth corner, left mouth corner
    pub landmarks: [[f32; 2]; 5],
}

impl Face {
    pub fn eye_midpoint(&self) -> [f32; 2] {
        [
            (self.landmarks[0][0] + self.landmarks[1][0]) / 2.0,
            (self.landmarks[0][1] + self.landmarks[1][1]) / 2.0,
        ]
    }

    pub fn mouth_midpoint(&self) -> [f32; 2] {
        [
            (self.landmarks[3][0] + self.landmarks[4][0]) / 2.0,
            (self.landmarks[3][1] + self.landmarks[4][1]) / 2.0,
        ]
    }

    /// Roll angle in degrees (positive = head tilted so the left eye is
    /// lower than the right in image coordinates).
    pub fn roll_deg(&self) -> f32 {
        let dx = self.landmarks[1][0] - self.landmarks[0][0];
        let dy = self.landmarks[1][1] - self.landmarks[0][1];
        dy.atan2(dx).to_degrees()
    }
}

/// YuNet's square input edge. The source is letterboxed into it
/// top-left-anchored, matching OpenCV's resize-to-input flow.
pub const INPUT: u32 = 640;
pub const STRIDES: [u32; 3] = [8, 16, 32];
const SCORE_THRESHOLD: f32 = 0.7;
const NMS_IOU: f32 = 0.3;

/// The four per-stride head tensors, as contiguous slices in the layout
/// the ONNX graph emits them: `cls`/`obj` are one value per grid cell,
/// `bbox` four, `kps` ten.
pub struct StrideHeads<'a> {
    pub stride: u32,
    pub cls: &'a [f32],
    pub obj: &'a [f32],
    pub bbox: &'a [f32],
    pub kps: &'a [f32],
}

/// Letterbox `frame` into YuNet's 640x640 input and fill an NCHW BGR
/// tensor. Returns the flat tensor (`1*3*640*640`, C-order) alongside the
/// scale factor `decode` needs to map detections back to source pixels.
///
/// YuNet is trained on BGR input (OpenCV convention) with no
/// normalisation — raw 0-255 values.
pub fn preprocess(frame: &Frame) -> Result<(Vec<f32>, f32)> {
    let (w, h) = (frame.width(), frame.height());
    let scale = INPUT as f32 / w.max(h) as f32;
    let nw = ((w as f32 * scale).round() as u32).clamp(1, INPUT);
    let nh = ((h as f32 * scale).round() as u32).clamp(1, INPUT);
    let resized = resize_exact(frame, nw, nh)?;

    let plane = (INPUT * INPUT) as usize;
    let mut input = vec![0.0f32; 3 * plane];
    for (x, y, p) in resized.pixels.enumerate_pixels() {
        let i = (y * INPUT + x) as usize;
        input[i] = p[2] as f32;
        input[plane + i] = p[1] as f32;
        input[2 * plane + i] = p[0] as f32;
    }
    Ok((input, scale))
}

/// Decode the three strided heads into faces in source-image pixel
/// coordinates, sorted by descending score and NMS-filtered.
///
/// `scale` is the second value returned by [`preprocess`].
pub fn decode(heads: &[StrideHeads<'_>], scale: f32) -> Result<Vec<Face>> {
    let mut faces = Vec::new();
    for head in heads {
        let stride = head.stride;
        let cols = (INPUT / stride) as usize;
        let rows = (INPUT / stride) as usize;
        let n = cols * rows;
        let (cls, obj, bbox, kps) = (head.cls, head.obj, head.bbox, head.kps);
        if cls.len() < n || obj.len() < n || bbox.len() < n * 4 || kps.len() < n * 10 {
            return Err(FaceError::BadOutput(format!(
                "stride {stride}: unexpected lengths cls={} obj={} bbox={} kps={}",
                cls.len(),
                obj.len(),
                bbox.len(),
                kps.len()
            )));
        }

        for r in 0..rows {
            for c in 0..cols {
                let i = r * cols + c;
                let score = (cls[i].clamp(0.0, 1.0) * obj[i].clamp(0.0, 1.0)).sqrt();
                if score < SCORE_THRESHOLD {
                    continue;
                }
                let s = stride as f32;
                let cx = (c as f32 + bbox[i * 4]) * s;
                let cy = (r as f32 + bbox[i * 4 + 1]) * s;
                let bw = bbox[i * 4 + 2].exp() * s;
                let bh = bbox[i * 4 + 3].exp() * s;
                let mut landmarks = [[0.0f32; 2]; 5];
                for (k, lm) in landmarks.iter_mut().enumerate() {
                    lm[0] = (c as f32 + kps[i * 10 + k * 2]) * s / scale;
                    lm[1] = (r as f32 + kps[i * 10 + k * 2 + 1]) * s / scale;
                }
                faces.push(Face {
                    score,
                    bbox: [
                        (cx - bw / 2.0) / scale,
                        (cy - bh / 2.0) / scale,
                        bw / scale,
                        bh / scale,
                    ],
                    landmarks,
                });
            }
        }
    }

    faces.sort_by(|a, b| b.score.total_cmp(&a.score));
    Ok(nms(faces))
}

#[cfg(feature = "inference")]
pub struct YuNet {
    session: frame_engine::EngineSession,
}

#[cfg(feature = "inference")]
impl YuNet {
    pub fn load(ep: frame_engine::Ep) -> Result<Self> {
        let path = frame_engine::ensure_model(&frame_engine::registry::YUNET_2023MAR)?;
        let session = frame_engine::EngineSession::load(&path, ep)?;
        Ok(YuNet { session })
    }

    /// The raw forward pass: output names and tensors, for a caller that
    /// does its own letterboxing through [`preprocess`] and decoding
    /// through [`decode`] — the phone apps drive `frame-session` this way.
    pub fn run_raw(&mut self, input: ArrayD<f32>) -> Result<(Vec<String>, Vec<ArrayD<f32>>)> {
        let names = self.session.output_names();
        let outputs = self.session.run_f32(input)?;
        Ok((names, outputs))
    }

    /// Detect faces, returned sorted by descending score.
    pub fn detect(&mut self, frame: &Frame) -> Result<Vec<Face>> {
        let (flat, scale) = preprocess(frame)?;
        let input = ArrayD::from_shape_vec(IxDyn(&[1, 3, INPUT as usize, INPUT as usize]), flat)
            .map_err(|e| FaceError::BadOutput(format!("input shape: {e}")))?;

        let output_names = self.session.output_names();
        let outputs = self.session.run_f32(input)?;
        let get = |name: &str| -> Result<&[f32]> {
            output_names
                .iter()
                .position(|n| n == name)
                .map(|i| &outputs[i])
                .ok_or_else(|| {
                    FaceError::BadOutput(format!(
                        "missing output {name}; outputs are {output_names:?}"
                    ))
                })?
                .as_slice()
                .ok_or_else(|| non_contig(name))
        };

        let mut heads = Vec::with_capacity(STRIDES.len());
        for stride in STRIDES {
            heads.push(StrideHeads {
                stride,
                cls: get(&format!("cls_{stride}"))?,
                obj: get(&format!("obj_{stride}"))?,
                bbox: get(&format!("bbox_{stride}"))?,
                kps: get(&format!("kps_{stride}"))?,
            });
        }
        decode(&heads, scale)
    }
}

#[cfg(feature = "inference")]
fn non_contig(name: &str) -> FaceError {
    FaceError::BadOutput(format!("{name} output not contiguous"))
}

fn iou(a: &[f32; 4], b: &[f32; 4]) -> f32 {
    let x1 = a[0].max(b[0]);
    let y1 = a[1].max(b[1]);
    let x2 = (a[0] + a[2]).min(b[0] + b[2]);
    let y2 = (a[1] + a[3]).min(b[1] + b[3]);
    let inter = (x2 - x1).max(0.0) * (y2 - y1).max(0.0);
    let union = a[2] * a[3] + b[2] * b[3] - inter;
    if union <= 0.0 {
        0.0
    } else {
        inter / union
    }
}

fn nms(sorted: Vec<Face>) -> Vec<Face> {
    let mut keep: Vec<Face> = Vec::new();
    for face in sorted {
        if keep.iter().all(|k| iou(&k.bbox, &face.bbox) < NMS_IOU) {
            keep.push(face);
        }
    }
    keep
}
