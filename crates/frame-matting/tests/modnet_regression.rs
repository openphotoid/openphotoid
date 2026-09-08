//! Model regression test against the checked-in test portrait.
//!
//! Ignored by default (downloads a 25 MB model + runs real inference —
//! not suitable for a no-network CI lane by default). Run explicitly with:
//!   cargo test -p frame-matting --test modnet_regression -- --ignored

use frame_matting::eval::coverage;
use frame_matting::modnet::Modnet;

#[test]
#[ignore = "downloads a model and runs real inference; run with --ignored"]
fn modnet_finds_the_subject_in_the_test_portrait() {
    let frame = frame_core::io::load(std::path::Path::new("../../testdata/portrait-obama.jpg"))
        .expect("test portrait should be present (see testdata/README.md)");

    let mut model = Modnet::load(frame_engine::Ep::Cpu).expect("model should load");
    let matte = model.infer(&frame).expect("inference should succeed");

    // A half-body portrait crop should land comfortably in this band;
    // catches gross regressions (e.g. an all-background or all-foreground
    // matte from a broken preprocessing/postprocessing change) without
    // being so tight it flags normal model-to-model variance.
    let cov = coverage(&matte, 0.5);
    assert!(
        (0.35..0.70).contains(&cov),
        "unexpected foreground coverage: {cov:.3} (expected roughly 0.35-0.70)"
    );
}
