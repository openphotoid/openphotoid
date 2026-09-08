//! Photo spec data model — a pragmatic subset of the full JSON schema
//! (docs/research/04-compliance-specs.md); grows as the dataset grows.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhotoSpec {
    pub id: String,
    /// ISO 3166-1 alpha-2
    pub country: String,
    pub document: String,
    #[serde(default)]
    pub sources: Vec<String>,
    #[serde(default)]
    pub last_verified: Option<String>,
    pub print: Option<PrintSpec>,
    pub digital: Option<DigitalSpec>,
    pub background: BackgroundSpec,
    pub face: FaceSpec,
    #[serde(default)]
    pub rules: Rules,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintSpec {
    pub width_mm: f32,
    pub height_mm: f32,
    #[serde(default = "default_dpi")]
    pub dpi_default: u32,
}

fn default_dpi() -> u32 {
    300
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigitalSpec {
    pub width_px: Option<u32>,
    pub height_px: Option<u32>,
    pub min_kb: Option<u32>,
    pub max_kb: Option<u32>,
    #[serde(default)]
    pub formats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundSpec {
    pub name: String,
    /// What the compositor paints, e.g. "#FFFFFF".
    pub hex_render: String,
}

impl BackgroundSpec {
    pub fn rgb(&self) -> [u8; 3] {
        let hex = self.hex_render.trim_start_matches('#');
        let parse = |s: &str| u8::from_str_radix(s, 16).unwrap_or(255);
        if hex.len() == 6 {
            [parse(&hex[0..2]), parse(&hex[2..4]), parse(&hex[4..6])]
        } else {
            [255, 255, 255]
        }
    }
}

/// Face geometry as fractions of the output image (percent bands),
/// following the convention used by US digital specs; mm-band specs are
/// converted to pct at authoring time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceSpec {
    /// Head (chin to crown) height as a fraction of image height.
    pub head_min_pct: f32,
    pub head_max_pct: f32,
    /// Eye midpoint height measured FROM THE BOTTOM as a fraction of
    /// image height.
    pub eye_min_from_bottom_pct: f32,
    pub eye_max_from_bottom_pct: f32,
    #[serde(default = "default_centering")]
    pub centering_tolerance_pct: f32,
    #[serde(default = "default_max_roll")]
    pub max_roll_deg: f32,
}

fn default_centering() -> f32 {
    5.0
}

fn default_max_roll() -> f32 {
    5.0
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Rules {
    #[serde(default)]
    pub glasses: Option<String>,
    #[serde(default)]
    pub editing_forbidden: bool,
    #[serde(default)]
    pub notes: Option<String>,
}

impl PhotoSpec {
    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }

    /// Output pixel dimensions: digital exact px wins; else print mm at DPI.
    pub fn output_px(&self) -> (u32, u32) {
        if let Some(d) = &self.digital {
            if let (Some(w), Some(h)) = (d.width_px, d.height_px) {
                return (w, h);
            }
        }
        if let Some(p) = &self.print {
            let px = |mm: f32| (mm / 25.4 * p.dpi_default as f32).round() as u32;
            return (px(p.width_mm), px(p.height_mm));
        }
        (600, 600)
    }
}

/// Built-in starter specs used directly by name (spike/demo convenience);
/// the full dataset is loaded from disk via [`load_dir`]/[`load_all`].
pub const US_PASSPORT_JSON: &str = include_str!("../../../data/specs/us-passport.json");
pub const SCHENGEN_35X45_JSON: &str = include_str!("../../../data/specs/schengen-passport.json");

/// The `data/specs/` directory relative to the workspace root. Only valid
/// for reading files on the machine that built this crate (`cargo test`,
/// `cargo run` from source) — [`env!("CARGO_MANIFEST_DIR")`] is a
/// compile-time path baked into the binary, so it does not exist on an
/// end user's machine once the app is packaged and distributed. Used by
/// this crate's own dev-time validation test; production spec loading
/// goes through [`load_all`], which embeds the files at compile time
/// instead (see `SPECS_DIR`).
pub fn default_specs_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/specs")
}

/// `data/specs/*.json`, embedded into the binary at compile time so the
/// shipped app never depends on a filesystem path that only existed on
/// the machine that built it — see [`load_all`] and the doc comment on
/// [`default_specs_dir`] for why that matters.
static SPECS_DIR: include_dir::Dir<'_> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/../../data/specs");

/// Load every `*.json` spec in `dir`, returning `(path, spec-or-error)` so
/// a caller can report which file failed rather than aborting the load.
pub fn load_dir(
    dir: &std::path::Path,
) -> std::io::Result<Vec<(std::path::PathBuf, serde_json::Result<PhotoSpec>)>> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let text = std::fs::read_to_string(&path)?;
        out.push((path, PhotoSpec::from_json(&text)));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}

/// Load every valid spec from the compile-time-embedded dataset
/// (`SPECS_DIR`). Specs that fail to parse are skipped — this can't
/// realistically happen outside of a corrupted build, since
/// `every_spec_file_is_well_formed` validates the same files at test
/// time from disk. Use [`load_dir`] directly if you need per-file parse
/// errors.
pub fn load_all() -> Vec<PhotoSpec> {
    // Parsed once per process. The browser's crop solver looked a spec
    // up on every slider move, which re-parsed all twenty-one files each
    // time; the dataset is compile-time data and cannot change under us.
    static ALL: std::sync::OnceLock<Vec<PhotoSpec>> = std::sync::OnceLock::new();
    ALL.get_or_init(|| {
        SPECS_DIR
            .files()
            .filter(|f| f.path().extension().and_then(|e| e.to_str()) == Some("json"))
            .filter_map(|f| f.contents_utf8())
            .filter_map(|text| PhotoSpec::from_json(text).ok())
            .collect()
    })
    .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_specs_parse() {
        let us = PhotoSpec::from_json(US_PASSPORT_JSON).unwrap();
        assert_eq!(us.id, "us-passport");
        assert_eq!(us.output_px(), (600, 600));
        assert_eq!(us.background.rgb(), [255, 255, 255]);

        let eu = PhotoSpec::from_json(SCHENGEN_35X45_JSON).unwrap();
        assert_eq!(eu.output_px(), (413, 531)); // 35x45 mm @ 300 dpi
    }

    /// Every file in data/specs/ must parse, have a unique id matching its
    /// filename stem, sane monotonic bands, and a resolvable output size —
    /// catches the "reversed min/max" or "typo'd field" class of authoring
    /// mistake across the whole dataset at once.
    #[test]
    fn every_spec_file_is_well_formed() {
        let entries = load_dir(&default_specs_dir()).expect("data/specs/ should be readable");
        assert!(
            entries.len() >= 20,
            "expected >=20 spec files, found {}",
            entries.len()
        );

        let mut seen_ids = std::collections::HashSet::new();
        for (path, result) in &entries {
            let stem = path.file_stem().unwrap().to_string_lossy().to_string();
            let spec = result
                .as_ref()
                .unwrap_or_else(|e| panic!("{}: parse error: {e}", path.display()));

            assert_eq!(
                spec.id,
                stem,
                "{}: id must match filename stem",
                path.display()
            );
            assert!(
                seen_ids.insert(spec.id.clone()),
                "duplicate spec id {}",
                spec.id
            );

            let f = &spec.face;
            assert!(
                f.head_min_pct < f.head_max_pct,
                "{}: head band inverted",
                spec.id
            );
            assert!(f.head_max_pct <= 1.0, "{}: head_max_pct > 100%", spec.id);
            assert!(
                f.eye_min_from_bottom_pct < f.eye_max_from_bottom_pct,
                "{}: eye band inverted",
                spec.id
            );
            assert!(!spec.sources.is_empty(), "{}: missing sources", spec.id);
            assert!(
                spec.last_verified.is_some(),
                "{}: missing last_verified",
                spec.id
            );

            let (w, h) = spec.output_px();
            assert!(
                w > 0 && h > 0,
                "{}: degenerate output size {w}x{h}",
                spec.id
            );

            if let Some(d) = &spec.digital {
                if let (Some(min), Some(max)) = (d.min_kb, d.max_kb) {
                    assert!(min < max, "{}: digital KB band inverted", spec.id);
                }
            }
        }
    }

    #[test]
    fn load_all_returns_every_spec() {
        let specs = load_all();
        assert!(specs.len() >= 20);
        assert!(specs.iter().any(|s| s.id == "us-passport"));
        assert!(specs.iter().any(|s| s.id == "china-visa-cova"));
    }
}
