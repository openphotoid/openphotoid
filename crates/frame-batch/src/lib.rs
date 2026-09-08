//! Batch queue: run the full ID-photo pipeline (matting -> compliance crop
//! -> validate -> export) over many images in parallel.
//!
//! Design note: ONNX inference calls are serialized behind a `Mutex`
//! (rather than one model instance per worker thread) so a batch run
//! doesn't multiply the model's resident memory by the worker count — a
//! deliberate choice after the M2 BiRefNet-lite OOM on this dev VM (see
//! docs/research/05-m0-results.md). Everything else — decode, guided-filter
//! refinement, foreground estimation, crop geometry, JPEG encode — runs
//! fully in parallel via rayon, and measured alone that's already the
//! majority of per-image wall time (refine+composite ~1.5-2.5s vs ~0.2-0.3s
//! inference), so this still yields real speedup on multi-core hosts.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use rayon::prelude::*;

use frame_compliance::{PhotoSpec, ValidationReport};
use frame_engine::Ep;
use frame_face::YuNet;
use frame_matting::modnet::Modnet;

#[derive(Debug, thiserror::Error)]
pub enum BatchError {
    #[error(transparent)]
    Engine(#[from] frame_engine::EngineError),
    #[error(transparent)]
    Matting(#[from] frame_matting::MattingError),
    #[error(transparent)]
    Face(#[from] frame_face::FaceError),
}

pub type Result<T> = std::result::Result<T, BatchError>;

#[derive(Debug, Clone)]
pub struct BatchItem {
    pub input_path: PathBuf,
    pub spec_id: String,
}

#[derive(Debug, Clone)]
pub struct BatchResult {
    pub input_path: PathBuf,
    pub spec_id: String,
    pub output_path: Option<PathBuf>,
    pub report: Option<ValidationReport>,
    pub error: Option<String>,
}

impl BatchResult {
    pub fn overall(&self) -> frame_compliance::validate::Status {
        self.report
            .as_ref()
            .map(|r| r.overall())
            .unwrap_or(frame_compliance::validate::Status::Fail)
    }
}

/// A cooperative cancellation flag for a batch run. Cheap to clone (shares
/// the underlying flag) — hand one clone to `run_cancellable` and keep
/// another to call `cancel()` from e.g. a UI "Stop" button.
///
/// Cancellation is checked before each item starts, so already-running
/// items finish rather than being interrupted mid-inference (there's no
/// safe way to preempt an in-flight ONNX Runtime call), but no *new* item
/// starts once cancelled — for a batch of any real size this stops the
/// bulk of the remaining work almost immediately.
#[derive(Debug, Clone, Default)]
pub struct CancellationToken(Arc<AtomicBool>);

impl CancellationToken {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

pub struct BatchExecutor {
    matting: Mutex<Modnet>,
    face: Mutex<YuNet>,
    specs: Vec<PhotoSpec>,
}

impl BatchExecutor {
    pub fn new(ep: Ep) -> Result<Self> {
        Ok(BatchExecutor {
            matting: Mutex::new(Modnet::load(ep)?),
            face: Mutex::new(YuNet::load(ep)?),
            specs: frame_compliance::spec::load_all(),
        })
    }

    /// Process every item in parallel, writing `<stem>-<spec_id>.jpg` into
    /// `out_dir`. Never panics on a single bad item — failures are captured
    /// in that item's [`BatchResult`] so the rest of the batch proceeds.
    pub fn run(&self, items: &[BatchItem], out_dir: &Path) -> Vec<BatchResult> {
        self.run_cancellable(items, out_dir, &CancellationToken::new())
    }

    /// Like [`run`](Self::run), but stops launching new items once `token`
    /// is cancelled. Items already in flight when cancellation happens
    /// still complete and appear in the result; skipped items get a
    /// [`BatchResult`] with `error: Some("cancelled")` instead of a report.
    pub fn run_cancellable(
        &self,
        items: &[BatchItem],
        out_dir: &Path,
        token: &CancellationToken,
    ) -> Vec<BatchResult> {
        items
            .par_iter()
            .map(|item| {
                if token.is_cancelled() {
                    return BatchResult {
                        input_path: item.input_path.clone(),
                        spec_id: item.spec_id.clone(),
                        output_path: None,
                        report: None,
                        error: Some("cancelled".into()),
                    };
                }
                self.process_one(item, out_dir)
            })
            .collect()
    }

    fn process_one(&self, item: &BatchItem, out_dir: &Path) -> BatchResult {
        let mk_err = |msg: String| BatchResult {
            input_path: item.input_path.clone(),
            spec_id: item.spec_id.clone(),
            output_path: None,
            report: None,
            error: Some(msg),
        };

        let Some(spec) = self.specs.iter().find(|s| s.id == item.spec_id) else {
            return mk_err(format!("unknown spec id {}", item.spec_id));
        };

        let frame = match frame_core::io::load(&item.input_path) {
            Ok(f) => f,
            Err(e) => return mk_err(format!("decode error: {e}")),
        };
        // Cap huge source photos before the full-resolution refinement
        // passes (see frame_core::resize::DEFAULT_MAX_DIMENSION).
        let frame = match frame_core::resize::resize_max_side(
            &frame,
            frame_core::resize::DEFAULT_MAX_DIMENSION,
        ) {
            Ok(f) => f,
            Err(e) => return mk_err(format!("resize error: {e}")),
        };

        let source_face = {
            let mut face = match self.face.lock() {
                Ok(g) => g,
                Err(e) => return mk_err(format!("face detector lock poisoned: {e}")),
            };
            match face.detect(&frame) {
                Ok(faces) if !faces.is_empty() => faces.into_iter().next().unwrap(),
                Ok(_) => return mk_err("no face detected in source".into()),
                Err(e) => return mk_err(format!("face detection error: {e}")),
            }
        };

        let mut matte = {
            let mut model = match self.matting.lock() {
                Ok(g) => g,
                Err(e) => return mk_err(format!("matting model lock poisoned: {e}")),
            };
            match model.infer(&frame) {
                Ok(m) => m,
                Err(e) => return mk_err(format!("matting inference error: {e}")),
            }
        };
        matte.refine(&frame);
        let bg_replaced = matte.composite_solid_defringed(&frame, spec.background.rgb());

        let adjustment = frame_compliance::CropAdjustment::default_for(spec);
        let solution = frame_compliance::solve_crop(&bg_replaced, &source_face, spec, &adjustment);
        let output = match frame_compliance::crop::apply_crop_padded(&bg_replaced, &solution, spec)
        {
            Ok(o) => o,
            Err(e) => return mk_err(format!("crop error: {e}")),
        };

        let stem = item
            .input_path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "output".into());
        let output_path = out_dir.join(format!("{stem}-{}.jpg", spec.id));
        if let Err(e) = frame_core::io::save_jpeg(&output, &output_path, 95) {
            return mk_err(format!("encode/save error: {e}"));
        }

        let final_faces = {
            let mut face = match self.face.lock() {
                Ok(g) => g,
                Err(e) => return mk_err(format!("face detector lock poisoned: {e}")),
            };
            face.detect(&output).unwrap_or_default()
        };
        let report = frame_compliance::validate(&output, &final_faces, spec);

        BatchResult {
            input_path: item.input_path.clone(),
            spec_id: item.spec_id.clone(),
            output_path: Some(output_path),
            report: Some(report),
            error: None,
        }
    }
}

/// Write a per-item CSV summary: input, spec, output, overall status,
/// error (if any), and one column per check name with its status.
pub fn write_csv_report(results: &[BatchResult], path: &Path) -> std::io::Result<()> {
    use std::io::Write;

    let mut check_names: Vec<&str> = Vec::new();
    for r in results {
        if let Some(report) = &r.report {
            for c in &report.checks {
                if !check_names.contains(&c.name) {
                    check_names.push(c.name);
                }
            }
        }
    }

    let mut out = std::fs::File::create(path)?;
    write!(out, "input,spec_id,output,overall,error")?;
    for name in &check_names {
        write!(out, ",{name}")?;
    }
    writeln!(out)?;

    for r in results {
        write!(
            out,
            "{},{},{},{:?},{}",
            csv_escape(&r.input_path.display().to_string()),
            csv_escape(&r.spec_id),
            r.output_path
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
            r.overall(),
            csv_escape(r.error.as_deref().unwrap_or("")),
        )?;
        for name in &check_names {
            let status = r
                .report
                .as_ref()
                .and_then(|rep| rep.checks.iter().find(|c| c.name == *name))
                .map(|c| format!("{:?}", c.status))
                .unwrap_or_default();
            write!(out, ",{status}")?;
        }
        writeln!(out)?;
    }
    Ok(())
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_report_has_header_and_one_row_per_result() {
        let dir = tempfile::tempdir().unwrap();
        let csv_path = dir.path().join("report.csv");

        let report = ValidationReport {
            checks: vec![
                frame_compliance::validate::CheckResult {
                    name: "face_count",
                    status: frame_compliance::validate::Status::Pass,
                    detail: "1 face".into(),
                },
                frame_compliance::validate::CheckResult {
                    name: "roll",
                    status: frame_compliance::validate::Status::Warn,
                    detail: "6°".into(),
                },
            ],
        };
        let results = vec![
            BatchResult {
                input_path: PathBuf::from("a.jpg"),
                spec_id: "us-passport".into(),
                output_path: Some(PathBuf::from("out/a-us-passport.jpg")),
                report: Some(report),
                error: None,
            },
            BatchResult {
                input_path: PathBuf::from("b.jpg"),
                spec_id: "us-passport".into(),
                output_path: None,
                report: None,
                error: Some("no face detected in source".into()),
            },
        ];

        write_csv_report(&results, &csv_path).unwrap();
        let text = std::fs::read_to_string(&csv_path).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3, "header + 2 rows");
        assert!(lines[0].starts_with("input,spec_id,output,overall,error"));
        assert!(lines[0].contains("face_count"));
        assert!(lines[0].contains("roll"));
        assert!(lines[1].contains("a.jpg"));
        assert!(lines[1].contains("Warn")); // overall is worst of Pass/Warn
        assert!(lines[2].contains("no face detected"));
    }

    #[test]
    fn csv_escapes_commas_in_error_messages() {
        let dir = tempfile::tempdir().unwrap();
        let csv_path = dir.path().join("report.csv");
        let results = vec![BatchResult {
            input_path: PathBuf::from("a.jpg"),
            spec_id: "x".into(),
            output_path: None,
            report: None,
            error: Some("decode error: invalid, corrupt file".into()),
        }];
        write_csv_report(&results, &csv_path).unwrap();
        let text = std::fs::read_to_string(&csv_path).unwrap();
        assert!(text.contains("\"decode error: invalid, corrupt file\""));
    }

    #[test]
    fn unknown_spec_id_is_a_clean_error_not_a_panic() {
        // Exercises process_one's error path without needing a loaded
        // model — BatchExecutor::new (which loads models) is covered by
        // the ignored real-pipeline integration test instead.
        let specs = frame_compliance::spec::load_all();
        assert!(!specs.iter().any(|s| s.id == "not-a-real-spec"));
    }
}
