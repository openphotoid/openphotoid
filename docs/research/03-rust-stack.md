# Tech-Stack Research Report: Rust Desktop Photo Tool (BG removal + ID-photo + batch)

Research date: 2026-08-01. Target: Windows + macOS desktop; MVP = ONNX background removal, ID-photo compliance (face detect, crop, DPI/print), background replacement, batch processing; later free/premium tiers with Ed25519 license gating. Sibling project (openpdfedit) already chose Tauri 2.

---

## A. GUI Framework Comparison

### Summary table

| Framework | Version (mid-2026) | Rendering model | Large-image canvas story | Shipped image/photo precedents | Binary size | Dev velocity | License |
|---|---|---|---|---|---|---|---|
| **Tauri 2** | 2.x stable (2.0 Oct 2024, mature) | OS webview (WebView2/WKWebView) + Rust backend | Canvas/`<img>` via asset protocol; GPU-composited in Chromium; canvas 2D perf caveats for per-pixel ops | **RMBG (Tekipeps/rmbg — Tauri+ort BG removal, 15 models incl. BiRefNet)**, zhbhun/rmbg, Tauview image viewer | 3–15 MB installer | **Highest** (web ecosystem: React/Svelte, drag-drop, CSS) | MIT/Apache |
| **egui** (eframe) | ~0.3x, very active | Immediate mode, wgpu/glow | Texture upload per image; documented perf problems with very large images (5200×2500 hover lag, issue #797); pan/zoom container not built-in (issue #1811) — rerun solved it with a custom wgpu renderer | **Oculante** (image viewer/editor, ~25 MB static), **rerun.io** (custom `re_renderer` on wgpu) | ~20–25 MB static | Medium (custom widgets DIY, "debug UI that grew up" aesthetics) | MIT/Apache |
| **iced** | 0.14 (Dec 2025, last pre-1.0) | Elm-style retained, wgpu | `image::viewer` widget exists; good but custom shader work needed for before/after compositing | Halloy, Sniffnet, COSMIC DE, Kraken Desktop; hobbyist image viewers | ~15–25 MB | Medium-low (Elm architecture boilerplate, pre-1.0 API churn) | MIT |
| **Slint** | 1.1x | Declarative DSL, own renderer (GPU/software) | Designed for embedded/HMI; image element scaling OK, but no precedent for pro photo canvas | **Czkawka/Krokiet** (duplicate-image finder GUI — chosen over GTK for pure-Rust cross-compile) | small (~10–20 MB) | Medium (DSL learning curve, good tooling) | **GPLv3 OR Royalty-Free (with attribution) OR commercial** — RF license fine for proprietary desktop |
| **Dioxus** | 0.7 (native WGPU renderer "Blitz" new) | React-like; desktop = webview (like Tauri) or experimental Blitz native | Same as Tauri for webview mode; Blitz is WIP with CSS gaps | Few; Blitz explicitly not production-ready | webview mode small | High for React devs, but native renderer immature | MIT/Apache |
| **GPUI** | pre-1.0, now on crates.io | Hybrid immediate/retained, GPU | 120fps capable, Zed-proven text/UI perf; gpui-component gives 60+ widgets; awesome-gpui lists an image viewer | Zed itself; small ecosystem apps | moderate | Low-medium (pre-1.0, docs sparse, API churn tracking Zed) | Apache 2.0 |

### Key findings

- **egui's large-image weakness is documented**: displaying large images can "tank performance" (multi-second hover delays at 5200×2500 in discussion #797); texture transform/crop/zoom requires re-uploading textures (issue #1279); no built-in pan/zoom container (#1811). Oculante and rerun both work around this with custom rendering layers — rerun wrote an entire `re_renderer` crate on wgpu. So egui *can* do it, but the canvas is a from-scratch wgpu project, not free.
- **Tauri canvas caveat**: old issue #4891 (no HW accel for canvas/CSS on some Linux WebKitGTK) is largely a Linux problem; Windows WebView2 (Chromium) and macOS WKWebView have GPU-accelerated canvas/compositing. The proven pattern for image display is **do not push pixels over IPC** — write processed images to disk (or in-memory custom protocol handler) and load via `convertFileSrc`/asset protocol, which bypasses JSON IPC. Tauri 2's IPC also supports raw binary bodies now (no JSON overhead; issue #7127 landed in 2.0). Zoom/pan of a 50 MP photo in Chromium via CSS transforms/`<canvas>`/WebGL is a solved web problem (deep-zoom tiling if needed — the Tauview viewer uses Leaflet tile abstraction for "infinite smooth zooming on any image resolution").
- **Direct precedent for exactly this app exists in Tauri**: `Tekipeps/rmbg` — Tauri + React + TS background remover with 15 models (U2Net, BiRefNet, ISNet…), in-app model manager, **download-on-first-launch (~176 MB U2Net)**. Listed in awesome-tauri. Also `zhbhun/rmbg` (multi-platform, local-only). This is strong validation of Tauri 2 + ort for this exact MVP.
- **iced 0.14** is good and proven (Kraken, COSMIC) but pre-1.0 API churn is real and its widget set for drag-drop batch queues/complex panels is thinner than the web's.
- **Slint** licensing is fine for a proprietary-tier desktop app (royalty-free desktop license w/ attribution), and Czkawka's author chose it for pure-Rust cross-compilation; but no serious photo-editing precedent.
- **GPUI/Dioxus-native**: most interesting tech, weakest maturity story for 2026 shipping.

**A-verdict input**: For an image-heavy consumer-polish app with drag-drop batch UX + before/after sliders, Tauri 2 wins on dev velocity, UI polish, precedent, and **consistency with the sibling PDF editor** (shared bundler config, updater infra, signing pipeline, license-gating code, team knowledge). egui is the fallback if webview canvas perf proves inadequate — Oculante proves it viable, at the cost of building canvas infrastructure.

---

## B. Local ONNX Inference in Rust

| Engine | Version | Model compat (BiRefNet/MODNet/U2-Net) | CPU speed | GPU/NPU | Distribution | License |
|---|---|---|---|---|---|---|
| **ort** (pykeio/ort) | **2.0.0-rc.13**, wraps ONNX Runtime **1.28**; active (990+ commits), used by HuggingFace TEI, Google Magika | **Yes — all of them** (it's real ONNX Runtime; rembg-rs, HivisionIDPhotos ports, RMBG app all run these models on ORT) | Best-in-class CPU (MLAS + threading) | CoreML EP (macOS, ANE/GPU), DirectML EP (Win, any DX12 GPU), CUDA, OpenVINO, QNN | dylib (~15–30 MB per platform); `download-binaries` feature; `load-dynamic` (dlopen, recommended for shipping); static link possible via `ORT_LIB_PATH` but painful with EPs | MIT/Apache-2.0 (ONNX Runtime itself MIT) |
| **tract** (Sonos) | 0.21.x, pure Rust | U2-Net/MODNet: yes (CNNs, well-supported opset). BiRefNet (large transformer hybrid): risky — opset coverage gaps likely | Good but generally slower than ORT on big CNNs (no comparable SIMD kernels/threading tuning); fine for small models | **None** (CPU only) | Perfect — pure Rust static, zero dylibs | MIT/Apache |
| **candle** (HuggingFace) | 0.8.x + `candle-onnx` | candle-onnx op coverage incomplete; ONNX support is a side feature (issue #348 lineage); segmentation demos exist (SAM) but via native ports, not ONNX | OK | Metal + CUDA (no DirectML; Windows GPU = CUDA only) | Static, clean | MIT/Apache |
| **burn** | 0.16/0.17 + `burn-import` | **Converts ONNX → Rust code at build time**; "limited set of ONNX operators", active development; BiRefNet almost certainly hits unsupported ops | Improving (fusion) | wgpu backend = true cross-platform GPU (interesting!), plus CUDA/Metal | Static, clean | MIT/Apache |

### Key data points

- **BiRefNet perf reality**: designed for 1024×1024; ~17 FPS on RTX 4090 (3.45 GB VRAM); **on CPU it is seconds-per-image, sometimes pathologically slow** (upstream issue #26 reports CPU inference that "might go on indefinitely"). BiRefNet-lite variants exist (HivisionIDPhotos ships `birefnet-v1-lite`). Plan: U2-Net (320×320, ~176 MB, sub-second CPU) or MODNet (portrait matting, small, real-time-ish CPU) as default; ISNet (~44M params) as quality mid-tier; BiRefNet(-lite) as "HD" mode, GPU-preferred with time warnings on CPU. INT8 quantization gives ~4–5× smaller and significantly faster CPU inference with marginal quality loss.
- **CoreML EP caveats** (matters for BiRefNet on macOS): dynamic input shapes historically fall back to CPU (`RequireStaticInputShapes`; ORT issue #14212); unsupported ops cause silent partition round-trips (e.g. Pad reflect, issue #28022). **Mitigation: export fixed-shape models** (BG removal has fixed input sizes anyway — 320/512/1024) and test op coverage per model. CoreML EP works from Rust via ort's `coreml` feature.
- **DirectML EP**: works on any DX12 GPU (~all modern GPUs incl. Intel iGPU); ort exposes it; known driver-level wrong-results bugs on Intel Iris Xe (ORT #18652 / DirectML #541) → ship a "GPU acceleration" toggle with CPU fallback and per-image sanity check. Note Microsoft is migrating Windows AI to Windows ML (shared ORT runtime) — future option to shrink installers on Windows 11.
- **ort distribution**: recommended shipping pattern is `load-dynamic` + bundle `onnxruntime.dll` / `libonnxruntime.dylib` in app resources, set `ORT_DYLIB_PATH` at startup (ort docs: "often far less troublesome"). `copy-dylibs` feature helps in dev. DirectML additionally needs `DirectML.dll` bundled; CoreML is an OS framework (nothing to bundle). This is exactly the pattern Tauri resource bundling handles; note Tauri GH issue #11992 — sign/notarize hiccups with `externalBin` sidecars on macOS; bundling as resources (not sidecar) avoids it.
- **face detection for ID-photo**: ready-made ONNX routes on ort — `rust-faces` crate (BlazeFace/MTCNN, ort-based), `face_id` crate (SCRFD detect + ArcFace embed + landmarks/alignment — landmarks are what you need for eye-line/crown-chin compliance geometry), or run YuNet ONNX (tiny, ms-level CPU, OpenCV Zoo) directly in ort. HivisionIDPhotos (Python, 18k+ stars) is the reference algorithm pipeline to port: MODNet/BiRefNet matting + MTCNN/RetinaFace detection + compliance cropping — all ONNX, all portable to ort. (License note: insightface SCRFD weights are non-commercial — see 02-models-licenses.md; prefer YuNet.)

**B-verdict input**: **ort 2.0-rc is the only engine that runs all target models with GPU/NPU acceleration on both OSes.** Its rc-status is the main blemish (rc.13, API churn between rcs; pin exactly). tract is a credible CPU-only fallback for U2-Net/MODNet if dylib-free builds ever become mandatory. candle/burn: not ready for arbitrary ONNX segmentation models. Alternative worth noting: ncnn+Vulkan (Upscayl's engine) gives cross-platform GPU without ORT, but is C++ and model conversion adds friction.

---

## C. Image Processing Crates

| Need | Crate | Version | Notes |
|---|---|---|---|
| Core decode/encode/types | `image` | 0.25.x | JPEG/PNG/WebP/TIFF/BMP + AVIF; de-facto standard; interops with everything |
| Fast decode | `zune-image` / `zune-jpeg` / `zune-png` | zune-jpeg 0.5.13 (active through 2026) | libjpeg-turbo-class speed (±10 ms), AVX2/SSE, fuzz-tested; `image` crate can use zune-jpeg internally |
| Resize (print DPI, thumbnails) | `fast_image_resize` | 5.x | SIMD (SSE4/AVX2/NEON/WASM), Lanczos3, **alpha-premultiply correct**, gamma/sRGB-linear mappers (`create_srgb_mapper`) — use linear-light resize for quality; several× faster than `image` resize |
| Ops (thresholds, morphology for mask cleanup) | `imageproc` | 0.25 | Fine for mask post-processing (erode/dilate/feather via blur) |
| `photon` | — | Skip: WASM-oriented, weaker than the above |
| Color management (ICC, accurate bg colors, print) | `lcms2` (bindings, stable years, production) 6.x; or `qcms` (pure Rust, Firefox's) 0.5 | lcms2 for full print-grade CMM (rendering intents, CMYK-ready later); qcms if pure-Rust preferred; also a new pure-Rust lcms2 reimplementation exists (bit-identical, differential-tested) |
| EXIF (orientation, DPI metadata) | `kamadak-exif` (crate name `exif`) | 0.6 | Pure Rust; reads JPEG/TIFF/PNG/WebP/HEIF/AVIF |
| HEIC decode | `libheif-rs` (libheif 1.17–1.21) | 2.x | **Patent minefield**: HEVC decode implicates Access Advance/MPEG-LA pools; libheif itself LGPL but libde265/x265 GPL + patents; pure-Rust `imazen/heic` decoder is AGPL-3.0/commercial and explicitly disclaims patent grants. **Recommendation: v1 = no in-app HEIC decode.** On macOS use OS CoreGraphics/`sips` (Apple pays the license); on Windows require the user's HEIF+HEVC extension via WIC, or tell users to export JPEG. Revisit later with legal advice |
| Print sheet PDF (4×6 with N ID photos) | `printpdf` | **0.9.1** | MIT, active (fschutt), embeds images via `image`, precise mm layout — ideal for photo-sheet generation with exact DPI |
| License keys | `ed25519-dalek` | 2.x | Standard, audited; offline signature-verified license keys |

Pipeline note: decode (zune/image) → EXIF orient → CMS to sRGB working space (lcms2) → inference pre/post (fast_image_resize to model input; ndarray HWC↔CHW) → mask feather/composite → export with ICC profile + correct DPI in JPEG/PNG headers → printpdf sheet. All stages parallelize with `rayon` for batch mode.

---

## D. Precedents, Packaging, Distribution

**Precedents**
- **Tekipeps/rmbg** (Tauri+ort BG removal, model manager, download-on-first-run) — closest codebase to borrow patterns from; https://github.com/Tekipeps/rmbg
- **HivisionIDPhotos** (Python; MODNet/BiRefNet + MTCNN/RetinaFace ID-photo pipeline) — the algorithm blueprint to port; https://github.com/Zeyi-Lin/HivisionIDPhotos
- **Oculante** (egui image viewer/editor, SIMD edit ops, ~25 MB) — egui-route reference; https://github.com/woelper/oculante
- **rerun.io** (egui + custom wgpu `re_renderer`) — shows the cost of the native-canvas route
- **Czkawka/Krokiet** (Slint) — Slint viability + why-not-GTK
- **Upscayl** (Electron + ncnn-Vulkan sidecar) — proof the Electron+sidecar route ships, and of its ~200 MB+ footprint
- **rembg-rs** crate (ort + U2-Net, sigmoid/threshold/RGBA pipeline) — direct code to borrow

**Packaging plan (Tauri route)**
- Tauri bundler: NSIS/MSI (Win), .app/DMG (mac), out of the box; cargo-bundle is comparatively unmaintained — another point for Tauri even with an egui core (egui-in-Tauri is not a thing; egui would use cargo-bundle/cargo-packager).
- Signing: macOS Developer ID + notarization automated by Tauri build; Windows via Azure Key Vault / EV cert; documented CI recipes (dev.to "Ship Your Tauri v2 App Like a Pro" 1&2). Caveat: `externalBin` sidecar notarization bug #11992 → keep ONNX dylibs as bundled resources, not sidecars.
- Auto-update: `tauri-plugin-updater`, separate Ed25519-style update-signing key (distinct from OS signing) — infra shareable with openpdfedit.
- **Models: download-on-first-run** (RMBG precedent, Windows ML guidance). Installer stays ~10–25 MB (app + ORT dylibs); default model MODNet/ISNet (25–176 MB) fetched with checksum + resume into app-data dir; BiRefNet-HD (300 MB+) optional download. Bundling 300 MB in the installer would blow past DMG/NSIS comfort and update bandwidth.
- Realistic installed footprint: app 10–20 MB + onnxruntime ~20–40 MB + models 44–500 MB (user-selected). Memory: webview ~100–200 MB + image buffers + inference peak (BiRefNet ~3.5 GB GPU / several GB RAM at 1024; U2-Net far less).

---

## E. Verdict Inputs (Rust vs alternatives)

| Option | For | Against |
|---|---|---|
| **Rust (Tauri 2 + ort)** | Single binary + resources; no runtime deps; memory safety across a batch pipeline (the "extremely stable" requirement); ORT CPU inference at full native speed; 10–25 MB installer; 30–50 MB idle RAM; shares infra/skills with openpdfedit; rayon batch parallelism trivially | ort is rc; webview canvas needs the asset-protocol pattern; Rust iteration slower than TS-only |
| Electron + Python sidecar (rembg) | Fastest prototyping; rembg mature | 150–300 MB installs, 300–500 MB idle RAM, Python packaging "a nightmare" for non-technical users (PyInstaller AV false-positives, path issues), two runtimes to crash — directly conflicts with "extremely stable and fast" positioning |
| Electron + onnxruntime-node | No Python; Upscayl-style viability | Still Electron footprint; native-module ABI churn; image pipeline in JS/native-addon glue |
| Swift + C# native (two apps) | Best per-OS polish, CoreML native | Two codebases, double maintenance, kills velocity for a small team |

**Rust makes sense.** The precedent set (RMBG, rembg-rs, HivisionIDPhotos-as-blueprint, Oculante) shows every MVP component has a proven Rust/ONNX path, and the perf/stability/footprint profile matches the product positioning. The honest weak spot is not Rust — it's that the *canvas layer* is best done in the webview (web tech), which Tauri gives you anyway.

### Recommended stack
- **GUI**: Tauri 2 + Svelte (or React, match openpdfedit) — canvas via asset protocol/custom URI handler, never pixel data over JSON IPC; deep-zoom tiling only if >50 MP support needed.
- **Inference**: `ort` 2.0.0-rc (pin exact rc), `load-dynamic`, bundled ORT dylibs; EPs: CoreML (fixed-shape exports) on macOS, DirectML on Windows with CPU fallback toggle; models: MODNet/ISNet default, U2-Net compat, BiRefNet-lite/HR as HD mode; INT8 variants for CPU.
- **Faces**: YuNet ONNX on the same ort session infra (rust-faces/face_id as reference code); landmarks drive compliance geometry.
- **Imaging**: image 0.25 + zune-jpeg decode, fast_image_resize (linear-light Lanczos) for print DPI, imageproc mask ops, lcms2 ICC, kamadak-exif; **defer HEIC** (patents) → OS decoders per platform.
- **Print**: printpdf 0.9.1 for 4×6/A4 photo sheets with exact mm/DPI.
- **Licensing**: ed25519-dalek offline key verification.
- **Distribution**: Tauri bundler + notarization + tauri-plugin-updater; models download-on-first-run with checksums.

### Risks
1. **ort still rc** — pin version; abstraction trait over the inference call so tract (CPU-only) remains an escape hatch for U2-Net/MODNet.
2. **CoreML/DirectML EP flakiness** (dynamic shapes, Intel Iris Xe wrong results) — fixed-shape exports, GPU toggle + silent CPU fallback + result sanity checks; CPU-only must remain acceptable for default models.
3. **BiRefNet CPU pathology** — gate HD mode behind GPU detection or explicit warning.
4. **Webview canvas perf ceiling** — prototype week 1: 40 MP zoom/pan + before/after slider in WebView2 and WKWebView; fallback plan is egui/wgpu native window for the canvas only (or full egui pivot à la Oculante).
5. **HEIC patents** — do not ship libheif/libde265 in binaries without counsel; OS-decoder route sidesteps it.
6. **Tauri sidecar notarization bug (#11992)** — avoid `externalBin`; use resources.
7. **Linux later**: WebKitGTK canvas HW-accel is the weak platform if Linux gets added.

### Key URLs
Frameworks: https://github.com/tauri-apps/awesome-tauri • https://v2.tauri.app/blog/tauri-20/ • https://github.com/emilk/egui/discussions/797 • https://github.com/emilk/egui/issues/1811 • https://github.com/woelper/oculante • https://github.com/rerun-io/rerun/blob/main/ARCHITECTURE.md • https://byteiota.com/iced-0-14-rust-gui-gets-reactive-rendering-time-travel/ • https://medium.com/@qarmin/czkawka-7-0-a465036e8788 • https://slint.dev/pricing • https://dioxuslabs.com/blog/release-070/ • https://github.com/zed-industries/awesome-gpui • https://www.boringcactus.com/2025/04/13/2025-survey-of-rust-gui-libraries.html • http://lukaskalbertodt.github.io/2023/02/03/tauri-iced-egui-performance-comparison.html
Inference: https://github.com/pykeio/ort • https://ort.pyke.io/setup/linking • https://ort.pyke.io/perf/execution-providers • https://onnxruntime.ai/docs/execution-providers/CoreML-ExecutionProvider.html • https://onnxruntime.ai/docs/execution-providers/DirectML-ExecutionProvider.html • https://github.com/microsoft/onnxruntime/issues/14212 • https://github.com/microsoft/onnxruntime/issues/18652 • https://github.com/ZhengPeng7/BiRefNet/issues/26 • https://lib.rs/crates/rembg-rs • https://github.com/rustybuilder/rust-faces • https://docs.rs/face_id • https://crates.io/crates/candle-onnx • https://crates.io/crates/burn-candle
Imaging: https://github.com/Cykooz/fast_image_resize • https://github.com/etemesi254/zune-image • https://crates.io/crates/lcms2 • https://crates.io/crates/qcms • https://github.com/kamadak/exif-rs • https://github.com/strukturag/libheif/issues/591 • https://github.com/imazen/heic-decoder-rs • https://github.com/fschutt/printpdf
Precedents/packaging: https://github.com/Tekipeps/rmbg • https://github.com/zhbhun/rmbg • https://github.com/Zeyi-Lin/HivisionIDPhotos • https://github.com/upscayl/upscayl • https://v2.tauri.app/distribute/sign/macos/ • https://github.com/tauri-apps/tauri/issues/11992 • https://github.com/tauri-apps/tauri/issues/7127 • https://github.com/orgs/tauri-apps/discussions/7145 • https://dev.to/tomtomdu73/ship-your-tauri-v2-app-like-a-pro-code-signing-for-macos-and-windows-part-12-3o9n • https://evilmartians.com/chronicles/making-desktop-apps-with-revved-up-potential-rust-tauri-sidecar

**Bottom line**: Rust is validated for this product. Tauri 2 + ort + image/fast_image_resize/lcms2/printpdf is the recommended stack; the two things to prototype before committing are (1) 40 MP canvas zoom/pan in the webview on both OSes, and (2) BiRefNet-lite + U2-Net through ort with CoreML/DirectML on real hardware.
