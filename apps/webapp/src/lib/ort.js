/**
 * onnxruntime-web, loaded in whichever build this browser can actually
 * use.
 *
 * The WebGPU build runs MODNet roughly an order of magnitude faster than
 * the CPU one, which on a phone is the difference between "a moment" and
 * "did it freeze". It is also a ~6.6 MB gzipped download against ~3.6 MB.
 * Since they are separate files, the right move is to decide at runtime
 * and fetch exactly one — a static import of both would cost every
 * visitor the union of the two.
 *
 * The `.wasm` binaries are copied into `./ort/` by the build script and
 * served from this origin: the default resolves them against a CDN, which
 * this app has no business talking to.
 */

let ready;

/** True when the browser exposes WebGPU at all. */
export function hasWebGPU() {
  return typeof navigator !== "undefined" && "gpu" in navigator;
}

export function loadOrt() {
  ready ??= init();
  return ready;
}

async function init() {
  const webgpu = hasWebGPU();
  const ort = webgpu
    ? await import("onnxruntime-web/webgpu")
    : await import("onnxruntime-web/wasm");

  ort.env.wasm.wasmPaths = new URL("./ort/", document.baseURI).href;
  // Multi-threading needs cross-origin isolation (COOP/COEP). A plain
  // static host does not send those headers, so the service worker adds
  // them to every same-origin response it serves (see service-worker.js);
  // the page is isolated from its second load onwards. Asking for threads
  // without isolation makes onnxruntime probe, fail and fall back, so the
  // count is decided here rather than left to the probe.
  const threads = globalThis.crossOriginIsolated
    ? Math.max(1, Math.min(4, (navigator.hardwareConcurrency ?? 2) - 1))
    : 1;
  ort.env.wasm.numThreads = threads;
  ort.env.logLevel = "error";

  return { ort, webgpu, threads };
}

/**
 * Create a session, falling back to CPU if the WebGPU provider refuses
 * the graph.
 *
 * That fallback is not defensive padding: WebGPU support is uneven enough
 * across drivers that a provider which reports as present can still fail
 * on a specific model, and the useful behaviour there is a slower photo,
 * not an error page.
 */
export async function createSession(bytes) {
  const { ort, webgpu, threads } = await loadOrt();
  if (webgpu) {
    try {
      const session = await ort.InferenceSession.create(bytes, {
        executionProviders: ["webgpu"],
        graphOptimizationLevel: "all",
      });
      return { session, provider: "webgpu", threads, ort };
    } catch (e) {
      console.warn("WebGPU session failed, falling back to CPU:", e);
    }
  }
  const session = await ort.InferenceSession.create(bytes, {
    executionProviders: ["wasm"],
    graphOptimizationLevel: "all",
  });
  return { session, provider: "cpu", threads, ort };
}
