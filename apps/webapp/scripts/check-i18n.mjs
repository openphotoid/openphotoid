// Three static checks on the catalogues, no browser and no build:
//   1. every declared locale has a catalogue;
//   2. every catalogue covers the same keys as English;
//   3. no value equals its English source, unless allowlisted below.
// A missing entry is invisible at runtime by design (t() falls back to
// English); this is what keeps that fallback from hiding a gap for a
// release or two. Runs from scripts/build.sh, so a gap fails the build.
import en from "../src/lib/i18n/en.js";
import zhHans from "../src/lib/i18n/zh-Hans.js";
import zhHant from "../src/lib/i18n/zh-Hant.js";
import ja from "../src/lib/i18n/ja.js";
import ko from "../src/lib/i18n/ko.js";
import de from "../src/lib/i18n/de.js";
import es from "../src/lib/i18n/es.js";
import pt from "../src/lib/i18n/pt.js";

const DECLARED = ["en", "zh-Hans", "zh-Hant", "ja", "ko", "de", "es", "pt"];
const DICTS = { en, "zh-Hans": zhHans, "zh-Hant": zhHant, ja, ko, de, es, pt };

// Genuine identities: proper nouns, formats and templates that read the
// same in every language. Add here rather than loosening the check.
const IDENTICAL_OK = new Set([
  "app.name",
  "picker.size", // "{w} × {h} px" in de/es/pt/ja/ko
  "picker.print", // "{w} × {h} mm"
  "export.digital.hint", // "{w} × {h} px, JPEG{size}"
  "export.digital.window", // ", {min}–{max} KB"
  "batch.running", // "{done} / {total}" in Chinese
  "studio.before", // "Original" is German and Spanish
  "diag.browser", // "Browser" is German
  "diag.total", // "Total" in es/pt
  "panel.background.color", // "Color" in Spanish
  "panel.adjust", // "Position" is German
  "panel.background.vignette", // "Studio" is German
]);

let failures = 0;
const fail = (m) => { failures++; console.error("  FAIL " + m); };
const enKeys = Object.keys(en);

for (const tag of DECLARED) {
  if (!DICTS[tag]) { fail(`${tag} is declared but has no catalogue`); continue; }
  if (tag === "en") continue;
  const dict = DICTS[tag];
  const missing = enKeys.filter((k) => !(k in dict));
  const extra = Object.keys(dict).filter((k) => !(k in en));
  if (missing.length) fail(`${tag} is missing ${missing.length}: ${missing.slice(0, 5).join(", ")}${missing.length > 5 ? "…" : ""}`);
  if (extra.length) fail(`${tag} has keys English does not: ${extra.join(", ")}`);
  const same = enKeys.filter((k) => k in dict && dict[k] === en[k] && !IDENTICAL_OK.has(k));
  if (same.length) fail(`${tag} leaves ${same.length} untranslated: ${same.join(", ")}`);
  // Every {placeholder} in the English source must survive translation.
  for (const k of enKeys) {
    if (!(k in dict)) continue;
    const want = (en[k].match(/\{[a-z]+\}/g) ?? []).sort().join(" ");
    const got = (dict[k].match(/\{[a-z]+\}/g) ?? []).sort().join(" ");
    if (want !== got) fail(`${tag} ${k}: placeholders ${JSON.stringify(got)} != ${JSON.stringify(want)}`);
  }
}
console.log(`i18n: ${DECLARED.length} locales, ${enKeys.length} keys each${failures ? "" : ", all covered"}`);
process.exit(failures ? 1 : 0);
