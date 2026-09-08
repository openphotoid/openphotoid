//! Visual spike: tile a real compliant photo onto a 4x6in sheet.
//! Usage: sheet-spike <photo> <out.jpg> <photo_w_mm> <photo_h_mm>

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let photo_path = args
        .next()
        .expect("usage: sheet-spike <photo> <out.jpg> <w_mm> <h_mm>");
    let out_path = args
        .next()
        .expect("usage: sheet-spike <photo> <out.jpg> <w_mm> <h_mm>");
    let w_mm: f32 = args.next().unwrap_or_else(|| "50.8".into()).parse()?;
    let h_mm: f32 = args.next().unwrap_or_else(|| "50.8".into()).parse()?;

    let photo = frame_core::io::load(std::path::Path::new(&photo_path))?;
    let (sw, sh) = frame_sheet::SHEET_4X6IN;
    let layout = frame_sheet::solve_layout(w_mm, h_mm, sw, sh, 3.0);
    println!(
        "layout: {}x{} on {:.1}x{:.1}mm sheet ({} photos)",
        layout.cols,
        layout.rows,
        layout.sheet_w_mm,
        layout.sheet_h_mm,
        layout.count()
    );
    let sheet = frame_sheet::render_sheet(&photo, &layout, 300, true);
    frame_core::io::save_jpeg(&sheet, std::path::Path::new(&out_path), 92)?;
    println!("wrote {out_path} ({}x{})", sheet.width(), sheet.height());
    Ok(())
}
