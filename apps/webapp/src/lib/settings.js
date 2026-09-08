/**
 * Settings that outlive a photo: the last document used, and the
 * finishing choices someone has already made once.
 *
 * Deliberately *not* stored: anything derived from the photo itself. The
 * app's claim is that the image never leaves the device; leaving pieces
 * of it in localStorage after the tab closes would undercut that even
 * though it never crosses the network.
 */

import { writable } from "svelte/store";

const KEY = "openphotoid.settings";

const defaults = {
  recentSpecs: [],
  quality: true,
  sharpen: true,
  retouch: 0,
};

function load() {
  try {
    // "openpassport.settings" is the key the page used before the September 2026 rename.
    return { ...defaults, ...JSON.parse(localStorage.getItem(KEY) ?? localStorage.getItem("openpassport.settings") ?? "{}") };
  } catch {
    return { ...defaults };
  }
}

export const settings = writable(load());

settings.subscribe((v) => {
  try {
    localStorage.setItem(KEY, JSON.stringify(v));
  } catch {
    /* private mode: settings simply do not persist */
  }
});

export function rememberSpec(id) {
  settings.update((s) => ({
    ...s,
    recentSpecs: [id, ...s.recentSpecs.filter((x) => x !== id)].slice(0, 5),
  }));
}
