//! The OpenPhotoId pipeline as one session, with inference left to the
//! caller.
//!
//! This is the crate behind both the browser build (`frame-wasm`, which
//! hands the ONNX graphs to onnxruntime-web) and the phone apps (which run
//! them natively through `tract`). Everything except the forward pass lives
//! here, once.
//!
//! # Why this shape
//!
//! The desktop build owns its whole pipeline, ONNX included, through
//! `frame-engine`. A browser cannot: `ort` has no wasm32 target, and the
//! pure-Rust alternative (`tract`) is far too slow for MODNet on a phone.
//! So in this build **JavaScript runs the two ONNX graphs**, through
//! onnxruntime-web, which reaches WebGPU and SIMD that a Rust wasm module
//! cannot.
//!
//! What that must not become is a second implementation of the pipeline.
//! Everything except the forward pass itself lives here and is the same
//! code the desktop runs — decode and EXIF orientation, YuNet's
//! letterboxing and head decode, MODNet's normalisation and matte
//! upsample, the guided-filter refinement, foreground estimation,
//! background compositing, the crop solver, the 11-check validator, the
//! KB-window JPEG search, and the print-sheet layout. JavaScript's share
//! is: hand us bytes, run a tensor through a model, hand back the output.
//!
//! The call sequence a front end follows:
//!
//! ```text
//! Session::decode(file bytes)
//!   -> face_input()          --[ORT: YuNet]-->  set_source_face(heads)
//!   -> matte_input(model)    --[ORT: MODNet]--> set_matte(map, model)
//!   -> prepare(options)                        // retouch, backdrop, garment
//!   -> solve(spec, adjustment)                 // crop + resize (+ enhance)
//!   -> output_face_input()   --[ORT: YuNet]-->  validate_output(heads)
//!   -> output_jpeg_within() / output_png() / sheet_jpeg()
//! ```
//!
//! Every step after `set_matte` is cheap, so a front end re-runs
//! `prepare`/`solve` freely as the user drags a slider without touching
//! the models again — the same caching the desktop's `AppState` does.

use frame_compliance::{CropAdjustment, PhotoSpec};
use frame_core::Frame;
use frame_face::{Face, StrideHeads};
use frame_matting::Matte;
use frame_retouch::{backdrop::Backdrop, enhance, garment, skin};
use serde::{Deserialize, Serialize};


/// One error type for the whole session: a message a front end can show.
#[derive(Debug, Clone, thiserror::Error)]
#[error("{0}")]
pub struct SessionError(pub String);

impl SessionError {
    pub fn new(msg: impl Into<String>) -> Self {
        SessionError(msg.into())
    }
}

fn err<E: std::fmt::Display>(context: &str) -> impl FnOnce(E) -> SessionError + '_ {
    move |e| SessionError(format!("{context}: {e}"))
}

// ---------------------------------------------------------------- specs

/// Every built-in spec, as JSON — the picker, the coverage page and the
/// per-spec detail all read from this one source, which is the same
/// compile-time-embedded `data/specs/` dataset the desktop app uses.
///
/// The research (§九 切入点2) is explicit that *publishing* the covered
/// list, with the date each entry was last checked against its source, is
/// the differentiator here; vagueness about coverage is the complaint
/// competitors attract. So `last_verified` and `sources` ride along rather
/// than being stripped for size.
pub fn specs_json() -> String {
    let mut specs = frame_compliance::spec::load_all();
    specs.sort_by(|a, b| a.id.cmp(&b.id));
    serde_json::to_string(&specs).unwrap_or_else(|_| "[]".into())
}

// ------------------------------------------------------------ ONNX I/O

/// The four per-stride tensors YuNet emits, collected on the JS side and
/// handed back in one object.
///
/// Twelve separate arguments would be the alternative; this keeps the
/// stride association explicit so a caller cannot silently pair `cls_8`
/// with `bbox_16`.
#[derive(Default)]
pub struct YunetHeads {
    strides: Vec<u32>,
    cls: Vec<Vec<f32>>,
    obj: Vec<Vec<f32>>,
    bbox: Vec<Vec<f32>>,
    kps: Vec<Vec<f32>>,
}

impl YunetHeads {
    pub fn new() -> YunetHeads {
        YunetHeads::default()
    }

    pub fn push(
        &mut self,
        stride: u32,
        cls: Vec<f32>,
        obj: Vec<f32>,
        bbox: Vec<f32>,
        kps: Vec<f32>,
    ) {
        self.strides.push(stride);
        self.cls.push(cls);
        self.obj.push(obj);
        self.bbox.push(bbox);
        self.kps.push(kps);
    }

    pub fn decode(&self, scale: f32) -> Result<Vec<Face>, SessionError> {
        let heads: Vec<StrideHeads<'_>> = (0..self.strides.len())
            .map(|i| StrideHeads {
                stride: self.strides[i],
                cls: &self.cls[i],
                obj: &self.obj[i],
                bbox: &self.bbox[i],
                kps: &self.kps[i],
            })
            .collect();
        frame_face::decode(&heads, scale).map_err(err("face decode"))
    }
}

/// Which matting model produced a map handed to [`Session::set_matte`].
fn matte_from_map(model: &str, map: &[f32], w: u32, h: u32) -> Result<Matte, SessionError> {
    match model {
        "modnet" => frame_matting::modnet::postprocess(map, w, h).map_err(err("modnet output")),
        "birefnet" => {
            frame_matting::birefnet::postprocess(map, w, h).map_err(err("birefnet output"))
        }
        other => Err(SessionError::new(format!("unknown matting model {other}"))),
    }
}

// ------------------------------------------------------------- options

/// Everything the finishing stage can do, as one object so a front end can
/// re-render from a single settings blob.
#[derive(Debug, Clone, Deserialize)]
pub struct PrepareOptions {
    /// Skin smoothing, 0.0-0.6. **Defaults to zero and stays there unless
    /// asked**: the research found forced beautification is itself one of
    /// this category's top complaints, and an over-smoothed ID photo gets
    /// rejected.
    #[serde(default)]
    pub retouch: f32,
    #[serde(default)]
    pub backdrop: BackdropOption,
    /// `None` leaves the subject in their own clothes.
    #[serde(default)]
    pub garment: Option<GarmentOption>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum BackdropOption {
    /// Flat fill — what every document spec actually requires.
    Solid {
        rgb: [u8; 3],
    },
    Gradient {
        top: [u8; 3],
        bottom: [u8; 3],
    },
    Vignette {
        center: [u8; 3],
        edge: [u8; 3],
        #[serde(default = "default_center_y")]
        center_y: f32,
    },
    /// Uses the image most recently passed to
    /// [`Session::set_backdrop_image`].
    Image {
        #[serde(default)]
        blur: usize,
    },
    /// Keep the original background — no matting composite at all.
    Keep,
}

fn default_center_y() -> f32 {
    0.35
}

impl Default for BackdropOption {
    fn default() -> Self {
        BackdropOption::Solid {
            rgb: [255, 255, 255],
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GarmentOption {
    pub style: String,
    pub jacket: [u8; 3],
    pub shirt: [u8; 3],
    pub tie: [u8; 3],
}

impl GarmentOption {
    fn to_params(&self) -> Result<garment::GarmentParams, SessionError> {
        let style = match self.style.as_str() {
            "suit-tie" => garment::Style::SuitTie,
            "suit-open" => garment::Style::SuitOpen,
            "blouse" => garment::Style::Blouse,
            "shirt" => garment::Style::Shirt,
            other => return Err(SessionError::new(format!("unknown garment style {other}"))),
        };
        Ok(garment::GarmentParams {
            style,
            jacket: self.jacket,
            shirt: self.shirt,
            tie: self.tie,
        })
    }
}

// -------------------------------------------------------------- output

#[derive(Debug, Serialize)]
pub struct CheckDto {
    pub name: String,
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Serialize)]
pub struct SolveDto {
    pub width: u32,
    pub height: u32,
    /// True when the requested crop ran past the source photo's edges and
    /// was filled with the spec's background colour — legitimate for a
    /// tight head-and-shoulders source, but worth telling the user.
    pub padded: bool,
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
pub struct ReportDto {
    pub overall: String,
    pub checks: Vec<CheckDto>,
}

#[derive(Debug, Serialize)]
pub struct SheetDto {
    pub cols: u32,
    pub rows: u32,
    pub count: u32,
    pub sheet_w_mm: f32,
    pub sheet_h_mm: f32,
}

// ------------------------------------------------------------- session

/// The browser's working ceiling on the long side, in pixels.
///
/// The desktop uses `frame_core::resize::DEFAULT_MAX_DIMENSION` (4000).
/// Every stage after decode — the guided filter, foreground estimation,
/// the composite — is per-pixel over the working frame, and a 4000-px
/// frame is 12 MP of RGBA plus an f32 matte, which is what a phone's wasm
/// heap cannot hold twice over. The largest output any spec asks for is
/// 900 px on a side, and a head-and-shoulders crop is rarely under half
/// the frame, so 2600 px leaves the crop at least at output resolution
/// while halving the work on a 12-MP phone photo. Measured on a
/// 2687×3356 portrait: matte refinement 720 ms → 430 ms.
const WEB_MAX_DIMENSION: u32 = 2600;

/// One photo, carried through the pipeline. Mirrors the desktop's
/// `AppState` including its caching: the expensive stages (face detection,
/// matting) run once, and everything downstream re-runs from the cached
/// result.
pub struct Session {
    source: Frame,
    face_scale: f32,
    output_face_scale: f32,
    source_face: Option<Face>,
    matte: Option<Matte>,
    backdrop_image: Option<image::RgbaImage>,
    detail_rect: Option<[u32; 4]>,
    prepared: Option<Frame>,
    /// The options JSON `prepared` was built from. A slider drag calls
    /// `prepare` with the same options every time; matching this string
    /// is what turns the full-frame composite into a no-op for it.
    prepared_for: Option<String>,
    spec: Option<PhotoSpec>,
    output: Option<Frame>,
}

impl Session {
    /// Decode a picked file: any format the `image` crate reads, EXIF
    /// orientation applied, downscaled to the pipeline's working ceiling.
    ///
    /// The ceiling is not cosmetic — the refinement and foreground passes
    /// are full-resolution, and a 48-megapixel phone photo will exhaust a
    /// mobile browser's wasm heap without it. It is the same constant the
    /// desktop and batch paths use.
    pub fn decode(bytes: &[u8]) -> Result<Session, SessionError> {
        let frame = frame_core::io::load_from_bytes(bytes).map_err(err("decode"))?;
        let frame = frame_core::resize::resize_max_side(&frame, WEB_MAX_DIMENSION)
            .map_err(err("resize"))?;
        Ok(Session {
            source: frame,
            face_scale: 1.0,
            output_face_scale: 1.0,
            source_face: None,
            matte: None,
            backdrop_image: None,
            detail_rect: None,
            prepared: None,
            prepared_for: None,
            spec: None,
            output: None,
        })
    }

    pub fn width(&self) -> u32 {
        self.source.width()
    }

    pub fn height(&self) -> u32 {
        self.source.height()
    }

    /// The decoded source as JPEG, for showing the user what was picked
    /// without re-decoding the original file in the page.
    pub fn source_jpeg(&self, quality: u8) -> Result<Vec<u8>, SessionError> {
        frame_core::io::encode_jpeg(&self.source, quality).map_err(err("encode source"))
    }

    /// YuNet's input tensor for the source photo, NCHW BGR 1x3x640x640.
    pub fn face_input(&mut self) -> Result<Vec<f32>, SessionError> {
        let (input, scale) =
            frame_face::preprocess(&self.source).map_err(err("face preprocess"))?;
        self.face_scale = scale;
        Ok(input)
    }

    /// Decode YuNet's heads and keep the highest-scoring face. Returns
    /// false when there is no face, which is a normal outcome the UI has
    /// to handle, not an error.
    pub fn set_source_face(&mut self, heads: &YunetHeads) -> Result<bool, SessionError> {
        let faces = heads.decode(self.face_scale)?;
        self.source_face = faces.into_iter().next();
        Ok(self.source_face.is_some())
    }

    /// The matting model's input tensor. `model` is `"modnet"` (the fast
    /// default, 512px) or `"birefnet"` (the quality tier, 1024px).
    pub fn matte_input(&self, model: &str) -> Result<Vec<f32>, SessionError> {
        match model {
            "modnet" => {
                frame_matting::modnet::preprocess(&self.source).map_err(err("modnet input"))
            }
            "birefnet" => {
                frame_matting::birefnet::preprocess(&self.source).map_err(err("birefnet input"))
            }
            other => Err(SessionError::new(format!("unknown matting model {other}"))),
        }
    }

    /// Take the model's alpha map, upsample it to full resolution and snap
    /// it to real edges with the guided filter — the step that decides
    /// whether hair looks like hair or like a cut-out.
    pub fn set_matte(&mut self, map: &[f32], model: &str) -> Result<(), SessionError> {
        let mut matte = matte_from_map(model, map, self.source.width(), self.source.height())?;
        matte.refine(&self.source);
        self.matte = Some(matte);
        self.prepared = None;
        self.prepared_for = None;
        self.output = None;
        Ok(())
    }

    /// MODNet's input tensor for a crop around the head — the "detail
    /// pass" of the quality tier.
    ///
    /// Returns an empty vector when there is no face to crop around, in
    /// which case the caller should simply skip the pass; a half-body
    /// photo with no detected face has no region worth spending a second
    /// inference on.
    pub fn detail_input(&mut self) -> Result<Vec<f32>, SessionError> {
        let Some(face) = self.source_face.clone() else {
            self.detail_rect = None;
            return Ok(Vec::new());
        };
        let rect = detail_rect(&face, self.source.width(), self.source.height());
        let [x, y, w, h] = rect;
        if w < 32 || h < 32 {
            self.detail_rect = None;
            return Ok(Vec::new());
        }
        // Skip the pass when the head already fills most of the frame:
        // the crop would be nearly the whole image and the second
        // inference would cost the same and buy nothing.
        let coverage =
            (w as f32 * h as f32) / (self.source.width() as f32 * self.source.height() as f32);
        if coverage > 0.55 {
            self.detail_rect = None;
            return Ok(Vec::new());
        }
        let cropped = image::imageops::crop_imm(&self.source.pixels, x, y, w, h).to_image();
        let crop_frame = Frame {
            pixels: cropped,
            dpi: self.source.dpi,
        };
        self.detail_rect = Some(rect);
        frame_matting::modnet::preprocess(&crop_frame).map_err(err("detail input"))
    }

    /// Merge the detail pass back into the full-frame matte.
    pub fn set_detail_matte(&mut self, map: &[f32]) -> Result<(), SessionError> {
        let Some(rect) = self.detail_rect else {
            return Ok(());
        };
        let matte = self
            .matte
            .as_mut()
            .ok_or_else(|| SessionError::new("set_matte must run before set_detail_matte"))?;
        let [x, y, w, h] = rect;
        let detail = frame_matting::modnet::postprocess(map, w, h).map_err(err("detail output"))?;
        // Feather across a twentieth of the crop, which is a few pixels at
        // typical sizes — enough that the seam is invisible, small enough
        // that it does not dilute the detail it is there to deliver.
        matte.blend_region(&detail, rect, (w.min(h) / 20).max(2));

        // Re-run the guided filter over the merged result: the blend is a
        // weighted average of two mattes that disagree slightly, and the
        // refinement is what snaps the merged alpha back onto real edges.
        let source = self.source.clone();
        matte.refine(&source);
        self.prepared = None;
        self.prepared_for = None;
        self.output = None;
        let _ = (x, y);
        Ok(())
    }

    pub fn has_face(&self) -> bool {
        self.source_face.is_some()
    }

    pub fn has_matte(&self) -> bool {
        self.matte.is_some()
    }

    /// Supply the image for [`BackdropOption::Image`]. Kept separate from
    /// `prepare` so switching backdrops does not re-decode it.
    pub fn set_backdrop_image(&mut self, bytes: &[u8]) -> Result<(), SessionError> {
        let frame = frame_core::io::load_from_bytes(bytes).map_err(err("backdrop decode"))?;
        self.backdrop_image = Some(frame.pixels);
        // The same options JSON now means a different picture.
        self.prepared = None;
        self.prepared_for = None;
        Ok(())
    }

    /// Run the finishing stage: retouch, background, garment. Cheap enough
    /// to call on every control change.
    pub fn prepare(&mut self, options_json: &str) -> Result<(), SessionError> {
        // The position sliders re-render on every input event and never
        // change these options, so most calls find the composite already
        // done. Measured on a 2687×3356 photo in the browser: this took
        // the per-drag render from 1.5 s to the crop alone.
        if self.prepared.is_some() && self.prepared_for.as_deref() == Some(options_json) {
            return Ok(());
        }
        let opts: PrepareOptions =
            serde_json::from_str(options_json).map_err(err("prepare options"))?;

        // Retouch first, on the source: the matte was computed from the
        // unsmoothed pixels and stays valid, so changing the slider never
        // costs another inference pass. With the slider at zero the
        // source is borrowed, not copied — a 20 MB clone per render.
        let faces: Vec<Face> = self.source_face.clone().into_iter().collect();
        let working: std::borrow::Cow<'_, Frame> = if opts.retouch > 0.0 {
            std::borrow::Cow::Owned(skin::smooth(&self.source, &faces, opts.retouch))
        } else {
            std::borrow::Cow::Borrowed(&self.source)
        };

        let mut prepared = match (&opts.backdrop, &self.matte) {
            (BackdropOption::Keep, _) | (_, None) => working.into_owned(),
            (choice, Some(matte)) => {
                let backdrop = match choice {
                    BackdropOption::Solid { rgb } => Backdrop::Solid(*rgb),
                    BackdropOption::Gradient { top, bottom } => Backdrop::Gradient {
                        top: *top,
                        bottom: *bottom,
                    },
                    BackdropOption::Vignette {
                        center,
                        edge,
                        center_y,
                    } => Backdrop::Vignette {
                        center: *center,
                        edge: *edge,
                        center_y: *center_y,
                    },
                    BackdropOption::Image { blur } => {
                        let rgba = self.backdrop_image.clone().ok_or_else(|| {
                            SessionError::new("no backdrop image set — call set_backdrop_image first")
                        })?;
                        Backdrop::Image {
                            rgba,
                            blur_radius: *blur,
                        }
                    }
                    BackdropOption::Keep => unreachable!("handled above"),
                };
                backdrop.composite(&working, matte)
            }
        };

        if let Some(g) = &opts.garment {
            match (&self.source_face, &self.matte) {
                (Some(face), Some(matte)) => {
                    prepared = garment::apply(&prepared, matte, face, g.to_params()?);
                }
                _ => {
                    return Err(SessionError::new(
                        "garment needs both a detected face and a matte",
                    ))
                }
            }
        }

        self.prepared = Some(prepared);
        self.prepared_for = Some(options_json.to_string());
        self.output = None;
        Ok(())
    }

    /// Solve and apply the compliance crop for `spec_id`.
    ///
    /// `head_pct` / `eye_pct` / `center_pct` place the crop within the
    /// spec's allowed bands; pass `f32::NAN` for any of them to take the
    /// automatic middle-of-band default. The returned bounds are what a
    /// slider should offer — but re-read the checklist after every change
    /// rather than trusting them, because the checks that depend on source
    /// pixel detail (inter-eye distance, blur) can still fail inside the
    /// band. That caveat is inherited from `CropAdjustment`, where it was
    /// found with a real photo rather than reasoned about.
    ///
    /// `enhance` applies the print-sharpening pass after the final resize.
    pub fn solve(
        &mut self,
        spec_id: &str,
        head_pct: f32,
        eye_pct: f32,
        center_pct: f32,
        enhance_output: bool,
    ) -> Result<String, SessionError> {
        let spec = match self.spec.take().filter(|s| s.id == spec_id) {
            Some(spec) => spec,
            None => frame_compliance::spec::load_all()
                .into_iter()
                .find(|s| s.id == spec_id)
                .ok_or_else(|| SessionError::new(format!("unknown spec {spec_id}")))?,
        };
        let face = self
            .source_face
            .clone()
            .ok_or_else(|| SessionError::new("no face detected"))?;
        let prepared = self
            .prepared
            .as_ref()
            .ok_or_else(|| SessionError::new("call prepare() before solve()"))?;

        let auto = CropAdjustment::default_for(&spec);
        let adjustment = CropAdjustment {
            head_pct: pick(head_pct, auto.head_pct),
            eye_from_bottom_pct: pick(eye_pct, auto.eye_from_bottom_pct),
            center_offset_pct: pick(center_pct, auto.center_offset_pct),
        }
        .clamp_to(&spec);

        let solution = frame_compliance::solve_crop(prepared, &face, &spec, &adjustment);
        let padded = solution.out_of_bounds;
        let mut output = frame_compliance::crop::apply_crop_padded(prepared, &solution, &spec)
            .map_err(err("crop"))?;

        if enhance_output {
            output = enhance::unsharp(&output, enhance::EnhanceParams::default());
        }
        // Carry the spec's print DPI so an exported file prints at the
        // right physical size rather than at whatever the viewer assumes.
        if let Some(print) = &spec.print {
            output.dpi = frame_core::Dpi(print.dpi_default as f32);
        }

        let f = &spec.face;
        let dto = SolveDto {
            width: output.width(),
            height: output.height(),
            padded,
            head_pct: adjustment.head_pct,
            eye_from_bottom_pct: adjustment.eye_from_bottom_pct,
            center_offset_pct: adjustment.center_offset_pct,
            head_min_pct: f.head_min_pct,
            head_max_pct: f.head_max_pct,
            eye_min_pct: f.eye_min_from_bottom_pct,
            eye_max_pct: f.eye_max_from_bottom_pct,
            center_offset_max_pct: f.centering_tolerance_pct,
        };
        self.output = Some(output);
        self.spec = Some(spec);
        serde_json::to_string(&dto).map_err(err("solve result"))
    }

    /// YuNet's input tensor for the *cropped output*.
    ///
    /// Validation re-detects on the result rather than trusting the
    /// analytical solve — a lesson from the M2 real-photo work, where the
    /// two disagreed often enough to matter.
    pub fn output_face_input(&mut self) -> Result<Vec<f32>, SessionError> {
        let output = self
            .output
            .as_ref()
            .ok_or_else(|| SessionError::new("no output yet"))?;
        let (input, scale) = frame_face::preprocess(output).map_err(err("face preprocess"))?;
        self.output_face_scale = scale;
        Ok(input)
    }

    /// Run the 11-check compliance report against the output.
    pub fn validate_output(&self, heads: &YunetHeads) -> Result<String, SessionError> {
        let output = self
            .output
            .as_ref()
            .ok_or_else(|| SessionError::new("no output yet"))?;
        let spec = self
            .spec
            .as_ref()
            .ok_or_else(|| SessionError::new("no spec selected"))?;
        let faces = heads.decode(self.output_face_scale)?;
        let report = frame_compliance::validate(output, &faces, spec);
        let dto = ReportDto {
            overall: status_str(report.overall()).to_string(),
            checks: report
                .checks
                .iter()
                .map(|c| CheckDto {
                    name: c.name.to_string(),
                    status: status_str(c.status).to_string(),
                    detail: c.detail.clone(),
                })
                .collect(),
        };
        serde_json::to_string(&dto).map_err(err("report"))
    }

    pub fn output_png(&self) -> Result<Vec<u8>, SessionError> {
        frame_core::io::encode_png(self.output_ref()?).map_err(err("png"))
    }

    pub fn output_jpeg(&self, quality: u8) -> Result<Vec<u8>, SessionError> {
        frame_core::io::encode_jpeg(self.output_ref()?, quality).map_err(err("jpeg"))
    }

    /// Encode inside a KB window, which several portals enforce on both
    /// sides (DS-160 <= 240 KB, India PAN 20-50 KB, China COVA 40-120 KB).
    pub fn output_jpeg_within(&self, min_kb: u32, max_kb: u32) -> Result<Vec<u8>, SessionError> {
        frame_core::io::encode_jpeg_within(self.output_ref()?, min_kb, max_kb)
            .map_err(err("jpeg within size window"))
    }

    /// Tile the output onto a print sheet. `sheet` is one of `4x6`, `5x7`,
    /// `a4`, `letter`.
    pub fn sheet_jpeg(
        &self,
        sheet: &str,
        dpi: u32,
        cut_marks: bool,
        quality: u8,
    ) -> Result<Vec<u8>, SessionError> {
        let (photo, layout) = self.sheet_layout(sheet)?;
        let rendered = frame_sheet::render_sheet(photo, &layout, dpi, cut_marks);
        frame_core::io::encode_jpeg(&rendered, quality).map_err(err("sheet jpeg"))
    }

    /// How many photos a sheet would hold, without rendering it — so the
    /// UI can say "8 per 4x6" before the user commits.
    pub fn sheet_info(&self, sheet: &str) -> Result<String, SessionError> {
        let (_, layout) = self.sheet_layout(sheet)?;
        let dto = SheetDto {
            cols: layout.cols,
            rows: layout.rows,
            count: layout.count(),
            sheet_w_mm: layout.sheet_w_mm,
            sheet_h_mm: layout.sheet_h_mm,
        };
        serde_json::to_string(&dto).map_err(err("sheet info"))
    }

    fn sheet_layout(&self, sheet: &str) -> Result<(&Frame, frame_sheet::Layout), SessionError> {
        let output = self.output_ref()?;
        let spec = self
            .spec
            .as_ref()
            .ok_or_else(|| SessionError::new("no spec selected"))?;
        let print = spec.print.as_ref().ok_or_else(|| {
            SessionError::new(format!(
                "{} is a digital-only spec with no print size, so it cannot be tiled onto a sheet",
                spec.id
            ))
        })?;
        let (sw, sh) = match sheet {
            "4x6" => frame_sheet::SHEET_4X6IN,
            "5x7" => frame_sheet::SHEET_5X7IN,
            "a4" => frame_sheet::SHEET_A4,
            "letter" => frame_sheet::SHEET_LETTER,
            other => return Err(SessionError::new(format!("unknown sheet {other}"))),
        };
        let layout = frame_sheet::solve_layout(print.width_mm, print.height_mm, sw, sh, 2.0);
        if layout.count() == 0 {
            return Err(SessionError::new(format!(
                "a {:.0}x{:.0}mm photo does not fit on this sheet",
                print.width_mm, print.height_mm
            )));
        }
        Ok((output, layout))
    }

    fn output_ref(&self) -> Result<&Frame, SessionError> {
        self.output
            .as_ref()
            .ok_or_else(|| SessionError::new("no output yet — call solve() first"))
    }
}

/// The crop the detail pass runs over: the head plus enough margin for
/// hair and shoulders, squared off so the model's square input does not
/// distort it, and clamped to the frame.
fn detail_rect(face: &Face, w: u32, h: u32) -> [u32; 4] {
    let fw = face.bbox[2].max(1.0);
    let cx = face.bbox[0] + face.bbox[2] / 2.0;
    let cy = face.bbox[1] + face.bbox[3] * 0.45;
    // 2.4 face widths covers hair well past the crown and the top of the
    // shoulders, which is the whole region a compliance crop keeps.
    let half = fw * 1.2;
    let x0 = (cx - half).floor().max(0.0) as u32;
    let y0 = (cy - half).floor().max(0.0) as u32;
    let x1 = ((cx + half).ceil() as i64).clamp(0, w as i64) as u32;
    let y1 = ((cy + half).ceil() as i64).clamp(0, h as i64) as u32;
    [x0, y0, x1.saturating_sub(x0), y1.saturating_sub(y0)]
}

/// NaN means "use the automatic default" — a sentinel rather than an
/// `Option<f32>` because that crosses the wasm boundary as a plain number.
fn pick(requested: f32, auto: f32) -> f32 {
    if requested.is_finite() {
        requested
    } else {
        auto
    }
}

fn status_str(s: frame_compliance::validate::Status) -> &'static str {
    use frame_compliance::validate::Status;
    match s {
        Status::Pass => "pass",
        Status::Warn => "warn",
        Status::Fail => "fail",
        Status::NotChecked => "not_checked",
    }
}
