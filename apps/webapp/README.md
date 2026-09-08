# OpenPhotoId on the web

The same engine as the desktop app, served as an ordinary web page: the
Rust core compiled to WebAssembly, running entirely in the browser.
Nothing is uploaded — there is no server to upload it to.

```sh
npm --prefix apps/webapp install
npm --prefix apps/webapp run build     # -> apps/webapp/dist
npm --prefix apps/webapp run preview   # http://localhost:8082
```

Serve it over http, not `file://` — a service worker needs an origin. The
output is static; any file host will do.

## Why the web at all

`docs/research/` §八 concluded that mobile is the form this category is
won in, and that the *second* thing to build is "a light responsive web
page, not a desktop client" — because the task-shaped searches this
audience makes ("how to make a passport photo online") land on web pages,
and because the one time a phone-first user reaches for a computer is
filling in a visa or exam form, where a web page is exactly the right
size of tool. This is that page. It is mobile-first; the widest layout
here is a reading column, not a dashboard.

## Why the name

The research's central finding (§九 切入点2) is that the moat is the
ID-photo spec engine — free, complete, and openly documented — not
background removal, which `rembg` made common property years ago. The
product name says the job: "passport photo" is the highest-intent search
term this category has, and it is what the page is for.

The crates stay `frame-*`; everything user-facing is OpenPhotoId since the
September 2026 rename (the page shipped as OpenPassport before that).

## How it is put together

The interesting decision is where the seam falls between Rust and
JavaScript.

`ort`, the desktop's inference backend, has no wasm32 target. The
pure-Rust alternative already wired into `frame-engine` (`tract`) does
build for wasm, but is far too slow for MODNet on a phone. So **the
browser runs the two ONNX graphs in JavaScript**, through
onnxruntime-web, which can reach WebGPU and SIMD that a Rust wasm module
cannot.

Everything else is Rust, and is the same code the desktop runs:

| | |
|---|---|
| decode, EXIF orientation, working-resolution cap | `frame-core` |
| YuNet letterboxing and head decode | `frame-face::preprocess` / `::decode` |
| MODNet normalisation, matte upsample | `frame-matting::modnet` |
| guided-filter refinement, foreground estimation, compositing | `frame-matting` |
| retouch, print sharpening, backdrops, garments | `frame-retouch` |
| spec dataset, crop solver, 11-check validator | `frame-compliance` |
| KB-window JPEG search, PNG with DPI | `frame-core::io` |
| print-sheet layout | `frame-sheet` |

JavaScript's entire share is: hand Rust the bytes, run a tensor through a
model, hand back the output. That split exists so there is exactly one
implementation of the letterboxing, the normalisation constants and the
head decode. A JS reimplementation of those would agree with the Rust one
on the day it was written and drift silently afterwards, and the failure
mode is not a crash — it is compliance checks that quietly stop matching
what the desktop reports for the same photo.

`crates/frame-wasm` is the binding layer; its module docs carry the call
sequence.

## Three ways this differs from the desktop app

**Models are served from this origin.** The desktop downloads weights
from their upstream release URLs on first run. Doing that here would mean
a cross-origin request timed to the moment someone opens their passport
photo, which undercuts the privacy claim even though the image itself
never moves. `scripts/fetch-models.sh` pins the same checksums as
`crates/frame-engine/src/registry.rs` and vendors them into the build.

**The quality tier is a second MODNet pass, not a second model.** The
desktop can fall back to BiRefNet-lite for hard edges; that is 224 MB,
which is not a download to hand a phone, and `MODELS.md` still lists its
inference as unverified. Instead, "extra edge detail" runs MODNet again
over a crop around the head, so hair is resolved at the model's full
input resolution rather than at whatever fraction of it the head happened
to occupy, and merges the two mattes (`Matte::blend_region`). One extra
inference at the same size, no extra weights.

**Batch has no cap and no report file.** It runs in the page, sequential
so peak memory stays flat, and writes the same per-item CSV the desktop
batch writes — as a download rather than to a directory.

## What a visitor actually downloads

| | |
|---|---|
| app JS + CSS | 48 KB gzipped |
| fonts (4 faces, Latin subset, WOFF2) | 68 KB |
| `frame_wasm_bg.wasm` | 1.8 MB (734 KB gzipped) |
| ONNX runtime | 3.6 MB gzipped (CPU) **or** 6.4 MB gzipped (WebGPU) |
| MODNet + YuNet weights | ~26 MB |

Exactly one ONNX runtime is fetched: `src/lib/ort.js` picks the WebGPU
build when the browser has WebGPU and the CPU build when it does not.
Both are shipped, which costs disk on the host and nothing on the wire.

The weights are the bulk, and they are fetched on first use with a real
progress bar rather than precached at first paint — someone reading the
coverage page should not silently pull 26 MB. After that they live in
Cache Storage and a repeat visit needs the network for nothing.

Where those numbers come from, because each was larger once:

- The wasm is built with its own Cargo profile (`wasm-release`: fat LTO,
  one codegen unit, `panic = "abort"`, `opt-level = "s"`) and the `image`
  crate is limited to the formats a photo tool receives. The default
  feature set compiled an AVIF encoder, OpenEXR and half a dozen other
  codecs into the browser build: 2.8 MB → 1.8 MB.
- The fonts are the same Geist faces the desktop ships, subset to Latin
  and converted to WOFF2 by `scripts/subset-fonts.sh`: 660 KB → 68 KB.
  Geist has no CJK glyphs, so the Chinese UI used the system font anyway.
- `wasm-opt` (`brew install binaryen`) runs when installed; the build
  passes it the feature flags Rust's wasm target now emits by default,
  without which a current binaryen refuses the module outright.

The CPU path runs onnxruntime multi-threaded from the second visit: the
service worker adds the cross-origin-isolation headers a static host
cannot, and `src/lib/ort.js` asks for threads once the page reports
itself isolated. The first visit is single-threaded by construction —
the worker that installs on a page does not control it.

Serve it with compression.

Screenshots of the built app are in [`screenshots/`](screenshots/),
captured by `e2e/capture.mjs` driving the real UI rather than drawn by hand.

## Testing

```sh
npm --prefix apps/webapp run build    # e2e and screenshots run against dist/
npm --prefix apps/webapp run e2e      # Playwright, serves dist/ on :5195
npm --prefix apps/webapp run screenshots
```

Always the full build: `vite build` alone empties `dist/` and does not
put the ONNX runtime, the weights or the service worker back, so the app
looks built and every photo then fails to load.

## Layout

| | |
|---|---|
| `src/lib/pipeline.js` | the sequence: which Rust call, which model run, in what order |
| `src/lib/ort.js` | runtime selection and the WebGPU→CPU fallback |
| `src/lib/models.js` | weight fetching, caching, progress |
| `src/lib/i18n.js` | English and Simplified Chinese |
| `src/views/` | the six screens |
| `src/ui/` | local primitives — see `Icon.svelte` for why the design kit's is not used |
| `scripts/` | wasm build, model fetch, site assembly |

The OpenApps design tokens are imported from `apps/desktop/src/design`
rather than copied. The kit's Svelte components are not, because its
`Icon` fetches glyphs from a CDN and this build has to render with no
network at all.
