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
  // `?threads=N` on the URL overrides the count, for testing a device that
  // misbehaves with several — the diagnostics page reports what it got.
  const forced = Number(new URLSearchParams(location.search).get("threads"));
  const threads = forced > 0
    ? forced
    : globalThis.crossOriginIsolated
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
/** What happened on the way to a session, for the diagnostics page. */
export const notes = [];

export async function createSession(bytes) {
  const { ort, webgpu, threads } = await loadOrt();
  // WebGPU is opt-in (`?gpu=1`) until it is right. With onnxruntime-web
  // 1.29 the WebGPU provider returns a MODNet matte full of holes — the
  // same wrong pixels in Chrome and WebKit, at every optimisation level and
  // with memory planning off — while the CPU path is correct. A slower
  // photo beats a wrong one; the threaded CPU path is 400 ms on a laptop.
  if (webgpu && new URLSearchParams(location.search).get("gpu") === "1") {
    try {
      const q = new URLSearchParams(location.search);
      const session = await ort.InferenceSession.create(bytes, {
        executionProviders: ["webgpu"],
        // MODNet adds its decoder output to a resized copy of its input;
        // on WebGPU the runtime's memory planner reuses a buffer across
        // that add and the matte comes back with holes — the same shape
        // of failure OpenPixels hit. Planning off costs a little memory.
        enableMemPattern: q.get("mempattern") === "1",
        graphOptimizationLevel: q.get("gpuopt") ?? "all",
      });
      return { session, provider: "webgpu", threads, ort };
    } catch (e) {
      notes.push(`WebGPU refused the model, using the CPU: ${e?.message ?? e}`);
      console.warn("WebGPU session failed, falling back to CPU:", e);
    }
  }
  // The optimiser can fail on a graph a browser's wasm build cannot handle
  // — iOS 17 Safari refused MODNet at "all" with "Could not find OrtValue
  // with name '686'" — so step down before giving up. A less-optimised
  // session is slower; no session is a blank page.
  if (webgpu && !notes.some((n) => n.startsWith("WebGPU is present"))) notes.push("WebGPU is present but not used: onnxruntime-web 1.29's WebGPU matte is wrong (holes through hair); the CPU path is exact");
  let last;
  for (const level of ["all", "basic", "disabled"]) {
    try {
      const session = await ort.InferenceSession.create(bytes, {
        executionProviders: ["wasm"],
        graphOptimizationLevel: level,
      });
      if (level !== "all") notes.push(`CPU session needed graph optimisation "${level}"`);
      return { session, provider: "cpu", threads, ort };
    } catch (e) {
      last = e;
      notes.push(`CPU session at optimisation "${level}" failed: ${e?.message ?? e}`);
    }
  }
  throw last;
}
