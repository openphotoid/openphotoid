#!/usr/bin/env bash
# Assembles the whole static site into apps/webapp/dist.
#
# Output is a plain folder of files: any static host will serve it, and
# there is no server component to run, which is the same fact the product
# claims on its privacy page.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WEBAPP_DIR="$(dirname "$SCRIPT_DIR")"
ROOT="$(cd "$WEBAPP_DIR/../.." && pwd)"
DIST="$WEBAPP_DIR/dist"

log() { printf '\033[1m==> %s\033[0m\n' "$1"; }

log "Building the Rust core for wasm32"
bash "$SCRIPT_DIR/build-wasm.sh"

log "Fetching the ONNX weights"
bash "$SCRIPT_DIR/fetch-models.sh"

log "Bundling the app"
# The catalogues are checked before the bundle is built: a locale in the
# picker with a gap in its catalogue would otherwise fall back to English
# silently, and nothing at runtime would say so.
(cd "$WEBAPP_DIR" && node scripts/check-i18n.mjs)
(cd "$WEBAPP_DIR" && npx vite build)

[ -f "$DIST/index.html" ] || { echo "build.sh: vite produced no index.html" >&2; exit 1; }

# The ONNX runtime binaries. onnxruntime-web resolves these at runtime
# from `ort.env.wasm.wasmPaths` (see src/lib/ort.js), so no bundler ever
# sees them and nothing copies them unless this does.
#
# Both are shipped and exactly one is fetched per visitor: the app picks
# the WebGPU build when the browser has WebGPU and the CPU build when it
# does not. That costs disk on the server, not bandwidth for the user.
log "Copying the ONNX runtime"
mkdir -p "$DIST/ort"
ORT_DIST="$WEBAPP_DIR/node_modules/onnxruntime-web/dist"
# The .mjs glue as well as the .wasm: vite.config.js resolves the
# non-bundled onnxruntime builds, which load both from `wasmPaths`.
#
# Which pair each build wants is not guessable and changed between
# onnxruntime versions — 1.29's WebGPU build asks for the *asyncify*
# runtime, not the `jsep` one the file names suggest. These four names are
# read straight out of the two bundles below; if a future upgrade renames
# them, the check underneath fails loudly instead of at runtime in
# someone's browser.
for f in \
  ort-wasm-simd-threaded.wasm ort-wasm-simd-threaded.mjs \
  ort-wasm-simd-threaded.asyncify.wasm ort-wasm-simd-threaded.asyncify.mjs; do
  [ -f "$ORT_DIST/$f" ] || { echo "build.sh: missing $ORT_DIST/$f — run npm install" >&2; exit 1; }
  cp "$ORT_DIST/$f" "$DIST/ort/$f"
done

# Nothing should have emitted a copy of these into assets/. If something
# does again, it is 40 MB of dead weight that no code path ever loads, so
# fail rather than ship it quietly.
if compgen -G "$DIST/assets/ort-wasm-*.wasm" >/dev/null; then
  echo "build.sh: onnxruntime wasm leaked into dist/assets — check the aliases in vite.config.js" >&2
  exit 1
fi

# Every runtime the two bundles actually name must be present. This is
# what turns "the WebGPU path 404s on a visitor's machine" into a build
# failure here.
for bundle in ort.webgpu.min.mjs ort.wasm.min.mjs; do
  for needed in $(grep -o 'ort-wasm-simd-threaded[a-z.]*\.mjs' "$ORT_DIST/$bundle" | sort -u); do
    [ -f "$DIST/ort/$needed" ] || {
      echo "build.sh: $bundle needs ort/$needed, which was not copied" >&2
      exit 1
    }
    wasm="${needed%.mjs}.wasm"
    [ -f "$DIST/ort/$wasm" ] || {
      echo "build.sh: $bundle needs ort/$wasm, which was not copied" >&2
      exit 1
    }
  done
done

log "Copying the models"
mkdir -p "$DIST/models"
for f in modnet_photographic_portrait_matting.onnx face_detection_yunet_2023mar.onnx; do
  src="$ROOT/.vendor/models/$f"
  [ -f "$src" ] || { echo "build.sh: missing $src — run scripts/fetch-models.sh" >&2; exit 1; }
  cp "$src" "$DIST/models/$f"
done

# The manifest and icons come through public/, so they keep their names.
# The service worker precaches ./manifest.webmanifest by that exact path;
# a content-hashed copy would make its install step fail, and offline
# support would disappear without anything looking broken.
[ -f "$DIST/manifest.webmanifest" ] || {
  echo "build.sh: dist/manifest.webmanifest missing — is it still in public/?" >&2
  exit 1
}

# The service worker's cache name carries a digest of everything in dist/,
# so it changes exactly when the build does. Deriving it beats bumping a
# version string by hand in both directions: a rebuild of an unchanged
# version can't serve the previous build's index.html forever, and a
# rebuild that changed nothing can't pointlessly evict a returning
# visitor's tens of megabytes of cached wasm and weights.
#
# Hashed from *inside* dist/ so the paths that reach the digest are
# relative — otherwise moving the checkout produces a brand-new id for a
# byte-identical build.
log "Stamping the service worker"
BUILD_ID=$(
  cd "$DIST" &&
    find . -type f -print0 |
    LC_ALL=C sort -z |
    xargs -0 shasum -a 256 |
    shasum -a 256 |
    cut -c1-12
)
sed "s/__BUILD_ID__/$BUILD_ID/" "$WEBAPP_DIR/service-worker.js" > "$DIST/service-worker.js"
if grep -q "__BUILD_ID__" "$DIST/service-worker.js"; then
  echo "build.sh: service-worker.js still has an unstamped __BUILD_ID__" >&2
  exit 1
fi

log "Registering the service worker"
node - "$DIST/index.html" <<'NODE'
import { readFileSync, writeFileSync } from "node:fs";
const file = process.argv[2];
const html = readFileSync(file, "utf8");
if (html.includes("service-worker.js")) process.exit(0);
if (!html.includes("</head>")) {
  console.error("build.sh: no </head> in index.html");
  process.exit(1);
}
// Injected here rather than in index.html, because registering a service
// worker only makes sense for the hosted build — `vite dev` would keep
// serving a stale cached bundle behind every edit.
const inject = `
<script>
  window.__installPrompt = null;
  addEventListener("beforeinstallprompt", (e) => {
    e.preventDefault();
    window.__installPrompt = e;
    dispatchEvent(new Event("openphotoid:installable"));
  });
  if ("serviceWorker" in navigator) {
    addEventListener("load", () => {
      navigator.serviceWorker.register("./service-worker.js").catch(() => {});
    });
  }
<\/script>`;
writeFileSync(file, html.replace("</head>", inject + "\n</head>"));
NODE

log "Done"
echo "  output: $DIST"
echo "  size:   $(du -sh "$DIST" | cut -f1)"
echo
echo "Serve it (a service worker needs an http origin, not file://):"
echo "  npm --prefix apps/webapp run preview   # http://localhost:8082"
