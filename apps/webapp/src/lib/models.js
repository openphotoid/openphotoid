/**
 * Model files, fetched once and kept in the Cache Storage API.
 *
 * They are served from this app's own origin rather than from the
 * upstream release URLs the desktop build uses. Three reasons, in order
 * of how much they matter:
 *
 *  1. The product's claim is that the photo never leaves the device.
 *     Fetching weights cross-origin at runtime would hand a third party a
 *     request timed to the exact moment someone opens their passport
 *     photo. Same-origin keeps that promise whole.
 *  2. Neither upstream host commits to CORS headers, so it would work
 *     until the day it didn't.
 *  3. Cached in Cache Storage, a second visit needs the network for
 *     nothing — which is the difference between "offline-capable" as a
 *     bullet point and as a fact.
 *
 * `scripts/build.sh` fetches and checksums them at build time; the
 * checksums live in MODELS.md alongside their licences.
 */

const CACHE = "openphotoid-models-v1";
// Builds before the September 2026 rename cached models under this name.
caches.delete("openpassport-models-v1").catch(() => {});

export const MODELS = {
  /** YuNet face detector — 5 landmarks, MIT (OpenCV Zoo). */
  face: { file: "face_detection_yunet_2023mar.onnx", approxBytes: 232_000 },
  /** MODNet portrait matting — Apache-2.0. The only matting model this
   *  build ships; the quality tier runs it a second time over the head
   *  rather than downloading a heavier one. */
  matting: { file: "modnet_photographic_portrait_matting.onnx", approxBytes: 25_900_000 },
};

const base = () => new URL("./models/", document.baseURI);

/**
 * Fetch a model, preferring the cache. `onProgress(fraction)` is called
 * as it streams — a 25 MB download on a phone needs a real progress bar,
 * not a spinner that looks identical to a hang.
 */
export async function fetchModel(id, onProgress) {
  const spec = MODELS[id];
  if (!spec) throw new Error(`unknown model ${id}`);
  const url = new URL(spec.file, base()).href;

  const cache = await caches.open(CACHE).catch(() => null);
  const hit = cache && (await cache.match(url));
  if (hit) {
    onProgress?.(1);
    return new Uint8Array(await hit.arrayBuffer());
  }

  const res = await fetch(url);
  if (!res.ok) throw new Error(`${spec.file}: ${res.status} ${res.statusText}`);

  // Clone before consuming: a body can only be read once, and we want
  // both the bytes now and a cache entry for next time.
  if (cache) cache.put(url, res.clone()).catch(() => {});

  const total = Number(res.headers.get("content-length")) || spec.approxBytes;
  if (!res.body) {
    onProgress?.(1);
    return new Uint8Array(await res.arrayBuffer());
  }

  const reader = res.body.getReader();
  const chunks = [];
  let received = 0;
  for (;;) {
    const { done, value } = await reader.read();
    if (done) break;
    chunks.push(value);
    received += value.length;
    onProgress?.(Math.min(1, received / total));
  }
  const out = new Uint8Array(received);
  let at = 0;
  for (const c of chunks) {
    out.set(c, at);
    at += c.length;
  }
  onProgress?.(1);
  return out;
}

/**
 * Whether the app can genuinely run with no network — which drives the
 * "ready to work offline" badge, so it has to be true and not merely
 * likely.
 *
 * Both halves are required. Cached weights alone are not enough: on a
 * first visit the service worker installs but does not control the page
 * that installed it, so the app shell has not been cached yet and a
 * reload with no signal would fail. Waiting for `controller` is what
 * makes the badge mean what it says.
 */
export async function readyOffline() {
  if (!("serviceWorker" in navigator) || !navigator.serviceWorker.controller) return false;
  const cache = await caches.open(CACHE).catch(() => null);
  if (!cache) return false;
  for (const spec of Object.values(MODELS)) {
    if (!(await cache.match(new URL(spec.file, base()).href))) return false;
  }
  return true;
}
