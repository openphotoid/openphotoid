// Serves the built app (dist/) for the e2e suite and the screenshot run.
//
// A static server and nothing else, which is the same thing a real host
// does for this app; the point of testing against dist/ is that the
// suite exercises the files a visitor gets, not the dev server's.
//
// `COI=1` adds the cross-origin-isolation headers, which is what lets a
// test compare single- and multi-threaded onnxruntime on the same build.
import { createServer } from "node:http";
import { createReadStream, statSync } from "node:fs";
import { join, extname, normalize } from "node:path";
import { fileURLToPath } from "node:url";

// `SITE_ROOT` serves the composed site — the product page from the private
// site repo with this app inside it — instead of the app's bare `dist/`.
// After the website moved out, `dist/index.html` is a shell: it has the app
// and none of the copy the suite asserts on, so a plain run here would be
// testing a page nobody is served. `deploy.sh` in the site repo sets it.
const root = process.env.SITE_ROOT
  ? process.env.SITE_ROOT
  : join(fileURLToPath(new URL(".", import.meta.url)), "..", "dist");
const port = Number(process.env.PORT ?? 5199);
const coi = process.env.COI === "1";

const types = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".mjs": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".json": "application/json",
  ".webmanifest": "application/manifest+json",
  ".wasm": "application/wasm",
  ".onnx": "application/octet-stream",
  ".woff2": "font/woff2",
  ".png": "image/png",
  ".svg": "image/svg+xml",
  ".jpg": "image/jpeg",
};

createServer((req, res) => {
  let path;
  try {
    path = decodeURIComponent(new URL(req.url, "http://x").pathname);
  } catch {
    // A malformed percent-escape took the whole server down mid-suite
    // once; a static host answers 400 and carries on.
    res.writeHead(400).end("bad request");
    return;
  }
  if (path.endsWith("/")) path += "index.html";
  const file = normalize(join(root, path));
  if (!file.startsWith(root)) {
    res.writeHead(403).end();
    return;
  }
  let stat;
  try {
    stat = statSync(file);
    if (!stat.isFile()) throw new Error("not a file");
  } catch {
    res.writeHead(404).end("not found");
    return;
  }
  const headers = {
    "content-type": types[extname(file)] ?? "application/octet-stream",
    "content-length": stat.size,
    "cache-control": "no-store",
  };
  if (coi) {
    headers["cross-origin-opener-policy"] = "same-origin";
    headers["cross-origin-embedder-policy"] = "require-corp";
  }
  res.writeHead(200, headers);
  createReadStream(file).pipe(res);
}).listen(port, () => console.log(`serving ${root} on http://localhost:${port}/${coi ? " (cross-origin isolated)" : ""}`));
