/** Save bytes as a file, and read a picked file as bytes. */

import { isNative } from "./platform.js";

/**
 * Hand the bytes to the user.
 *
 * In a browser this is an `<a download>` click. Inside the phone apps it
 * cannot be: a WebView has no download manager of its own, and with no
 * `setDownloadListener` registered on the Android side it does not forward the
 * click anywhere either — it drops it. The three Save buttons therefore did
 * nothing at all on Android: no error, no file, no sheet. Nothing to notice
 * except that the photo never arrived.
 *
 * So the native path asks the system for a destination — on Android that is
 * `ACTION_CREATE_DOCUMENT`, the same picker every other app saves through —
 * and writes there. The desktop front end already did exactly this; the phones
 * were running the browser build, which did not.
 *
 * Both plugins are imported lazily, so a browser never downloads them.
 */
export async function save(bytes, filename, mime) {
  if (isNative) return saveNatively(bytes, filename, mime);

  const blob = new Blob([bytes], { type: mime });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  a.remove();
  // Revoking immediately can cancel the download in some browsers; a
  // frame's grace is enough and the blob is freed either way.
  setTimeout(() => URL.revokeObjectURL(url), 10_000);
  return true;
}

async function saveNatively(bytes, filename, mime) {
  const [{ save: pickDestination }, { writeFile }] = await Promise.all([
    import("@tauri-apps/plugin-dialog"),
    import("@tauri-apps/plugin-fs"),
  ]);
  const ext = filename.split(".").pop();
  const path = await pickDestination({
    defaultPath: filename,
    filters: [{ name: ext.toUpperCase(), extensions: [ext] }],
  });
  // Null is the user backing out of the picker, which is not a failure and
  // must not be reported as one.
  if (!path) return false;
  // `writeFile` goes through the plugin's own command rather than std::fs
  // because on Android the picker returns a `content://` URI, which is not a
  // path and cannot be opened as one.
  await writeFile(path, bytes);
  return true;
}

export async function readFile(file) {
  return new Uint8Array(await file.arrayBuffer());
}

/** An object URL for showing bytes in an <img>, with its own revoke. */
export function objectUrl(bytes, mime = "image/jpeg") {
  return URL.createObjectURL(new Blob([bytes], { type: mime }));
}

/** Slugify a spec id and index into a predictable filename. */
export function filename(specId, ext, index) {
  const n = index == null ? "" : `-${String(index + 1).padStart(3, "0")}`;
  return `${specId}${n}.${ext}`;
}
