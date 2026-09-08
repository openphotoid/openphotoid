/** Save bytes as a file, and read a picked file as bytes. */

export function save(bytes, filename, mime) {
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
