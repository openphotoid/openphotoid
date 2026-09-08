//! Tauri commands wrapping the single-photo pipeline: matting -> spec-driven
//! crop (background-padded) -> validate -> base64 JPEG for display.
//!
//! Photos are sent to/from the frontend as bytes (base64 in, base64 out)
//! rather than filesystem paths — the browser file picker (`<input
//! type="file">`) only gives JS a `File` object with no real OS path in a
//! sandboxed webview, and compliance-photo JPEGs are at most a few hundred
//! KB, so a JSON round-trip is fine here (unlike the 40MP-canvas case in
//! the M0 spike, where pushing pixels over IPC was explicitly the thing to
//! avoid).

use std::sync::Mutex;

use base64::Engine;
use frame_compliance::validate::Status;
use frame_compliance::{CropAdjustment, PhotoSpec};
use frame_core::Frame;
use frame_engine::Ep;
use frame_face::{Face, YuNet};
use frame_matting::modnet::Modnet;
use serde::Serialize;

/// The output of the expensive part of the pipeline (matting + face
/// detection), cached so [`adjust_crop`] can re-solve the crop and
/// re-validate cheaply — without re-running ONNX inference — every time
/// a user nudges an adjustment slider.
struct PreparedPhoto {
    bg_replaced: Frame,
    source_face: Face,
    spec: PhotoSpec,
}

pub struct AppState {
    matting: Mutex<Option<Modnet>>,
    face: Mutex<Option<YuNet>>,
    current: Mutex<Option<PreparedPhoto>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            matting: Mutex::new(None),
            face: Mutex::new(None),
            current: Mutex::new(None),
        }
    }
}

#[derive(Serialize)]
pub struct SpecSummary {
    pub id: String,
    pub country: String,
    pub document: String,
    pub background_hex: String,
    pub output_w: u32,
    pub output_h: u32,
}

#[derive(Debug, Serialize)]
pub struct CheckDto {
    pub name: String,
    pub status: String,
    pub detail: String,
}

/// The adjustment actually used to produce this result, plus the range a
/// UI slider can offer — `*_min`..`*_max` bounds the percentage-band
/// geometry checks (head height, eye line, centering), but not checks
/// that depend on the source photo's actual pixel detail (inter-eye
/// distance, blur), which can still fail near the edges of the range —
/// see [`CropAdjustment`]'s doc comment. Always look at `checks`/`overall`
/// after an adjustment rather than assuming the bounds alone guarantee it.
#[derive(Debug, Serialize)]
pub struct CropAdjustmentDto {
    pub head_pct: f32,
    pub eye_from_bottom_pct: f32,
    pub center_offset_pct: f32,
    pub head_min_pct: f32,
    pub head_max_pct: f32,
    pub eye_min_pct: f32,
    pub eye_max_pct: f32,
    pub center_offset_max_pct: f32,
}

#[derive(Debug, Serialize)]
pub struct ProcessResult {
    pub output_jpeg_base64: String,
    pub checks: Vec<CheckDto>,
    pub overall: String,
    pub adjustment: CropAdjustmentDto,
}

#[tauri::command]
pub fn list_specs() -> Vec<SpecSummary> {
    let mut specs: Vec<SpecSummary> = frame_compliance::spec::load_all()
        .into_iter()
        .map(|s| {
            let (w, h) = s.output_px();
            SpecSummary {
                id: s.id.clone(),
                country: s.country.clone(),
                document: s.document.clone(),
                background_hex: s.background.hex_render.clone(),
                output_w: w,
                output_h: h,
            }
        })
        .collect();
    specs.sort_by(|a, b| a.id.cmp(&b.id));
    specs
}

// Only referenced via tauri::generate_handler!'s string-based dispatch in
// main.rs, so the compiler sees it as unused in isolated compilations —
// e.g. the integration test in tests/pipeline_regression.rs, which
// includes this file as a separate module via #[path] specifically so
// process_photo_inner is callable without a running Tauri App instance.
#[allow(dead_code)]
#[tauri::command]
pub fn process_photo(
    state: tauri::State<AppState>,
    image_base64: String,
    spec_id: String,
) -> Result<ProcessResult, String> {
    process_photo_inner(&state, &image_base64, &spec_id)
}

/// The actual pipeline, taking a plain `&AppState` instead of
/// `tauri::State` so it's callable from a normal Rust test with a real
/// photo — `tauri::State` needs a running App to construct, which would
/// otherwise put this logic out of reach of anything but the full GUI
/// (unavailable in this headless dev VM — see docs/research/05-m0-results.md).
pub fn process_photo_inner(
    state: &AppState,
    image_base64: &str,
    spec_id: &str,
) -> Result<ProcessResult, String> {
    let specs = frame_compliance::spec::load_all();
    let spec = specs
        .into_iter()
        .find(|s| s.id == spec_id)
        .ok_or_else(|| format!("unknown spec {spec_id}"))?;

    let bytes = base64::engine::general_purpose::STANDARD
        .decode(image_base64.split(',').next_back().unwrap_or(image_base64))
        .map_err(|e| format!("invalid base64: {e}"))?;
    let frame =
        frame_core::io::load_from_bytes(&bytes).map_err(|e| format!("decode error: {e}"))?;
    // Cap huge source photos before the full-resolution refinement passes
    // (see frame_core::resize::DEFAULT_MAX_DIMENSION).
    let frame =
        frame_core::resize::resize_max_side(&frame, frame_core::resize::DEFAULT_MAX_DIMENSION)
            .map_err(|e| format!("resize error: {e}"))?;

    // Lazily load models once and keep them in managed state — avoids
    // reloading (and re-downloading, on first run) per photo.
    let mut face_guard = state
        .face
        .lock()
        .map_err(|e| format!("lock poisoned: {e}"))?;
    if face_guard.is_none() {
        *face_guard = Some(YuNet::load(Ep::Cpu).map_err(|e| format!("face model load: {e}"))?);
    }
    let detector = face_guard.as_mut().unwrap();

    let source_faces = detector
        .detect(&frame)
        .map_err(|e| format!("detect error: {e}"))?;
    let source_face = source_faces
        .into_iter()
        .next()
        .ok_or_else(|| "no face detected in the photo".to_string())?;

    let mut matting_guard = state
        .matting
        .lock()
        .map_err(|e| format!("lock poisoned: {e}"))?;
    if matting_guard.is_none() {
        *matting_guard =
            Some(Modnet::load(Ep::Cpu).map_err(|e| format!("matting model load: {e}"))?);
    }
    let model = matting_guard.as_mut().unwrap();

    let mut matte = model
        .infer(&frame)
        .map_err(|e| format!("matting inference error: {e}"))?;
    matte.refine(&frame);
    let bg_replaced = matte.composite_solid_defringed(&frame, spec.background.rgb());

    let adjustment = CropAdjustment::default_for(&spec);
    let result = finalize(detector, &bg_replaced, &source_face, &spec, adjustment)?;

    // Cache the expensive part (matting + face detection) so adjust_crop
    // can re-solve/re-validate cheaply as the user drags a slider, without
    // re-running ONNX inference on every change.
    *state
        .current
        .lock()
        .map_err(|e| format!("lock poisoned: {e}"))? = Some(PreparedPhoto {
        bg_replaced,
        source_face,
        spec,
    });

    Ok(result)
}

// Same dead-code note as `process_photo` above.
#[allow(dead_code)]
#[tauri::command]
pub fn adjust_crop(
    state: tauri::State<AppState>,
    head_pct: f32,
    eye_from_bottom_pct: f32,
    center_offset_pct: f32,
) -> Result<ProcessResult, String> {
    adjust_crop_inner(&state, head_pct, eye_from_bottom_pct, center_offset_pct)
}

/// Re-solves and re-validates the crop against the photo cached by the
/// last [`process_photo_inner`] call, using a caller-chosen point within
/// the spec's allowed bands instead of the automatic middle-of-band
/// default — cheap (no ONNX inference), suitable for calling on every
/// slider drag.
pub fn adjust_crop_inner(
    state: &AppState,
    head_pct: f32,
    eye_from_bottom_pct: f32,
    center_offset_pct: f32,
) -> Result<ProcessResult, String> {
    let current_guard = state
        .current
        .lock()
        .map_err(|e| format!("lock poisoned: {e}"))?;
    let prepared = current_guard
        .as_ref()
        .ok_or_else(|| "no photo processed yet".to_string())?;

    let mut face_guard = state
        .face
        .lock()
        .map_err(|e| format!("lock poisoned: {e}"))?;
    if face_guard.is_none() {
        *face_guard = Some(YuNet::load(Ep::Cpu).map_err(|e| format!("face model load: {e}"))?);
    }
    let detector = face_guard.as_mut().unwrap();

    let adjustment = CropAdjustment {
        head_pct,
        eye_from_bottom_pct,
        center_offset_pct,
    };
    finalize(
        detector,
        &prepared.bg_replaced,
        &prepared.source_face,
        &prepared.spec,
        adjustment,
    )
}

/// Shared by [`process_photo_inner`] (auto default) and
/// [`adjust_crop_inner`] (user-chosen point): solve the crop, apply it,
/// re-detect the face on the *output* to independently confirm where it
/// actually landed (not just trusting the analytical solve_crop math —
/// same reasoning as the M2 real-photo findings in
/// docs/research/05-m0-results.md), validate, and encode.
fn finalize(
    detector: &mut YuNet,
    bg_replaced: &Frame,
    source_face: &Face,
    spec: &PhotoSpec,
    adjustment: CropAdjustment,
) -> Result<ProcessResult, String> {
    let adjustment = adjustment.clamp_to(spec);
    let solution = frame_compliance::solve_crop(bg_replaced, source_face, spec, &adjustment);
    let output = frame_compliance::crop::apply_crop_padded(bg_replaced, &solution, spec)
        .map_err(|e| format!("crop error: {e}"))?;

    let final_faces = detector.detect(&output).unwrap_or_default();
    let report = frame_compliance::validate(&output, &final_faces, spec);

    let jpeg_bytes =
        frame_core::io::encode_jpeg(&output, 95).map_err(|e| format!("encode error: {e}"))?;
    let output_jpeg_base64 = format!(
        "data:image/jpeg;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(jpeg_bytes)
    );

    let f = &spec.face;
    Ok(ProcessResult {
        output_jpeg_base64,
        checks: report
            .checks
            .iter()
            .map(|c| CheckDto {
                name: c.name.to_string(),
                status: status_str(c.status).to_string(),
                detail: c.detail.clone(),
            })
            .collect(),
        overall: status_str(report.overall()).to_string(),
        adjustment: CropAdjustmentDto {
            head_pct: adjustment.head_pct,
            eye_from_bottom_pct: adjustment.eye_from_bottom_pct,
            center_offset_pct: adjustment.center_offset_pct,
            head_min_pct: f.head_min_pct,
            head_max_pct: f.head_max_pct,
            eye_min_pct: f.eye_min_from_bottom_pct,
            eye_max_pct: f.eye_max_from_bottom_pct,
            center_offset_max_pct: f.centering_tolerance_pct,
        },
    })
}

fn status_str(s: Status) -> &'static str {
    match s {
        Status::Pass => "pass",
        Status::Warn => "warn",
        Status::Fail => "fail",
        Status::NotChecked => "not_checked",
    }
}
