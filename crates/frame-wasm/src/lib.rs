//! The OpenPhotoId pipeline, compiled to WebAssembly for the browser.
//!
//! Everything lives in `frame-session`; this crate is the wasm-bindgen
//! surface over it, and nothing else. The phone apps wrap the same crate
//! with Tauri commands, so the browser and the phones cannot drift apart.
//!
//! The call sequence a front end follows is documented on
//! `frame_session::Session`: decode → face_input → set_source_face →
//! matte_input → set_matte → prepare → solve → output_face_input →
//! validate_output → output_*.

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

fn js(e: frame_session::SessionError) -> JsError {
    JsError::new(&e.0)
}

#[wasm_bindgen]
pub fn specs_json() -> String {
    frame_session::specs_json()
}

/// The four per-stride tensors YuNet emits, collected on the JS side.
#[wasm_bindgen]
#[derive(Default)]
pub struct YunetHeads(frame_session::YunetHeads);

#[wasm_bindgen]
impl YunetHeads {
    #[wasm_bindgen(constructor)]
    pub fn new() -> YunetHeads {
        YunetHeads::default()
    }
    pub fn push(&mut self, stride: u32, cls: Vec<f32>, obj: Vec<f32>, bbox: Vec<f32>, kps: Vec<f32>) {
        self.0.push(stride, cls, obj, bbox, kps)
    }
}

#[wasm_bindgen]
pub struct Session(frame_session::Session);

#[wasm_bindgen]
impl Session {
    pub fn decode(bytes: &[u8]) -> Result<Session, JsError> {
        frame_session::Session::decode(bytes).map(Session).map_err(js)
    }
    #[wasm_bindgen(getter)]
    pub fn width(&self) -> u32 {
        self.0.width()
    }
    #[wasm_bindgen(getter)]
    pub fn height(&self) -> u32 {
        self.0.height()
    }
    pub fn source_jpeg(&self, quality: u8) -> Result<Vec<u8>, JsError> {
        self.0.source_jpeg(quality).map_err(js)
    }
    pub fn face_input(&mut self) -> Result<Vec<f32>, JsError> {
        self.0.face_input().map_err(js)
    }
    pub fn set_source_face(&mut self, heads: &YunetHeads) -> Result<bool, JsError> {
        self.0.set_source_face(&heads.0).map_err(js)
    }
    pub fn matte_input(&self, model: &str) -> Result<Vec<f32>, JsError> {
        self.0.matte_input(model).map_err(js)
    }
    pub fn set_matte(&mut self, map: &[f32], model: &str) -> Result<(), JsError> {
        self.0.set_matte(map, model).map_err(js)
    }
    pub fn detail_input(&mut self) -> Result<Vec<f32>, JsError> {
        self.0.detail_input().map_err(js)
    }
    pub fn set_detail_matte(&mut self, map: &[f32]) -> Result<(), JsError> {
        self.0.set_detail_matte(map).map_err(js)
    }
    #[wasm_bindgen(getter)]
    pub fn has_face(&self) -> bool {
        self.0.has_face()
    }
    #[wasm_bindgen(getter)]
    pub fn has_matte(&self) -> bool {
        self.0.has_matte()
    }
    pub fn set_backdrop_image(&mut self, bytes: &[u8]) -> Result<(), JsError> {
        self.0.set_backdrop_image(bytes).map_err(js)
    }
    pub fn prepare(&mut self, options_json: &str) -> Result<(), JsError> {
        self.0.prepare(options_json).map_err(js)
    }
    pub fn solve(&mut self, spec_id: &str, head_pct: f32, eye_pct: f32, center_pct: f32, enhance_output: bool) -> Result<String, JsError> {
        self.0.solve(spec_id, head_pct, eye_pct, center_pct, enhance_output).map_err(js)
    }
    pub fn output_face_input(&mut self) -> Result<Vec<f32>, JsError> {
        self.0.output_face_input().map_err(js)
    }
    pub fn validate_output(&self, heads: &YunetHeads) -> Result<String, JsError> {
        self.0.validate_output(&heads.0).map_err(js)
    }
    pub fn output_png(&self) -> Result<Vec<u8>, JsError> {
        self.0.output_png().map_err(js)
    }
    pub fn output_jpeg(&self, quality: u8) -> Result<Vec<u8>, JsError> {
        self.0.output_jpeg(quality).map_err(js)
    }
    pub fn output_jpeg_within(&self, min_kb: u32, max_kb: u32) -> Result<Vec<u8>, JsError> {
        self.0.output_jpeg_within(min_kb, max_kb).map_err(js)
    }
    pub fn sheet_jpeg(&self, sheet: &str, dpi: u32, cut_marks: bool, quality: u8) -> Result<Vec<u8>, JsError> {
        self.0.sheet_jpeg(sheet, dpi, cut_marks, quality).map_err(js)
    }
    pub fn sheet_info(&self, sheet: &str) -> Result<String, JsError> {
        self.0.sheet_info(sheet).map_err(js)
    }
}
