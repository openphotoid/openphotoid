#!/usr/bin/env python3
"""Patch MODNet's ONNX graph so it loads under `tract` (mobile inference
backend candidate — `ort`'s prebuilt onnxruntime binaries don't cover
Android/iOS; see docs/research/ for the investigation).

Two independent tract incompatibilities in the stock MODNet export
(HivisionIDPhotos' Apache-2.0 weights, see MODELS.md), both in this
script's scope:

1. All 9 `Resize` nodes set coordinate_transformation_mode=
   "pytorch_half_pixel", which tract-onnx 0.21 doesn't implement. Per the
   ONNX spec, pytorch_half_pixel differs from half_pixel only when an
   output spatial dimension is 1 — never true here (MODNet's decoder only
   upsamples to larger feature maps). Rewritten to "half_pixel", which is
   supported and numerically identical for this model's actual shapes.
2. The graph's input dims (batch_size, height, width) are left symbolic,
   which tract's shape inference can't carry through a fractional-scale
   Resize. frame-matting already fixes MODNet's input to 512x512 before
   inference (crates/frame-matting/src/modnet.rs) — resolving the input
   shape here just makes that existing fixed-size contract explicit in
   the graph itself, consistent with the "fixed-shape exports" mitigation
   already used for CoreML/DirectML (PLAN.md's risk table).

Validated: comparing this patched model (tract) against the original
(ort) on testdata/portrait-obama.jpg, at the raw 512x512 alpha stage —
mean absolute difference 0.000179, max 0.0818 (single-pixel edge cases),
foreground coverage 50.965% (ort) vs 50.962% (tract). Effectively
equivalent output.

Usage:
    pip install onnx
    python3 scripts/patch-modnet-for-tract.py \
        <path to cached modnet_photographic_portrait_matting.onnx> \
        <output path>

The input model is the same file frame_engine::registry::MODNET_PHOTOGRAPHIC
downloads and checksums — not committed to this repo (see MODELS.md for
why: models are fetched at runtime, not stored in git). This script is
what's committed; run it to reproduce the patched model, then record its
SHA-256 in MODELS.md when it's wired into a mobile build.
"""

import sys

import onnx
from onnx import shape_inference


def patch(src_path: str, dst_path: str) -> None:
    model = onnx.load(src_path)

    resize_nodes = [n for n in model.graph.node if n.op_type == "Resize"]
    patched = 0
    for node in resize_nodes:
        for attr in node.attribute:
            if attr.name == "coordinate_transformation_mode" and attr.s == b"pytorch_half_pixel":
                attr.s = b"half_pixel"
                patched += 1
    print(f"Rewrote coordinate_transformation_mode on {patched}/{len(resize_nodes)} Resize nodes")

    # Fix the input to 512x512x3 (frame-matting's existing preprocessing
    # contract) so tract's shape inference can resolve every downstream
    # fractional-scale Resize instead of failing on symbolic dims.
    inp = model.graph.input[0]
    dims = inp.type.tensor_type.shape.dim
    dims[0].dim_value = 1
    dims[2].dim_value = 512
    dims[3].dim_value = 512
    print(f"Fixed input shape to {[d.dim_value for d in dims]}")

    onnx.checker.check_model(model)
    shape_inference.infer_shapes(model)  # raises if the graph is now inconsistent
    onnx.save(model, dst_path)
    print(f"Saved patched model to {dst_path}")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print(__doc__)
        sys.exit(1)
    patch(sys.argv[1], sys.argv[2])
