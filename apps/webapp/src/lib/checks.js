/**
 * The checklist, in the reader's language.
 *
 * The core reports each check twice: an English sentence (`detail`) and the
 * figures it was built from (`values`). The sentence is what a log or the
 * desktop build prints; here the figures are what matter, because the eleven
 * checks are the product's main claim and a German reader met them in
 * English with the word "Laplacian" in them.
 *
 * Anything without a translation falls back to the English sentence rather
 * than to a key or a blank: a checklist that hides a number is worse than one
 * that shows it in the wrong language.
 */
import { hasKey } from "./i18n.js";

/** Numbers as this locale writes them — 61,9 in German, 61.9 in English. */
function localise(values, locale) {
  const out = {};
  const fmt = new Intl.NumberFormat(locale, { maximumFractionDigits: 1 });
  for (const [k, v] of Object.entries(values ?? {})) out[k] = fmt.format(v);
  return out;
}

/** The check's name: "Head height", "头部高度". */
export function checkName(t, name) {
  const key = `check.${name}`;
  return hasKey(key) ? t(key) : name.replaceAll("_", " ");
}

/**
 * Which sentence a check gets.
 *
 * Three checks say different things depending on how they came out, and the
 * figures are what tell them apart — the status alone does not: inter-eye
 * distance passes both when the photo is comfortably wide and when the
 * recommendation is out of reach at this document's size, and those are
 * different sentences.
 */
function detailKey(check) {
  const v = check.values ?? {};
  switch (check.name) {
    case "face_count":
      if (v.count === 0) return "check.face_count.detail.none";
      return v.count === 1 ? "check.face_count.detail" : "check.face_count.detail.many";
    case "inter_eye_distance":
      if (v.value < v.min) return "check.inter_eye_distance.detail";
      return v.recommendation_reachable || v.value >= v.recommended
        ? "check.inter_eye_distance.detail.recommended"
        : "check.inter_eye_distance.detail.later";
    case "file_size":
      return "kb" in v ? "check.file_size.detail" : "check.file_size.detail.impossible";
    default:
      return `check.${check.name}.detail`;
  }
}

/**
 * The check's own sentence, written here rather than in the core.
 *
 * `spec` carries the two things the figures cannot: the background colour the
 * document asks for, by name and by hex.
 */
export function checkDetail(t, check, spec, locale) {
  const values = check.values ?? {};
  // No figures means the core had nothing to measure -- the face was missing,
  // the region too small, the document has no size window. Those sentences are
  // keyed by what they say, since there is no number in them.
  if (!Object.keys(values).length) {
    const key = `check.detail.${NO_FIGURES[check.detail] ?? ""}`;
    return NO_FIGURES[check.detail] && hasKey(key) ? t(key) : check.detail;
  }
  const key = detailKey(check);
  if (!hasKey(key)) return check.detail;
  return t(key, { ...localise(values, locale), hex: spec?.background?.hex_render ?? "#FFFFFF" });
}

/** The English sentences that carry no figure, and the key each maps to. */
const NO_FIGURES = {
  "no face detected": "no_face",
  "face region too small to measure": "face_too_small",
  "face region too small to split": "face_too_narrow",
  "no reliable background region above the head to sample": "no_background_region",
  "spec has no digital file-size constraint": "no_digital_window",
  "no max KB constraint to target": "no_max_kb",
};
