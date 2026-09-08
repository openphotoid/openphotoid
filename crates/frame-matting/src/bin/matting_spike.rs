//! Matting spike/benchmark: run a matting model on a test portrait with the
//! refinement chain, write outputs, print timings.
//!
//! Usage: matting-spike <input-image> <output-dir> [modnet|birefnet]

use std::time::Instant;

enum Model {
    Modnet(frame_matting::modnet::Modnet),
    Birefnet(frame_matting::birefnet::BiRefNetLite),
}

impl Model {
    fn infer(&mut self, f: &frame_core::Frame) -> frame_matting::Result<frame_matting::Matte> {
        match self {
            Model::Modnet(m) => m.infer(f),
            Model::Birefnet(m) => m.infer(f),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let input = args
        .next()
        .expect("usage: matting-spike <input> <outdir> [modnet|birefnet]");
    let outdir = std::path::PathBuf::from(
        args.next()
            .expect("usage: matting-spike <input> <outdir> [modnet|birefnet]"),
    );
    let which = args.next().unwrap_or_else(|| "modnet".into());
    std::fs::create_dir_all(&outdir)?;

    let frame = frame_core::io::load(std::path::Path::new(&input))?;
    println!("input: {}x{} ({which})", frame.width(), frame.height());

    #[allow(unused_mut)]
    let mut eps = vec![frame_engine::Ep::Cpu];
    #[cfg(feature = "coreml")]
    eps.push(frame_engine::Ep::CoreMl);

    for ep in eps {
        let t0 = Instant::now();
        let mut model = match which.as_str() {
            "modnet" => Model::Modnet(frame_matting::modnet::Modnet::load(ep)?),
            "birefnet" => {
                // Low-memory hosts: OPENPHOTOID_THREADS=1 trims ORT's
                // per-thread arena overhead (see docs/research/05-m0-results.md).
                let threads: usize = std::env::var("OPENPHOTOID_THREADS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(|| std::thread::available_parallelism().map_or(4, |n| n.get()));
                Model::Birefnet(frame_matting::birefnet::BiRefNetLite::load_with_threads(
                    ep, threads,
                )?)
            }
            other => anyhow::bail!("unknown model {other}"),
        };
        let load_ms = t0.elapsed().as_millis();

        let mut matte = model.infer(&frame)?; // warmup
        let mut times = Vec::new();
        for _ in 0..3 {
            let t = Instant::now();
            matte = model.infer(&frame)?;
            times.push(t.elapsed().as_millis());
        }
        println!(
            "[{ep}] load {load_ms} ms; infer warm {:?} ms (median {})",
            times,
            times[times.len() / 2]
        );

        if ep == frame_engine::Ep::Cpu {
            let unrefined_white = matte.composite_solid(&frame, [255, 255, 255]);
            frame_core::io::save_jpeg(
                &unrefined_white,
                &outdir.join(format!("{which}-unrefined-white.jpg")),
                92,
            )?;

            let t_ref = Instant::now();
            matte.refine(&frame);
            println!(
                "refine (default params): {} ms",
                t_ref.elapsed().as_millis()
            );

            let cutout = matte.cutout(&frame);
            frame_core::io::save_png(&cutout, &outdir.join(format!("{which}-cutout.png")))?;

            let t_fg = Instant::now();
            let white = matte.composite_solid_defringed(&frame, [255, 255, 255]);
            println!("defringed composite: {} ms", t_fg.elapsed().as_millis());
            frame_core::io::save_jpeg(&white, &outdir.join(format!("{which}-white.jpg")), 92)?;

            let grad = matte.composite_gradient(&frame, [214, 228, 240], [176, 196, 222]);
            frame_core::io::save_jpeg(&grad, &outdir.join(format!("{which}-gradient.jpg")), 92)?;

            let fg =
                matte.alpha.iter().filter(|a| **a > 0.5).count() as f32 / matte.alpha.len() as f32;
            println!("foreground coverage: {:.1}%", fg * 100.0);
        }
    }
    println!("outputs in {}", outdir.display());
    Ok(())
}
