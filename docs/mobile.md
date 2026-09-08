# The phone apps

The end state for every product in this suite is a listing in the stores: an
APK on Google Play and an IPA on the App Store, for iPhone and iPad. The web
app at app.openphotoid.com is the first surface, not the destination.

## Where we are (8 September 2026, evening)

| Surface | State |
|---|---|
| Web app on a phone or tablet | Verified: the full Playwright suite under WebKit with iPhone and iPad emulation and Chromium as a Pixel; `#/diagnostics/run` on iOS 26.5 and Android 15 simulators runs the pipeline end to end. iOS 17.0 Safari cannot start onnxruntime-web at all (see `apps/webapp/README.md`); the native app below is the answer there. |
| **iPhone and iPad app** | **Built and verified.** `apps/mobile` (Tauri 2), the same UI, inference native in Rust through `tract`. The device test reports "native · tract · 10 threads" and about 3.0 s end to end on the iPhone 17 Pro Max (iOS 26.5), iPhone 15 Pro (iOS 17.0), iPad Pro 11 (iOS 17.0) and iPad Pro 11 M5 (iOS 26.5) simulators, every check filled in — including the iOS 17 devices the web app cannot serve. Captures in `apps/webapp/screenshots/native/`. |
| **Android app** | **Built and verified.** Same crate, universal APK. Android 15 emulator: "native · tract · 4 threads", about 3.5 s. Debug-signed APK installs and runs; the release workflow signs the same way. |
| Desktop | Tauri 2 + Svelte, `apps/desktop`, unsigned bundles. |
| Release candidate | `.github/workflows/release.yml` builds the APK, the unsigned IPA and the desktop bundles on any `v*` tag and attaches them to a prerelease. Not yet cut: the first tag is `v0.1.0-rc.1`. |
| Store listings | Not started; step 6 below. |

Two things the phone builds taught, both fixed in the tree:

- **Android's resource directory is not a directory.** `resource_dir()`
  returns an `asset://localhost/` URI inside the APK; `std::fs` cannot open
  it and the registry's `create_dir_all` fails with "Read-only file system".
  The app copies the two models out through the fs plugin (which resolves
  assets via the AssetManager) into the app data directory on first launch,
  and points the registry there (`materialise_assets` in
  `apps/mobile/src-tauri/src/lib.rs`). iOS's resource directory is real and
  is used directly.
- **Neither WebView pads for the status bar on its own.** On iOS the CSS
  `env(safe-area-inset-top)` works, but only if the longhand comes *after*
  the `padding` shorthand in `.topbar` (the first attempt had it before, and
  the shorthand silently reset it). On Android the WebView reports zero
  insets whatever the CSS says, so `MainActivity.kt` pads the content view
  by the system bars and the theme's window background matches the page.

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

Steps 1–4 are done (8 September 2026). Step 5 is ready to run and waits for
the push. Step 6 waits for the accounts.

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

Steps 1–5 are routine engineering; step 6 is the one that cannot be started
without the accounts.

## Building the phone apps here

```sh
# once: apps/mobile/node_modules (tauri-cli), the models in .vendor/models
# incl. the tract MODNet from scripts/patch-modnet-for-tract.py
cd apps/mobile
export CARGO_TARGET_DIR=~/.cache/openphotoid-target
npm run ios:build -- --target aarch64-sim          # gen/apple/build/arm64-sim/OpenPhotoId.app
export JAVA_HOME=/Library/Java/JavaVirtualMachines/temurin-21.jdk/Contents/Home
export ANDROID_HOME=/opt/homebrew/share/android-commandlinetools
export NDK_HOME=$ANDROID_HOME/ndk/27.0.12077973
npm run android:build -- --target aarch64          # …/apk/universal/release/app-universal-release-unsigned.apk
npm run ios:build -- --target aarch64 --no-sign --archive-only   # the device build, unsigned:
#   gen/apple/build/openphotoid-mobile_iOS.xcarchive/Products/Applications/OpenPhotoId.app
scripts/run-simulators.sh <ios-udid> emulator-5554 /tmp/captures
```

The iOS app launched with `-diag` runs the device test on its own; on
Android the intent extras never reach the process, so the test is reached
through the footer link. If `tauri ios build` ends with "failed to rename
app … Directory not empty", the build itself succeeded: remove the previous
`build/arm64-sim/OpenPhotoId.app` and run it again.
