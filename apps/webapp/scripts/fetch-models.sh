#!/usr/bin/env bash
# Fetches the ONNX weights into .vendor/models/ and verifies their
# checksums, so the web build can serve them from its own origin.
#
# Same files, same pins, same licences as the desktop registry
# (crates/frame-engine/src/registry.rs and MODELS.md) — deliberately not a
# second list of URLs that can drift from that one. Only commercially
# clean weights are permitted here; see MODELS.md for the rule.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
DEST="$ROOT/.vendor/models"
mkdir -p "$DEST"

# name|url|sha256
MODELS=(
  "modnet_photographic_portrait_matting.onnx|https://github.com/Zeyi-Lin/HivisionIDPhotos/releases/download/pretrained-model/modnet_photographic_portrait_matting.onnx|07c308cf0fc7e6e8b2065a12ed7fc07e1de8febb7dc7839d7b7f15dd66584df9"
  "face_detection_yunet_2023mar.onnx|https://media.githubusercontent.com/media/opencv/opencv_zoo/main/models/face_detection_yunet/face_detection_yunet_2023mar.onnx|8f2383e4dd3cfbb4553ea8718107fc0423210dc964f9f4280604804ed2552fa4"
)

sha() { shasum -a 256 "$1" | cut -d' ' -f1; }

for entry in "${MODELS[@]}"; do
  IFS='|' read -r name url want <<<"$entry"
  path="$DEST/$name"
  if [ -f "$path" ] && [ "$(sha "$path")" = "$want" ]; then
    echo "  ok   $name (cached)"
    continue
  fi
  echo "  get  $name"
  # Download to .part and only move it into place once the checksum
  # passes, so an interrupted fetch can never leave a truncated model that
  # looks cached on the next run.
  curl -fL --progress-bar -o "$path.part" "$url"
  got="$(sha "$path.part")"
  if [ "$got" != "$want" ]; then
    rm -f "$path.part"
    echo "fetch-models.sh: checksum mismatch for $name" >&2
    echo "  expected $want" >&2
    echo "  got      $got" >&2
    exit 1
  fi
  mv "$path.part" "$path"
  echo "  ok   $name"
done

echo "Models in $DEST"
