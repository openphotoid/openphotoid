# Running and testing OpenPhotoId locally

This is a hands-on guide for building OpenPhotoId from source and actually
using it — on a real machine with a real display. (Everything up to this
point was built and tested on a headless shared dev VM with no attached
display, so a real click-through has never happened — this guide is
written so you can be the first to do that.)

This is *not* the release runbook — for signing, notarization, and
cutting a tagged release, see [releasing.md](releasing.md). This guide
only covers building and running the app locally for your own testing.

Tested against macOS and Windows (the only two platforms CI builds for).
Linux desktop builds are untested — Tauri supports Linux in principle,
but nothing here has verified it for this project.

## 1. Prerequisites

| Tool | Version used in this repo | Check with |
|---|---|---|
| Rust (stable) | 1.97.1 | `rustc --version` |
| Node.js | 22.23 (20+ should work — CI uses 20) | `node --version` |
| npm | bundled with Node | `npm --version` |
| Git | any recent | `git --version` |

No `protoc`, no database, no other system services — this is a desktop
app with everything except two ONNX model files vendored or fetched
directly by Cargo/npm.

**macOS-specific:**
- Xcode Command Line Tools: `xcode-select --install` (needed for the
  linker and for `codesign`, even for local ad-hoc signing).

**Windows-specific:**
- Microsoft C++ Build Tools (Tauri needs the MSVC linker) — install via
  the [Visual Studio Build Tools installer](https://visualstudio.microsoft.com/visual-cpp-build-tools/),
  "Desktop development with C++" workload.
- WebView2 Runtime — pre-installed on current Windows 10/11; if missing,
  the bundle's `webviewInstallMode: downloadBootstrapper` setting
  (`apps/desktop/src-tauri/tauri.conf.json`) fetches it automatically on
  first install.

Install the Tauri CLI as a dev dependency (already in
`apps/desktop/package.json`, so `npm install` handles it — no global
install needed).

## 2. Clone and get your bearings

```sh
git clone <your fork/remote> openphotoid
cd openphotoid

# Recommended: keep cargo's build output off a networked/shared mount if
# you're on one (irrelevant on ordinary local disk, but harmless either way)
export CARGO_TARGET_DIR="$HOME/.cache/openphotoid-target"
```

Repo layout, if you want to look around first: `crates/frame-*` are the
Rust libraries (image IO, ONNX inference, matting, face detection,
compliance rules, print sheets, batch executor); `apps/desktop` is the
Tauri + Svelte app; `data/specs` holds the country photo-spec JSON files;
`testdata/portrait-obama.jpg` is a real test photo checked in for exactly
this kind of manual walkthrough.

## 3. Build and run the automated tests first

This confirms your toolchain is set up correctly before you touch the UI.

```sh
cargo build --release
cargo test --workspace --exclude openphotoid-mobile --release
```

Expect this to take a few minutes on first build (compiling `ort`,
`image`, and friends from scratch) and finish with a long list of
`test result: ok.` lines, `0 failed`, across ~24 test binaries.

Then run the real-inference integration tests. These are marked
`#[ignore]` because they download real ONNX models and actually run
inference (not just synthetic fixtures), so they're slower and need
network access the first time:

```sh
cargo test --workspace --exclude openphotoid-mobile --release -- --ignored
```

This downloads (once, cached under your platform's app-data dir —
`~/Library/Application Support/OpenPhotoId/models` on macOS,
`%APPDATA%\OpenPhotoId\models` on Windows):

| Model | Size | Used by |
|---|---|---|
| MODNet photographic portrait matting | ~25 MB | default background-removal model |
| YuNet face detector | ~230 KB | face/landmark detection for auto-crop |
| BiRefNet-lite | ~176 MB | higher-quality matting tier |

Total ~200 MB, one-time. If you're on a slow or metered connection, set
`OPENPHOTOID_MODELS_DIR` to point at a pre-populated directory instead (see
`crates/frame-engine/src/registry.rs`).

Expect ~6 passing tests here (MODNet + BiRefNet regression checks against
`testdata/portrait-obama.jpg`, a batch end-to-end test, and a desktop
pipeline round-trip test). If a download times out, just re-run — it
resumes into a `.part` file and only renames it after the SHA-256 check
passes, so a partial/corrupt download can't silently poison the cache.

## 4. Run the app itself (dev mode)

```sh
cd apps/desktop
npm install
npm run tauri dev
```

First run compiles the Rust backend in dev mode (slower than `--release`)
and starts a Vite dev server on `http://localhost:1420`; a native window
should open automatically. If port 1420 is already taken by something
else, stop that process or edit `devUrl` in `tauri.conf.json`.

### Manual smoke test — the compliance flow

The app has two tabs. On the main tab:

1. **Drag `testdata/portrait-obama.jpg` onto the drop zone** (or click it
   to open a file picker).
2. **Pick a document spec** from the dropdown — try `us-passport` first
   (2x2 in, plain white/off-white background required).
3. You should see a **before/after** view: the original photo next to the
   auto-cropped, background-processed result.
4. Check the **validation checklist** below it. Each row now shows both a
   colored dot *and* a text label (`PASS`/`WARN`/`FAIL`) — that text label
   is new work from this session (the color-only version was an
   accessibility bug). For `us-passport` against this test photo you
   should see mostly `PASS` rows; a passport-strict spec may flag a
   `WARN` or two depending on framing.
5. Click **download** and confirm a JPEG lands wherever your OS file save
   dialog points, and that it actually opens and looks like a cropped
   ID photo (not corrupted, not blank).
6. Try at least one more spec from a different country to confirm the
   spec-switching path re-runs crop + validation correctly.

### Manual smoke test — the canvas spike tab

Switch to the second tab (the M0 zoom/pan performance spike — this is a
dev tool, not a polished feature, but it's part of what shipped):

1. Confirm the FPS counter updates and a status line is visible.
2. **Mouse**: drag to pan, scroll wheel to zoom, the before/after slider
   at the top should reveal/hide the second image.
3. **Keyboard** (added this session as an accessibility fix — click into
   the viewport first so it has focus, you should see a green focus
   ring): arrow keys pan, `+`/`-` zoom. This is the actual thing to
   verify — the mouse path already worked before, the keyboard path is
   new and untested outside `svelte-check`'s static analysis.

If either tab misbehaves, that's exactly the kind of "only a headless VM
tested this" gap this guide exists to catch — worth filing as a real
finding rather than assuming it's your setup.

## 5. Build and install a real bundle (not dev mode)

Dev mode proves the code works; this step proves *packaging* works —
that a normal user could download and install this without touching a
terminal.

```sh
cd apps/desktop
npm run tauri build
```

This produces, under
`$CARGO_TARGET_DIR/release/bundle/` (or `apps/desktop/src-tauri/target/release/bundle/`
if you didn't set `CARGO_TARGET_DIR`):

- **macOS**: `macos/OpenPhotoId.app` (verified this session — builds and
  launches) and `dmg/OpenPhotoId_0.1.0_<arch>.dmg` (not built this session;
  `npm run tauri build` with no `--bundles` filter produces both — the
  `.app` build here only used `--bundles app` to save time)
- **Windows**: `msi/OpenPhotoId_0.1.0_<arch>.msi` and `nsis/OpenPhotoId_0.1.0_<arch>-setup.exe`
  (neither built or tested — this dev VM is macOS-only; Windows bundling
  is only exercised in CI's `desktop-build` job, which builds but doesn't
  install/launch it)

### macOS: installing an unsigned build

This build is **ad-hoc signed only** (no Apple Developer ID — see
[releasing.md](releasing.md) §2), so Gatekeeper will refuse to open it
normally. To run it locally anyway:

```sh
# Confirm it's ad-hoc, not a real Developer ID signature:
codesign -dv --verbose=4 "path/to/OpenPhotoId.app"   # expect "Signature=adhoc"
```

Then either right-click → **Open** (and confirm through the dialog,
instead of double-clicking), or:

```sh
xattr -cr "path/to/OpenPhotoId.app"
open "path/to/OpenPhotoId.app"
```

Mounting the `.dmg` and dragging to `/Applications` works the same way —
you'll hit the same unsigned-app prompt the first time you open it from
there.

### Windows: installing an unsigned build

Running the `.msi` or the NSIS `-setup.exe` will trigger a Windows
SmartScreen "unrecognized app" warning (no code-signing certificate yet —
see [releasing.md](releasing.md) §2). Click **More info → Run anyway** to
proceed. This is expected for an unsigned local build, not a bug.

### What to verify once it's installed

- The app launches standalone (not via `npm run tauri dev`) with no
  console/terminal attached.
- Repeat the manual smoke test from §4 against the installed app.
- The icon in the Dock/taskbar and the window title bar shows the actual
  OpenPhotoId icon (not Tauri's default placeholder).
- Click **"Check for updates"** in the UI. Since `plugins.updater.endpoints`
  points at a GitHub Releases URL that doesn't exist yet
  (`openapps/openphotoid` has no releases), expect this to fail cleanly
  with a "no update available" or network-error message — not a crash.
  This confirms the updater plugin is wired correctly; it does not
  confirm the update flow works end-to-end, since that needs a real
  published release to update *to* (see releasing.md).

## 6. Optional: exercise the batch pipeline and other spikes

```sh
export CARGO_TARGET_DIR="$HOME/.cache/openphotoid-target"

# Single-photo matting only
cargo run --release -p frame-matting --bin matting-spike -- \
  testdata/portrait-obama.jpg /tmp/out modnet

# Full compliance crop for one document spec
cargo run --release -p frame-compliance --bin crop-spike -- \
  testdata/portrait-obama.jpg /tmp/out us-passport

# Print-sheet layout (4x6 sheet of a 50.8x50.8mm photo)
cargo run --release -p frame-sheet --bin sheet-spike -- \
  /tmp/out/us-passport.jpg /tmp/out/sheet.jpg 50.8 50.8

# Batch executor over a directory of photos
cargo run --release -p frame-batch --bin batch-spike
```

Each writes output images to `/tmp/out` (or wherever you point it) —
open them and eyeball the result. This project's own history
(`docs/research/05-m0-results.md`) has two real bugs that only showed up
by looking at actual output, not by reading green test results — worth
keeping that habit going.

## Troubleshooting

- **"No space left on device" mid-build**: the Rust target directory
  grows large (10GB+ across debug+release+bundle artifacts). `cargo
  clean` recovers it — Cargo just recompiles from scratch afterward.
- **Model download hangs or times out**: check network/proxy access to
  `github.com` and `huggingface.co`; re-running is safe (see §3).
- **`npm run tauri dev` opens a blank window**: usually means the Vite
  dev server on port 1420 isn't reachable — check nothing else is bound
  to that port, and check the terminal for a Vite error above the Tauri
  window launch.
- **macOS says "OpenPhotoId is damaged and can't be opened"**: this is
  Gatekeeper's quarantine flag on an unsigned app downloaded/copied from
  elsewhere (not "actually damaged") — `xattr -cr` as shown in §5 clears
  it.
- **`cargo test -- --ignored` fails with a checksum mismatch**: delete
  the offending file from the models dir (§3) and re-run; don't ignore a
  checksum failure, it's the download-integrity check working as
  intended.
