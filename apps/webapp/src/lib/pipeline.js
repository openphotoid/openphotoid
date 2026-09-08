/**
 * The pipeline, as the browser runs it.
 *
 * Almost nothing happens here. The Rust core (`crates/frame-wasm`) owns
 * every stage — decode, letterboxing, matte upsampling, refinement,
 * compositing, the crop solver, the checklist, the KB-window JPEG search
 * — and this file does the two things Rust in a browser cannot: run the
 * ONNX graphs, and await. That split is deliberate; see the crate docs
 * for why the alternative (reimplementing pre/post-processing in JS)
 * would be a slow-motion correctness bug.
 *
 * Read the sequence in `run()` alongside `Session`'s doc comment in Rust
 * — they are the same list.
 */

import { createSession, notes as sessionNotes } from "./ort.js";
import { fetchModel } from "./models.js";
import { isNative, NativePhoto, nativeSpecs, nativeInfo } from "./platform.js";

/**
 * Stage timings as `performance.measure` entries named `openphotoid:<stage>`.
 * They show in a browser's Performance panel and the e2e suite reads them
 * back, so the numbers in the test guide are the ones the page measured.
 */
async function timed(stage, work) {
  const start = performance.now();
  try {
    return await work();
  } finally {
    performance.measure(`openphotoid:${stage}`, { start, end: performance.now() });
  }
}

let wasm;
let faceModel;
let mattingModel;

/** The wasm module, instantiated once. */
export async function loadCore() {
  wasm ??= import("../wasm/frame_wasm.js").then(async (m) => {
    await m.default();
    return m;
  });
  return wasm;
}

/**
 * Load both models. `onProgress({ id, fraction })` reports the download.
 *
 * Sessions are kept for the life of the page: creating one costs more
 * than running it, and a batch of thirty photos must not pay that thirty
 * times.
 */
export async function loadModels(onProgress) {
  if (isNative) {
    // The phone app ships its models and runs them in Rust; nothing to fetch.
    const info = await nativeInfo();
    globalThis.__openphotoid = { provider: info.provider, threads: info.threads, notes: [] };
    onProgress?.({ id: "matting", fraction: 1 });
    return { core: null, provider: info.provider };
  }
  const core = await loadCore();
  if (!faceModel) {
    const bytes = await fetchModel("face", (f) => onProgress?.({ id: "face", fraction: f }));
    faceModel = await createSession(bytes);
  }
  if (!mattingModel) {
    const bytes = await fetchModel("matting", (f) => onProgress?.({ id: "matting", fraction: f }));
    mattingModel = await createSession(bytes);
  }
  // Which provider ran, readable from the console and by the e2e suite.
  globalThis.__openphotoid = { provider: mattingModel.provider, threads: mattingModel.threads, notes: sessionNotes };
  return { core, provider: mattingModel.provider };
}

/** YuNet's three strides, in the order the graph names its outputs. */
const STRIDES = [8, 16, 32];

async function detectFaces(session, ort, input, heads) {
  const tensor = new ort.Tensor("float32", input, [1, 3, 640, 640]);
  const out = await session.run({ [session.inputNames[0]]: tensor });
  for (const stride of STRIDES) {
    heads.push(
      stride,
      out[`cls_${stride}`].data,
      out[`obj_${stride}`].data,
      out[`bbox_${stride}`].data,
      out[`kps_${stride}`].data,
    );
  }
}

async function runMatting(session, ort, input, size) {
  const tensor = new ort.Tensor("float32", input, [1, 3, size, size]);
  const out = await session.run({ [session.inputNames[0]]: tensor });
  // MODNet exports vary in what they call the output; take the first one
  // that is the right length rather than hard-coding a name that differs
  // between the several MODNet exports in circulation.
  const wanted = size * size;
  for (const key of session.outputNames) {
    const t = out[key];
    if (t && t.data.length === wanted) return t.data;
  }
  const first = out[session.outputNames[0]];
  throw new Error(
    `matting output was ${first?.data.length ?? "absent"} values, expected ${wanted}`,
  );
}

/**
 * Everything the studio can be asked to produce, in one object so the UI
 * can re-render from a single settings blob.
 *
 * `retouch` defaults to 0 and the UI does not move it on its own. The
 * research is unambiguous that forced beautification is a top complaint
 * in this category, and an ID photo that stops looking like its holder
 * fails at the counter.
 */
export const defaultOptions = () => ({
  retouch: 0,
  backdrop: { kind: "solid", rgb: [255, 255, 255] },
  garment: null,
});

/**
 * One photo, from file bytes to a validated, exportable result.
 *
 * Kept as a class because the expensive stages are cached inside it: a
 * slider drag re-runs `render()`, which touches no model at all.
 */
export class Photo {
  #session;
  #core;

  constructor(core, session) {
    this.#core = core;
    this.#session = session;
    this.report = null;
    this.solve = null;
    this.faceFound = false;
  }

  /**
   * Decode and run the two model stages. `quality` adds the detail pass:
   * a second matting run over a crop around the head, merged back in, so
   * hair is resolved at the model's full input resolution instead of at
   * whatever fraction of it the head happened to occupy.
   */
  static async open(bytes, { quality = false, onStage } = {}) {
    if (isNative) return NativePhoto.open(bytes, { quality, onStage });
    const { core } = await loadModels();
    const session = await timed("decode", async () => core.Session.decode(bytes));
    const photo = new Photo(core, session);

    onStage?.("detecting");
    const heads = new core.YunetHeads();
    await timed("detect", () =>
      detectFaces(faceModel.session, faceModel.ort, session.face_input(), heads),
    );
    photo.faceFound = session.set_source_face(heads);

    onStage?.("matting");
    const map = await timed("matting", () =>
      runMatting(mattingModel.session, mattingModel.ort, session.matte_input("modnet"), 512),
    );
    await timed("matte-refine", async () => session.set_matte(map, "modnet"));

    if (quality && photo.faceFound) {
      const detail = session.detail_input();
      // An empty tensor means the crate decided the pass would not pay —
      // no face, or a head already filling the frame.
      if (detail.length > 0) {
        onStage?.("detail");
        const detailMap = await timed("detail", () =>
          runMatting(mattingModel.session, mattingModel.ort, detail, 512),
        );
        session.set_detail_matte(detailMap);
      }
    }

    return photo;
  }

  get width() {
    return this.#session.width;
  }
  get height() {
    return this.#session.height;
  }

  sourceJpeg(quality = 88) {
    return this.#session.source_jpeg(quality);
  }

  setBackdropImage(bytes) {
    this.#session.set_backdrop_image(bytes);
  }

  /**
   * Re-run the cheap half: finishing options, then the compliance crop,
   * then re-detect on the output and validate.
   *
   * Validation deliberately re-detects the face on the cropped result
   * rather than trusting the crop solver's own arithmetic. The two
   * disagree often enough on real photos to matter, and the checklist is
   * only worth anything if it describes the file the user is about to
   * hand to a government portal.
   */
  async render(specId, options, adjust = {}, { enhance = false } = {}) {
    const solve = await timed("render", async () => {
      this.#session.prepare(JSON.stringify(options));
      return JSON.parse(
        this.#session.solve(
          specId,
          adjust.head ?? NaN,
          adjust.eye ?? NaN,
          adjust.center ?? NaN,
          enhance,
        ),
      );
    });

    const heads = new this.#core.YunetHeads();
    await timed("validate", () =>
      detectFaces(faceModel.session, faceModel.ort, this.#session.output_face_input(), heads),
    );
    const report = JSON.parse(this.#session.validate_output(heads));

    this.solve = solve;
    this.report = report;
    return { solve, report };
  }

  outputJpeg(quality = 95) {
    return this.#session.output_jpeg(quality);
  }
  outputJpegWithin(minKb, maxKb) {
    return this.#session.output_jpeg_within(minKb, maxKb);
  }
  outputPng() {
    return this.#session.output_png();
  }
  sheetInfo(sheet) {
    return JSON.parse(this.#session.sheet_info(sheet));
  }
  sheetJpeg(sheet, dpi = 300, cutMarks = true, quality = 95) {
    return this.#session.sheet_jpeg(sheet, dpi, cutMarks, quality);
  }

  free() {
    this.#session.free();
  }
}

/** The spec dataset, read out of the wasm module once. */
export async function loadSpecs() {
  if (isNative) return nativeSpecs();
  const core = await loadCore();
  return JSON.parse(core.specs_json());
}
