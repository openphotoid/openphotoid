//! Exercises the exact command logic the UI calls (base64 in -> base64
//! out, JSON-shaped result), bypassing only the part that genuinely
//! requires a real display (the native Tauri window) — this dev VM has
//! no attached display (`screencapture`/`system_profiler` confirm no
//! CGDisplay), so the click-through has to happen on real hardware.
//! Everything else — decode, matting, crop, validate, re-encode, the
//! base64/JSON glue unique to this crate — runs for real here.
//!
//! Ignored by default (downloads models + runs real inference). Run with:
//!   cargo test -p openphotoid-desktop --test pipeline_regression -- --ignored

use base64::Engine;

#[path = "../src/pipeline.rs"]
mod pipeline;

#[test]
#[ignore = "downloads models and runs real inference; run with --ignored"]
fn process_photo_matches_the_browser_round_trip() {
    let portrait_bytes = std::fs::read("../../../testdata/portrait-obama.jpg")
        .expect("test portrait should be present (see testdata/README.md)");

    // Mirror exactly what the Svelte side produces:
    // `reader.readAsDataURL(file)` -> "data:image/jpeg;base64,<...>" ->
    // `.split(",").pop()` before sending over invoke().
    let data_url = format!(
        "data:image/jpeg;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(&portrait_bytes)
    );
    let image_base64 = data_url.split(',').next_back().unwrap();

    let state = pipeline::AppState::new();
    let result = pipeline::process_photo_inner(&state, image_base64, "us-passport")
        .expect("pipeline should succeed on a real portrait with a face");

    assert!(result
        .output_jpeg_base64
        .starts_with("data:image/jpeg;base64,"));
    let out_b64 = result.output_jpeg_base64.split(',').next_back().unwrap();
    let out_bytes = base64::engine::general_purpose::STANDARD
        .decode(out_b64)
        .expect("output must be valid base64");
    let decoded = frame_core::io::load_from_bytes(&out_bytes).expect("output must be a valid JPEG");
    assert_eq!(
        (decoded.width(), decoded.height()),
        (600, 600),
        "us-passport spec is 600x600"
    );

    assert!(
        !result.checks.is_empty(),
        "should return the full checklist"
    );
    assert!(result
        .checks
        .iter()
        .any(|c| c.name == "face_count" && c.status == "pass"));
    assert_ne!(
        result.overall, "fail",
        "a compliant real photo should not fail overall"
    );

    // Second call reuses the cached models in AppState (lazy-load path).
    let result2 = pipeline::process_photo_inner(&state, image_base64, "schengen-passport")
        .expect("second call with a different spec should also succeed");
    assert!(result2
        .output_jpeg_base64
        .starts_with("data:image/jpeg;base64,"));

    // The auto-crop result should report the dead-center-of-band defaults.
    let a = &result.adjustment;
    assert!((a.head_pct - (a.head_min_pct + a.head_max_pct) / 2.0).abs() < 1e-4);
    assert_eq!(a.center_offset_pct, 0.0);
}

#[test]
#[ignore = "downloads models and runs real inference; run with --ignored"]
fn adjust_crop_re_solves_without_reprocessing_the_source() {
    let portrait_bytes = std::fs::read("../../../testdata/portrait-obama.jpg")
        .expect("test portrait should be present (see testdata/README.md)");
    let data_url = format!(
        "data:image/jpeg;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(&portrait_bytes)
    );
    let image_base64 = data_url.split(',').next_back().unwrap();

    let state = pipeline::AppState::new();
    let auto = pipeline::process_photo_inner(&state, image_base64, "us-passport")
        .expect("initial pipeline run should succeed");

    // Nudge zoom slightly (not to an extreme) — a real slider drag, not
    // the auto default. Bounds only guarantee the percentage-band
    // geometry checks (head height, eye line, centering) stay
    // satisfiable; checks tied to the source photo's actual pixel detail
    // (inter_eye_distance, blur) aren't structurally guaranteed by the
    // bounds alone (see CropAdjustment's doc comment) — a modest nudge
    // like this one is expected to stay compliant on a real portrait,
    // but "in bounds" alone isn't a universal compliance guarantee.
    let bounds = &auto.adjustment;
    let nudged_head = bounds.head_min_pct * 0.25 + bounds.head_pct * 0.75;
    let adjusted =
        pipeline::adjust_crop_inner(&state, nudged_head, bounds.eye_from_bottom_pct, 0.0)
            .expect("adjust_crop should succeed against the cached photo");

    assert!((adjusted.adjustment.head_pct - nudged_head).abs() < 1e-4);
    assert_ne!(
        adjusted.output_jpeg_base64, auto.output_jpeg_base64,
        "a different head-size target should actually change the crop"
    );
    assert_ne!(
        adjusted.overall, "fail",
        "a modest in-bounds adjustment should not fail overall: {:?}",
        adjusted.checks
    );

    // The band's tightest end is a real edge case: zooming out further
    // shrinks the face in the fixed-resolution output, and this test
    // photo's eye-to-eye pixel distance is close enough to the minimum
    // that pushing all the way to head_min_pct genuinely fails
    // inter_eye_distance — found via a real photo during manual testing,
    // not a hypothetical. The mechanism (the value is honored, and
    // validation reflects reality) is what matters here, not that it
    // happens to pass — a caller must look at `overall`/`checks` after
    // every adjustment rather than trusting the bounds alone.
    let extreme =
        pipeline::adjust_crop_inner(&state, bounds.head_min_pct, bounds.eye_from_bottom_pct, 0.0)
            .expect("adjust_crop must not error even when the result fails validation");
    assert!((extreme.adjustment.head_pct - bounds.head_min_pct).abs() < 1e-4);
    assert!(
        !extreme.checks.is_empty(),
        "should still return a full checklist even when overall fails"
    );

    // A caller-supplied value outside the spec's bounds must be clamped,
    // not silently accepted or allowed to produce a broken crop.
    let clamped = pipeline::adjust_crop_inner(&state, 999.0, 999.0, 999.0)
        .expect("out-of-range input should be clamped, not rejected");
    assert_eq!(clamped.adjustment.head_pct, bounds.head_max_pct);
    assert_eq!(clamped.adjustment.eye_from_bottom_pct, bounds.eye_max_pct);
    assert_eq!(
        clamped.adjustment.center_offset_pct,
        bounds.center_offset_max_pct
    );
}

#[test]
fn adjust_crop_without_a_processed_photo_reports_a_clean_error() {
    let state = pipeline::AppState::new();
    let err = pipeline::adjust_crop_inner(&state, 0.6, 0.6, 0.0).unwrap_err();
    assert!(err.contains("no photo processed"));
}

#[test]
fn list_specs_returns_the_full_dataset() {
    // No models needed — exercises the exact call list_specs() makes.
    let specs = pipeline::list_specs();
    assert!(specs.len() >= 20);
    assert!(specs.iter().any(|s| s.id == "us-passport"));
}

#[test]
fn process_photo_reports_unknown_spec_cleanly() {
    let state = pipeline::AppState::new();
    let err = pipeline::process_photo_inner(&state, "aGVsbG8=", "not-a-real-spec").unwrap_err();
    assert!(err.contains("unknown spec"));
}
