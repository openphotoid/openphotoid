/**
 * Offline support.
 *
 * The claim on the home page is that after one visit the app works with
 * no signal. This is the whole of what makes that true, so it is written
 * to be boring: precache the shell on install, serve cached-first for
 * everything immutable, and never let a failed network request turn into
 * a blank page.
 *
 * The models are deliberately *not* precached here. They are ~26 MB, and
 * `src/lib/models.js` puts them in its own cache as they stream, with a
 * progress bar. Precaching them would mean a silent 26 MB download on
 * first paint for someone who might only be reading the coverage page.
 */

const BUILD = "__BUILD_ID__";
const CACHE = `openphotoid-${BUILD}`;

// The app shell. Everything else — the code-split ONNX runtime chunks,
// the wasm binaries, the fonts — is cached on first use by the fetch
// handler, because which of them a given browser needs is only knowable
// at runtime.
const SHELL = ["./", "./index.html", "./manifest.webmanifest"];

/**
 * Cross-origin isolation, from a static host.
 *
 * onnxruntime's CPU path runs single-threaded unless the page is
 * cross-origin isolated, and isolation is granted by two response headers
 * a file host never sends. A service worker may add them to the responses
 * it serves, so from the second load on — the first is not controlled by
 * the worker — the CPU path gets its threads. This app loads nothing
 * cross-origin, which is what makes `require-corp` safe to send.
 */
function isolated(res) {
  if (!res || res.status === 0 || res.type === "opaque") return res;
  const headers = new Headers(res.headers);
  headers.set("Cross-Origin-Opener-Policy", "same-origin");
  headers.set("Cross-Origin-Embedder-Policy", "require-corp");
  return new Response(res.body, { status: res.status, statusText: res.statusText, headers });
}

self.addEventListener("install", (event) => {
  event.waitUntil(
    caches
      .open(CACHE)
      .then((c) => c.addAll(SHELL))
      .then(() => self.skipWaiting()),
  );
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((keys) =>
        Promise.all(
          keys
            .filter((k) => (k.startsWith("openphotoid-") || k.startsWith("openpassport-")) && k !== CACHE)
            .map((k) => caches.delete(k)),
        ),
      )
      .then(() => self.clients.claim()),
  );
});

self.addEventListener("fetch", (event) => {
  const { request } = event;
  if (request.method !== "GET") return;

  const url = new URL(request.url);
  if (url.origin !== self.location.origin) return;

  // Navigations: network first, so a deployed update is picked up on the
  // next visit rather than after an unpredictable cache expiry — but fall
  // back to a cached copy, which is what makes offline launch work.
  //
  // Each page is cached under its own URL. With eight languages, /de.html
  // and / are different documents; storing every navigation as
  // ./index.html made the offline English home page whichever language
  // was visited last. The key drops the query and fragment, which do not
  // change the document. Offline, a page never visited falls back to the
  // shell, since the app inside it runs the same either way.
  //
  // Only a good, same-origin answer is kept. A 404 or a proxy's error page
  // cached here would be served as the page for as long as the device
  // stays offline (openpixels, bba0ee2).
  if (request.mode === "navigate") {
    const key = url.origin + url.pathname;
    event.respondWith(
      fetch(request)
        .then((res) => {
          if (res.ok && res.type === "basic") {
            const copy = res.clone();
            caches.open(CACHE).then((c) => c.put(key, copy));
          }
          return isolated(res);
        })
        .catch(() =>
          caches
            .match(key)
            .then((r) => r ?? caches.match("./index.html"))
            .then((r) => (r ? isolated(r) : Response.error())),
        ),
    );
    return;
  }

  // Everything else is content-hashed or version-pinned, so cache-first
  // is both correct and the fast path.
  event.respondWith(
    caches.match(request).then((hit) => {
      if (hit) return isolated(hit);
      return fetch(request).then((res) => {
        if (res.ok && res.type === "basic") {
          const copy = res.clone();
          caches.open(CACHE).then((c) => c.put(request, copy));
        }
        return isolated(res);
      });
    }),
  );
});
