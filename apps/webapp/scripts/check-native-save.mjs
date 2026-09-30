#!/usr/bin/env node
/**
 * Saving a photo reaches the system, inside the phone apps as well as a browser.
 *
 * The phone apps are the browser build inside a WebView, and a WebView has no
 * download manager: an `<a download>` click is dropped unless the Android side
 * registers a download listener, which ours does not. So the three Save buttons
 * did nothing at all there -- no file, no picker, no error, nothing in any log.
 * The only visible symptom was that the photo never arrived.
 *
 * `save()` therefore has two paths, and this checks both, because neither can
 * be checked by the browser suite: Playwright cannot be a WebView, and the
 * native path is chosen at import time from `window.__TAURI_INTERNALS__`.
 * Each case runs in its own process for that reason.
 *
 *   node scripts/check-native-save.mjs
 */
import { execFileSync } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";

const HERE = fileURLToPath(new URL(".", import.meta.url));
// `SAVE_MODULE` points the check at another copy of the module, which is how
// the shipped build is shown failing:
//   git show <tag>:apps/webapp/src/lib/download.js > /tmp/old.js
//   SAVE_MODULE=/tmp/old.js node scripts/check-native-save.mjs
const DOWNLOAD = process.env.SAVE_MODULE
  ? pathToFileURL(process.env.SAVE_MODULE).href
  : new URL("../src/lib/download.js", import.meta.url).href;

/** A document with just enough in it for the browser path to run. */
function fakeDocument(clicks) {
  return {
    // `looksNative` reads this attribute, so it has to answer like an element.
    documentElement: { dataset: {}, hasAttribute: () => false },
    createElement: () => ({
      set href(v) {},
      set download(v) {
        clicks.push(v);
      },
      click() {},
      remove() {},
    }),
    body: { appendChild() {} },
  };
}

async function runCase(name) {
  const calls = [];
  const clicks = [];
  globalThis.window = globalThis;
  globalThis.document = fakeDocument(clicks);
  globalThis.URL.createObjectURL = () => "blob:fake";
  globalThis.URL.revokeObjectURL = () => {};
  globalThis.Blob = class {
    constructor(parts, opts) {
      this.parts = parts;
      this.type = opts?.type;
    }
  };

  if (name !== "browser") {
    globalThis.__TAURI_INTERNALS__ = {
      invoke: async (cmd) => {
        calls.push(cmd);
        // The picker returns the chosen destination, or null when dismissed.
        if (cmd === "plugin:dialog|save") return name === "cancelled" ? null : "/tmp/us-passport.jpg";
        return null;
      },
      transformCallback: (cb) => cb,
      convertFileSrc: (p) => p,
    };
  }

  const { save } = await import(DOWNLOAD);
  const returned = await save(new Uint8Array([0xff, 0xd8, 0xff]), "us-passport.jpg", "image/jpeg");
  return { returned, calls, clicks };
}

const EXPECTED = {
  // The phone apps: the system's own picker, then a write through the plugin.
  // `content://` is not a path, so the write cannot go through std::fs.
  native: (r) =>
    r.returned === true &&
    r.calls.includes("plugin:dialog|save") &&
    r.calls.includes("plugin:fs|write_file") &&
    r.clicks.length === 0,
  // Dismissing the picker is not a failure, and must not report a save.
  cancelled: (r) =>
    r.returned === false && r.calls.includes("plugin:dialog|save") && !r.calls.includes("plugin:fs|write_file"),
  // A browser keeps the download it always had, and asks Tauri for nothing.
  browser: (r) => r.returned === true && r.calls.length === 0 && r.clicks[0] === "us-passport.jpg",
};

if (process.argv[2]) {
  const result = await runCase(process.argv[2]);
  process.stdout.write(JSON.stringify(result));
  process.exit(0);
}

let failed = 0;
for (const name of Object.keys(EXPECTED)) {
  let result;
  try {
    result = JSON.parse(
      execFileSync(process.execPath, [fileURLToPath(import.meta.url), name], {
        cwd: HERE,
        encoding: "utf8",
        stdio: ["ignore", "pipe", "pipe"],
      }),
    );
  } catch (e) {
    console.log(`  FAIL ${name.padEnd(10)} ${String(e.stderr ?? e.message).trim().split("\n")[0]}`);
    failed++;
    continue;
  }
  const ok = EXPECTED[name](result);
  if (!ok) failed++;
  const detail = result.calls.length ? result.calls.join(" → ") : `<a download="${result.clicks[0] ?? ""}">`;
  console.log(`  ${ok ? "ok  " : "FAIL"} ${name.padEnd(10)} returned ${String(result.returned).padEnd(5)} ${detail}`);
}
if (failed) {
  console.error(`save: ${failed} case(s) failed — the Save buttons do not reach the system`);
  process.exit(1);
}
console.log("save: browser and phone paths both reach the system");
