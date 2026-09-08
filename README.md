# OpenPhotoId

Formerly OpenFrame; the web build shipped as OpenPassport until September 2026.

Open-source, local-first photo tool written in Rust: AI background
removal, an ID-photo compliance engine (auto crop/resize/validate against
country standards), background replacement, and batch processing — all
fully offline, on the desktop and in a browser.

There are two front ends over one engine:

- **`apps/desktop`** — Tauri 2 + Svelte, Windows and macOS.
- **`apps/webapp`** — the same Rust core compiled to WebAssembly, served
  as a static page; see [apps/webapp/README.md](apps/webapp/README.md)
  for where the seam between Rust and JavaScript falls.

Everything is free, including the five features this category normally
sells by subscription — see [docs/premium-features.md](docs/premium-features.md)
for what each one does and, where it is less than the marketing elsewhere,
why that is deliberate.

**Status: pre-alpha, M0–M7 functionally complete, M8 partially done,
M9–M11 added since.** See
[PLAN.md](PLAN.md) (milestones M0–M11, with per-milestone status) and
[docs/research/](docs/research/) for the market, model-licensing, stack,
and compliance research behind the plan — `05-m0-results.md` in
particular logs several real-photo bugs found during implementation that
synthetic unit tests alone missed. See [docs/releasing.md](docs/releasing.md)
for what's left before a real release (signing, notarization — these need
credentials only the project owner should hold).

## Layout

- `crates/frame-core` — image decode/orient/resize/export (KB-window JPEG targeting)
- `crates/frame-engine` — ONNX inference (ort), model registry + checksummed download-on-first-run
- `crates/frame-matting` — matting pipeline: MODNet (validated default), BiRefNet-lite (implemented, unverified on real hardware), guided-filter refinement + foreground-estimation de-fringing, IoU/MAE eval harness
- `crates/frame-face` — YuNet face detection + 5-point landmarks
- `crates/frame-compliance` — 20-document country spec dataset + auto-crop solver + 11-check validation report + padded background-extension crop
- `crates/frame-sheet` — print-sheet layout (4x6/A4/5x7, cut marks), raster output
- `crates/frame-batch` — parallel batch executor (shared model locks + rayon), per-item CSV report
- `crates/frame-retouch` — optional finishing: capped skin smoothing (off by default), print sharpening, gradient/vignette/image backdrops, parametric formal wear
- `crates/frame-wasm` — wasm-bindgen bindings that run the whole pipeline in a browser; JS supplies only the ONNX forward pass
- `crates/frame-license` — Ed25519 scaffold (all features currently ungated)
- `apps/desktop` — Tauri 2 + Svelte: single-photo compliance flow (drag-drop → spec → before/after → checklist → download), an update-check button (`tauri-plugin-updater`), plus the M0 canvas-perf spike as a second tab
- `apps/webapp` — the browser build: mobile-first, English + Simplified Chinese, installable, works offline after the first visit
- `data/specs` — country/document photo specs (JSON, mm-first, sourced + dated)

## Build

```sh
export CARGO_TARGET_DIR=~/.cache/openphotoid-target   # keep target off shared mounts
cargo build --release && cargo test --workspace --release

# real-inference integration tests (download models, run actual inference —
# not in the default test run):
cargo test --workspace --release -- --ignored

# spikes / manual tools
cargo run --release -p frame-matting --bin matting-spike -- testdata/portrait-obama.jpg /tmp/out modnet
cargo run --release -p frame-compliance --bin crop-spike -- testdata/portrait-obama.jpg /tmp/out us-passport
cargo run --release -p frame-sheet --bin sheet-spike -- /tmp/out/us-passport.jpg /tmp/out/sheet.jpg 50.8 50.8
cargo run --release -p frame-batch --bin batch-spike

# desktop shell (dev mode)
cd apps/desktop && npm install && npm run tauri dev

# the web app (static output in apps/webapp/dist)
npm --prefix apps/webapp install
npm --prefix apps/webapp run build
npm --prefix apps/webapp run preview   # http://localhost:8082

# real installable bundle (.app/.dmg on macOS, .msi/.nsis on Windows —
# ad-hoc signed only; see docs/releasing.md for real code signing)
cd apps/desktop && npm run tauri build
```

CI (`.github/workflows/ci.yml`) runs fmt/clippy(-D warnings)/cargo-deny,
the default test suite on macOS + Linux, a desktop build on macOS +
Windows, and the real-inference integration suite (cached models) on
pushes to `main`.

Every crate that owns an inference session (`frame-face`, `frame-matting`,
and through them `frame-compliance`) puts it behind a default-on
`inference` feature. The browser build turns it off, which is what keeps
`ort` — no wasm32 target — out of that graph; the model-independent
pre/post-processing compiles either way, so both builds share one
implementation of it. Build the wasm crate with `-p frame-wasm`
specifically: cargo unifies features across the *selected* workspace
members, so a `--workspace` build for wasm32 would switch `inference` back
on through the desktop crates.

Models download on first run (checksummed) to the platform data dir; see
[MODELS.md](MODELS.md) for licenses — only commercially-clean weights are
allowed (no BRIA RMBG, no insightface weights).

## License

MIT OR Apache-2.0 (same as the openapps workspace). Third-party dependency
licenses are in [THIRD-PARTY-LICENSES.html](THIRD-PARTY-LICENSES.html)
(generated via `cargo about generate about.hbs -o THIRD-PARTY-LICENSES.html`
— regenerate after any dependency change before cutting a release).
