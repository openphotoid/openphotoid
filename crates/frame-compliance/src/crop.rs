//! The auto-crop solver: face landmarks + spec -> crop rectangle.
//!
//! Head extent from 5 landmarks uses the standard anthropometric prior
//! (same family of heuristic as dpar39/ppp and HivisionIDPhotos): the eye
//! line sits at ~the vertical middle of the head, and eye->mouth distance
//! is ~28% of chin-to-crown height.

use frame_core::{resize::resize_exact, Frame};
use frame_face::Face;
use image::{Rgba, RgbaImage};

use crate::{spec::PhotoSpec, ComplianceError, Result};

const HEAD_PER_EYE_MOUTH: f32 = 3.55; // head height ≈ 3.55 × eye-to-mouth distance

/// Estimate chin-to-crown head height in source pixels from 5 landmarks —
/// shared by the crop solver and the post-crop validator so both use the
/// exact same anthropometric model (no drift between "what we aimed for"
/// and "what we check").
pub fn estimate_head_height(face: &Face) -> f32 {
    let eye = face.eye_midpoint();
    let mouth = face.mouth_midpoint();
    let eye_mouth = ((mouth[0] - eye[0]).powi(2) + (mouth[1] - eye[1]).powi(2)).sqrt();
    eye_mouth * HEAD_PER_EYE_MOUTH
}

#[derive(Debug)]
pub struct CropSolution {
    /// Crop rectangle in source pixels (x, y, w, h) — may exceed the source
    /// bounds, in which case `out_of_bounds` is set and the caller should
    /// pad (M4) or reject.
    pub rect: [f32; 4],
    pub out_of_bounds: bool,
    /// Where the head/eyes actually land in the output, for reporting.
    pub achieved_head_pct: f32,
    pub achieved_eye_from_bottom_pct: f32,
    pub roll_deg: f32,
}

/// Where within the spec's allowed bands to place the crop — the auto
/// solver aims for the middle of every band ([`CropAdjustment::default_for`]);
/// a UI can offer bounded manual adjustment by varying these within
/// [`CropAdjustment::clamp_to`]'s range instead of a free-form crop.
///
/// That range only bounds the *percentage-band* geometry checks in
/// [`crate::validate`] (head height, eye line, centering) — it does not
/// guarantee every check passes. Checks that depend on the source photo's
/// actual pixel detail (`inter_eye_distance`, `blur`) can still fail near
/// the edges of the range: e.g. zooming out toward `head_min_pct` shrinks
/// the face in the fixed-resolution output, and a photo with a
/// borderline-low eye-to-eye pixel distance at the auto default can drop
/// below the minimum once zoomed out further. This is exactly why a
/// caller should re-validate after every adjustment rather than trusting
/// the bounds alone — found via a real photo during manual testing, not
/// a hypothetical.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CropAdjustment {
    /// Target head (chin-to-crown) height as a fraction of crop height.
    pub head_pct: f32,
    /// Target eye-line position, as a fraction of crop height measured
    /// from the bottom.
    pub eye_from_bottom_pct: f32,
    /// Horizontal shift of the crop center away from the eye midpoint, as
    /// a percentage of crop width. Positive shifts the crop rectangle
    /// right (so the face lands further left in the output).
    pub center_offset_pct: f32,
}

impl CropAdjustment {
    /// The original auto-crop behavior: dead center of every allowed band,
    /// zero horizontal offset.
    pub fn default_for(spec: &PhotoSpec) -> Self {
        let f = &spec.face;
        CropAdjustment {
            head_pct: (f.head_min_pct + f.head_max_pct) / 2.0,
            eye_from_bottom_pct: (f.eye_min_from_bottom_pct + f.eye_max_from_bottom_pct) / 2.0,
            center_offset_pct: 0.0,
        }
    }

    /// Clamp every field to the range `spec`'s checklist will actually
    /// pass — the min/max a UI slider should offer, and a defense-in-depth
    /// clamp before solving so a caller can never request an
    /// out-of-spec crop even by mistake.
    pub fn clamp_to(mut self, spec: &PhotoSpec) -> Self {
        let f = &spec.face;
        self.head_pct = self.head_pct.clamp(f.head_min_pct, f.head_max_pct);
        self.eye_from_bottom_pct = self
            .eye_from_bottom_pct
            .clamp(f.eye_min_from_bottom_pct, f.eye_max_from_bottom_pct);
        self.center_offset_pct = self
            .center_offset_pct
            .clamp(-f.centering_tolerance_pct, f.centering_tolerance_pct);
        self
    }
}

/// Solve the crop for `face` against `spec`, placing the head/eye-line at
/// `adjustment`'s targets (use [`CropAdjustment::default_for`] for the
/// original fully-automatic behavior).
pub fn solve_crop(
    frame: &Frame,
    face: &Face,
    spec: &PhotoSpec,
    adjustment: &CropAdjustment,
) -> CropSolution {
    let adjustment = adjustment.clamp_to(spec);
    let eye = face.eye_midpoint();
    let head_h = estimate_head_height(face);

    let target_head = adjustment.head_pct;
    let target_eye = adjustment.eye_from_bottom_pct;

    let (out_w, out_h) = spec.output_px();
    let aspect = out_w as f32 / out_h as f32;

    // Crop height so the head occupies the target fraction.
    let crop_h = head_h / target_head;
    let crop_w = crop_h * aspect;
    // Vertical placement: eyes at target height from the bottom.
    let top = eye[1] - (1.0 - target_eye) * crop_h;
    // Horizontal: centered on the eye midpoint, plus the requested offset.
    let left = eye[0] - crop_w / 2.0 + (adjustment.center_offset_pct / 100.0) * crop_w;

    let out_of_bounds = left < 0.0
        || top < 0.0
        || left + crop_w > frame.width() as f32
        || top + crop_h > frame.height() as f32;

    CropSolution {
        rect: [left, top, crop_w, crop_h],
        out_of_bounds,
        achieved_head_pct: head_h / crop_h,
        achieved_eye_from_bottom_pct: (crop_h - (eye[1] - top)) / crop_h,
        roll_deg: face.roll_deg(),
    }
}

/// Apply a solution: crop (clamped to bounds for the spike) and resize to
/// the spec's output pixels.
pub fn apply_crop(frame: &Frame, solution: &CropSolution, spec: &PhotoSpec) -> Result<Frame> {
    let [x, y, w, h] = solution.rect;
    let x = x.max(0.0) as u32;
    let y = y.max(0.0) as u32;
    let w = (w as u32).min(frame.width().saturating_sub(x));
    let h = (h as u32).min(frame.height().saturating_sub(y));
    if w == 0 || h == 0 {
        return Err(ComplianceError::OutOfBounds(format!(
            "degenerate crop {:?}",
            solution.rect
        )));
    }
    let cropped = image::imageops::crop_imm(&frame.pixels, x, y, w, h).to_image();
    let cropped = Frame {
        pixels: cropped,
        dpi: frame.dpi,
    };
    let (out_w, out_h) = spec.output_px();
    Ok(resize_exact(&cropped, out_w, out_h)?)
}

/// Like [`apply_crop`] but for out-of-bounds solutions: extends the canvas
/// with the spec's background color instead of clamping (which would
/// distort the aspect ratio / lose head-height accuracy on tight
/// half-body source photos). Assumes `frame` already has its background
/// replaced to a uniform color matching `spec.background` — the intended
/// pipeline order is matting/bg-replace, THEN compliance crop.
pub fn apply_crop_padded(
    frame: &Frame,
    solution: &CropSolution,
    spec: &PhotoSpec,
) -> Result<Frame> {
    let [x, y, w, h] = solution.rect;
    let (cw, ch) = (w.round() as u32, h.round() as u32);
    if cw == 0 || ch == 0 {
        return Err(ComplianceError::OutOfBounds(format!(
            "degenerate crop {:?}",
            solution.rect
        )));
    }
    let bg = spec.background.rgb();
    let mut canvas = RgbaImage::from_pixel(cw, ch, Rgba([bg[0], bg[1], bg[2], 255]));

    let src_x0 = x.max(0.0);
    let src_y0 = y.max(0.0);
    let src_x1 = (x + w).min(frame.width() as f32);
    let src_y1 = (y + h).min(frame.height() as f32);
    if src_x1 > src_x0 && src_y1 > src_y0 {
        let (sx0, sy0) = (src_x0 as u32, src_y0 as u32);
        let (sw, sh) = ((src_x1 - src_x0) as u32, (src_y1 - src_y0) as u32);
        let sub = image::imageops::crop_imm(&frame.pixels, sx0, sy0, sw, sh).to_image();
        let dst_x = (src_x0 - x).round() as i64;
        let dst_y = (src_y0 - y).round() as i64;
        image::imageops::overlay(&mut canvas, &sub, dst_x, dst_y);
    }

    let padded = Frame {
        pixels: canvas,
        dpi: frame.dpi,
    };
    let (out_w, out_h) = spec.output_px();
    Ok(resize_exact(&padded, out_w, out_h)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbaImage as ImgRgba;

    fn white_frame(w: u32, h: u32) -> Frame {
        Frame::new(ImgRgba::from_pixel(w, h, Rgba([255, 255, 255, 255])))
    }

    #[test]
    fn padded_crop_fills_out_of_bounds_with_background_color() {
        let frame = white_frame(100, 100);
        // Rect extends 20px above the top edge and 20px right of the right
        // edge — a tight source photo scenario.
        let solution = CropSolution {
            rect: [-10.0, -20.0, 120.0, 130.0],
            out_of_bounds: true,
            achieved_head_pct: 0.6,
            achieved_eye_from_bottom_pct: 0.6,
            roll_deg: 0.0,
        };
        let spec_json = r##"{
            "id": "test", "country": "US", "document": "custom",
            "sources": [], "last_verified": "2026-08-01",
            "print": null,
            "digital": {"width_px": 60, "height_px": 65, "min_kb": null, "max_kb": null, "formats": []},
            "background": {"name": "solid-red-test", "hex_render": "#FF0000"},
            "face": {"head_min_pct": 0.5, "head_max_pct": 0.6, "eye_min_from_bottom_pct": 0.5, "eye_max_from_bottom_pct": 0.6}
        }"##;
        let spec = PhotoSpec::from_json(spec_json).unwrap();

        let out = apply_crop_padded(&frame, &solution, &spec).unwrap();
        assert_eq!((out.width(), out.height()), (60, 65));

        // Top-left corner of the padded region (was out-of-bounds -> red).
        let corner = out.pixels.get_pixel(0, 0);
        assert_eq!(
            [corner[0], corner[1], corner[2]],
            [255, 0, 0],
            "padding should use spec background color"
        );

        // Center should be the original white content.
        let center = out.pixels.get_pixel(30, 35);
        assert_eq!(
            [center[0], center[1], center[2]],
            [255, 255, 255],
            "overlap region should keep source pixels"
        );
    }
}
