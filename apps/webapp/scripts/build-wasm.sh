#!/usr/bin/env bash
# Builds crates/frame-wasm for the browser and emits the JS glue into
# apps/webapp/src/wasm/, which pipeline.js imports.
#
# Two things here are load-bearing and easy to get wrong:
#
#   --no-default-features  turns off `inference`, which is what keeps
#   `ort` (no wasm32 target, needs a C++ toolchain) out of the graph. See
#   crates/frame-wasm/src/lib.rs for why the browser runs ONNX in JS.
#
#   -p frame-wasm rather than --workspace: cargo unifies features across
#   the *selected* members, so building the whole workspace for wasm32
#   would switch `inference` back on through the desktop crates and fail.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WEBAPP_DIR="$(dirname "$SCRIPT_DIR")"
ROOT="$(cd "$WEBAPP_DIR/../.." && pwd)"
OUT="$WEBAPP_DIR/src/wasm"

: "${CARGO_TARGET_DIR:=$HOME/.cache/openphotoid-target}"
export CARGO_TARGET_DIR

log() { printf '\033[1m==> %s\033[0m\n' "$1"; }

if ! rustup target list --installed | grep -qx wasm32-unknown-unknown; then
  log "Adding the wasm32-unknown-unknown target"
  rustup target add wasm32-unknown-unknown
fi

if ! command -v wasm-bindgen >/dev/null 2>&1; then
  echo "build-wasm.sh: wasm-bindgen CLI not found. Install it with:" >&2
  echo "  cargo install wasm-bindgen-cli" >&2
  exit 1
fi

log "Compiling frame-wasm for wasm32"
# SIMD is what makes the image work (resampling, the guided filter, the
# foreground estimator) tolerable in a browser; every target the app
# supports has had it for years.
RUSTFLAGS="${RUSTFLAGS:-} -C target-feature=+simd128" \
  cargo build \
  --manifest-path "$ROOT/Cargo.toml" \
  -p frame-wasm \
  --target wasm32-unknown-unknown \
  --profile wasm-release \
  --no-default-features

log "Generating the JS bindings"
rm -rf "$OUT"
wasm-bindgen \
  --target web \
  --out-dir "$OUT" \
  --no-typescript \
  "$CARGO_TARGET_DIR/wasm32-unknown-unknown/wasm-release/frame_wasm.wasm"

if command -v wasm-opt >/dev/null 2>&1; then
  log "Optimising with wasm-opt"
  # Rust's wasm32 target emits bulk-memory, sign-ext, non-trapping
  # float-to-int, mutable globals, reference types and multivalue by
  # default now, and binaryen validates strictly: without these flags a
  # current wasm-opt refuses the module ("memory.copy operations require
  # bulk memory") and the build fails. Listed explicitly rather than
  # --all-features so the optimiser cannot emit anything a browser lacks.
  wasm-opt -Oz \
    --enable-simd --enable-bulk-memory --enable-nontrapping-float-to-int \
    --enable-sign-ext --enable-mutable-globals --enable-reference-types \
    --enable-multivalue \
    "$OUT/frame_wasm_bg.wasm" -o "$OUT/frame_wasm_bg.wasm"
else
  echo "  (wasm-opt not installed — skipping size optimisation, output is ~30% larger)"
fi

log "Done: $OUT"
ls -la "$OUT"
