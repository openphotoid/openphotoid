/**
 * Eight languages, the same set as every app in the suite (openapps-i18n).
 *
 * Strings are deliberately plain. The audience for this product is
 * someone who needs a visa photo tonight, not someone who wants to learn
 * what a "spec engine" is — so the UI says "size and rules for the
 * country you picked", and the word "matting" appears nowhere.
 *
 * The catalogues are imported statically: eight of them are ~20 KB gzipped,
 * there is no loading state, and the app is designed to open with no
 * network. A missing entry falls back to English at runtime; the static
 * check in scripts/check-i18n.mjs is what stops that fallback hiding a gap.
 */

import { writable, derived, get } from "svelte/store";
import en from "./i18n/en.js";
import zhHans from "./i18n/zh-Hans.js";
import zhHant from "./i18n/zh-Hant.js";
import ja from "./i18n/ja.js";
import ko from "./i18n/ko.js";
import de from "./i18n/de.js";
import es from "./i18n/es.js";
import pt from "./i18n/pt.js";

// Script subtags on Chinese, not regions: the difference that matters is
// the writing system, and a reader in Singapore wants Hans while one in
// Hong Kong wants Hant.
export const DICTS = { en, "zh-Hans": zhHans, "zh-Hant": zhHant, ja, ko, de, es, pt };

/** Endonyms, never translated: 日本語 stays 日本語 in every locale. */
export const LANGUAGES = [
  { value: "en", label: "English" },
  { value: "zh-Hans", label: "简体中文" },
  { value: "zh-Hant", label: "繁體中文" },
  { value: "ja", label: "日本語" },
  { value: "ko", label: "한국어" },
  { value: "de", label: "Deutsch" },
  { value: "es", label: "Español" },
  { value: "pt", label: "Português" },
];

const STORAGE_KEY = "openphotoid.lang";

/**
 * Widen a browser tag to a locale we ship:
 * exact tag → zh script variant → base language → nothing.
 *
 * The middle step is what stops a browser reporting `zh-TW` falling
 * through to `zh` and being handed Simplified — worse than English. Per
 * CLDR's likely subtags, Hant is Taiwan, Hong Kong and Macau; every other
 * `zh` is Hans.
 */
export function matchLocale(tag) {
  if (!tag) return null;
  const t = String(tag).toLowerCase();
  const exact = Object.keys(DICTS).find((k) => k.toLowerCase() === t);
  if (exact) return exact;
  if (t === "zh" || t.startsWith("zh-")) {
    return /hant|-tw|-hk|-mo/.test(t) ? "zh-Hant" : "zh-Hans";
  }
  const base = t.split("-")[0];
  return Object.keys(DICTS).find((k) => k.toLowerCase() === base) ?? null;
}

function detect() {
  // Private browsing throws on access, not just on write; the browser's
  // own preference is a good enough answer when it does.
  try {
    const saved = localStorage.getItem(STORAGE_KEY) ?? localStorage.getItem("openpassport.lang");
    // The tag this app used to persist before the suite settled on script
    // subtags; the widening maps it to Hans as it always meant.
    const migrated = saved && matchLocale(saved);
    if (migrated) return migrated;
  } catch {
    /* fall through to the browser's list */
  }
  let nav = [];
  try {
    nav = navigator.languages ?? [navigator.language ?? "en"];
  } catch {
    /* no navigator: SSR or a very locked-down context */
  }
  for (const tag of nav) {
    const hit = matchLocale(tag);
    if (hit) return hit;
  }
  return "en";
}

export const lang = writable(detect());

lang.subscribe((v) => {
  try {
    localStorage.setItem(STORAGE_KEY, v);
  } catch {
    /* private mode: the language just does not persist */
  }
  // Not decoration: `lang` picks the right glyphs for Han characters,
  // which differ between Chinese and Japanese, and a screen reader
  // switches voice on it.
  if (typeof document !== "undefined") document.documentElement.lang = v;
});

/** `t("key", { n: 3 })` — a missing key renders as the key, loudly. */
export const t = derived(lang, ($lang) => (key, vars) => {
  const dict = DICTS[$lang] ?? en;
  let s = dict[key] ?? en[key] ?? key;
  if (vars) for (const [k, v] of Object.entries(vars)) s = s.replaceAll(`{${k}}`, String(v));
  return s;
});

export const currentLang = () => get(lang);

/** Keys a catalogue lacks relative to English; for the coverage check. */
export function missing(locale) {
  const dict = DICTS[locale] ?? {};
  return Object.keys(en).filter((k) => !(k in dict));
}
