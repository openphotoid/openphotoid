# Model manifest

Every ONNX weight OpenPhotoId downloads must be listed here with license and
provenance (PLAN.md §4). Non-commercial weights (BRIA RMBG, insightface,
MatAnyone, Adobe-DIM-trained matting checkpoints) are banned.

| Model | File | License | Source | SHA-256 | Status |
|---|---|---|---|---|---|
| MODNet photographic portrait matting | `modnet_photographic_portrait_matting.onnx` (24.7 MB) | Apache-2.0 (MODNet; weights redistributed by HivisionIDPhotos) | https://github.com/Zeyi-Lin/HivisionIDPhotos/releases/tag/pretrained-model | `07c308cf0fc7e6e8b2065a12ed7fc07e1de8febb7dc7839d7b7f15dd66584df9` | shipped (fast default) |
| YuNet face detection (2023mar) | `face_detection_yunet_2023mar.onnx` (227 KB) | MIT (OpenCV Zoo) | https://github.com/opencv/opencv_zoo/tree/main/models/face_detection_yunet | `8f2383e4dd3cfbb4553ea8718107fc0423210dc964f9f4280604804ed2552fa4` | shipped (detector) |
| BiRefNet-lite (ONNX community export) | `birefnet_lite.onnx` (224 MB) | MIT (BiRefNet) | https://huggingface.co/onnx-community/BiRefNet_lite-ONNX | `5600024376f572a557870a5eb0afb1e5961636bef4e1e22132025467d0f03333` | code shipped, **inference unverified** — OOM on shared dev VM (docs/research/05-m0-results.md); benchmark on real hardware before enabling as default |
| MODNet, tract-compatible variant (mobile candidate) | `modnet_photographic_portrait_matting.tract.onnx` (24.7 MB) | Apache-2.0 (same weights, mechanically patched — see below) | derived from the row above via `scripts/patch-modnet-for-tract.py` (not a separate download; run the script to reproduce) | `a09a06eebfbe75e3c7bf0241c4cc26b2709d55ae5193277c6812ce98b3a0447f` (output of the script as committed 2026-08-02) | **exploratory, not wired into the model registry or `EngineSession` yet** (PLAN.md §M9) — validated to run under `tract` and match the original `ort` output (mean abs. alpha diff 0.000179 on `testdata/portrait-obama.jpg`), but not yet an actual runtime code path |
