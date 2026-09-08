//! End-to-end spike: matting (bg replace to spec color) -> YuNet detect ->
//! spec-driven auto-crop (background-padded) -> re-detect on the FINAL
//! output -> full validation checklist.
//!
//! Usage: crop-spike <input-image> <output-dir> [spec-id]

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let input = args
        .next()
        .expect("usage: crop-spike <input> <outdir> [spec]");
    let outdir = std::path::PathBuf::from(
        args.next()
            .expect("usage: crop-spike <input> <outdir> [spec]"),
    );
    let spec_id = args.next().unwrap_or_else(|| "us-passport".into());
    std::fs::create_dir_all(&outdir)?;

    let specs = frame_compliance::spec::load_all();
    let spec = specs
        .into_iter()
        .find(|s| s.id == spec_id)
        .ok_or_else(|| anyhow::anyhow!("unknown spec {spec_id}"))?;

    let frame = frame_core::io::load(std::path::Path::new(&input))?;
    println!("input: {}x{}", frame.width(), frame.height());

    let mut detector = frame_face::YuNet::load(frame_engine::Ep::Cpu)?;
    let faces = detector.detect(&frame)?;
    anyhow::ensure!(!faces.is_empty(), "no face detected in source");
    let face = &faces[0];
    println!(
        "source face: score {:.3}, roll {:.2}°",
        face.score,
        face.roll_deg()
    );

    // Background replacement BEFORE crop, so out-of-bounds padding can use
    // the spec's background color and the corners are genuinely uniform.
    let mut model = frame_matting::modnet::Modnet::load(frame_engine::Ep::Cpu)?;
    let mut matte = model.infer(&frame)?;
    matte.refine(&frame);
    let bg_replaced = matte.composite_solid_defringed(&frame, spec.background.rgb());

    let adjustment = frame_compliance::CropAdjustment::default_for(&spec);
    let solution = frame_compliance::solve_crop(&bg_replaced, face, &spec, &adjustment);
    println!(
        "crop rect: {:?} (out of bounds: {})",
        solution.rect, solution.out_of_bounds
    );

    let out = frame_compliance::crop::apply_crop_padded(&bg_replaced, &solution, &spec)?;
    let out_path = outdir.join(format!("{}.jpg", spec.id));
    frame_core::io::save_jpeg(&out, &out_path, 95)?;
    println!(
        "wrote {} ({}x{})",
        out_path.display(),
        out.width(),
        out.height()
    );

    // Re-detect on the FINAL output — validates the whole pipeline
    // end-to-end rather than trusting the crop solver's internal math.
    let final_faces = detector.detect(&out)?;
    let report = frame_compliance::validate(&out, &final_faces, &spec);
    println!("\n--- validation report ({}) ---", spec.id);
    for check in &report.checks {
        println!("  [{:?}] {}: {}", check.status, check.name, check.detail);
    }
    println!("overall: {:?}", report.overall());

    Ok(())
}
