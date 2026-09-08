//! Formal-wear replacement, drawn parametrically rather than generated.
//!
//! "AI 换正装" is a paid feature in every competitor that has it, and the
//! research names the underlying pain plainly: someone needs a photo in a
//! jacket for a visa, a job application or an exam registration, and does
//! not own one. That is a real need and it is the one premium feature in
//! the set that a browser genuinely cannot solve with a model — a
//! garment-aware diffusion pass is hundreds of megabytes and tens of
//! seconds, and it invents a body.
//!
//! What it *can* do is what the open-source prior art in this category
//! does: place a garment whose geometry is derived from the face
//! landmarks. That is what this module is. It is honest about being a
//! template — the UI says so — and it is free, offline, instant, and
//! deterministic, which the generative version would not be.
//!
//! Geometry is expressed in units of the detected face width, anchored at
//! the chin, so it tracks the subject rather than the image size.

use frame_core::Frame;
use frame_face::Face;
use frame_matting::Matte;

/// Which garment to draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    /// Jacket, shirt, tie — the safest read for visa and exam photos.
    SuitTie,
    /// Jacket over an open-collar shirt.
    SuitOpen,
    /// Collarless blouse / knit top.
    Blouse,
    /// Shirt only, no jacket.
    Shirt,
}

/// Garment colours. Defaults are a charcoal jacket, white shirt, navy tie
/// — deliberately conventional, because the whole point of this feature is
/// to look unremarkable to whoever is checking the document.
#[derive(Debug, Clone, Copy)]
pub struct GarmentParams {
    pub style: Style,
    pub jacket: [u8; 3],
    pub shirt: [u8; 3],
    pub tie: [u8; 3],
}

impl Default for GarmentParams {
    fn default() -> Self {
        GarmentParams {
            style: Style::SuitTie,
            jacket: [38, 41, 48],
            shirt: [244, 246, 249],
            tie: [30, 52, 92],
        }
    }
}

/// Draw `params`' garment onto `frame`, positioned from `face`.
///
/// `matte` is used only as a keep-out: wherever the subject's own alpha is
/// solid *above* the chin line, the garment is not drawn. That is what
/// stops a collar being painted over hair that falls past the jaw, which
/// is the single most obvious way template garments give themselves away.
pub fn apply(frame: &Frame, matte: &Matte, face: &Face, params: GarmentParams) -> Frame {
    let (w, h) = (frame.width(), frame.height());
    let fw = face.bbox[2].max(1.0);
    let cx = face.bbox[0] + face.bbox[2] / 2.0;
    // Chin: the face box bottom sits near it; the mouth landmark is the
    // reliable anchor, so derive from the mouth-to-box-bottom span.
    let chin_y = face.bbox[1] + face.bbox[3];
    let roll = face.roll_deg().to_radians();

    let mut out = frame.pixels.clone();
    for y in 0..h {
        for x in 0..w {
            // Into face-relative units, de-rotated by the head's roll so a
            // slightly tilted subject gets a slightly tilted collar rather
            // than a level one that reads as pasted on.
            let px = (x as f32 + 0.5 - cx) / fw;
            let py = (y as f32 + 0.5 - chin_y) / fw;
            let (u, v) = (
                px * roll.cos() + py * roll.sin(),
                -px * roll.sin() + py * roll.cos(),
            );

            let cover = coverage(u, v, fw);
            if cover <= 0.0 {
                continue;
            }
            let guard = head_guard(matte, x, y, w, u, v);
            let a = cover * (1.0 - guard);
            if a <= 0.0 {
                continue;
            }

            let color = shade(u, v, params);
            let dst = out.get_pixel_mut(x, y);
            for c in 0..3 {
                dst[c] = (dst[c] as f32 * (1.0 - a) + color[c] as f32 * a).round() as u8;
            }
            dst[3] = 255;
        }
    }

    Frame {
        pixels: out,
        dpi: frame.dpi,
    }
}

/// Neck half-width, in face widths.
const NECK_HALF: f32 = 0.30;
/// Shoulder point, in face widths from the centre.
const SHOULDER_HALF: f32 = 1.45;
/// Where the garment meets the neck, below the chin.
const COLLAR_Y: f32 = 0.42;
/// Where the garment's top edge sits at the shoulder point.
const SHOULDER_Y: f32 = 0.78;

/// How far below the chin the garment's top edge sits at horizontal
/// position `u`, in face widths. Smoothstep between the neck and the
/// shoulder point gives the shoulder line its slope; a straight
/// interpolation reads as a triangle rather than a body.
fn garment_top(u: f32) -> f32 {
    let t = ((u.abs().clamp(NECK_HALF, SHOULDER_HALF) - NECK_HALF) / (SHOULDER_HALF - NECK_HALF))
        .clamp(0.0, 1.0);
    COLLAR_Y + (SHOULDER_Y - COLLAR_Y) * (t * t * (3.0 - 2.0 * t))
}

/// How much of the garment covers this point, 0.0 to 1.0, antialiased
/// across roughly a pixel at the top edge.
fn coverage(u: f32, v: f32, fw: f32) -> f32 {
    // Feather over ~1.2 source pixels, expressed in face-width units.
    let feather = (1.2 / fw).max(0.004);
    ((v - garment_top(u)) / feather).clamp(0.0, 1.0)
}

/// How far below the garment's own top edge the keep-out reaches.
const GUARD_BAND: f32 = 0.45;

/// 1.0 where the garment must not be drawn because the subject is already
/// there and is not clothing.
///
/// Telling hair from a shirt needs a segmentation model this build does
/// not have, so the discriminator is geometric, and it is the garment's
/// own shoulder line: solid subject alpha that sits *above* where a
/// shoulder would be, and outside the neck column, is hair — a real
/// shoulder cannot be there. Inside the neck column the same rule keeps
/// the neck itself in front of the collar, which is also what a
/// photograph of a person in a jacket looks like.
///
/// Below the band the garment always wins, because that is the torso and
/// replacing what the person is wearing is the entire point. Hair longer
/// than roughly a face width past the chin therefore gets overpainted at
/// its ends; that is the deliberate trade, and it is why this is offered
/// as a template rather than sold as a garment-aware model.
fn head_guard(matte: &Matte, x: u32, y: u32, w: u32, u: f32, v: f32) -> f32 {
    let depth = v - garment_top(u);
    if depth >= GUARD_BAND {
        return 0.0;
    }
    let i = (y * w + x) as usize;
    let alpha = matte.alpha.get(i).copied().unwrap_or(0.0).clamp(0.0, 1.0);
    // Feather over the last 40% of the band, so hair ends fade into the
    // garment instead of being sheared off at a line.
    let fade = ((GUARD_BAND - depth) / (GUARD_BAND * 0.4)).clamp(0.0, 1.0);
    alpha * fade
}

/// Which part of the garment this point belongs to, and its shaded colour.
fn shade(u: f32, v: f32, p: GarmentParams) -> [u8; 3] {
    let below_collar = (v - COLLAR_Y).max(0.0);

    // The shirt V, widening as it descends from the collar.
    let v_half = 0.06 + 0.30 * (below_collar / 0.75).clamp(0.0, 1.6);
    let in_v = u.abs() < v_half;

    let base = match p.style {
        Style::Shirt | Style::Blouse => p.shirt,
        Style::SuitTie | Style::SuitOpen => {
            if in_v {
                p.shirt
            } else {
                p.jacket
            }
        }
    };

    let mut color = base;

    // Tie: a tapered column down the middle of the V.
    if p.style == Style::SuitTie && v > COLLAR_Y + 0.16 {
        let tie_half = 0.055 + 0.055 * ((v - COLLAR_Y - 0.16) / 0.8).clamp(0.0, 1.0);
        if u.abs() < tie_half {
            color = p.tie;
        }
    }

    // Collarless styles get no lapel edge; the others get a darkened seam
    // where jacket meets shirt, which is what gives the flat fill depth.
    if matches!(p.style, Style::SuitTie | Style::SuitOpen) {
        let d = (u.abs() - v_half).abs();
        if d < 0.035 {
            let k = 1.0 - (d / 0.035) * 0.45;
            let shade = (0.62 + 0.38 * k).min(1.0);
            for ch in &mut color {
                *ch = (*ch as f32 * shade).round() as u8;
            }
        }
    }

    // A gentle top-to-bottom falloff: fabric nearest the light is the
    // fabric nearest the face.
    let fall = 1.0 - (below_collar * 0.10).clamp(0.0, 0.16);
    for ch in &mut color {
        *ch = (*ch as f32 * fall).round().clamp(0.0, 255.0) as u8;
    }
    color
}

#[cfg(test)]
mod tests {
    use super::*;
    use frame_core::RgbaImage;
    use image::Rgba;

    fn face(cx: f32, chin: f32, fw: f32) -> Face {
        Face {
            score: 0.99,
            bbox: [cx - fw / 2.0, chin - fw * 1.2, fw, fw * 1.2],
            landmarks: [
                [cx - fw * 0.18, chin - fw * 0.75],
                [cx + fw * 0.18, chin - fw * 0.75],
                [cx, chin - fw * 0.45],
                [cx - fw * 0.12, chin - fw * 0.25],
                [cx + fw * 0.12, chin - fw * 0.25],
            ],
        }
    }

    fn solid_matte(w: u32, h: u32, a: f32) -> Matte {
        Matte {
            width: w,
            height: h,
            alpha: vec![a; (w * h) as usize],
        }
    }

    fn scene(w: u32, h: u32) -> Frame {
        Frame::new(RgbaImage::from_pixel(w, h, Rgba([200, 180, 170, 255])))
    }

    #[test]
    fn draws_below_the_collar_and_not_above_the_chin() {
        let (w, h) = (200u32, 260u32);
        let src = scene(w, h);
        let f = face(100.0, 120.0, 60.0);
        let out = apply(&src, &solid_matte(w, h, 1.0), &f, GarmentParams::default());

        // Well above the chin: untouched.
        assert_eq!(out.pixels.get_pixel(100, 40).0, [200, 180, 170, 255]);
        // Well below the shoulder line at the frame edge: garment.
        assert_ne!(out.pixels.get_pixel(10, 230).0, [200, 180, 170, 255]);
    }

    #[test]
    fn head_guard_protects_solid_alpha_just_below_the_chin() {
        let (w, h) = (200u32, 260u32);
        let src = scene(w, h);
        let f = face(100.0, 120.0, 60.0);

        // A point just below the garment's shoulder line and inside the
        // keep-out band: u = -0.6, v = 0.55 face widths from the chin.
        let guarded = apply(&src, &solid_matte(w, h, 1.0), &f, GarmentParams::default());
        let unguarded = apply(&src, &solid_matte(w, h, 0.0), &f, GarmentParams::default());
        let (x, y) = (64u32, 153u32);
        assert_eq!(
            guarded.pixels.get_pixel(x, y).0,
            src.pixels.get_pixel(x, y).0,
            "solid alpha above the shoulder line must block the garment"
        );
        assert_ne!(
            unguarded.pixels.get_pixel(x, y).0,
            src.pixels.get_pixel(x, y).0,
            "with no subject there, the garment should draw"
        );
    }

    #[test]
    fn tie_only_appears_for_the_tie_style() {
        let (w, h) = (200u32, 300u32);
        let src = scene(w, h);
        let f = face(100.0, 120.0, 60.0);
        let m = solid_matte(w, h, 0.0);

        let tied = apply(&src, &m, &f, GarmentParams::default());
        let open = apply(
            &src,
            &m,
            &f,
            GarmentParams {
                style: Style::SuitOpen,
                ..Default::default()
            },
        );
        // Centre of the chest, where a tie would be.
        let (x, y) = (100u32, 220u32);
        assert_ne!(tied.pixels.get_pixel(x, y).0, open.pixels.get_pixel(x, y).0);
    }

    #[test]
    fn shirt_style_has_no_jacket_anywhere() {
        let (w, h) = (200u32, 300u32);
        let src = scene(w, h);
        let f = face(100.0, 120.0, 60.0);
        let out = apply(
            &src,
            &solid_matte(w, h, 0.0),
            &f,
            GarmentParams {
                style: Style::Shirt,
                ..Default::default()
            },
        );
        // Far from the centre line, a suit would be dark; a shirt is light.
        let p = out.pixels.get_pixel(8, 280).0;
        assert!(
            p[0] > 150,
            "shirt style should stay light at the shoulder, got {p:?}"
        );
    }

    #[test]
    fn garment_tracks_the_face_position() {
        let (w, h) = (240u32, 300u32);
        let src = scene(w, h);
        let m = solid_matte(w, h, 0.0);
        let left = apply(&src, &m, &face(70.0, 120.0, 60.0), GarmentParams::default());
        let right = apply(
            &src,
            &m,
            &face(170.0, 120.0, 60.0),
            GarmentParams::default(),
        );
        // The V of the shirt follows the face centre, so the two differ at
        // a point that is on the centre line for one and not the other.
        assert_ne!(
            left.pixels.get_pixel(70, 200).0,
            right.pixels.get_pixel(70, 200).0
        );
    }
}
