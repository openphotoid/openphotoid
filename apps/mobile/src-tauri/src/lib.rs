//! OpenPhotoId on a phone: the web app's UI, the same `frame-session`
//! pipeline the browser runs, and inference done natively through tract
//! rather than as WebAssembly inside the WebView.
//!
//! The JavaScript side (`apps/webapp/src/lib/platform.js`) calls these
//! commands instead of onnxruntime-web and the wasm module. Photos travel
//! as base64 — a compliance photo is at most a few hundred KB.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use base64::Engine as _;
use frame_engine::Ep;
use frame_face::YuNet;
use frame_matting::modnet::Modnet;
use frame_session::{Session, YunetHeads};
use ndarray::{ArrayD, IxDyn};
use serde::Serialize;
use tauri::Manager;

struct Models {
    face: YuNet,
    matting: Modnet,
}

#[derive(Default)]
pub struct AppState {
    models: Mutex<Option<Models>>,
    sessions: Mutex<HashMap<u32, Session>>,
    next_id: Mutex<u32>,
}

type CmdResult<T> = Result<T, String>;

fn b64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}
fn unb64(s: &str) -> CmdResult<Vec<u8>> {
    base64::engine::general_purpose::STANDARD
        .decode(s)
        .map_err(|e| format!("bad base64: {e}"))
}

#[cfg(target_os = "android")]
const MODEL_FILES: [&str; 2] = [
    "face_detection_yunet_2023mar.onnx",
    "modnet_photographic_portrait_matting.tract.onnx",
];

/// Where the bundled models can be opened as ordinary files.
///
/// On iOS and desktop the bundle's resource directory is a real directory
/// and is used as-is. On Android it is an `asset://localhost/` URI inside
/// the APK, which `std::fs` cannot open and which is read-only, so the two
/// models are copied through the fs plugin (which resolves assets via the
/// Android AssetManager) into the app's data directory once, and that
/// directory is used from then on.
fn models_dir(app: &tauri::AppHandle) -> CmdResult<PathBuf> {
    let resources: PathBuf = app
        .path()
        .resource_dir()
        .map_err(|e| format!("resource dir: {e}"))?
        .join("models");
    #[cfg(not(target_os = "android"))]
    {
        Ok(resources)
    }
    #[cfg(target_os = "android")]
    {
        materialise_assets(app, &resources)
    }
}

/// Android only: copy the models out of the APK into the data directory.
#[cfg(target_os = "android")]
fn materialise_assets(app: &tauri::AppHandle, resources: &std::path::Path) -> CmdResult<PathBuf> {
    use tauri_plugin_fs::FsExt as _;
    if !resources
        .to_string_lossy()
        .starts_with(tauri::utils::platform::ANDROID_ASSET_PROTOCOL_URI_PREFIX)
    {
        return Ok(resources.to_path_buf());
    }
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("data dir: {e}"))?
        .join("models");
    std::fs::create_dir_all(&dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
    for name in MODEL_FILES {
        let dst = dir.join(name);
        if dst.exists() {
            continue;
        }
        let src = resources.join(name);
        let bytes = app
            .fs()
            .read(src.clone())
            .map_err(|e| format!("read asset {}: {e}", src.display()))?;
        let part = dir.join(format!("{name}.part"));
        std::fs::write(&part, &bytes).map_err(|e| format!("write {}: {e}", part.display()))?;
        std::fs::rename(&part, &dst).map_err(|e| format!("place {}: {e}", dst.display()))?;
    }
    Ok(dir)
}

/// The two models, loaded once from the app bundle's resources.
fn models<'a>(app: &tauri::AppHandle, guard: &'a mut Option<Models>) -> CmdResult<&'a mut Models> {
    if guard.is_none() {
        let dir = models_dir(app)?;
        // The registry would download on first run; a phone app ships them.
        std::env::set_var("OPENPHOTOID_MODELS_DIR", &dir);
        let face = YuNet::load(Ep::Cpu).map_err(|e| format!("face model: {e}"))?;
        let matting = Modnet::load_from_path(
            &dir.join("modnet_photographic_portrait_matting.tract.onnx"),
            Ep::Cpu,
        )
        .map_err(|e| format!("matting model: {e}"))?;
        *guard = Some(Models { face, matting });
    }
    Ok(guard.as_mut().unwrap())
}

/// Run YuNet on a prepared 640×640 input and hand the heads to the session.
fn detect(face: &mut YuNet, input: Vec<f32>) -> CmdResult<YunetHeads> {
    let arr = ArrayD::from_shape_vec(IxDyn(&[1, 3, 640, 640]), input)
        .map_err(|e| format!("face input: {e}"))?;
    let (names, outputs) = face.run_raw(arr).map_err(|e| format!("face: {e}"))?;
    let mut heads = YunetHeads::new();
    for stride in [8u32, 16, 32] {
        let get = |n: &str| -> CmdResult<Vec<f32>> {
            let i = names
                .iter()
                .position(|x| x == n)
                .ok_or(format!("no output {n}"))?;
            Ok(outputs[i].iter().copied().collect())
        };
        heads.push(
            stride,
            get(&format!("cls_{stride}"))?,
            get(&format!("obj_{stride}"))?,
            get(&format!("bbox_{stride}"))?,
            get(&format!("kps_{stride}"))?,
        );
    }
    Ok(heads)
}

fn matte(matting: &mut Modnet, input: Vec<f32>) -> CmdResult<Vec<f32>> {
    let arr = ArrayD::from_shape_vec(IxDyn(&[1, 3, 512, 512]), input)
        .map_err(|e| format!("matte input: {e}"))?;
    let out = matting.run_raw(arr).map_err(|e| format!("matting: {e}"))?;
    Ok(out.iter().copied().collect())
}

#[derive(Serialize)]
pub struct Opened {
    id: u32,
    width: u32,
    height: u32,
    face_found: bool,
    source_jpeg: String,
}

/// Decode, detect, matte (and the detail pass): the expensive half.
#[tauri::command]
fn session_open(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    bytes: String,
    quality: bool,
) -> CmdResult<Opened> {
    let bytes = unb64(&bytes)?;
    let mut session = Session::decode(&bytes).map_err(|e| e.to_string())?;
    let mut guard = state.models.lock().unwrap();
    let m = models(&app, &mut guard)?;

    let heads = detect(
        &mut m.face,
        session.face_input().map_err(|e| e.to_string())?,
    )?;
    let face_found = session.set_source_face(&heads).map_err(|e| e.to_string())?;
    let map = matte(
        &mut m.matting,
        session.matte_input("modnet").map_err(|e| e.to_string())?,
    )?;
    session
        .set_matte(&map, "modnet")
        .map_err(|e| e.to_string())?;
    if quality && face_found {
        let detail = session.detail_input().map_err(|e| e.to_string())?;
        if !detail.is_empty() {
            let dm = matte(&mut m.matting, detail)?;
            session.set_detail_matte(&dm).map_err(|e| e.to_string())?;
        }
    }
    let source_jpeg = b64(&session.source_jpeg(80).map_err(|e| e.to_string())?);
    let (width, height) = (session.width(), session.height());
    let id = {
        let mut n = state.next_id.lock().unwrap();
        *n += 1;
        *n
    };
    state.sessions.lock().unwrap().insert(id, session);
    Ok(Opened {
        id,
        width,
        height,
        face_found,
        source_jpeg,
    })
}

#[derive(Serialize)]
pub struct Rendered {
    solve: serde_json::Value,
    report: serde_json::Value,
    output_jpeg: String,
}

/// The cheap half: finishing, crop, re-detect on the output, validate.
#[allow(clippy::too_many_arguments)] // one command per render; the arguments are the UI sliders
#[tauri::command]
fn session_render(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    id: u32,
    spec_id: String,
    options: String,
    head: Option<f32>,
    eye: Option<f32>,
    center: Option<f32>,
    enhance: bool,
) -> CmdResult<Rendered> {
    let mut sessions = state.sessions.lock().unwrap();
    let s = sessions.get_mut(&id).ok_or("no such session")?;
    s.prepare(&options).map_err(|e| e.to_string())?;
    let solve = s
        .solve(
            &spec_id,
            head.unwrap_or(f32::NAN),
            eye.unwrap_or(f32::NAN),
            center.unwrap_or(f32::NAN),
            enhance,
        )
        .map_err(|e| e.to_string())?;
    let mut guard = state.models.lock().unwrap();
    let m = models(&app, &mut guard)?;
    let heads = detect(
        &mut m.face,
        s.output_face_input().map_err(|e| e.to_string())?,
    )?;
    let report = s.validate_output(&heads).map_err(|e| e.to_string())?;
    Ok(Rendered {
        solve: serde_json::from_str(&solve).map_err(|e| e.to_string())?,
        report: serde_json::from_str(&report).map_err(|e| e.to_string())?,
        output_jpeg: b64(&s.output_jpeg(92).map_err(|e| e.to_string())?),
    })
}

/// One of: jpeg(quality) · jpeg_within(min_kb,max_kb) · png · sheet(name,dpi,cut_marks,quality).
#[tauri::command]
fn session_export(
    state: tauri::State<'_, AppState>,
    id: u32,
    kind: String,
    a: Option<f64>,
    b: Option<f64>,
    sheet: Option<String>,
    cut_marks: Option<bool>,
) -> CmdResult<String> {
    let sessions = state.sessions.lock().unwrap();
    let s = sessions.get(&id).ok_or("no such session")?;
    let bytes = match kind.as_str() {
        "jpeg" => s.output_jpeg(a.unwrap_or(95.0) as u8),
        "jpeg_within" => {
            s.output_jpeg_within(a.unwrap_or(0.0) as u32, b.unwrap_or(10_000.0) as u32)
        }
        "png" => s.output_png(),
        "sheet" => s.sheet_jpeg(
            sheet.as_deref().unwrap_or("4x6"),
            a.unwrap_or(300.0) as u32,
            cut_marks.unwrap_or(true),
            b.unwrap_or(95.0) as u8,
        ),
        other => return Err(format!("unknown export {other}")),
    }
    .map_err(|e| e.to_string())?;
    Ok(b64(&bytes))
}

#[tauri::command]
fn session_sheet_info(
    state: tauri::State<'_, AppState>,
    id: u32,
    sheet: String,
) -> CmdResult<serde_json::Value> {
    let sessions = state.sessions.lock().unwrap();
    let s = sessions.get(&id).ok_or("no such session")?;
    serde_json::from_str(&s.sheet_info(&sheet).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn session_set_backdrop_image(
    state: tauri::State<'_, AppState>,
    id: u32,
    bytes: String,
) -> CmdResult<()> {
    let mut sessions = state.sessions.lock().unwrap();
    let s = sessions.get_mut(&id).ok_or("no such session")?;
    s.set_backdrop_image(&unb64(&bytes)?)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn session_free(state: tauri::State<'_, AppState>, id: u32) {
    state.sessions.lock().unwrap().remove(&id);
}

#[tauri::command]
fn specs_json() -> String {
    frame_session::specs_json()
}

/// What the diagnostics page shows for "ran on".
#[tauri::command]
fn platform_info() -> serde_json::Value {
    serde_json::json!({
        "provider": "native",
        "engine": "tract",
        "threads": std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1),
        // `-diag` on the command line (simctl launch … -diag) opens the device test.
        "autorun_diagnostics": std::env::args().any(|a| a == "-diag"),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            session_open,
            session_render,
            session_export,
            session_sheet_info,
            session_set_backdrop_image,
            session_free,
            specs_json,
            platform_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running OpenPhotoId");
}
