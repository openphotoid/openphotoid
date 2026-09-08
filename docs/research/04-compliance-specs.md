# ID / Passport Photo Compliance Standards — Research Report

Research date: 2026-08-01. All numbers cross-checked across at least two sources; official-source URLs given where reachable (several government sites 403 automated fetches; numbers were then confirmed via multiple secondary compliance-tool sources).

---

## A. ICAO 9303 / ISO-IEC 19794-5 baseline portrait geometry

ICAO Doc 9303 (Part 3) defers photo quality to ISO/IEC 19794-5 (Full Frontal image type; being superseded by ISO/IEC 39794-5). Key numbers as implemented by compliance tooling:

| Parameter | Value | Notes |
|---|---|---|
| Head height (chin→crown) | **70–80 % of image height** | The most-cited ICAO number; many national specs restate as mm ranges |
| Inter-eye distance (IED) | **≥ 90 px, preferably ≥ 120 px** | ISO 19794-5 digital-image minimum; "a little less than 1/4 of image width" |
| Eye-line vertical position | Eye midpoint between **~50–70 % of image height from bottom** (best-practice band; token image fixes eyes at a constant line) | National specs give mm bands instead |
| Horizontal centering | Face midpoint at **image center ± ~5 % of width** (45–55 %) | |
| Pose (yaw/pitch/roll) | **≤ ±5° from frontal in every direction** (ISO full-frontal); some implementations tolerate roll to ±8° | Automated tools use these as hard limits |
| Dynamic range | ≥ **7 bits (128 intensity levels)** in the face region | |
| Background | Plain, **uniform, light, monochrome**, no texture/gradient, contrast with subject; no shadows on background | ISO does not mandate a hex; countries do |
| Lighting | Even, no shadows on face or background, no hot spots/flash reflection, natural skin tone (no color cast) | |
| Expression | Neutral, mouth closed, eyes open, gaze at camera | |
| Glasses | Allowed only if eyes clearly visible: no glare, no tint/sunglasses, frames must not cover eyes (many countries now ban entirely: US since Nov 2016, China, Korea, Australia) | |
| Head covering | Only religious/medical; full face chin-to-forehead visible | |
| Recency | ≤ 6 months old | |
| Print resolution | Scanned/printed images at **300 dpi minimum**; 600 dpi common for capture | |

Sources: [ICAO TR "Portrait Quality" PDF](https://www.icao.int/sites/default/files/TRIP/Publications/TR-Portrait-Quality-v1.0.pdf) (server blocks bots; retrieve manually — it contains the definitive tolerance tables), [photocollect ISO 19794-5 explainer](https://www.photocollect.io/en/blog/iso-iec-19794-5-the-international-standard-for-biometric-facial-images), [correlance ISO 19794-5 summary](https://www.correlance.com/cms/en/iso19794-5), [photogov ICAO 9303 guide](https://photogov.net/knowledge/standards/icao-9303-biometric-standards/), [BioLab-ICAO benchmark paper](https://www.researchgate.net/publication/221122160_BioLab-ICAO_A_new_benchmark_to_evaluate_applications_assessing_face_image_compliance_to_ISOIEC_19794-5_standard).

---

## B. Country geometric spec table

"Head" = chin to crown unless noted. Background hex values are *engine recommendations* — no government publishes hex; they publish color names. Suggested render values in parentheses.

| Country / doc | Print size | Head height | Eye line / other geometry | Background | Notes |
|---|---|---|---|---|---|
| **US passport & visa (print)** | 2×2 in (51×51 mm) | **1"–1 3/8" (25–35 mm)** = 50–69 % of height | Eye height **1 1/8"–1 3/8" (28–35 mm) from bottom** | White or off-white (#FFFFFF / #FAFAFA) | No glasses (since 2016). 300 dpi min print |
| **US DS-160 / DV lottery (digital)** | — | Head **50–69 %** of image height | Eyes **56–69 % from bottom** | White/off-white | Square 600×600 to 1200×1200 px, JPEG ≤ 240 KB, 24-bit sRGB |
| **US green card (I-485/I-90)** | 2×2 in, same spec as passport | 1"–1 3/8" | same | White/off-white | 2 identical prints |
| **Schengen/EU (incl. Germany, France; also standard EU ID)** | 35×45 mm | **32–36 mm** (70–80 %) | Face centered; ICAO geometry | Light grey recommended (#D3D3D3–#E8E8E8); "plain light"; Germany mandates light grey, not white | ≥ 400 dpi print quality cited; no glasses for most consulates |
| **UK passport** | 45×35 mm print | **Crown–chin 29–34 mm** | Head centered, not tilted | Light grey or cream (#DCDCDC / #F5F0E1); **not white** | Digital: ≥ 600×750 px, 50 KB–10 MB, JPEG |
| **Canada passport** | **50×70 mm** | Face height (chin→crown) **31–36 mm** | Face centered | Plain white or light-coloured (#FFFFFF) | Must be studio-taken, printed, photographer info on back; no edits/AI allowed |
| **India passport (print/BLS/VFS)** | 2×2 in (51×51 mm) | **25–35 mm**, tools target face ≈ 70–80 % (Passport Seva enforces "80–85 % face coverage") | Eyes 28.5–35 mm from bottom | Plain white (#FFFFFF) | Glasses banned since 2016 |
| **India Passport Seva (digital)** | — | face 80–85 % of frame | — | White | **630×810 px exactly**, JPEG 10–250 KB |
| **India OCI (online)** | 51×51 mm equiv. | face ~70–80 % | square | Plain light/white | **200×200 to 900×900 px square**, JPEG ≤ 200 KB (use 900×900) |
| **India PAN (NSDL/UTIITSL)** | **3.5×2.5 cm** (new Forms 95/96: 4.5×3.5 cm) | — | — | White | JPEG, ≥ 200 dpi, **20–50 KB** |
| **China passport / visa (print)** | **33×48 mm** | **28–33 mm**; head width **15–22 mm** | Top-of-head to top edge **3–5 mm**; chin to bottom edge **≥ 7 mm** | Pure white (#FFFFFF); passports also accept light blue historically, NIA/PSB now white | No glasses, ears visible, no smile |
| **China COVA visa online (digital)** | — | Head width ≈ 161–223 px band; face width 191–251 px per checkers | 3:4 aspect | Pure white — grey/cream cast rejected | **W 354–420 px, H 472–560 px**, JPEG **40–120 KB** |
| **Japan passport (MOFA)** | 35×45 mm | **34 ± 2 mm (32–36)** | Crown margin **4 ± 2 mm** from top | Plain light: white, light grey, light blue (#FFFFFF / #EDEDED / #D6E4F0) | No shadow, no smile |
| **Singapore (ICA)** | 35×45 mm | Face 70–80 %, ICAO | Full shoulders visible | **White** (#FFFFFF) | Digital: **exactly 400×514 px** (413×531 rejected), jpg/jpeg/png/heic, ≤ 2 MB |
| **Australia (DFAT)** | 35×45 mm (accepts up to 40×50) | **32–36 mm** | ICAO centered | Plain **light grey or off-white** (#E0E0E0 / #F2F2F2); not pure white for in-person | No glasses (since 2018). Digital guide ~1200×1600 px |
| **Brazil (Polícia Federal)** | **5×7 cm (50×70 mm)** | ICAO 70–80 % | — | White (#FFFFFF) | Same physical size as Canada |
| **Russia (internal & international)** | 35×45 mm | **32–36 mm**, top margin 5 mm | — | White for passport (#FFFFFF); light grey for visa | Matte vs glossy conventions; Gosuslugi online upload accepts JPEG ≥ 300 dpi |
| **South Korea passport** | 35×45 mm | **32–36 mm** (excluding hair) | Ears visible recommended | **Pure white only** — grey cast fails | Digital (Government24): **413×531 px**, 300 dpi, JPEG 100–500 KB |
| **UAE passport/ID** | **4×6 cm (40×60 mm)** | Face 70–80 % | — | White (#FFFFFF) | ICP e-channel uploads ~ 300–600 px wide typical |
| **Saudi eVisa (visitsaudi)** | — | Face 70–80 % | — | White | **200×200 px**, JPG/PNG/GIF/BMP, **5–100 KB** ([official spec page](https://visa.visitsaudi.com/Home/PhotoSpecifications)) |
| **Schengen visa (VFS/consulate)** | 35×45 mm | 32–36 mm / 70–80 % | ICAO | Light grey preferred, plain light | 2 prints; some consulates accept white |
| **Driver's licenses / student & work IDs** | Usually captured live (US/UK/CA). Where submitted: Japan DL 24×30 mm; India DL uses 35×45; China employment/student uses 1-inch 25×35 mm / 2-inch 35×49 mm classes | varies | — | White or blue common in CN (blue ≈ #438EDB, red ≈ #D74532, white) | Chinese "one inch" family is the dominant non-passport format (see Hivision CSV below) |

Chinese domestic size families (from HivisionIDPhotos `size_list_EN.csv`, px @300dpi): One inch 295×413; Small one inch 260×378; Large one inch 390×567; Two inch 413×626; Small two inch 413×531; Five inch 1050×1499; CET exams 144×192; graduate exam 531×709; social security card 358×441.

Background-color summary: **pure white** — US, Canada, India, China, Brazil, Russia (passport), Korea (strict), Singapore, UAE, Saudi, Japan (option); **off-white/cream** — US (option), UK (option); **light grey** — UK, Schengen/Germany, Australia, Russia visa, Japan (option); **light blue** — Japan (option), some Chinese domestic IDs.

---

## C. Digital-submission constraints table

| Portal | Pixels | File size | Format | Other |
|---|---|---|---|---|
| US DS-160/DS-260/DV | 600×600 min, 1200×1200 max, square | ≤ 240 KB | JPEG, 24-bit color | Not digitally altered; ≤ 6 months |
| US passport renewal online (MyTravelGov) | ≥ 600×600 | ≤ 10 MB (JPEG) | JPEG | Same geometry |
| UK gov.uk digital code | ≥ 600×750 | 50 KB–10 MB | JPEG | Taken against plain light background |
| India Passport Seva | exactly 630×810 | 10–250 KB | JPEG | face 80–85 % |
| India OCI portal | 200×200–900×900 square | ≤ 200 KB | JPEG | signature file separate |
| India PAN (NSDL) | 3.5×2.5 cm @ ≥200 dpi | 20–50 KB | JPEG | |
| China COVA | 354–420 (W) × 472–560 (H) | 40–120 KB | JPEG | face-width pixel band checked |
| Singapore ICA | exactly 400×514 | ≤ 2 MB | jpg/jpeg/png/heic/heif | |
| South Korea Government24 | 413×531 | 100–500 KB | JPEG | 300 dpi |
| Saudi eVisa | 200×200 | 5–100 KB | JPG/PNG/GIF/BMP | |
| Japan MOFA online | 35×45 equiv; ~480×600 min guide | ≤ ~5 MB typical | JPEG | |
| Australia online (AusPassport) | ~900×1200 to 1200×1600 guide | ≤ 10 MB | JPEG | |
| Schengen visa portals (varies) | commonly 35×45 @ 300 dpi = 413×531 | 50–500 KB typical | JPEG | per-consulate |

DPI convention: **300 dpi is the print standard** (35×45 mm → 413×531 px; 2×2 in → 600×600 px); 600 dpi (35×45 → 827×1063; 2×2 → 1200×1200) is the "premium/insurance" tier and the capture recommendation. Design the engine mm-first, derive px from dpi. ([Passlens DPI guide](https://passlens.com/blog/passport-photo-dpi-guide))

---

## D. Machine-readable spec databases (borrowability)

| Source | Format | Coverage | License | Verdict |
|---|---|---|---|---|
| [HivisionIDPhotos](https://github.com/Zeyi-Lin/HivisionIDPhotos) — `demo/assets/size_list_EN.csv` / `size_list_CN.csv`, `color_list_CN.csv` | CSV (name, height px, width px; colors as hex) | ~19 sizes, China-domestic heavy + US/JP/KR visa | **Apache-2.0** | Borrowable with attribution. Sizes only — no head-geometry, no file-size rules |
| [dpar39/ppp](https://github.com/dpar39/ppp) | Photo-standard definitions embedded in C++ (libppp); includes face-height tolerances (e.g. AU ±5.88 %) | ~dozens | **GPL-3.0** | Copyleft — read for ideas, don't vendor |
| [deidaraiorek/photogen](https://github.com/deidaraiorek/photogen) — `shared/photo_requirements.json` | JSON: size_inches/size_pixels/head_height_percent/background | ~10 countries | **MIT** | Best direct schema precedent; small coverage |
| [idphotoapi/Passport-Photo-API](https://github.com/idphotoapi/Passport-Photo-API) | Preset country configs | many | license unclear — verify before use | Caution |
| [Maphoz/BioGaze](https://github.com/Maphoz/BioGaze) | Python ICAO-check code (not a spec DB) | 15+ checks | no license stated | Reference only |
| [uam-biometrics/FaceQvec](https://github.com/uam-biometrics/FaceQvec) | 25 ISO-compliance tests | — | check repo | Reference for check design |
| PhotoAiD / visafoto / 123passportphoto / idphotodiy requirement pages | HTML per-document pages (visafoto has ~1000+ doc types with exact px/KB/DPI/background) | Most complete on the web | **Proprietary** | Use as verification corpus only; do not scrape-and-ship. Facts themselves (dimensions) are not copyrightable, but wholesale DB copying risks EU database-right issues |

**Conclusion: no comprehensive open dataset exists.** The practical route is to author an original JSON dataset (schema below) from primary government sources, cross-checked against visafoto/PhotoAiD pages, and optionally seed Chinese domestic sizes from Hivision's Apache-2.0 CSV.

---

## E. Compliance-check checklist with implementable heuristics

FaceQvec's 25 automated ISO-19794-5 tests (verbatim list from the paper, with method): 1 Blur (Laplacian variance), 2 Eyes direction (pupil vs eyeball center distance), 3 Ink marks (color-based detector on segmented face/bg), 4 Odd skin colour, 5 General illumination (mean pixel value too dark/bright), 6 Contrast (histogram concentration), 7 Pixelation (periodic edge detection), 8 Hair over face (hair segmentation CNN), 9 Eyes open/closed (eye-landmark distance = EAR), 10 Heterogeneous background (bg segmentation + k-means clustering), 11 Pose (roll/pitch/yaw CNN), 12 Light reflections on skin (overexposed zones), 13 Red eyes (color segmentation), 14 Shadows in background, 15 Shadows over face, 16 Sunglasses (dark pixels in eye region), 17 Reflections on glasses (overexposed pixels in eye region), 18 Wide frames, 19 Frames covering eyes (edges inside eye zone), 20 Hat (unnatural color upper forehead), 21 Veil, 22 Mouth open (mouth-landmark distance), 23 Other faces (multi-face detection), 24 White noise, 25 Expression (CNN). ([FaceQvec paper](https://arxiv.org/pdf/2111.02078); code at github.com/uam-biometrics/FaceQvec)

Recommended engine checklist with thresholds:

1. **Exactly one face**, detection confidence > 0.9.
2. **Head-height ratio**: landmark chin + estimated crown (top of skull ≈ eyes + 1.1×(eyes−chin) if hair occludes) vs country min/max mm mapped to px. This crown-estimation fudge is what dpar39/ppp does via landmarks.
3. **Eye-line height**: eye-midpoint y within country band (e.g. US 56–69 % from bottom).
4. **Centering**: |face-midpoint x − W/2| ≤ 5 % W.
5. **Pose**: roll from eye-line angle ≤ 5° (warn > 3°); yaw/pitch from solvePnP or CNN ≤ 5°.
6. **IED** ≥ 90 px (fail) / ≥ 120 px (recommend).
7. **Background uniformity**: segment person (matting model); on bg pixels compute stddev per channel (fail if σ > ~8/255) and mean ΔE to target color (warn ΔE > 10); k-means k=3 — largest cluster < 90 % ⇒ heterogeneous.
8. **Background shadows**: luminance gradient/dark-blob detection on bg mask adjacent to the silhouette.
9. **Face shadows / uneven lighting**: split face left/right halves; |ΔL| mean luminance > ~15 % ⇒ uneven; overexposed hot spots = connected regions > 240 luminance.
10. **Blur**: variance of Laplacian on face crop; threshold ~100 (tune per resolution — normalize crop to 112×112 as FaceQvec does, or scale threshold by area).
11. **Eyes open**: Eye Aspect Ratio > ~0.2 both eyes.
12. **Mouth closed / expression**: Mouth Aspect Ratio < ~0.1; optional emotion CNN = neutral.
13. **Glasses/glare**: eye-region dark-pixel ratio (sunglasses), specular highlight ratio (glare), edge density inside eye box (frames).
14. **Red-eye**: redness ratio in pupil region.
15. **Color cast**: gray-world deviation on face; a/b channel means in Lab outside ±10 ⇒ cast; saturation sanity.
16. **Resolution/output**: min px dims, dpi metadata, aspect ratio exact, JPEG re-encode binary-search quality to hit KB window (e.g. 40–120 KB COVA, ≤ 240 KB DS-160, 20–50 KB PAN).
17. **Head covering/other faces/ink-marks** as in FaceQvec.

PhotoAiD's public description: AI computes head tilt/rotation/position, background uniformity (no gradients/shadows/color inconsistency), lighting normalization + human expert review ([how it works](https://photoaid.com/how-it-works)). Visafoto advertises per-document auto-checks of size, head proportion, background replacement, and file-size targeting.

---

## F. Print-sheet tiling (computed; standard conventions)

Sheets: **4×6 in = 10.2×15.2 cm** (retail standard), **A4 210×297 mm**, sometimes 5×7 in. Convention: 3–5 mm gutters + thin cut lines (0.2 mm, #999), photos grouped to leave handling margin; retail kiosks typically print only 2 copies per 4×6.

| Photo size | 4×6 in sheet | A4 sheet |
|---|---|---|
| 2×2 in (51 mm) | **2×2 = 4** with gutters (6 borderless max: 2 cols × 3 rows exact) | 3×5 = 15 (with margins ~12) |
| 35×45 mm | **4×2 = 8** (4×35=140 ≤ 152; 2×45=90 ≤ 102) | 5×6 = 30 borderless; 4×5 = 20 with comfortable gutters |
| 50×70 mm (CA/BR) | **2×1 = 2** with margins (3×1 borderless) | 4×4 = 16 tight; 3×3 = 9 comfortable |
| 33×48 mm (CN) | **4×2 = 8** | 5×5 = 25 comfortable |
| 40×60 mm (UAE) | 3×1 = 3 (2 with margins) | 4×4 = 16 |
| 25×35 mm (1", PAN) | 5×2=10 (landscape: 6×2=12) | 7×7 ≈ 49; typical layout 40 |

Implementation: lay out on a mm grid at 300/600 dpi, center the block, draw crop marks extending 3 mm outside photo corners, optional 0.5 pt cut lines in gutters. ([Passlens 4×6 template guide](https://passlens.com/blog/4x6-passport-photo-template), [visapicpro print guide](https://visapicpro.com/article/print-passport-photos-at-home/))

---

## Proposed JSON schema for a country spec

```json
{
  "$id": "photo-spec.schema.json",
  "type": "object",
  "required": ["id", "country", "document", "print", "background", "face"],
  "properties": {
    "id": {"type": "string", "examples": ["us-passport", "cn-visa-cova"]},
    "country": {"type": "string", "description": "ISO 3166-1 alpha-2"},
    "document": {"type": "string", "enum": ["passport", "visa", "visa-online", "id-card", "green-card", "oci", "pan", "dv-lottery", "driver-license", "student-id", "custom"]},
    "sources": {"type": "array", "items": {"type": "string", "format": "uri"}},
    "last_verified": {"type": "string", "format": "date"},
    "print": {
      "type": "object",
      "properties": {
        "width_mm": {"type": "number"}, "height_mm": {"type": "number"},
        "dpi_default": {"type": "integer", "default": 300},
        "dpi_allowed": {"type": "array", "items": {"type": "integer"}, "default": [300, 600]}
      }
    },
    "digital": {
      "type": "object",
      "properties": {
        "width_px": {"type": ["integer", "null"], "description": "exact requirement, null if range"},
        "height_px": {"type": ["integer", "null"]},
        "min_width_px": {"type": "integer"}, "max_width_px": {"type": "integer"},
        "min_height_px": {"type": "integer"}, "max_height_px": {"type": "integer"},
        "aspect_ratio": {"type": "string", "examples": ["1:1", "3:4", "7:9"]},
        "min_kb": {"type": "number"}, "max_kb": {"type": "number"},
        "formats": {"type": "array", "items": {"type": "string"}},
        "color_depth_bits": {"type": "integer", "default": 24}
      }
    },
    "background": {
      "type": "object",
      "properties": {
        "name": {"type": "string", "enum": ["white", "off-white", "light-grey", "cream", "light-blue", "blue", "red", "any-plain-light"]},
        "hex_render": {"type": "string", "pattern": "^#[0-9A-Fa-f]{6}$"},
        "hex_alternatives": {"type": "array", "items": {"type": "string"}},
        "max_delta_e": {"type": "number", "description": "validation tolerance vs hex_render"},
        "max_stddev": {"type": "number", "description": "uniformity tolerance, 0-255 scale"}
      }
    },
    "face": {
      "type": "object",
      "properties": {
        "head_min_mm": {"type": "number"}, "head_max_mm": {"type": "number"},
        "head_min_pct": {"type": "number"}, "head_max_pct": {"type": "number"},
        "eye_min_from_bottom_pct": {"type": "number"}, "eye_max_from_bottom_pct": {"type": "number"},
        "eye_min_from_bottom_mm": {"type": "number"}, "eye_max_from_bottom_mm": {"type": "number"},
        "crown_top_margin_mm": {"type": ["object", "null"], "properties": {"min": {"type": "number"}, "max": {"type": "number"}}},
        "centering_tolerance_pct": {"type": "number", "default": 5},
        "max_roll_deg": {"type": "number", "default": 5},
        "max_yaw_deg": {"type": "number", "default": 5},
        "max_pitch_deg": {"type": "number", "default": 5}
      }
    },
    "rules": {
      "type": "object",
      "properties": {
        "glasses": {"type": "string", "enum": ["forbidden", "allowed-no-glare", "discouraged"]},
        "expression": {"type": "string", "default": "neutral"},
        "ears_visible": {"type": "boolean"},
        "shoulders_visible": {"type": "boolean"},
        "max_age_months": {"type": "integer", "default": 6},
        "editing_forbidden": {"type": "boolean"},
        "notes": {"type": "string"}
      }
    },
    "print_layouts": {
      "type": "array",
      "items": {"type": "object", "properties": {
        "sheet": {"type": "string", "enum": ["4x6in", "A4", "5x7in", "letter"]},
        "cols": {"type": "integer"}, "rows": {"type": "integer"},
        "gutter_mm": {"type": "number", "default": 3},
        "cut_marks": {"type": "boolean", "default": true}
      }}
    }
  }
}
```

Design notes: encode geometry mm-first with optional pct — validators convert via dpi; `hex_render` is what the compositor paints, `max_delta_e`/`max_stddev` are what the validator accepts; exact-pixel portals (Singapore 400×514, Seva 630×810, Saudi 200×200) use `width_px`/`height_px`, range portals (US, COVA) use min/max.

**Key caveats:** (1) No open dataset covers head-geometry + file-size rules — plan to author it (est. 40–60 document types for good coverage; visafoto's catalog shows the long tail is ~1000+). (2) The ICAO TR Portrait Quality PDF and travel.state.gov/canada.ca block automated fetch — pull them manually as primary citations. (3) Sub-country variance is real (Schengen consulates differ on white vs grey); the schema's `hex_alternatives` + `notes` fields absorb this. (4) Canada's "no software edits, studio-taken" rule means an auto-crop app can prepare but not certify Canadian passport photos — worth a per-spec `editing_forbidden` disclaimer flag, already in the schema.

Sources: [travel.state.gov photo page](https://travel.state.gov/content/travel/en/us-visas/visa-information-resources/photos.html), [canada.ca passport photos](https://www.canada.ca/en/immigration-refugees-citizenship/services/canadian-passports/photos.html), [visa.visitsaudi.com photo specs](https://visa.visitsaudi.com/Home/PhotoSpecifications), [schengenvisainfo.com/photo](https://schengenvisainfo.com/photo/), [visafoto US visa](https://visafoto.com/us-visa-photo), [visafoto OCI](https://visafoto.com/in-passport-oci-online-photo), [visafoto PAN](https://visafoto.com/in-pan-photo), [visafoto Saudi eVisa](https://visafoto.com/sa-evisa-200x200px-photo), [visafoto Russia](https://visafoto.com/ru-passport-photo), [visafoto Korea online](https://visafoto.com/kr-passport-online-photo), [Passlens Singapore](https://passlens.com/blog/singapore-passport-photo-guide), [Passlens US visa](https://passlens.com/blog/us-visa-photo-guide), [Passlens Japan](https://passlens.com/blog/japan-passport-photo-guide), [Snap2Pass COVA](https://www.snap2pass.com/doc/chinese-visa-photo), [photopass Passport Seva](https://www.photopass.ai/blog/how-to-upload-photo-passport-seva), [HivisionIDPhotos (Apache-2.0)](https://github.com/Zeyi-Lin/HivisionIDPhotos), [dpar39/ppp (GPL-3.0)](https://github.com/dpar39/ppp), [photogen (MIT)](https://github.com/deidaraiorek/photogen), [BioGaze](https://github.com/Maphoz/BioGaze), [FaceQvec paper](https://arxiv.org/pdf/2111.02078), [PhotoAiD how-it-works](https://photoaid.com/how-it-works), [mybiometricphotos Schengen](https://mybiometricphotos.com/visa-photo/schengen/), [onlinepassportphoto China](https://onlinepassportphoto.com/China-passport-photos.htm), [Passlens 4x6 template](https://passlens.com/blog/4x6-passport-photo-template), [dvlottery.com photo requirements](https://www.dvlottery.com/photo-requirements/), [UK passport size guide](https://passportphotodigital.com/article-detail/uk-passport-photo-size-dimensions-guide).
