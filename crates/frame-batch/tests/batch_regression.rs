//! Real end-to-end batch test: N items (same test portrait, different
//! specs) through the actual executor, verifying parallel execution
//! produces N correct results and a readable CSV.
//!
//! Ignored by default (downloads models + runs real inference). Run with:
//!   cargo test -p frame-batch --test batch_regression -- --ignored

use frame_batch::{BatchExecutor, BatchItem};

#[test]
#[ignore = "downloads models and runs real inference; run with --ignored"]
fn batch_processes_multiple_specs_in_parallel() {
    let portrait = std::path::Path::new("../../testdata/portrait-obama.jpg")
        .canonicalize()
        .expect("test portrait should be present (see testdata/README.md)");

    let items: Vec<BatchItem> = [
        "us-passport",
        "schengen-passport",
        "china-passport",
        "uk-passport",
    ]
    .iter()
    .map(|spec_id| BatchItem {
        input_path: portrait.clone(),
        spec_id: spec_id.to_string(),
    })
    .collect();

    let executor = BatchExecutor::new(frame_engine::Ep::Cpu).expect("executor should load");
    let outdir = tempfile::tempdir().unwrap();

    let t0 = std::time::Instant::now();
    let results = executor.run(&items, outdir.path());
    let elapsed = t0.elapsed();
    println!("batch of {} items: {} ms", items.len(), elapsed.as_millis());

    assert_eq!(results.len(), items.len());
    for r in &results {
        assert!(r.error.is_none(), "{}: {:?}", r.spec_id, r.error);
        assert!(
            r.output_path.as_ref().is_some_and(|p| p.exists()),
            "{}: no output file",
            r.spec_id
        );
        let report = r
            .report
            .as_ref()
            .expect("successful item should have a report");
        assert_ne!(
            report.overall(),
            frame_compliance::validate::Status::Fail,
            "{}: {:#?}",
            r.spec_id,
            report.checks
        );
    }

    let csv_path = outdir.path().join("report.csv");
    frame_batch::write_csv_report(&results, &csv_path).unwrap();
    let csv = std::fs::read_to_string(&csv_path).unwrap();
    assert_eq!(
        csv.lines().count(),
        items.len() + 1,
        "header + one row per item"
    );
}

#[test]
#[ignore = "downloads models and runs real inference; run with --ignored"]
fn batch_reports_a_clean_error_for_unknown_spec() {
    let portrait = std::path::Path::new("../../testdata/portrait-obama.jpg")
        .canonicalize()
        .unwrap();
    let items = vec![BatchItem {
        input_path: portrait,
        spec_id: "not-a-real-spec".into(),
    }];
    let executor = BatchExecutor::new(frame_engine::Ep::Cpu).unwrap();
    let outdir = tempfile::tempdir().unwrap();
    let results = executor.run(&items, outdir.path());
    assert_eq!(results.len(), 1);
    assert!(results[0]
        .error
        .as_deref()
        .unwrap()
        .contains("unknown spec"));
}

#[test]
#[ignore = "downloads models and runs real inference; run with --ignored"]
fn single_item_timing_baseline() {
    let portrait = std::path::Path::new("../../testdata/portrait-obama.jpg")
        .canonicalize()
        .unwrap();
    let items = vec![BatchItem {
        input_path: portrait,
        spec_id: "us-passport".into(),
    }];
    let executor = BatchExecutor::new(frame_engine::Ep::Cpu).unwrap();
    let outdir = tempfile::tempdir().unwrap();
    let t0 = std::time::Instant::now();
    let results = executor.run(&items, outdir.path());
    println!("single item: {} ms", t0.elapsed().as_millis());
    assert!(results[0].error.is_none());
}

#[test]
#[ignore = "downloads models (BatchExecutor::new loads them eagerly); run with --ignored"]
fn cancelled_token_skips_every_item_and_writes_nothing() {
    use frame_batch::CancellationToken;

    let portrait = std::path::Path::new("../../testdata/portrait-obama.jpg")
        .canonicalize()
        .unwrap();
    let items: Vec<BatchItem> = (0..3)
        .map(|_| BatchItem {
            input_path: portrait.clone(),
            spec_id: "us-passport".into(),
        })
        .collect();

    let executor = BatchExecutor::new(frame_engine::Ep::Cpu).unwrap();
    let outdir = tempfile::tempdir().unwrap();
    let token = CancellationToken::new();
    token.cancel();

    let results = executor.run_cancellable(&items, outdir.path(), &token);
    assert_eq!(results.len(), 3);
    for r in &results {
        assert_eq!(r.error.as_deref(), Some("cancelled"));
        assert!(r.output_path.is_none());
        assert!(r.report.is_none());
    }
    assert_eq!(
        std::fs::read_dir(outdir.path()).unwrap().count(),
        0,
        "a pre-cancelled run must not write any output files"
    );
}
