//! Proves the tract mobile backend (PLAN.md §M9) actually works end to
//! end through the real crate code — not just a throwaway spike outside
//! it. Only compiled/run under `--features tract-backend`.
//!
//! Ignored by default (downloads a model, shells out to a Python/onnx
//! prep step, runs real inference). Run explicitly with:
//!   cargo test -p frame-matting --no-default-features \
//!     --features tract-backend --test tract_backend_regression -- --ignored
//!
//! Requires `python3` with the `onnx` package installed (only for this
//! test's one-time model prep — the app itself never shells out to
//! Python; see scripts/patch-modnet-for-tract.py).

#![cfg(feature = "tract-backend")]

use frame_matting::eval::coverage;
use frame_matting::modnet::Modnet;

#[test]
#[ignore = "downloads a model, shells out to a Python prep step, runs real inference; run with --ignored"]
fn modnet_via_tract_finds_the_subject_in_the_test_portrait() {
    let frame = frame_core::io::load(std::path::Path::new("../../testdata/portrait-obama.jpg"))
        .expect("test portrait should be present (see testdata/README.md)");

    let original = frame_engine::ensure_model(&frame_engine::registry::MODNET_PHOTOGRAPHIC)
        .expect("stock model should download");

    let patched = frame_engine::registry::models_dir()
        .join("modnet_photographic_portrait_matting.tract.onnx");
    if !patched.exists() {
        let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repo root should resolve");
        let script = repo_root.join("scripts/patch-modnet-for-tract.py");
        let status = std::process::Command::new("python3")
            .arg(&script)
            .arg(&original)
            .arg(&patched)
            .status()
            .unwrap_or_else(|e| {
                panic!("failed to run {script:?} (need python3 + `pip install onnx`): {e}")
            });
        assert!(status.success(), "patch script failed: {script:?}");
    }

    let mut model =
        Modnet::load_from_path(&patched, frame_engine::Ep::Cpu).expect("tract model should load");
    let matte = model.infer(&frame).expect("inference should succeed");

    // Same plausibility band as the ort-backend regression test
    // (modnet_regression.rs) — this is the same model, just a different
    // inference backend, so the same real-photo output should land here.
    let cov = coverage(&matte, 0.5);
    assert!(
        (0.35..0.70).contains(&cov),
        "unexpected foreground coverage: {cov:.3} (expected roughly 0.35-0.70)"
    );
}
