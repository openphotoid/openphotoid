//! Model registry + download-on-first-run with SHA-256 pinning.
//!
//! Only commercial-clean weights are listed (PLAN.md §4). Every entry must
//! also appear in MODELS.md with license + provenance.

use std::io::{Read, Write};
use std::path::PathBuf;

use sha2::{Digest, Sha256};

use crate::{EngineError, Result};

pub struct ModelSpec {
    pub id: &'static str,
    pub filename: &'static str,
    pub url: &'static str,
    /// Lowercase hex SHA-256 of the file. `None` only during first-time
    /// spike bring-up; releases must pin.
    pub sha256: Option<&'static str>,
    pub license: &'static str,
}

/// MODNet photographic portrait matting, Apache-2.0 weights shipped by
/// HivisionIDPhotos (their release mirror). ~25 MB, 512-input, fast on CPU.
pub const MODNET_PHOTOGRAPHIC: ModelSpec = ModelSpec {
    id: "modnet-photographic",
    filename: "modnet_photographic_portrait_matting.onnx",
    url: "https://github.com/Zeyi-Lin/HivisionIDPhotos/releases/download/pretrained-model/modnet_photographic_portrait_matting.onnx",
    sha256: Some("07c308cf0fc7e6e8b2065a12ed7fc07e1de8febb7dc7839d7b7f15dd66584df9"),
    license: "Apache-2.0 (HivisionIDPhotos / MODNet)",
};

/// MODNet, the tract-compatible variant: the same Apache-2.0 weights with
/// its nine `Resize` nodes rewritten to `half_pixel` (provably identical
/// for this decoder) and the input fixed to 1×3×512×512, which is what
/// `frame-matting` feeds it anyway. Produced by
/// `scripts/patch-modnet-for-tract.py` from [`MODNET_PHOTOGRAPHIC`]; it is
/// not downloaded, so `url` is the script and [`ensure_model`] only finds
/// it. The phone apps ship it inside the bundle (PLAN.md §M9).
pub const MODNET_TRACT: ModelSpec = ModelSpec {
    id: "modnet-photographic-tract",
    filename: "modnet_photographic_portrait_matting.tract.onnx",
    url: "scripts/patch-modnet-for-tract.py",
    sha256: Some("a09a06eebfbe75e3c7bf0241c4cc26b2709d55ae5193277c6812ce98b3a0447f"),
    license: "Apache-2.0 (HivisionIDPhotos / MODNet), mechanically patched",
};

/// YuNet face detector, MIT (OpenCV Zoo). ~230 KB, 5 landmarks.
/// media.githubusercontent resolves the git-lfs pointer to real bytes.
pub const YUNET_2023MAR: ModelSpec = ModelSpec {
    id: "yunet-2023mar",
    filename: "face_detection_yunet_2023mar.onnx",
    url: "https://media.githubusercontent.com/media/opencv/opencv_zoo/main/models/face_detection_yunet/face_detection_yunet_2023mar.onnx",
    sha256: Some("8f2383e4dd3cfbb4553ea8718107fc0423210dc964f9f4280604804ed2552fa4"),
    license: "MIT (OpenCV Zoo)",
};

/// BiRefNet-lite, MIT, community ONNX export. ~176 MB fp32; "quality" tier.
pub const BIREFNET_LITE: ModelSpec = ModelSpec {
    id: "birefnet-lite",
    filename: "birefnet_lite.onnx",
    url: "https://huggingface.co/onnx-community/BiRefNet_lite-ONNX/resolve/main/onnx/model.onnx",
    sha256: Some("5600024376f572a557870a5eb0afb1e5961636bef4e1e22132025467d0f03333"),
    license: "MIT (BiRefNet)",
};

/// Model cache directory: `$OPENPHOTOID_MODELS_DIR` override, else the
/// platform data dir (`~/Library/Application Support/OpenPhotoId/models`,
/// `%APPDATA%/OpenPhotoId/models`).
pub fn models_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("OPENPHOTOID_MODELS_DIR") {
        return PathBuf::from(dir);
    }
    dirs::data_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("OpenPhotoId")
        .join("models")
}

/// Return the local path of a model, downloading + verifying it first if
/// missing. Downloads go to a `.part` file and are renamed only after the
/// checksum passes.
pub fn ensure_model(spec: &ModelSpec) -> Result<PathBuf> {
    let dir = models_dir();
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(spec.filename);
    if path.exists() {
        return Ok(path);
    }

    if !spec.url.starts_with("http") {
        // A derived model, produced by a script rather than fetched.
        return Err(EngineError::Download {
            id: spec.id.to_string(),
            msg: format!(
                "{} is not in {} and is not downloadable: produce it with `{}`",
                spec.filename,
                dir.display(),
                spec.url
            ),
        });
    }
    tracing::info!(model = spec.id, url = spec.url, "downloading model");
    let part = dir.join(format!("{}.part", spec.filename));
    let resp = ureq::get(spec.url)
        .timeout(std::time::Duration::from_secs(1800))
        .call()
        .map_err(|e| EngineError::Download {
            id: spec.id.into(),
            msg: e.to_string(),
        })?;

    let mut reader = resp.into_reader();
    let mut file = std::fs::File::create(&part)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 1 << 16];
    loop {
        let n = reader.read(&mut buf).map_err(|e| EngineError::Download {
            id: spec.id.into(),
            msg: e.to_string(),
        })?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        file.write_all(&buf[..n])?;
    }
    file.flush()?;
    drop(file);

    let got = hex::encode(hasher.finalize());
    match spec.sha256 {
        Some(expected) if expected != got => {
            let _ = std::fs::remove_file(&part);
            return Err(EngineError::Checksum {
                id: spec.id.into(),
                expected: expected.into(),
                got,
            });
        }
        Some(_) => {}
        None => {
            tracing::warn!(model = spec.id, sha256 = %got, "unpinned model checksum — pin this in registry.rs");
        }
    }
    std::fs::rename(&part, &path)?;
    Ok(path)
}
