//! Post-crop compliance validation: re-detects the face on the FINAL
//! exported image (not the crop solver's internal math) so a bug in the
//! solver, or a bad manual crop, gets caught independently. Checks
//! implementable with the current model stack (YuNet 5-landmark detector,
//! no eye-contour/pose/parsing models yet — see PLAN.md §2 for the fuller
//! checklist and which checks await future models).

use frame_core::Frame;
use frame_face::Face;

use crate::crop::estimate_head_height;
use crate::spec::PhotoSpec;

/// ISO/IEC 19794-5 minimum / recommended inter-eye distance in pixels.
const IED_MIN_PX: f32 = 90.0;
const IED_RECOMMENDED_PX: f32 = 120.0;
const BLUR_VARIANCE_MIN: f32 = 80.0;
const LIGHTING_ASYMMETRY_WARN_PCT: f32 = 15.0;
/// Corner sample size as a fraction of the shorter output dimension —
/// proxy for "background region" without needing an alpha mask, valid
/// because compliance crops always leave uniform-background corners.
const BG_SAMPLE_FRACTION: f32 = 0.06;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Status {
    Pass,
    Warn,
    Fail,
    /// No model/data available yet to compute this check.
    NotChecked,
}

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub name: &'static str,
    pub status: Status,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub checks: Vec<CheckResult>,
}

impl ValidationReport {
    /// Worst status across all checks that were actually computed
    /// (`NotChecked` never fails a report on its own).
    pub fn overall(&self) -> Status {
        self.checks
            .iter()
            .map(|c| c.status)
            .filter(|s| *s != Status::NotChecked)
            .max()
            .unwrap_or(Status::NotChecked)
    }

    pub fn passed(&self) -> bool {
        self.overall() != Status::Fail
    }
}

/// Validate a final (already cropped/composited/resized) output image
/// against `spec`. `faces` is the result of re-running detection on
/// `output` — passed in rather than re-detecting internally so callers
/// that already have a detector session don't pay for a second one.
pub fn validate(output: &Frame, faces: &[Face], spec: &PhotoSpec) -> ValidationReport {
    let mut checks = Vec::new();

    checks.push(face_count_check(faces));

    let Some(face) = faces.first() else {
        // Every geometry check depends on a face; report them NotChecked
        // rather than silently omitting them so the checklist shape stays
        // stable regardless of detection outcome.
        for name in [
            "head_height",
            "eye_line",
            "centering",
            "roll",
            "inter_eye_distance",
            "blur",
            "lighting_symmetry",
        ] {
            checks.push(CheckResult {
                name,
                status: Status::NotChecked,
                detail: "no face detected".into(),
            });
        }
        checks.push(background_uniformity_check(output, None, spec));
        checks.push(resolution_check(output, spec));
        checks.push(file_size_check(output, spec));
        return ValidationReport { checks };
    };

    checks.push(head_height_check(output, face, spec));
    checks.push(eye_line_check(output, face, spec));
    checks.push(centering_check(output, face, spec));
    checks.push(roll_check(face, spec));
    checks.push(inter_eye_distance_check(output, face, spec));
    checks.push(blur_check(output, face));
    checks.push(lighting_symmetry_check(output, face));
    checks.push(background_uniformity_check(output, Some(face), spec));
    checks.push(resolution_check(output, spec));
    checks.push(file_size_check(output, spec));

    ValidationReport { checks }
}

fn face_count_check(faces: &[Face]) -> CheckResult {
    match faces.len() {
        0 => CheckResult {
            name: "face_count",
            status: Status::Fail,
            detail: "no face detected".into(),
        },
        1 => CheckResult {
            name: "face_count",
            status: Status::Pass,
            detail: "1 face".into(),
        },
        n => CheckResult {
            name: "face_count",
            status: Status::Fail,
            detail: format!("{n} faces detected; exactly 1 required"),
        },
    }
}

fn band_status(value: f32, min: f32, max: f32, warn_margin: f32) -> Status {
    if value >= min && value <= max {
        Status::Pass
    } else if value >= min - warn_margin && value <= max + warn_margin {
        Status::Warn
    } else {
        Status::Fail
    }
}

fn head_height_check(output: &Frame, face: &Face, spec: &PhotoSpec) -> CheckResult {
    let pct = estimate_head_height(face) / output.height() as f32;
    let f = &spec.face;
    let status = band_status(pct, f.head_min_pct, f.head_max_pct, 0.03);
    CheckResult {
        name: "head_height",
        status,
        detail: format!(
            "{:.1}% (band {:.0}-{:.0}%)",
            pct * 100.0,
            f.head_min_pct * 100.0,
            f.head_max_pct * 100.0
        ),
    }
}

fn eye_line_check(output: &Frame, face: &Face, spec: &PhotoSpec) -> CheckResult {
    let eye_y = face.eye_midpoint()[1];
    let pct_from_bottom = (output.height() as f32 - eye_y) / output.height() as f32;
    let f = &spec.face;
    let status = band_status(
        pct_from_bottom,
        f.eye_min_from_bottom_pct,
        f.eye_max_from_bottom_pct,
        0.03,
    );
    CheckResult {
        name: "eye_line",
        status,
        detail: format!(
            "{:.1}% from bottom (band {:.0}-{:.0}%)",
            pct_from_bottom * 100.0,
            f.eye_min_from_bottom_pct * 100.0,
            f.eye_max_from_bottom_pct * 100.0
        ),
    }
}

fn centering_check(output: &Frame, face: &Face, spec: &PhotoSpec) -> CheckResult {
    let cx = face.eye_midpoint()[0];
    let offset_pct = ((cx - output.width() as f32 / 2.0) / output.width() as f32 * 100.0).abs();
    let tol = spec.face.centering_tolerance_pct;
    let status = if offset_pct <= tol {
        Status::Pass
    } else if offset_pct <= tol * 1.5 {
        Status::Warn
    } else {
        Status::Fail
    };
    CheckResult {
        name: "centering",
        status,
        detail: format!("{offset_pct:.1}% off-center (tolerance {tol:.0}%)"),
    }
}

fn roll_check(face: &Face, spec: &PhotoSpec) -> CheckResult {
    let roll = face.roll_deg().abs();
    let max = spec.face.max_roll_deg;
    let status = if roll <= max {
        Status::Pass
    } else if roll <= max * 1.6 {
        Status::Warn
    } else {
        Status::Fail
    };
    CheckResult {
        name: "roll",
        status,
        detail: format!("{roll:.1}° (max {max:.0}°)"),
    }
}

/// The minimum is a hard rule. The recommendation is only a caution when
/// the document could actually meet it: at 600×600 with the head cropped
/// to the middle of a 50–69% band, the eyes sit about 100 px apart however
/// well the photo was taken, so "recommended 120" cautioned every
/// compliant crop of every document (found on the reference portrait,
/// 11 September 2026). The reachable distance is what this face's eyes
/// span at the head size the automatic crop aims for — the middle of the
/// band, which is where every photo lands unless someone drags the
/// slider; a recommendation above it is stated, not warned.
fn inter_eye_distance_check(output: &Frame, face: &Face, spec: &PhotoSpec) -> CheckResult {
    let e = face.landmarks;
    let ied = ((e[1][0] - e[0][0]).powi(2) + (e[1][1] - e[0][1]).powi(2)).sqrt();
    let head_h = estimate_head_height(face);
    let head_mid = (spec.face.head_min_pct + spec.face.head_max_pct) / 2.0;
    let reachable = if head_h > 0.0 {
        ied / head_h * head_mid * output.height() as f32
    } else {
        ied
    };
    let (status, detail) = if ied < IED_MIN_PX {
        (Status::Fail, format!("{ied:.0}px (min {IED_MIN_PX:.0})"))
    } else if ied >= IED_RECOMMENDED_PX {
        (
            Status::Pass,
            format!("{ied:.0}px (min {IED_MIN_PX:.0}, recommended {IED_RECOMMENDED_PX:.0})"),
        )
    } else if reachable >= IED_RECOMMENDED_PX {
        (
            Status::Warn,
            format!("{ied:.0}px (min {IED_MIN_PX:.0}, recommended {IED_RECOMMENDED_PX:.0})"),
        )
    } else {
        (
            Status::Pass,
            format!("{ied:.0}px (min {IED_MIN_PX:.0}; {IED_RECOMMENDED_PX:.0} is recommended at larger output sizes)"),
        )
    };
    CheckResult {
        name: "inter_eye_distance",
        status,
        detail,
    }
}

/// Laplacian-variance blur estimate on a square region centered on the
/// face (approximates FaceQvec's normalized-crop approach).
fn blur_check(output: &Frame, face: &Face) -> CheckResult {
    let [bx, by, bw, bh] = face.bbox;
    let (w, h) = (output.width() as i64, output.height() as i64);
    let x0 = (bx as i64).clamp(0, w - 1);
    let y0 = (by as i64).clamp(0, h - 1);
    let x1 = ((bx + bw) as i64).clamp(x0 + 1, w);
    let y1 = ((by + bh) as i64).clamp(y0 + 1, h);

    let gray: Vec<f32> = (y0..y1)
        .flat_map(|y| {
            (x0..x1).map(move |x| {
                let p = output.pixels.get_pixel(x as u32, y as u32);
                0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32
            })
        })
        .collect();
    let gw = (x1 - x0) as usize;
    let gh = (y1 - y0) as usize;

    if gw < 3 || gh < 3 {
        return CheckResult {
            name: "blur",
            status: Status::NotChecked,
            detail: "face region too small to measure".into(),
        };
    }

    let mut lap = Vec::with_capacity((gw - 2) * (gh - 2));
    for y in 1..gh - 1 {
        for x in 1..gw - 1 {
            let c = gray[y * gw + x];
            let v = 4.0 * c
                - gray[(y - 1) * gw + x]
                - gray[(y + 1) * gw + x]
                - gray[y * gw + x - 1]
                - gray[y * gw + x + 1];
            lap.push(v);
        }
    }
    let mean = lap.iter().sum::<f32>() / lap.len() as f32;
    let variance = lap.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / lap.len() as f32;

    let status = if variance >= BLUR_VARIANCE_MIN {
        Status::Pass
    } else if variance >= BLUR_VARIANCE_MIN * 0.5 {
        Status::Warn
    } else {
        Status::Fail
    };
    CheckResult {
        name: "blur",
        status,
        detail: format!("Laplacian variance {variance:.0} (min {BLUR_VARIANCE_MIN:.0})"),
    }
}

/// Left/right mean-luminance asymmetry within the face bbox — a proxy for
/// uneven/directional lighting.
fn lighting_symmetry_check(output: &Frame, face: &Face) -> CheckResult {
    let [bx, by, bw, bh] = face.bbox;
    let (w, h) = (output.width() as i64, output.height() as i64);
    let x0 = (bx as i64).clamp(0, w - 1);
    let y0 = (by as i64).clamp(0, h - 1);
    let x1 = ((bx + bw) as i64).clamp(x0 + 1, w);
    let y1 = ((by + bh) as i64).clamp(y0 + 1, h);
    let mid = x0 + (x1 - x0) / 2;

    if mid <= x0 || mid >= x1 {
        return CheckResult {
            name: "lighting_symmetry",
            status: Status::NotChecked,
            detail: "face region too small to split".into(),
        };
    }

    let mean_luma = |xa: i64, xb: i64| -> f32 {
        let mut sum = 0.0;
        let mut n = 0u32;
        for y in y0..y1 {
            for x in xa..xb {
                let p = output.pixels.get_pixel(x as u32, y as u32);
                sum += 0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32;
                n += 1;
            }
        }
        sum / n.max(1) as f32
    };
    let left = mean_luma(x0, mid);
    let right = mean_luma(mid, x1);
    let asymmetry_pct = ((left - right).abs() / left.max(right).max(1.0)) * 100.0;

    let status = if asymmetry_pct <= LIGHTING_ASYMMETRY_WARN_PCT {
        Status::Pass
    } else if asymmetry_pct <= LIGHTING_ASYMMETRY_WARN_PCT * 1.5 {
        Status::Warn
    } else {
        Status::Fail
    };
    CheckResult {
        name: "lighting_symmetry",
        status,
        detail: format!("{asymmetry_pct:.1}% left/right luminance difference"),
    }
}

/// sRGB -> CIE Lab (D65), for a perceptual delta-E against the spec's
/// target background color.
fn srgb_to_lab(rgb: [u8; 3]) -> [f32; 3] {
    let to_linear = |c: u8| {
        let c = c as f32 / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    let (r, g, b) = (to_linear(rgb[0]), to_linear(rgb[1]), to_linear(rgb[2]));
    let x = r * 0.4124 + g * 0.3576 + b * 0.1805;
    let y = r * 0.2126 + g * 0.7152 + b * 0.0722;
    let z = r * 0.0193 + g * 0.1192 + b * 0.9505;

    // D65 white point.
    let (xn, yn, zn) = (0.95047, 1.0, 1.08883);
    let f = |t: f32| {
        if t > 0.008856 {
            t.cbrt()
        } else {
            7.787 * t + 16.0 / 116.0
        }
    };
    let (fx, fy, fz) = (f(x / xn), f(y / yn), f(z / zn));
    [116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz)]
}

fn delta_e76(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// Samples a background region and checks uniformity + color match.
///
/// Only the strip ABOVE the detected head is trustworthy as "definitely
/// background" for a standard head-and-shoulders ID photo: the bottom two
/// image corners routinely fall on the subject's shoulders/clothing once
/// framed per spec (caught by running this on a real photo — Obama's
/// shoulders reach both bottom corners of the US-passport crop — a
/// synthetic flat-background fixture could never have surfaced this).
/// Without a face position (e.g. no-face fallback) we conservatively use
/// only the top 8% strip, which is background in any compliant photo.
fn background_uniformity_check(
    output: &Frame,
    face: Option<&Face>,
    spec: &PhotoSpec,
) -> CheckResult {
    let (w, h) = (output.width(), output.height());
    let margin = ((w as f32 * BG_SAMPLE_FRACTION) as u32).max(1).min(w / 2);
    // YuNet's bbox top sits near the hairline/forehead boundary, not the
    // true crown — hair (esp. fluffy/grey hair) commonly extends well
    // above it. Back off by 25% of estimated head height so the sample
    // strip doesn't catch hair pixels (caught by running on a real photo:
    // Obama's grey hair inflated stddev even though the mean color stayed
    // near-white — variance, not mean, is what a flat synthetic fixture
    // can't exercise).
    let head_top = match face {
        Some(f) => (f.bbox[1] - estimate_head_height(f) * 0.25).max(0.0) as u32,
        None => (h as f32 * 0.08) as u32,
    };
    let strip_h = head_top.saturating_sub(2); // small extra buffer above that

    if strip_h < 3 || margin * 2 >= w {
        return CheckResult {
            name: "background_uniformity",
            status: Status::NotChecked,
            detail: "no reliable background region above the head to sample".into(),
        };
    }

    let mut samples: Vec<[u8; 3]> = Vec::new();
    for y in 0..strip_h {
        for x in margin..(w - margin) {
            let p = output.pixels.get_pixel(x, y);
            samples.push([p[0], p[1], p[2]]);
        }
    }

    let n = samples.len() as f32;
    let mean = [
        samples.iter().map(|s| s[0] as f32).sum::<f32>() / n,
        samples.iter().map(|s| s[1] as f32).sum::<f32>() / n,
        samples.iter().map(|s| s[2] as f32).sum::<f32>() / n,
    ];
    let stddev = (samples
        .iter()
        .map(|s| (0..3).map(|c| (s[c] as f32 - mean[c]).powi(2)).sum::<f32>())
        .sum::<f32>()
        / (n * 3.0))
        .sqrt();

    let mean_u8 = [mean[0] as u8, mean[1] as u8, mean[2] as u8];
    let delta_e = delta_e76(srgb_to_lab(mean_u8), srgb_to_lab(spec.background.rgb()));

    let status = if stddev <= 8.0 && delta_e <= 6.0 {
        Status::Pass
    } else if stddev <= 16.0 && delta_e <= 12.0 {
        Status::Warn
    } else {
        Status::Fail
    };
    CheckResult {
        name: "background_uniformity",
        status,
        detail: format!(
            "stddev {stddev:.1}, ΔE {delta_e:.1} vs {}",
            spec.background.hex_render
        ),
    }
}

fn resolution_check(output: &Frame, spec: &PhotoSpec) -> CheckResult {
    let (want_w, want_h) = spec.output_px();
    let (got_w, got_h) = (output.width(), output.height());
    let status = if (want_w, want_h) == (got_w, got_h) {
        Status::Pass
    } else {
        Status::Fail
    };
    CheckResult {
        name: "resolution",
        status,
        detail: format!("{got_w}x{got_h} (want {want_w}x{want_h})"),
    }
}

fn file_size_check(output: &Frame, spec: &PhotoSpec) -> CheckResult {
    let Some(d) = &spec.digital else {
        return CheckResult {
            name: "file_size",
            status: Status::NotChecked,
            detail: "spec has no digital file-size constraint".into(),
        };
    };
    let (Some(min_kb), Some(max_kb)) = (d.min_kb.or(Some(0)), d.max_kb) else {
        return CheckResult {
            name: "file_size",
            status: Status::NotChecked,
            detail: "no max KB constraint to target".into(),
        };
    };
    match frame_core::io::encode_jpeg_within(output, min_kb, max_kb) {
        Ok(bytes) => CheckResult {
            name: "file_size",
            status: Status::Pass,
            detail: format!("{} KB fits {min_kb}-{max_kb} KB", bytes.len() / 1024),
        },
        Err(_) => CheckResult {
            name: "file_size",
            status: Status::Fail,
            detail: format!("cannot hit {min_kb}-{max_kb} KB window at any JPEG quality"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    fn spec_fixture() -> PhotoSpec {
        PhotoSpec::from_json(crate::spec::US_PASSPORT_JSON).unwrap()
    }

    fn synthetic_face(cx: f32, cy: f32, head_h: f32, roll_deg: f32) -> Face {
        // eye-mouth distance = head_h / 3.55 (HEAD_PER_EYE_MOUTH); place
        // landmarks symmetrically around (cx, cy) with the given roll.
        let eye_mouth = head_h / 3.55;
        let half_eye_gap = eye_mouth * 0.9; // plausible eye separation
        let theta = roll_deg.to_radians();
        let rot = |dx: f32, dy: f32| {
            [
                cx + dx * theta.cos() - dy * theta.sin(),
                cy + dx * theta.sin() + dy * theta.cos(),
            ]
        };
        Face {
            score: 0.95,
            bbox: [
                cx - half_eye_gap * 1.3,
                cy - eye_mouth,
                half_eye_gap * 2.6,
                eye_mouth * 2.2,
            ],
            landmarks: [
                // Subject's right eye appears on the image-LEFT for a
                // frontal face — matches YuNet's landmark convention.
                rot(-half_eye_gap, 0.0), // right eye
                rot(half_eye_gap, 0.0),  // left eye
                rot(0.0, eye_mouth * 0.5),
                rot(-half_eye_gap * 0.5, eye_mouth),
                rot(half_eye_gap * 0.5, eye_mouth),
            ],
        }
    }

    fn white_output(w: u32, h: u32) -> Frame {
        Frame::new(RgbaImage::from_pixel(w, h, Rgba([255, 255, 255, 255])))
    }

    #[test]
    fn no_face_reports_fail_and_not_checked_geometry() {
        let spec = spec_fixture();
        let output = white_output(600, 600);
        let report = validate(&output, &[], &spec);
        assert_eq!(report.overall(), Status::Fail);
        let face_count = report
            .checks
            .iter()
            .find(|c| c.name == "face_count")
            .unwrap();
        assert_eq!(face_count.status, Status::Fail);
        let head = report
            .checks
            .iter()
            .find(|c| c.name == "head_height")
            .unwrap();
        assert_eq!(head.status, Status::NotChecked);
    }

    #[test]
    fn multiple_faces_fails() {
        let spec = spec_fixture();
        let output = white_output(600, 600);
        let f = synthetic_face(300.0, 330.0, 300.0, 0.0);
        let report = validate(&output, &[f.clone(), f], &spec);
        let fc = report
            .checks
            .iter()
            .find(|c| c.name == "face_count")
            .unwrap();
        assert_eq!(fc.status, Status::Fail);
    }

    #[test]
    fn well_placed_face_passes_geometry_checks() {
        let spec = spec_fixture(); // head 50-69%, eye 56-69% from bottom
        let (w, h) = (600u32, 600u32);
        let output = white_output(w, h);
        // Target mid-band: head 60% of 600 = 360; eye 62% from bottom.
        // synthetic_face's (cx, cy) IS the eye line, so pass eye_y directly.
        let head_h = 0.60 * h as f32;
        let eye_y = h as f32 * (1.0 - 0.62);
        let face = synthetic_face(300.0, eye_y, head_h, 0.0);

        let report = validate(&output, &[face], &spec);
        for name in [
            "face_count",
            "head_height",
            "eye_line",
            "centering",
            "roll",
            "inter_eye_distance",
        ] {
            let c = report.checks.iter().find(|c| c.name == name).unwrap();
            assert_eq!(c.status, Status::Pass, "{name}: {}", c.detail);
        }
    }

    #[test]
    fn inter_eye_recommendation_only_cautions_where_the_document_can_meet_it() {
        // A real face spans about 0.29 head heights between the eyes; the
        // synthetic face is wider, so the eyes are placed by hand.
        fn face_with(cx: f32, cy: f32, head_h: f32, ied: f32) -> Face {
            let mut f = synthetic_face(cx, cy, head_h, 0.0);
            f.landmarks[0][0] = cx - ied / 2.0;
            f.landmarks[1][0] = cx + ied / 2.0;
            f
        }
        let spec = spec_fixture();
        let mid = (spec.face.head_min_pct + spec.face.head_max_pct) / 2.0;

        // 600×600, head at the middle of the band, eyes 100 px apart: the
        // automatic crop can never reach 120 here, so it is a pass that
        // states the recommendation rather than a caution.
        let (w, h) = (600u32, 600u32);
        let output = Frame::new(RgbaImage::from_pixel(w, h, Rgba([255, 255, 255, 255])));
        let face = face_with(300.0, h as f32 * (1.0 - 0.62), h as f32 * mid, 100.0);
        let report = validate(&output, &[face], &spec);
        let c = report
            .checks
            .iter()
            .find(|c| c.name == "inter_eye_distance")
            .unwrap();
        assert_eq!(c.status, Status::Pass, "{}", c.detail);
        assert!(c.detail.contains("larger output"), "{}", c.detail);

        // 1200×1200 with the head well under the band's middle: the crop
        // the app aims for would enlarge it past 120 px, so 100 px here is
        // a caution (the head-height check objects too; only this one is
        // asserted).
        let (w, h) = (1200u32, 1200u32);
        let output = Frame::new(RgbaImage::from_pixel(w, h, Rgba([255, 255, 255, 255])));
        let face = face_with(600.0, h as f32 * (1.0 - 0.62), h as f32 * 0.45, 100.0);
        let report = validate(&output, &[face], &spec);
        let c = report
            .checks
            .iter()
            .find(|c| c.name == "inter_eye_distance")
            .unwrap();
        assert_eq!(c.status, Status::Warn, "{}", c.detail);

        // Below the minimum is a failure at any size.
        let face = face_with(600.0, h as f32 * (1.0 - 0.62), h as f32 * 0.45, 80.0);
        let report = validate(&output, &[face], &spec);
        let c = report
            .checks
            .iter()
            .find(|c| c.name == "inter_eye_distance")
            .unwrap();
        assert_eq!(c.status, Status::Fail, "{}", c.detail);
    }

    #[test]
    fn off_center_face_fails_centering() {
        let spec = spec_fixture();
        let (w, h) = (600u32, 600u32);
        let output = white_output(w, h);
        let head_h = 0.60 * h as f32;
        let eye_y = h as f32 * (1.0 - 0.62);
        // Shifted far right (cx = 500 of 600 -> ~33% off-center).
        let face = synthetic_face(500.0, eye_y, head_h, 0.0);
        let report = validate(&output, &[face], &spec);
        let c = report
            .checks
            .iter()
            .find(|c| c.name == "centering")
            .unwrap();
        assert_eq!(c.status, Status::Fail, "{}", c.detail);
    }

    #[test]
    fn tilted_face_fails_roll() {
        let spec = spec_fixture();
        let (w, h) = (600u32, 600u32);
        let output = white_output(w, h);
        let head_h = 0.60 * h as f32;
        let eye_y = h as f32 * (1.0 - 0.62);
        let face = synthetic_face(300.0, eye_y, head_h, 20.0); // way over the 5° max
        let report = validate(&output, &[face], &spec);
        let c = report.checks.iter().find(|c| c.name == "roll").unwrap();
        assert_eq!(c.status, Status::Fail, "{}", c.detail);
    }

    #[test]
    fn head_too_small_fails_head_height() {
        let spec = spec_fixture(); // band 50-69%
        let (w, h) = (600u32, 600u32);
        let output = white_output(w, h);
        let head_h = 0.20 * h as f32; // way under band
        let eye_y = h as f32 * 0.5;
        let face = synthetic_face(300.0, eye_y, head_h, 0.0);
        let report = validate(&output, &[face], &spec);
        let c = report
            .checks
            .iter()
            .find(|c| c.name == "head_height")
            .unwrap();
        assert_eq!(c.status, Status::Fail, "{}", c.detail);
    }

    #[test]
    fn matching_white_background_passes_uniformity() {
        let spec = spec_fixture(); // white
        let output = white_output(600, 600);
        let c = background_uniformity_check(&output, None, &spec);
        assert_eq!(c.status, Status::Pass, "{}", c.detail);
    }

    #[test]
    fn wrong_background_color_fails_uniformity() {
        let spec = spec_fixture(); // expects white
        let output = Frame::new(RgbaImage::from_pixel(600, 600, Rgba([40, 90, 40, 255])));
        let c = background_uniformity_check(&output, None, &spec);
        assert_eq!(c.status, Status::Fail, "{}", c.detail);
    }

    #[test]
    fn noisy_background_fails_uniformity_stddev() {
        let spec = spec_fixture();
        let mut img = RgbaImage::from_pixel(600, 600, Rgba([255, 255, 255, 255]));
        // Checkerboard the corners to blow up stddev while keeping mean ~white.
        for y in 0..600u32 {
            for x in 0..600u32 {
                if (x / 4 + y / 4) % 2 == 0 && !(40..=560).contains(&x) && !(40..=560).contains(&y)
                {
                    img.put_pixel(x, y, Rgba([120, 120, 120, 255]));
                }
            }
        }
        let output = Frame::new(img);
        let c = background_uniformity_check(&output, None, &spec);
        assert_ne!(c.status, Status::Pass, "{}", c.detail);
    }

    #[test]
    fn resolution_mismatch_fails() {
        let spec = spec_fixture(); // wants 600x600
        let output = white_output(500, 500);
        let c = resolution_check(&output, &spec);
        assert_eq!(c.status, Status::Fail);
    }

    #[test]
    fn resolution_match_passes() {
        let spec = spec_fixture();
        let output = white_output(600, 600);
        let c = resolution_check(&output, &spec);
        assert_eq!(c.status, Status::Pass);
    }

    #[test]
    fn file_size_within_window_passes() {
        let spec = spec_fixture(); // max_kb 240
        let output = white_output(600, 600); // flat white JPEGs compress tiny
        let c = file_size_check(&output, &spec);
        assert_eq!(c.status, Status::Pass, "{}", c.detail);
    }

    #[test]
    fn overall_takes_worst_status_ignoring_not_checked() {
        let report = ValidationReport {
            checks: vec![
                CheckResult {
                    name: "a",
                    status: Status::Pass,
                    detail: "".into(),
                },
                CheckResult {
                    name: "b",
                    status: Status::NotChecked,
                    detail: "".into(),
                },
                CheckResult {
                    name: "c",
                    status: Status::Warn,
                    detail: "".into(),
                },
            ],
        };
        assert_eq!(report.overall(), Status::Warn);
        assert!(report.passed());
    }
}
