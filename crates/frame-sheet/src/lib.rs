//! Print-sheet layout: tile N compliant photos onto 4x6 in / A4 / 5x7 in
//! sheets with gutters and cut marks, mm-exact at 300/600 DPI.
//!
//! See docs/research/04-compliance-specs.md §F for the reference table
//! this was validated against.

use frame_core::{resize::resize_exact, Frame};
use image::{Rgba, RgbaImage};

pub const SHEET_4X6IN: (f32, f32) = (101.6, 152.4);
pub const SHEET_5X7IN: (f32, f32) = (127.0, 177.8);
pub const SHEET_A4: (f32, f32) = (210.0, 297.0);
pub const SHEET_LETTER: (f32, f32) = (215.9, 279.4);

#[derive(Debug, Clone, Copy)]
pub struct Layout {
    /// Sheet dimensions as actually used (may be swapped from the input
    /// if rotating the sheet yields more photos — individual photos are
    /// never rotated, only the sheet orientation is chosen).
    pub sheet_w_mm: f32,
    pub sheet_h_mm: f32,
    pub photo_w_mm: f32,
    pub photo_h_mm: f32,
    pub cols: u32,
    pub rows: u32,
    pub gutter_mm: f32,
}

impl Layout {
    pub fn count(&self) -> u32 {
        self.cols * self.rows
    }

    /// Total content block size (all photos + internal gutters), used to
    /// center the block on the sheet.
    fn block_size_mm(&self) -> (f32, f32) {
        (
            self.cols as f32 * self.photo_w_mm
                + (self.cols.saturating_sub(1)) as f32 * self.gutter_mm,
            self.rows as f32 * self.photo_h_mm
                + (self.rows.saturating_sub(1)) as f32 * self.gutter_mm,
        )
    }

    fn margins_mm(&self) -> (f32, f32) {
        let (bw, bh) = self.block_size_mm();
        (
            ((self.sheet_w_mm - bw) / 2.0).max(0.0),
            ((self.sheet_h_mm - bh) / 2.0).max(0.0),
        )
    }
}

/// Solve the best-fit grid for tiling `photo_w_mm x photo_h_mm` photos
/// (kept upright) onto a `sheet_w_mm x sheet_h_mm` sheet, trying both
/// sheet orientations and picking whichever fits more photos.
pub fn solve_layout(
    photo_w_mm: f32,
    photo_h_mm: f32,
    sheet_w_mm: f32,
    sheet_h_mm: f32,
    gutter_mm: f32,
) -> Layout {
    let fit = |sw: f32, sh: f32| -> (u32, u32) {
        let cols = ((sw + gutter_mm) / (photo_w_mm + gutter_mm))
            .floor()
            .max(0.0) as u32;
        let rows = ((sh + gutter_mm) / (photo_h_mm + gutter_mm))
            .floor()
            .max(0.0) as u32;
        (cols, rows)
    };
    let (c1, r1) = fit(sheet_w_mm, sheet_h_mm);
    let (c2, r2) = fit(sheet_h_mm, sheet_w_mm);

    if c1 * r1 >= c2 * r2 {
        Layout {
            sheet_w_mm,
            sheet_h_mm,
            photo_w_mm,
            photo_h_mm,
            cols: c1,
            rows: r1,
            gutter_mm,
        }
    } else {
        Layout {
            sheet_w_mm: sheet_h_mm,
            sheet_h_mm: sheet_w_mm,
            photo_w_mm,
            photo_h_mm,
            cols: c2,
            rows: r2,
            gutter_mm,
        }
    }
}

/// Render `layout` tiling `photo` (already spec-compliant, correct px
/// dimensions) onto a sheet raster at `dpi`, with optional cut marks
/// extending into the gutters/margins.
pub fn render_sheet(photo: &Frame, layout: &Layout, dpi: u32, cut_marks: bool) -> Frame {
    let mm_to_px = |mm: f32| (mm / 25.4 * dpi as f32).round() as i64;
    let (sheet_w_px, sheet_h_px) = (mm_to_px(layout.sheet_w_mm), mm_to_px(layout.sheet_h_mm));
    let (margin_x_mm, margin_y_mm) = layout.margins_mm();
    let (margin_x, margin_y) = (mm_to_px(margin_x_mm), mm_to_px(margin_y_mm));
    let gutter_px = mm_to_px(layout.gutter_mm);

    let photo_resized = resize_exact(
        photo,
        mm_to_px(layout.photo_w_mm) as u32,
        mm_to_px(layout.photo_h_mm) as u32,
    )
    .expect("resize to a positive integer size cannot fail");
    let (pw, ph) = (photo_resized.width() as i64, photo_resized.height() as i64);

    let mut canvas = RgbaImage::from_pixel(
        sheet_w_px.max(1) as u32,
        sheet_h_px.max(1) as u32,
        Rgba([255, 255, 255, 255]),
    );

    let mut positions = Vec::with_capacity(layout.count() as usize);
    for row in 0..layout.rows {
        for col in 0..layout.cols {
            let x = margin_x + col as i64 * (pw + gutter_px);
            let y = margin_y + row as i64 * (ph + gutter_px);
            image::imageops::overlay(&mut canvas, &photo_resized.pixels, x, y);
            positions.push((x, y));
        }
    }

    if cut_marks {
        draw_cut_marks(&mut canvas, &positions, pw, ph);
    }

    Frame {
        pixels: canvas,
        dpi: frame_core::Dpi(dpi as f32),
    }
}

/// Small L-shaped marks at each photo's four corners, extending outward
/// into the surrounding gutter/margin.
fn draw_cut_marks(canvas: &mut RgbaImage, positions: &[(i64, i64)], pw: i64, ph: i64) {
    const LEN: i64 = 12;
    const COLOR: Rgba<u8> = Rgba([160, 160, 160, 255]);
    let (cw, ch) = (canvas.width() as i64, canvas.height() as i64);

    fn hline(canvas: &mut RgbaImage, x0: i64, x1: i64, y: i64, cw: i64, ch: i64, color: Rgba<u8>) {
        if y < 0 || y >= ch {
            return;
        }
        for x in x0.max(0)..x1.min(cw) {
            canvas.put_pixel(x as u32, y as u32, color);
        }
    }
    fn vline(canvas: &mut RgbaImage, x: i64, y0: i64, y1: i64, cw: i64, ch: i64, color: Rgba<u8>) {
        if x < 0 || x >= cw {
            return;
        }
        for y in y0.max(0)..y1.min(ch) {
            canvas.put_pixel(x as u32, y as u32, color);
        }
    }

    for &(x, y) in positions {
        let corners = [(x, y), (x + pw, y), (x, y + ph), (x + pw, y + ph)];
        for (cx, cy) in corners {
            let dx = if cx == x { -LEN } else { LEN };
            let dy = if cy == y { -LEN } else { LEN };
            hline(canvas, cx.min(cx + dx), cx.max(cx + dx), cy, cw, ch, COLOR);
            vline(canvas, cx, cy.min(cy + dy), cy.max(cy + dy), cw, ch, COLOR);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Borderless (gutter=0) counts are exact arithmetic, independently
    /// verifiable against docs/research/04-compliance-specs.md §F.
    #[test]
    fn borderless_2x2in_on_4x6in_matches_research_table() {
        let (sw, sh) = SHEET_4X6IN;
        let l = solve_layout(50.8, 50.8, sw, sh, 0.0);
        assert_eq!(
            l.count(),
            6,
            "2 cols x 3 rows borderless per research table"
        );
    }

    #[test]
    fn borderless_35x45mm_on_4x6in_picks_rotated_sheet_for_8() {
        let (sw, sh) = SHEET_4X6IN;
        let l = solve_layout(35.0, 45.0, sw, sh, 0.0);
        assert_eq!(
            l.count(),
            8,
            "4 cols x 2 rows on the rotated 6x4in sheet per research table"
        );
        assert_eq!(l.cols, 4);
        assert_eq!(l.rows, 2);
    }

    #[test]
    fn gutter_never_increases_count_vs_borderless() {
        let (sw, sh) = SHEET_4X6IN;
        let borderless = solve_layout(35.0, 45.0, sw, sh, 0.0).count();
        let gutter = solve_layout(35.0, 45.0, sw, sh, 3.0).count();
        assert!(gutter <= borderless);
    }

    #[test]
    fn layout_fits_within_sheet_bounds() {
        let (sw, sh) = SHEET_A4;
        let l = solve_layout(35.0, 45.0, sw, sh, 3.0);
        let (bw, bh) = l.block_size_mm();
        assert!(
            bw <= l.sheet_w_mm + 1e-3,
            "block width {bw} exceeds sheet {}",
            l.sheet_w_mm
        );
        assert!(
            bh <= l.sheet_h_mm + 1e-3,
            "block height {bh} exceeds sheet {}",
            l.sheet_h_mm
        );
        assert!(l.count() > 0);
    }

    #[test]
    fn oversized_photo_yields_zero_count_not_a_panic() {
        let l = solve_layout(500.0, 500.0, 101.6, 152.4, 3.0);
        assert_eq!(l.count(), 0);
    }

    #[test]
    fn render_sheet_produces_correct_pixel_dimensions() {
        let photo = Frame::new(RgbaImage::from_pixel(600, 600, Rgba([200, 0, 0, 255])));
        let layout = solve_layout(50.8, 50.8, 101.6, 152.4, 0.0);
        let sheet = render_sheet(&photo, &layout, 300, true);
        // 4in x 6in @ 300dpi = 1200 x 1800.
        assert_eq!((sheet.width(), sheet.height()), (1200, 1800));
    }

    #[test]
    fn render_sheet_places_photos_without_overlap() {
        // A distinctive color per photo isn't needed — just verify that
        // the count of "photo-colored" pixels roughly matches
        // count * photo_area, which would be lower than expected if tiles
        // overlapped and higher (impossible) if something double-painted.
        let photo = Frame::new(RgbaImage::from_pixel(400, 400, Rgba([10, 20, 30, 255])));
        let layout = solve_layout(35.0, 45.0, 101.6, 152.4, 2.0);
        let sheet = render_sheet(&photo, &layout, 300, false);
        let photo_pixels = sheet
            .pixels
            .pixels()
            .filter(|p| p[0] == 10 && p[1] == 20 && p[2] == 30)
            .count();
        let expected_per_photo = {
            let mm_to_px = |mm: f32| (mm / 25.4 * 300.0).round() as u32;
            (mm_to_px(35.0) * mm_to_px(45.0)) as usize
        };
        let expected_total = expected_per_photo * layout.count() as usize;
        assert_eq!(
            photo_pixels, expected_total,
            "overlap or gap would change this count"
        );
    }
}
