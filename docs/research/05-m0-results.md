# M0 validation spike results

Date: 2026-08-01. Hardware caveat: all numbers are from a **macOS VM on a
shared-folder mount** — treat CPU numbers as a pessimistic floor and GPU/ANE
numbers as unrepresentative (VMs get no ANE and limited GPU); re-run on real
hardware (M-series Mac + a Windows box) before locking perf budgets.

## M0b — ort inference (MODNet, 512×512 input)

Test image: `testdata/portrait-obama.jpg` (2687×3356, public domain).

| Path | Session load | Warm inference (median of 3) |
|---|---|---|
| ort 2.0.0-rc.10, CPU | ~36 ms (model cached) | **~231–316 ms** |
| ort + CoreML EP | ~176 ms | ~247–296 ms (no speedup — VM has no ANE; likely full CPU fallback) |

- End-to-end (decode → 512 resize → infer → bilinear alpha upsample to 9 MP →
  composite → encode) is ~1 s in release mode: **within the "< 1 s single
  photo" budget on CPU alone.** ✅
- Cutout quality on the test portrait: clean subject/hair separation; faint
  background-color fringe at sleeve edges — exactly what the M2 refinement
  chain (guided filter + foreground estimation) addresses.
- BiRefNet-lite not yet benchmarked (176 MB download deferred); it is the M2
  quality-tier task and needs real-hardware timings anyway.
- ort quirk vs research notes: rc.10's `try_extract_tensor` returns
  `(shape, &[f32])`, and `ort::inputs!` is infallible — minor API churn,
  confirms the "pin exact rc" policy.

## M0c — YuNet + compliance auto-crop

| Step | Time |
|---|---|
| YuNet session load | 612 ms (first run) |
| Detect (640 letterbox) | **61 ms** |

- 1 face, score 0.947, roll −1.35°, 5 landmarks all plausible.
- US-passport solve landed exactly mid-band: head 59.5% (spec 50–69%),
  eye line 62.5% from bottom (spec 56–69%). ✅
- The solver correctly flagged **out-of-bounds top** (crop extends above the
  source frame) — confirms M4 needs background-extension padding for tight
  headshots; the spike clamps instead.

## M0a — Tauri 2 + Svelte shell

- `apps/desktop`: Svelte 5 + Vite 6 frontend builds (33 KB JS bundle);
  `openphotoid-desktop` (Tauri 2, `protocol-asset` feature) compiles clean on
  macOS.
- Spike page: synthetic 40 MP (8000×5000) image, wheel-zoom/drag-pan via CSS
  transform, before/after clip-path slider, FPS meter.
- **Interactive go/no-go still needs a human eyeball** — run
  `cd apps/desktop && npm run tauri dev`, then pan/zoom and watch the FPS
  counter (target: ≥ 30 fps while dragging at any zoom). Windows WebView2
  run must wait for CI/a Windows box.

## Environment notes

- Shared-mount builds require `CARGO_TARGET_DIR` off the mount
  (`~/.cache/openphotoid-target`) — same as the other openapps projects.
- Local disk filled up mid-build (59 GB VM disk); reclaimed by deleting the
  regenerable `openapps-target`/fuzz cargo caches + Homebrew cache. Watch this.
- Tauri needs `icons/icon.png` even for plain `cargo build`
  (placeholder generated); real icon set is an M7 task.

## M2 addendum — guided-filter regression caught by eyeballing (2026-08-01)

The initial guided-filter refinement (cascaded radii `[8,5,2]`, scaled with
source resolution) **passed all unit tests but produced a visible halo
around hair** on the real 2687×3356 test portrait, plus made the composite
look worse than the unrefined bilinear-upsample baseline. Root cause:
bilinear upsampling from model resolution (512/1024) is already smooth, so
the true misalignment it needs correcting is a small, fixed few-pixel
effect — NOT proportional to source resolution. Scaling radius with image
size (as the original design assumed) made the filter treat individual hair
strands as texture to smooth away once radius exceeded ~2-3px.

Diagnosis method: swept radius 1/2/4/8 directly on the real photo (not the
tiny synthetic test fixture) — clean at 1-2, visible halo starting at 4,
severe at 8. The unit tests never caught this because a 64×32 synthetic
fixture makes an 8px radius look proportionally tiny; **synthetic fixture
tests did not substitute for eyeballing real output.** Fixed default:
single pass, `radius=2` (constant, not resolution-scaled), `eps=1e-4`.
Verified clean at both full res (2687×3356) and a downscaled 336×420 crop.

Separately noted (not a regression): a thin red sliver near the left
shoulder in all versions, including the unrefined baseline — a MODNet
segmentation gap letting the background curtain show through a small mask
hole. This is a matting-quality limitation to track, not something the
guided filter introduced or can fix (it operates on alpha, not on holes
inside high-alpha regions).

## M2 addendum — BiRefNet-lite blocked by shared-VM memory (2026-08-01)

`BiRefNetLite` (1024×1024 input, ImageNet-norm preprocessing, sigmoid
applied when outputs fall outside [0,1]) is implemented in
`crates/frame-matting/src/birefnet.rs` and the 224 MB ONNX export
downloads and checksums cleanly. **Running inference on this shared dev VM
gets OOM-killed (exit 137/SIGKILL)**, both at the default thread count and
retried single-threaded (`EngineSession::load_with_threads`, added to rule
out ORT's per-thread arena overhead as the cause — same result either way).
`vm_stat`/`ps aux -m` showed the 8 GB VM had only ~150 MB free with several
concurrent Claude Code sessions running; this is a host memory ceiling, not
a bug in the model wrapper.

This matches the research's standing warning (docs/research/02, 03:
"BiRefNet CPU inference … might go on indefinitely") and was already a
carried-forward M0 risk ("real-hardware EP benchmarks", "BiRefNet-lite
bench" — now confirmed environment-blocked rather than merely deferred).
**Not retried further** to avoid destabilizing other sessions sharing this
VM. Action: benchmark BiRefNet-lite on real end-user-class hardware (a
Mac with several GB headroom, or CI with a dedicated runner) before
enabling it as a shipped "HD" tier; MODNet remains the validated default.

## M4 addendum — validation checklist, two more bugs caught by real photos (2026-08-01)

Built `frame-compliance::validate` (11-check pass/warn/fail checklist —
face count, head-height, eye-line, centering, roll, inter-eye distance,
Laplacian blur, lighting symmetry, background uniformity/color via CIE
Lab ΔE76, exact resolution, JPEG file-size-window fit) plus
`crop::apply_crop_padded` (extends out-of-bounds crops with the spec's
background color instead of clamping/distorting). 17 unit tests pass on
synthetic fixtures, but **running the full pipeline end-to-end on the real
test portrait caught two design bugs synthetic tests missed**:

1. **Background-uniformity sampling assumed all 4 image corners are
   background.** On a real head-and-shoulders crop the subject's shoulders
   routinely reach the bottom two corners (confirmed visually — Obama's
   suit fills both bottom corners of the US-passport crop). Fixed: sample
   only the strip above the detected head, never the corners generically.
2. **That strip's cutoff (the face detector's bbox top) still caught hair
   pixels.** YuNet's bbox approximates the forehead/hairline, not the true
   crown, and grey/fluffy hair extends visibly above it — inflating stddev
   even though the mean color stayed near-white (a flat-background
   synthetic fixture cannot exercise this: it has no hair). Fixed: back
   off the sample cutoff by 25% of estimated head height.

After both fixes, the same real photo scores background_uniformity
`stddev 0.0, ΔE 0.0` (was `stddev 103.7, ΔE 35.5` → Fail) across white,
grey, and tight-crop specs, with tight-crop specs (India Seva, UK — no
safe strip above a near-full-frame head) correctly degrading to
`NotChecked` rather than false-failing. **Lesson reinforced from the M2
guided-filter bug: synthetic fixtures validate an algorithm's math; only a
real photo validates the algorithm's assumptions about real photos.**

## M6 addendum — batch executor, and a self-inflicted timing scare (2026-08-01)

Built `frame-batch::BatchExecutor`: one shared `Mutex<Modnet>` +
`Mutex<YuNet>` (not one model instance per worker thread — deliberately,
to avoid multiplying resident model memory across workers after the M2
BiRefNet-lite OOM finding above), with `rayon` parallelizing everything
else (decode, guided-filter refine, foreground estimation, crop, encode).

Initial timing looked alarming: a single item through
`BatchExecutor::run` took **22-30 seconds**, vs ~3s for the equivalent
work in the `crop-spike` binary. Chased this down two blind alleys first
— VM memory pressure (real, but not the cause here: re-checked with
`vm_stat` and it had eased without the timing improving) and rayon
test-thread contention (ruled out by forcing `--test-threads=1`) — before
finding the actual cause: **the integration test was run via
`cargo test` without `--release`**, while every prior spike binary had
been run with `cargo run --release`. Debug-mode nested loops (guided
filter's box-blur passes, three-pass foreground estimation) are exactly
the code shape that gets 5-10x slower unoptimized. With `--release`: a
single item drops to **2.0s** (matching the standalone spike) and a
4-item parallel batch completes in **4.7s** — real speedup from sharing
just two model locks across 8 cores, not the ~8s a fully-serial run would
take. **Lesson: benchmark release builds, always** — `cargo test`
defaults to debug and will produce misleading numbers for anything
compute-heavy.

## M5 addendum — desktop app, and a genuine environment limit (2026-08-01)

Built the real single-photo flow: Tauri commands (`list_specs`,
`process_photo`) wrapping matting → spec-driven crop → validate, a Svelte
UI (drag-drop, spec picker, before/after, color-coded checklist, JPEG
download), with the M0 canvas-perf spike preserved as a second tab
(`CanvasSpike.svelte`). Photos cross the IPC boundary as base64 JSON
(not raw pixels/paths) since a sandboxed webview's `<input type="file">`
has no real OS path, and compliance JPEGs are at most a few hundred KB —
a different tradeoff than the 40MP-canvas case the M0 spike was about.

**Tried to visually click-test it and hit a real infrastructure wall**:
this dev VM has no attached display — `system_profiler SPDisplaysDataType`
reports nothing and `screencapture` fails with "could not create image
from display" at the CGDisplay level, even though a full GUI login
session exists (`launchctl print gui/501` shows 431 services, `who` shows
an active console session). This is unlike the earlier memory/OOM
findings — not something more RAM or a smaller model fixes; there is
simply no framebuffer to screenshot. Confirmed: the Tauri binary compiles
(`cargo build -p openphotoid-desktop`) and **launches without crashing**
even in this display-less environment (process stays alive, PID stable),
and the Vite dev server correctly serves the Svelte app (`HTTP 200`,
correct `<title>`, correct JS bundle) — but the actual native window's
pixels can't be captured here, and the Tauri IPC bridge
(`window.__TAURI_INTERNALS__`) only exists inside that real window, so it
can't be driven from an external headless browser either.

Given that wall, verified everything reachable without a display:
refactored `process_photo` into a thin `#[tauri::command]` wrapper over
`process_photo_inner(&AppState, ...)` (plain function, no `tauri::State`
dependency-injection requirement) and wrote a real integration test that
mirrors the exact browser round-trip — reads the test portrait, base64
data-URLs it exactly like `FileReader.readAsDataURL` would, strips the
prefix exactly like the frontend's `.split(",").pop()`, and calls the
real command logic. It decodes, runs the actual matting+crop+validate
pipeline, and returns a valid 600×600 JPEG with a passing checklist —
proving the base64/JSON glue (the one part not already covered by the
crop-spike/batch-spike binaries) is correct. `list_specs()` (no models
needed) and the unknown-spec error path are covered too.
**Still outstanding, and only possible on real hardware**: an actual
click-through — drop a file, watch the before/after render, read the
checklist on screen. Recommend the user run `cd apps/desktop && npm run
tauri dev` on their own Mac/PC for that final visual pass before relying
on the UI.

## Verdict

Go. All three spike areas pass on this environment's evidence: models run
fast enough on CPU alone, the compliance geometry solver works on the first
try, and the full Rust+Tauri toolchain builds. Remaining M0 checkboxes
(human canvas-perf eyeball, real-hardware EP benchmarks, Windows build) do
not block starting M1/M2 — they're carried as risks in PLAN.md §7.
