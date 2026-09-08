Captured from `dist/` by `e2e/capture.mjs` driving the real UI in headless
Chromium — the same harness as the e2e suite, not hand-composed mockups.
Desktop shots are 1440×900 at 2×; `10-phone-studio` is 390×844 at 3×.
The reference photo is `testdata/portrait-obama.jpg`; the astronaut
portraits are NASA's public-domain official portraits in `e2e/fixtures/`.

| | |
|---|---|
| `01-home.png` | the three promises and the coverage count |
| `02-picker.png` | the document list filtered to "passport" |
| `03-studio.png` | US passport result with the checklist; `03-studio-dark.png` the same under a dark OS |
| `04-checklist-panels.png` | full page: eleven checks, Position and Background open, Save |
| `05-garment.png` | jacket and tie on a spacesuit, Canadian passport crop (see the test guide's findings) |
| `06-batch.png` | three portraits, one verdict each |
| `07-coverage.png` | the published document list with sources and check dates |
| `08-home-zh.png` | Simplified Chinese, offline badge active |
| `09-no-face.png` | the error path |
| `10-phone-studio.png` | China visa-online photo on a phone |
