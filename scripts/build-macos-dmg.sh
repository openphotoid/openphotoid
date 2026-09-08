#!/usr/bin/env bash
# Build OpenPhotoId's macOS .dmg on real hardware. The dev VM this repo is
# normally built in can't finish this build: bundling fails there with
# "Failed to create Info.plist: Inappropriate ioctl for device" once the
# target dir sits on the shared-folder mount, and the VM has no attached
# display to ever click-test the result. Run this on an actual Mac instead.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
desktop_dir="$repo_root/apps/desktop"

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "error: this builds a macOS .dmg and must run on macOS." >&2
  exit 1
fi

command -v cargo >/dev/null 2>&1 || {
  echo "error: cargo not found. Install Rust: https://rustup.rs" >&2
  exit 1
}
command -v npm >/dev/null 2>&1 || {
  echo "error: npm not found. Install Node.js: https://nodejs.org" >&2
  exit 1
}

# CoreML is macOS-only and off by default (the shared Cargo.toml has to
# stay buildable on Windows/Linux too) — enabled explicitly here since
# this script only ever runs on a real Mac. See the "coreml" feature in
# apps/desktop/src-tauri/Cargo.toml / crates/frame-engine/Cargo.toml.
features="coreml"

echo "==> $(uname -m) macOS $(sw_vers -productVersion 2>/dev/null || echo unknown)"
echo "==> Features: $features"
echo

cd "$desktop_dir"
npm install
npm run tauri build -- --features "$features"

bundle_dir="$repo_root/target/release/bundle"
app="$bundle_dir/macos/OpenPhotoId.app"
dmg="$(find "$bundle_dir/dmg" -maxdepth 1 -name '*.dmg' 2>/dev/null | head -1)"

echo
if [[ ! -d "$app" ]]; then
  echo "error: build finished but no OpenPhotoId.app was found under $bundle_dir/macos" >&2
  exit 1
fi
if [[ -z "${dmg:-}" ]]; then
  echo "error: build finished but no .dmg was found under $bundle_dir/dmg" >&2
  exit 1
fi

# target/release/bundle mixes the real deliverables with build-only
# clutter (bundle_dmg.sh, a loose icon.icns, a stray temp .dmg, a
# share/create-dmg dir) — stage just what's needed for local
# installation into one clean folder instead of pointing at that mess.
dist_dir="$repo_root/dist/macos"
rm -rf "$dist_dir"
mkdir -p "$dist_dir"
cp -R "$app" "$dist_dir/"
cp "$dmg" "$dist_dir/"

echo "==> Ready for local installation in: $dist_dir"
echo "      OpenPhotoId.app                 (run in place, or drag to /Applications)"
echo "      $(basename "$dmg")  (double-click to mount, then drag the app in)"
echo
echo "This build is unsigned — no Apple Developer ID is configured in"
echo "tauri.conf.json — so Gatekeeper will refuse to open it normally."
echo "For local testing: right-click OpenPhotoId.app -> Open, then confirm"
echo "in the dialog (only needed the first time)."
