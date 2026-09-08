fn main() {
    let portrait = std::path::Path::new("testdata/portrait-obama.jpg")
        .canonicalize()
        .unwrap();
    let items: Vec<frame_batch::BatchItem> = ["us-passport", "schengen-passport", "china-passport"]
        .iter()
        .map(|s| frame_batch::BatchItem {
            input_path: portrait.clone(),
            spec_id: s.to_string(),
        })
        .collect();
    let executor = frame_batch::BatchExecutor::new(frame_engine::Ep::Cpu).unwrap();
    let outdir =
        std::path::PathBuf::from(std::env::var("HOME").unwrap() + "/.cache/openphotoid-batch-out");
    let results = executor.run(&items, &outdir);
    frame_batch::write_csv_report(&results, &outdir.join("report.csv")).unwrap();
}
