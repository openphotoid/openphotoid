/**
 * Presentation helpers over the spec dataset.
 *
 * The dataset itself is Rust's — `data/specs/*.json`, embedded into the
 * wasm module. Nothing here re-derives a rule; it only formats what is
 * already there, so a spec change has one place to happen.
 */

const REGION = new Intl.DisplayNames(undefined, { type: "region" });

/** "US" -> "United States", falling back to the code itself. */
export function countryName(code, locale) {
  try {
    return new Intl.DisplayNames([locale], { type: "region" }).of(code) ?? code;
  } catch {
    try {
      return REGION.of(code) ?? code;
    } catch {
      return code;
    }
  }
}

/** A short human label: "Passport", "Visa (DV lottery)" etc. */
export function documentLabel(spec) {
  return spec.document
    .split("-")
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(" ");
}

export function outputPx(spec) {
  if (spec.digital?.width_px && spec.digital?.height_px) {
    return [spec.digital.width_px, spec.digital.height_px];
  }
  if (spec.print) {
    const px = (mm) => Math.round((mm / 25.4) * spec.print.dpi_default);
    return [px(spec.print.width_mm), px(spec.print.height_mm)];
  }
  return [600, 600];
}

/** The KB window a portal enforces, or null when it enforces none. */
export function sizeWindow(spec) {
  const d = spec.digital;
  if (!d) return null;
  if (d.min_kb == null && d.max_kb == null) return null;
  return { min: d.min_kb, max: d.max_kb };
}

export function backgroundRgb(spec) {
  const hex = (spec.background?.hex_render ?? "#FFFFFF").replace("#", "");
  if (hex.length !== 6) return [255, 255, 255];
  return [0, 2, 4].map((i) => parseInt(hex.slice(i, i + 2), 16));
}

/** Group specs by country for the picker, countries sorted by name. */
export function byCountry(specs, locale) {
  const groups = new Map();
  for (const spec of specs) {
    const name = countryName(spec.country, locale);
    if (!groups.has(spec.country)) groups.set(spec.country, { code: spec.country, name, specs: [] });
    groups.get(spec.country).specs.push(spec);
  }
  return [...groups.values()].sort((a, b) => a.name.localeCompare(b.name, locale));
}

/**
 * Loose search across the country name, its code, the document name and
 * the spec id — so "美国", "US", "passport" and "us-passport" all find the
 * same row. Task-shaped queries are how people actually look for this.
 */
export function matches(spec, query, locale) {
  if (!query) return true;
  const q = query.trim().toLowerCase();
  const haystack = [
    spec.id,
    spec.country,
    spec.document,
    countryName(spec.country, locale),
    countryName(spec.country, "en"),
    countryName(spec.country, "zh-CN"),
  ]
    .join(" ")
    .toLowerCase();
  return haystack.includes(q);
}
