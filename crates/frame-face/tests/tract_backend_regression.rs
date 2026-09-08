//! Proves the tract mobile backend (PLAN.md §M9) works for YuNet through
//! the real crate code. Unlike MODNet, the stock YuNet export needs no
//! patching — it loads under tract as-is (verified during the M9
//! feasibility spike), so this is a plain backend swap with no extra prep.
//!
//! Ignored by default (downloads a model, runs real inference). Run with:
//!   cargo test -p frame-face --no-default-features \
//!     --features tract-backend --test tract_backend_regression -- --ignored

#![cfg(feature = "tract-backend")]

use frame_face::YuNet;

#[test]
#[ignore = "downloads a model and runs real inference; run with --ignored"]
fn yunet_via_tract_finds_a_face_in_the_test_portrait() {
    let frame = frame_core::io::load(std::path::Path::new("../../testdata/portrait-obama.jpg"))
        .expect("test portrait should be present (see testdata/README.md)");

    let mut model = YuNet::load(frame_engine::Ep::Cpu).expect("tract model should load");
    let faces = model.detect(&frame).expect("detection should succeed");

    assert!(
        !faces.is_empty(),
        "expected at least one face in the test portrait, found none"
    );
    assert!(
        faces[0].score > 0.7,
        "top detection score too low: {}",
        faces[0].score
    );
}
