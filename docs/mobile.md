# The phone apps

The end state for every product in this suite is a listing in the stores: an
APK on Google Play and an IPA on the App Store, for iPhone and iPad. The web
app at app.openphotoid.com is the first surface, not the destination.

## Where we are (8 September 2026)

| Surface | State |
|---|---|
| Web app on a phone or tablet | Verified: the full Playwright suite under WebKit with iPhone and iPad emulation and Chromium as a Pixel; `#/diagnostics/run` on iOS 26.5 and Android 15 simulators runs the pipeline end to end. iOS 17.0 Safari cannot start onnxruntime-web at all (see `apps/webapp/README.md`). |
| Desktop | Tauri 2 + Svelte, `apps/desktop`, unsigned bundles. |
| Native inference | `frame-engine` has a `tract` backend (pure Rust, no native build step). YuNet runs unchanged; MODNet runs through the variant `scripts/patch-modnet-for-tract.py` produces, matching `ort` to a mean alpha difference of 0.000179 — but that variant is not yet in the model registry as a runtime path (PLAN.md §M9). |
| Store apps | None yet. |

## The road: Tauri 2 mobile

Two roads are in use in the suite. `opendocscan` went Flutter over the Rust
crates because it had no UI to reuse. This product has a finished,
mobile-first Svelte UI and a Tauri desktop shell already, so the shorter
road is **Tauri 2 mobile**: the same `apps/webapp` front end inside the
platform WebView, with the pipeline moved behind Tauri commands so that
**inference runs natively in Rust** through `tract`, not as wasm in the
WebView. That is what sidesteps the iOS 17 failure — it lives in WebKit's
wasm engine — and what keeps the app from being the thin wrapper both
stores reject.

The toolchain is already on this machine: tauri-cli 2.11, Xcode 26.6,
Android NDK 27, and the `aarch64-apple-ios`, `aarch64-apple-ios-sim`,
`aarch64-linux-android` and `x86_64-linux-android` targets.

## The steps

1. **Wire the tract MODNet into the registry.** `frame-engine::registry`
   gains the patched MODNet as the model the `tract-backend` feature loads,
   with its checksum from MODELS.md; the `-- --ignored` regression tests
   already exercise it. Models ship inside the app bundle rather than
   downloading on first run — a store app that pulls 26 MB after install
   reads as broken on a plane.
2. **A platform seam in the web app.** `apps/webapp/src/lib/pipeline.js`
   currently does two things itself: run ONNX through onnxruntime-web and
   call the wasm `Session`. Behind Tauri, both become `invoke()` calls into
   a `Session` held in Rust (`apps/mobile/src-tauri`), which runs the same
   `frame-*` crates natively with `tract`. One `platform.js` decides at
   start-up; the Svelte views do not change.
3. **`apps/mobile`**: `tauri android init` / `tauri ios init` against the
   web app's `dist`, with the app icons from `scripts/make-assets.mjs` (the
   Android set must be adaptive — see the `flutter-rust-bridge-on-this-mac`
   memory for why a legacy icon looks wrong), the bundle id
   `com.openapps.openphotoid`, camera and photo-library permission strings,
   and the file save/share sheet through Tauri's dialog and fs plugins.
4. **Run it on the simulators and the emulator** here, through the same
   diagnostics route, and drive the real flow with the platform's own
   automation (XCUITest / `adb`) rather than by hand.
5. **Release candidates**: `.github/workflows/release.yml` builds the APK
   (debug-signed until a submission is planned; the upload key can never
   be rotated) and the unsigned IPA on `v*` tags, attached to a prerelease.
   `v0.1.0-rc.1` is the first.
6. **The store gates**, which need Darius: an Apple Developer account and
   a Play Console account for the openphotoid identity, the signing keys,
   the listing copy and screenshots (from step 4's captures, the same mark
   as the favicon), the privacy policy URL (`openphotoid.com/privacy.html`,
   already live), and the review answers. A store upload is never done
   without being asked that time.

Steps 1–5 are routine engineering. Step 6 is the one that cannot be started
without the accounts.
