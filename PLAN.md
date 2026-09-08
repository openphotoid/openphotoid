# openphotoid — Implementation Plan

> Renamed from **openframe / OpenFrame** on 2026-09-06; the web build was **OpenPassport**. Older entries below keep the names they were written with.

Open-source, local-first desktop photo tool for Windows + macOS, written in Rust.
MVP: AI background removal · ID Photo Compliance Engine · background replacement · batch processing.

**Status: AWAITING USER APPROVAL. No implementation until the user gives an explicit go.**

---

## Decision log

| Date | Decision | Status |
|---|---|---|
| 2026-08-01 | Product is a **photo** editor (ID-photo focus), not PDF; name **openframe**; lives in `openapps/openframe/` | ✅ Decided (user) |
| 2026-08-01 | MVP strictly limited to 4 features: bg removal, compliance engine, bg replacement, batch | ✅ Decided (user) |
| 2026-08-01 | Research complete: `docs/research/01-04*.md` | ✅ Done |
| 2026-08-01 | Free/premium split & pricing: **deferred** — build all features ungated, keep Ed25519 scaffold (same policy as openpdfedit) | ✅ Decided (user) |
| 2026-08-01 | Front-end framework: **Svelte** | ✅ Decided (user) |
| 2026-08-01 | Product display name: **OpenFrame** | ✅ Decided (user) |
| 2026-08-01 | **Plan approved; go for M0** | ✅ Decided (user) |
| 2026-08-01 | M0 spikes executed — verdict **go** (see `docs/research/05-m0-results.md`); carried items: human canvas-perf eyeball, real-hardware EP benchmarks, Windows CI build | ✅ Done |
| 2026-09-06 | Renamed to **openphotoid / OpenPhotoId** — folder, package names, bundle id, storage keys; the web build drops the OpenPassport name. Crates stay `frame-*` | ✅ Decided (user) |
| 2026-09-06 | Web build optimised and tested: wasm 993→734 KB gz (own `wasm-release` profile, `image` codecs trimmed, wasm-opt fixed), fonts 658→68 KB, slider re-render 1194→41 ms (composite cached across drags), CPU inference threaded from the second visit (service-worker COOP/COEP). Playwright suite (`apps/webapp/e2e`, 11 tests, real photos) + `capture.mjs`. Found and fixed: garment choice never applied, no dark theme, stale build. Open: inter-eye "recommended 120 px" cautions every US passport photo; lighting-symmetry fail line vs studio portraits | ✅ Done |

---

## 1. Product thesis (from `docs/research/01-competitors.md`)

**The core combination is unclaimed by any product on the market:** modern hair-grade local matting (BiRefNet-class) + multi-country ID compliance engine + batch + Windows *and* macOS + fully offline. Every competitor has at most two of these:

- Cloud services (PhotoAiD $9.95–23.85/photo, visafoto $9, iVisa, Smartphone iD) — per-photo pricing, privacy exposure, and documented "rejected despite expert approval" complaints.
- Windows-only desktop incumbents — AMS Passport Photo Maker ($39.95–69.95 lifetime, legacy AI) and ID Photos Pro 8 (£240, pro-only). Both validate willingness to pay; both lack macOS and modern matting.
- Free tools are dead-ends: IDPhotoStudio (sheet tiling only), GIMP/Krita (no hair-grade matting), browser-local tools (wasm-constrained models).
- HivisionIDPhotos (~16–19k stars, Apache-2.0) proves OSS demand but is a Gradio/dev deployment, CN-centric — the polished cross-platform desktop niche is empty.

**Differentiators, in priority order:**
1. **Privacy**: "your passport photo never leaves your computer" — backed by the Australian Passport Office's official warning against online photo services and GDPR biometric-data concerns.
2. **Compliance depth as the moat** (matting is becoming commodity): an authored, source-cited country-spec database + landmark-driven auto-crop + pass/fail validation with explanations.
3. **"Compliant-but-natural" output**: since Jan 2026 the US State Dept rejects photos "generated, enhanced, or modified using AI tools" — conservative background replacement, no beautification, clean EXIF/metadata handling is a *feature*, not a limitation.
4. **Quality + stability + speed** (Rust, native inference, no Electron/Python runtime).
5. **Honest pricing** vs the market's dark patterns (fake-free, watermarks, credit expiry).

Threats to watch: Canva (owns remove.bg + Affinity), Apple/Microsoft OS-level segmentation as free baselines. Insulation = compliance engine + batch workflow.

## 2. Feature list

### MVP (strictly limited — user decision 2026-08-01)
1. **Background removal**
   - Models: MODNet (fast default, 25 MB) → BiRefNet-lite (quality default) → BiRefNet-HR/portrait/matting (HD mode, GPU-preferred). All MIT/Apache-2.0.
   - Refinement pipeline: guided-filter alpha upsample to full res → closed-form matting in the uncertainty band (pymatting port) → foreground color estimation (de-fringing).
   - Quality gate: IoU/S-measure eval harness against a curated test set (P3M-500 style + own photos); regression-tested per model/version.
   - Output: transparent PNG, or composite (see 3).
2. **ID Photo Compliance Engine**
   - Spec database: original JSON dataset (schema in `docs/research/04-compliance-specs.md`), mm-first, ~40–60 document types at launch covering US/EU/UK/CA/IN/CN/JP/SG/AU/KR/BR/RU/UAE/SA + Chinese domestic sizes (seeded from Hivision's Apache-2.0 CSV, attributed).
   - Auto-crop: YuNet face detect (5 landmarks) → roll-align → crown estimation → spec-driven crop (head-height %, eye-line band, centering) → exact px/DPI output.
   - Validation checklist (pass/warn/fail with plain-language reasons): single face, head-height ratio, eye-line, centering, pose (roll/yaw/pitch via Face Landmarker mesh), inter-eye distance, background uniformity (σ + ΔE vs target), shadows, blur (Laplacian), eyes open (EAR), mouth closed, glasses/glare, red-eye, color cast, resolution/aspect/file-size.
   - Digital-portal export: exact-pixel presets (Singapore 400×514, Seva 630×810, Saudi 200×200…) + JPEG quality binary-search to hit KB windows (DS-160 ≤ 240 KB, PAN 20–50 KB, COVA 40–120 KB).
   - Print sheets: 4×6 / A4 / 5×7 tiling with gutters + cut marks via printpdf, exact mm at 300/600 DPI.
   - `editing_forbidden` disclaimer flag per spec (e.g. Canada requires studio photos).
3. **Background replacement**
   - Solid color (spec-driven hex or custom picker) and simple linear/radial gradients.
   - Alpha-aware compositing in linear light with de-fringed foreground; edge feather control.
4. **Batch processing**
   - Drag-drop queue; per-item spec assignment or one spec for all; rayon-parallel pipeline; progress + per-item pass/fail report; batch export (files + sheets + CSV report).

### Post-MVP roadmap (build order TBD after MVP ships)
- Interactive mask touch-up (brush restore/erase, MobileSAM click-to-select feeding the matting refiner).
- Blur/bokeh & image backgrounds; background library.
- Basic adjustments (exposure/white-balance/rotate/straighten) — conservative, compliance-safe. **Crop done 2026-08-02** (bounded manual adjustment, see §6 M5 addendum) — exposure/white-balance/rotate/straighten remain.
- More formats: HEIC via OS decoders, WebP/AVIF export, TIFF.
- Spec-pack updates delivered via app update channel; community spec contributions (open data).
- CLI companion binary for scripted/batch use.
- B2B/studio features: order queue, custom layouts, printer profiles (the AMS Pro / Face Crop Jet niche).
- Linux build.

### Free/premium candidates (DEFERRED — build everything ungated first)
Market-observed gates that don't trigger resentment: batch volume, studio/print-shop features, HD model pack, background library, spec-pack update channel. Anti-patterns to avoid: resolution caps, watermarks, per-photo credits, fake-free. Keep the Ed25519 license scaffold so gating is a later flip. One-time price anchor if/when monetized: $30–70 (under Pixelmator/Affinity-V2 anchor).

## 3. Tech stack (from `docs/research/03-rust-stack.md`)

| Layer | Choice | Notes |
|---|---|---|
| Shell/GUI | **Tauri 2** + Svelte or React (⏳ user choice; match openpdfedit) | Consistency with sibling app: shared bundler/signing/updater/licensing infra. Canvas via asset protocol / custom URI — never pixels over JSON IPC. Direct precedent: Tekipeps/rmbg (Tauri+ort bg-removal). |
| Inference | **ort 2.0-rc (pinned)**, `load-dynamic`, ORT dylibs as bundled resources (not sidecars — notarization bug #11992) | EPs: CoreML (macOS, fixed-shape exports), DirectML (Windows, toggle + CPU fallback + sanity check), CPU default. Behind an `Engine` trait; tract as CPU-only escape hatch for small models. |
| Models | MODNet, BiRefNet family, BEN2 (optional), YuNet, MediaPipe Face Landmarker (ONNX-converted) | Download-on-first-run with checksums + resume; installer stays 10–25 MB. fp16/int8 variants for CPU. |
| Imaging | image 0.25 + zune-jpeg, fast_image_resize (linear-light Lanczos), imageproc, lcms2 (ICC), kamadak-exif | HEIC deferred to OS decoders (patent risk). |
| Print | printpdf 0.9 | mm-exact sheets, crop marks. |
| Licensing | ed25519-dalek scaffold | Offline key verification; gating deferred. |
| Parallelism | rayon | Batch pipeline. |
| Policy | cargo-deny | Ban AGPL/GPL in-process; license allowlist. |

**Rust verdict: validated.** Every MVP component has a proven Rust/ONNX path; Electron+Python alternative (Upscayl-style) costs 150–300 MB installs and two crash-prone runtimes, contradicting the stability/speed positioning.

## 4. Model & license policy (from `docs/research/02-models-licenses.md`)

**Allowed (commercial-clean):** BiRefNet all variants (MIT), MODNet (Apache-2.0), U2-Net/IS-Net (Apache-2.0), BEN2 (MIT — verify LICENSE file at fetch), InSPyReNet (MIT), ORMBG (Apache-2.0), SAM2/MobileSAM (Apache-2.0), YuNet (MIT), MediaPipe models (Apache-2.0), HivisionIDPhotos weights `hivision_modnet`/`modnet_photographic` (Apache-2.0), pymatting algorithms (MIT).

**Banned:** BRIA RMBG-1.4/2.0 (non-commercial), MatAnyone (S-Lab NC), insightface SCRFD/RetinaFace official weights (NC), FBA/ViTMatte released checkpoints (Adobe DIM dataset taint), CelebAMask-trained face parsing. Code from AGPL/GPL projects (imgly, ComfyUI-RMBG, dpar39/ppp) is read-only reference — never copied.

**Code-borrow map (port, with attribution):** HivisionIDPhotos (Apache-2.0) — the entire ID-photo domain pipeline; rembg (MIT) — session/model-zoo architecture + alpha-matting chain; pymatting (MIT) — closed-form matting + foreground estimation math; Tekipeps/rmbg (check license) — Tauri+ort patterns; rembg-rs (MIT) — ort pipeline code.

Enforcement: cargo-deny in CI; a `MODELS.md` manifest recording each shipped weight's license + source URL + SHA-256.

## 5. Architecture

```
openphotoid/
  Cargo.toml               # workspace
  crates/
    frame-core/            # image pipeline: decode/orient/CMS/resize/composite/export
    frame-engine/          # Engine trait + ort backend (+ tract fallback); model registry,
                           # download/cache/checksum; EP selection & fallback
    frame-matting/         # segmentation→alpha pipeline; guided filter; closed-form band
                           # refinement; foreground estimation
    frame-face/            # YuNet detect, landmark mesh, pose/EAR/MAR metrics
    frame-compliance/      # spec dataset (JSON) + loader; auto-crop solver; validation
                           # checklist; portal export (px/KB targeting)
    frame-sheet/           # print-sheet layout → printpdf
    frame-batch/           # queue, rayon executor, progress events, reports
    frame-license/         # ed25519 scaffold (ungated)
  apps/
    desktop/               # Tauri 2 shell (src-tauri + web front-end)
    cli/                   # (post-MVP) headless companion
  data/
    specs/*.json           # country spec dataset + JSON schema
    layouts/*.json         # sheet layouts
  testdata/                # eval images + golden outputs (git-lfs or scripted fetch)
  docs/research/           # 01-04 research reports
```

Pipeline: decode → EXIF orient → sRGB (lcms2) → [detect face] → [matting @ model res] → guided-filter upsample → band refine → foreground estimate → [compliance crop solve] → composite bg → validate → export (px/DPI/KB exact) → sheet PDF. Every stage pure-function-ish over `frame-core` types → unit-testable, batch-parallel.

## 6. Milestones

Each milestone ends with demoable state + tests green. Build with local CARGO_TARGET_DIR (shared-mount caveat — see openapps build-env memory).

- **M0 — Validation spikes.** ✅ Done 2026-08-01, verdict **go** (`docs/research/05-m0-results.md`). Tauri 2 + Svelte shell compiles and runs; MODNet matting ~200-300ms CPU on a 9MP portrait; YuNet detect ~50-60ms; US-passport auto-crop lands mid-band on the first real photo tried.
- **M1 — Workspace + core pipeline.** ✅ Done. 8 crates scaffolded; `frame-core` decode/EXIF-orient/resize/export with JPEG-KB-window targeting; CI added (`.github/workflows/ci.yml`: fmt/clippy/deny, test on macOS+Linux, desktop build on macOS+Windows, a real-inference `integration` job on push to main). Eval harness landed as part of M2 (below) rather than standalone — same content, different milestone boundary.
- **M2 — Matting engine.** ✅ Done. `frame-engine` (ort session wrapper, sha256-pinned download-on-first-run registry, CoreML/DirectML feature flags), `frame-matting` (MODNet shipped as the validated default; BiRefNet-lite implemented and code-reviewed but **inference unverified** — OOM on this dev VM, needs real-hardware benchmarking before enabling as default HD tier), guided-filter refinement + foreground-estimation de-fringing, `frame-matting::eval` (IoU/MAE) with a golden regression test against the real test portrait. **Caught and fixed a real bug here**: the initial refinement produced a visible hair halo on the real photo despite passing all synthetic unit tests — see the M2 addendum in `docs/research/05-m0-results.md`.
- **M3 — Background replacement.** ✅ Functionally done as part of M2/M4 rather than a standalone milestone: solid-color and gradient compositing (`Matte::composite_solid`, `composite_gradient`, `composite_solid_defringed`) are implemented and tested. **Not done**: linear-light compositing (currently gamma-space, matching most competitors — noted as a quality follow-up in `03-rust-stack.md`) and an explicit user-facing feather-amount control (edges are already soft via the alpha matte, but there's no separate feather slider).
- **M4 — Face + compliance engine.** ✅ Done, with one scope note. `frame-face` (YuNet decode/NMS from scratch), spec dataset at **20 documents** (not the 40-60 originally estimated — US/EU/UK/CA/IN(×3)/CN(×3)/JP/SG/AU/KR/BR/RU/AE/SA, each with sources + `last_verified`), auto-crop solver, padded-crop background extension, an 11-check validation report (re-detects on the FINAL image rather than trusting solver math — this caught a second real bug: the background-uniformity check assumed all 4 image corners were background, which is false once shoulders reach the frame edges). Portal KB-targeting via `encode_jpeg_within`. **Scope deviation**: print sheets are **raster JPEG/PNG** (`frame-sheet`), not PDF — printpdf was researched but not wired in; raster sheets are printable at the target DPI but a PDF option remains a follow-up if specifically needed.
- **M5 — Desktop app.** ✅ Functional core done: Tauri commands (`list_specs`, `process_photo`), Svelte UI (drag-drop, spec picker, before/after, color-coded checklist, JPEG download), M0 canvas spike preserved as a second tab. Verified with a real end-to-end test that mirrors the exact browser round-trip (base64 in/out) using the real test portrait — see the M5 addendum in `05-m0-results.md`. **Not done**: model-manager UI, spec search/filter UI (currently a flat dropdown), export-format options beyond JPEG. **Blocked in this environment**: this dev VM has no attached display (`screencapture`/`system_profiler` confirm no CGDisplay), so an actual click-through has not been visually verified — needs a real machine (`cd apps/desktop && npm run tauri dev`).

  **Real bug found 2026-08-02, first actual click-through on real hardware**: the spec dropdown was empty — no compliance spec could be selected at all. Root cause: `frame_compliance::spec::load_all()` (backing the `list_specs` Tauri command) resolved `data/specs/` via `env!("CARGO_MANIFEST_DIR")`, which is a *compile-time* path baked into the binary — pointing at this dev VM's source tree, not anywhere that exists on an end user's machine. In dev mode (`cargo run`/`npm run tauri dev` on the same machine that built it) this coincidentally works, which is exactly why it went undetected until someone ran a genuinely *packaged* `.dmg`-installed build on a different machine — a real-photo-only-style gap that no amount of testing on the build machine itself would have caught, same lesson as the M0/M2 findings. Fixed by embedding `data/specs/*.json` into the binary at compile time via `include_dir!` (`crates/frame-compliance/src/spec.rs`) instead of a runtime filesystem read; `default_specs_dir()`/`load_dir()` kept as-is for the crate's own dev-time `every_spec_file_is_well_formed` test, which legitimately wants real file paths for its error messages and only ever runs via `cargo test` on a dev machine. Verified: full workspace test suite green, and a fresh `.dmg` rebuilt with the fix.

  **Manual crop adjustment added 2026-08-02**, at user request after testing found the auto-crop-only flow too rigid (deliberately out of MVP scope per §1/post-MVP roadmap below, but reasonable to want once the app was in hand). Design: `frame_compliance::crop::CropAdjustment` generalizes `solve_crop` from a hardcoded "middle of every band" target to three caller-chosen values (head size, eye-line position, horizontal offset), each independently clamped to the range the spec's *percentage-band* checks (head height, eye line, centering) can satisfy. The desktop app caches the expensive part of the pipeline (matting + face detection) in `AppState` after `process_photo`, so a new `adjust_crop` command can re-solve/re-validate cheaply — no ONNX inference — on every slider drag (debounced 150ms in the Svelte frontend). Real UI: three range sliders under the output image, live-updating both the image and the checklist.

  **Important limitation found during testing, not assumed**: the adjustment bounds only guarantee the percentage-band checks stay satisfiable — they do **not** guarantee every check passes. Pushing the real test portrait's zoom to the tightest end of `us-passport`'s allowed head-size band dropped `inter_eye_distance` (a check tied to the source photo's actual pixel detail, not a percentage) below its minimum and failed overall — a genuine interaction the design didn't originally account for, caught by testing the actual edge of the range rather than just the middle. Fixed by correcting the "guaranteed compliant by construction" claim everywhere it appeared (code comments, UI hint text) to accurately scope it to the geometry checks, and by leaning on live re-validation (which was already there) as the real safety net rather than the bounds alone. Verified end-to-end with real inference: `apps/desktop/src-tauri/tests/pipeline_regression.rs`'s `adjust_crop_re_solves_without_reprocessing_the_source` test exercises a modest adjustment (expected to stay compliant) and the extreme edge (expected to genuinely fail `inter_eye_distance` on this photo) as two distinct, intentional cases — not just a happy path.
- **M6 — Batch.** ✅ Core done: `frame-batch::BatchExecutor` (shared-lock model instances + rayon-parallel everything else — deliberately, to avoid multiplying model memory across workers after the M2 BiRefNet OOM finding), per-item `BatchResult`, CSV report with one column per check. Verified end-to-end (4 real photos, different specs, in parallel) — release-mode timing: 2.0s for one item, 4.7s for four, real speedup from sharing 2 locks across 8 cores. **Not done**: batch UI (currently CLI/binary-only via `batch-spike`), batch print-sheet assembly (per-item sheets exist via `frame-sheet`; a "combine N processed photos into shared sheets" step doesn't).
- **M7 — Packaging & release eng.** ✅ Core done for everything achievable without real signing credentials. Real `.app`/`.dmg`/`.msi`/`.nsis` bundling enabled (`bundle.active = true`), a full icon set generated (`icons/*.icns/.ico/.png`, source kept at `icons/app-icon-source.png`), `tauri-plugin-updater` + `tauri-plugin-process` wired in with a working "Check for updates" UI button, an update-signing keypair generated (public key in `tauri.conf.json`; private key deliberately kept out of the repo), `THIRD-PARTY-LICENSES.html` generated via `cargo about`, and `docs/releasing.md` documents every step that needs credentials an agent can't hold (Apple Developer ID + notarization, Windows code signing, real update-key custody). **Not done, and shouldn't be done without the project owner**: actually signing/notarizing a build, cutting a v0.1.0 tag, or publishing a GitHub release — see `docs/releasing.md` §2 and §4.
- **M8 — Hardening & polish.** Partially done. ✅ A resolution cap (`frame_core::resize::DEFAULT_MAX_DIMENSION`, 4000px) now guards both the batch executor and the desktop command against huge source photos blowing up memory in `frame-matting`'s full-resolution refinement passes; a `CancellationToken` lets a batch run stop launching new items mid-flight (already-started items still finish — there's no safe way to preempt an in-flight ONNX call). ✅ Accessibility pass: fixed the one real a11y issue (`svelte-check` now reports 0 errors/0 warnings) — added real keyboard pan/zoom + a focus ring to the M0 canvas spike, and fixed a genuine WCAG 1.4.1 violation in the compliance checklist (pass/warn/fail was color-only; added a text label). **Not done**: i18n scaffold, and perf-budget validation on real (non-shared-VM) hardware — this dev VM's timing numbers are established as noisy (see the M6 addendum in `docs/research/05-m0-results.md`) and a beta feedback loop needs actual users.

- **M9 — Mobile (Android/iOS), exploratory.** Not in the original MVP scope (§1) — started 2026-08-02 at the user's request, requirement: inference must stay fully on-device, no server-side cost. **Key finding**: `ort` (desktop's inference backend) ships no prebuilt `onnxruntime` binary for Android or iOS at the pinned version (`=2.0.0-rc.10`) — confirmed by reading its actual download manifest, not assumption. Building onnxruntime from source for mobile is possible but a substantial separate toolchain; the chosen alternative is **`tract`**, a pure-Rust ONNX engine with no native/C++ build step, which `frame-engine`'s `EngineSession` abstraction was already designed to accommodate (`Cargo.toml`'s `ort` comment: "Engine trait is the seam that keeps tract available as a CPU-only escape hatch" — written before this investigation, never exercised until now). ✅ **YuNet** loads and runs under tract with zero changes. ✅ **MODNet** needed two fixes, both now scripted and validated: (1) its `Resize` nodes use `coordinate_transformation_mode=pytorch_half_pixel`, which tract 0.21 doesn't implement — rewritten to `half_pixel` (provably identical for this model, since the two modes only differ when an output dimension is 1, which never happens in MODNet's decoder); (2) tract's shape inference can't carry symbolic input dims through a fractional-scale `Resize` — fixed by specializing the graph to the 512x512 input `frame-matting` already enforces before inference. Both fixes are in `scripts/patch-modnet-for-tract.py`, and were validated by running the *actual* patched model through tract against the real `ort` pipeline on `testdata/portrait-obama.jpg`: mean absolute alpha difference 0.000179, foreground coverage 50.965% (ort) vs 50.962% (tract) — effectively equivalent output, not just "doesn't crash." BiRefNet-lite deliberately excluded from mobile scope — it's already the fragile/heavy tier on desktop (MODELS.md: "inference unverified — OOM on shared dev VM"), not worth the same risk on a phone.

  **Update, same day**: `tract` is now a real, wired-in `frame-engine` backend, not just a throwaway harness. `EngineSession`/`Ep` are feature-gated (`ort-backend`, default vs. `tract-backend`, mutually exclusive — enforced with `compile_error!` if both or neither are enabled) across two parallel modules (`session_ort.rs`, `session_tract.rs`) with an identical public API, so `frame-matting`/`frame-face` don't change at all to use either. `frame-matting`, `frame-face`, `frame-batch`, `frame-compliance`, and `apps/desktop/src-tauri` all got the same `ort-backend`/`tract-backend` passthrough features `coreml`/`directml` already used as precedent. Proven with real, `#[ignore]`d integration tests (not just compilation) — `frame-matting/tests/tract_backend_regression.rs` and `frame-face/tests/tract_backend_regression.rs` — both run the real models against the real test portrait through the actual tract-backed `EngineSession` and pass. `Modnet` gained a `load_from_path` (alongside the registry-backed `load`) so it can load the locally-patched variant. Verified: the desktop (`ort-backend`, default) path is completely unaffected — full workspace test suite, fmt, and clippy all still green after a clean rebuild.

  **Update, 2026-09-08**: done, on this Mac. `crates/frame-session` holds the pipeline once for both the browser and the phones; `apps/mobile` (Tauri 2) runs it natively through `tract` with the patched MODNet from `registry::MODNET_TRACT`, shipped in the bundle. Verified end to end on iPhone/iPad simulators at iOS 17.0 and 26.5 (about 3 s) and on the Android 15 emulator (about 4 s); `docs/mobile.md` has the status, the two platform traps (Android's `asset://` resource dir, status-bar insets) and the build commands. `.github/workflows/release.yml` builds the APK and unsigned IPA on `v*` tags. The paragraph below is the state before that.
  **Was still not done (2026-08-02)**: BiRefNet's own tract compatibility (untested, likely worse given its transformer backbone — no plan to pursue given the exclusion above), publishing the tract-patched MODNet variant anywhere the model registry could fetch it from (right now it only exists as something the test suite generates locally on demand via the Python script — there's no hosted URL/checksum for a real mobile build to download it from), `tauri android/ios init` and the touch/camera UI rework, and — same limitation as desktop — this dev VM has no Android SDK/emulator or iOS Simulator, so nothing mobile-specific can be built or run here; that needs a machine with Android Studio and/or Xcode installed.

  **Unrelated finding surfaced while verifying this work**: `cargo deny check` now fails on 3 real (not "unmaintained") vulnerability advisories — `time` (stack exhaustion DoS) and `quick-xml` (quadratic runtime + memory exhaustion), both pulled in transitively through `tauri`→`cookie`/`plist`, nothing to do with this M9 work (confirmed via `git diff Cargo.lock`: zero changes to either crate's version). Likely the RUSTSEC advisory database picked up new entries since the M7/M8 verification pass earlier the same day — this repo's `cargo deny check` was clean then. Not fixed as part of this work (out of scope, and the fix — if one exists without breaking tauri's version constraints — deserves its own look); flagged here so it isn't mistaken for a regression this session caused.

- **M10 — Web app (OpenPhotoId), 2026-08-26.** Not in the original MVP scope; built at the user's request, and the form the research had already put second behind mobile (§八⑤: "a light responsive web page, not a desktop client", to catch the task-shaped search traffic and the one scenario where a phone-first user is at a computer — filling in a visa or exam form). `apps/webapp` is a static site: the Rust core compiled to wasm32 via `crates/frame-wasm`, running entirely in the browser, with no server component. **Key finding**: the seam between Rust and JavaScript has to fall around the ONNX forward pass and nowhere else. `ort` has no wasm32 target, and `tract` (§M9's answer for mobile) is far too slow for MODNet in a browser — so onnxruntime-web runs the graphs, reaching WebGPU where the browser has it. Everything else is the desktop's own code: `frame-face::preprocess`/`::decode` and `frame-matting::modnet::preprocess`/`::postprocess` were extracted from their runners for exactly this, so the letterboxing, the normalisation constants and the head decode have one implementation rather than a Rust original and a JavaScript copy that agree on the day they are written. That refactor is behind a new `inference` feature (default on) on `frame-face`/`frame-matting`/`frame-compliance`, which the wasm build turns off; `cargo test --workspace --release` and `-- --ignored` (real inference) both still pass unchanged, which is what confirms the extraction preserved numerics.

  Verified end-to-end in headless Chrome against `testdata/portrait-obama.jpg`, driving the real UI rather than the internals: US passport 600×600 with all 11 checks reporting real values and a 196 KB JPEG inside the 240 KB cap; India PAN 295×413 landing at 47.5 KB inside its 20–50 KB window (the KB-window binary search working through wasm); Schengen 413×531; print-sheet and PNG exports; a batch run including a deliberately corrupt file, which failed its own row without stopping the run; and the whole flow again with the network cut at the browser, which is the offline claim actually tested rather than asserted. Three real defects were found and fixed this way: onnxruntime 1.29's WebGPU build wants the `asyncify` runtime rather than the `jsep` one its file names suggest (the build script now derives the list from the bundles and fails if a needed file is missing); Vite was emitting 40 MB of unreachable duplicate ONNX runtimes into `assets/`; and Svelte 5's `$state` does not proxy class instances, so the checklist never rendered even though the pipeline had produced it.

- **M11 — The paid-tier features, shipped free, 2026-08-26.** The five features §五 found every competitor gates behind a subscription (`docs/premium-features.md` covers each in full): hair-level edge detail, uncapped batch, non-flat backgrounds, print sharpening, and formal wear. New crate `crates/frame-retouch` holds the four that needed new code; the quality tier is `Matte::blend_region` plus a second MODNet pass over a head crop, which buys real resolution where hair is without the 224 MB BiRefNet download that `MODELS.md` still lists as unverified. Two are deliberately less than their marketing elsewhere, and say so in the UI: print sharpening is threshold-gated unsharp masking rather than a super-resolution model, because an ID photo is evidence and inventing facial detail is the wrong failure mode; formal wear is a parametric template positioned from the face landmarks, because a garment-aware generative model cannot run in a browser and doing it server-side would mean uploading the photo. Retouching exists, starts at zero and is capped — the same research names forced beautification as one of the category's top complaints.

**Overall**: M0-M7 are functionally complete for everything achievable without external credentials or a real display; M8 is partially done. Workspace-wide: `cargo fmt --check` clean, `cargo clippy --workspace --all-targets -- -D warnings` clean, `cargo deny check` clean, `cargo test --workspace --release` and `-- --ignored` (real-photo inference across matting/crop/validate/batch/desktop) both fully green, and a real `.app` bundle builds and launches. See `docs/research/05-m0-results.md` for the full list of real-photo bugs found and fixed along the way — several would not have been caught by synthetic-fixture unit tests alone. Remaining before a real release: an actual visual click-through on hardware with a display, and the credentialed steps in `docs/releasing.md` (signing, notarization, tagging).

## 7. Risks

| Risk | Mitigation |
|---|---|
| ort still rc | Pin exact rc; Engine trait; tract fallback for small models |
| CoreML/DirectML flakiness (dynamic shapes; Intel Iris Xe wrong results) | Fixed-shape exports; GPU toggle + silent CPU fallback + output sanity check |
| BiRefNet CPU pathologically slow | HD mode gated on GPU detection or explicit warning; lite/MODNet defaults |
| Webview canvas perf ceiling | M0 spike; fallback = native wgpu canvas window or egui pivot (Oculante precedent) |
| HEIC patents | No in-app decode v1; OS decoders per platform |
| "AI-edited photo" government rejections | Conservative processing mode; no beautification; clean metadata; document per-spec disclaimers |
| Spec dataset accuracy/staleness | Per-entry sources + `last_verified`; update channel; community PRs with review checklist |
| Weights' training-data license taint (ecosystem-wide) | Ship only weights whose released license is permissive; record provenance in MODELS.md; industry-standard position documented |
| Canva/OS free baselines | Compliance engine + batch + cross-platform is the moat; track macOS Vision/Windows ML quality |

## 8. Open questions for the user (blocking M0)

1. **Approve this plan?** (scope, architecture, milestones)
2. **Front-end framework**: Svelte or React (ideally same answer as openpdfedit).
3. **Tiering**: confirm openpdfedit-style policy — build all features ungated, defer pricing (recommended), or decide split now.
4. App display name / branding ("OpenFrame"?) — repo dir is `openframe/`. *(Resolved 2026-09-06: OpenPhotoId.)*
