/**
 * Where the pipeline runs.
 *
 * In a browser: onnxruntime-web runs the two graphs and the Rust core runs
 * as WebAssembly (`pipeline.js`). Inside the phone apps, the same Svelte UI
 * sits in a WebView and everything — decode, the two models through tract,
 * the crop, the checks, the exports — runs natively in Rust behind Tauri
 * commands. This module is the seam; the views never know which.
 */

export const isNative = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

// The document the phone apps package is the same one openphotoid.com
// serves, marketing copy included ("runs in your browser", the GitHub
// link). Inside an app that copy is wrong and a store review would say
// so; app.css hides it on this attribute.
if (isNative) document.documentElement.dataset.native = "";

// Presentation only: true in the phone apps, and in a browser that was
// told to look like one (the store-screenshot capture sets the attribute
// before the page loads). Never used to choose a code path.
export const looksNative =
  isNative || (typeof document !== "undefined" && document.documentElement.hasAttribute("data-native"));


let invokeFn;
async function invoke(cmd, args) {
  invokeFn ??= (await import("@tauri-apps/api/core")).invoke;
  return invokeFn(cmd, args);
}

const fromB64 = (s) => Uint8Array.from(atob(s), (c) => c.charCodeAt(0));
const toB64 = (bytes) => {
  let out = "";
  for (let i = 0; i < bytes.length; i += 0x8000) out += String.fromCharCode.apply(null, bytes.subarray(i, i + 0x8000));
  return btoa(out);
};

/** `Photo`-shaped, backed by a native session id. See `pipeline.js`. */
export class NativePhoto {
  #id;
  constructor(opened) {
    this.#id = opened.id;
    this.width = opened.width;
    this.height = opened.height;
    this.faceFound = opened.face_found;
    this.#source = fromB64(opened.source_jpeg);
    this.report = null;
    this.solve = null;
    this.#output = null;
  }
  #source;
  #output;

  static async open(bytes, { quality = false, onStage } = {}) {
    onStage?.("detecting");
    const start = performance.now();
    const opened = await invoke("session_open", { bytes: toB64(bytes), quality });
    // One measure for the whole native open, so the diagnostics page has
    // a number: the stages run inside one Rust call here.
    performance.measure("openphotoid:native-open", { start, end: performance.now() });
    return new NativePhoto(opened);
  }
  sourceJpeg() {
    return this.#source;
  }
  async setBackdropImage(bytes) {
    await invoke("session_set_backdrop_image", { id: this.#id, bytes: toB64(bytes) });
  }
  async render(specId, options, adjust = {}, { enhance = false } = {}) {
    const r = await invoke("session_render", {
      id: this.#id,
      specId,
      options: JSON.stringify(options),
      head: Number.isFinite(adjust.head) ? adjust.head : null,
      eye: Number.isFinite(adjust.eye) ? adjust.eye : null,
      center: Number.isFinite(adjust.center) ? adjust.center : null,
      enhance,
    });
    this.#output = fromB64(r.output_jpeg);
    this.solve = r.solve;
    this.report = r.report;
    return { solve: r.solve, report: r.report };
  }
  // The studio shows the render's own JPEG; exports go back to Rust.
  outputJpeg() {
    return this.#output;
  }
  async outputJpegWithin(minKb, maxKb) {
    return fromB64(await invoke("session_export", { id: this.#id, kind: "jpeg_within", a: minKb, b: maxKb }));
  }
  async outputPng() {
    return fromB64(await invoke("session_export", { id: this.#id, kind: "png" }));
  }
  async sheetInfo(sheet) {
    return invoke("session_sheet_info", { id: this.#id, sheet });
  }
  async sheetJpeg(sheet, dpi = 300, cutMarks = true, quality = 95) {
    return fromB64(await invoke("session_export", { id: this.#id, kind: "sheet", sheet, a: dpi, b: quality, cutMarks }));
  }
  free() {
    invoke("session_free", { id: this.#id }).catch(() => {});
  }
}

export async function nativeSpecs() {
  return JSON.parse(await invoke("specs_json"));
}

export async function nativeInfo() {
  return invoke("platform_info");
}

/**
 * The account, on a phone. Google refuses to sign in inside an embedded
 * WebView, so the app opens the web app's account page in the system
 * browser: same account, same balance, and the sign-in is Google's own
 * page in Safari or Chrome.
 */
export async function openAccountInBrowser() {
  const { openUrl } = await import("@tauri-apps/plugin-opener");
  await openUrl("https://app.openphotoid.com/#/account");
}
