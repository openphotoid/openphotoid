# Store submission

Everything a Play Console or App Store Connect listing asks for, prepared;
the two things only an account holder can do are marked **you**.

```
store/
├── README.md              this file: what is done, what you do, in order
├── listing/<lang>.json    name, subtitle, descriptions, keywords, release notes × 8 languages
├── screenshots/           captured from the app at each store's required pixel sizes, × 8 languages
│   ├── appstore-iphone-6.9/   1320×2868
│   ├── appstore-ipad-13/      2064×2752
│   ├── play-phone/            1080×2400
│   └── play-tablet-10/        1600×2560
├── feature-graphic.png    Google Play, 1024×500
└── answers.md             the review, privacy, data-safety and age-rating answers
```

Regenerate the screenshots after any UI change with the app served locally
(`npm --prefix apps/webapp run serve`, then
`node apps/webapp/e2e/capture-store.mjs store/screenshots`).

## Google Play

**Done.** A signed App Bundle is what Play takes; Play then splits it per
device, so the 259 MB universal APK on the GitHub release never reaches a
phone through the store.

- Upload key: `~/.config/openphotoid/upload-keystore.jks`, alias `upload`,
  RSA 4096, generated 11 September 2026 on this Mac. Its password is in
  `~/.config/openphotoid/keystore.properties` (mode 600). **Back both up
  somewhere that is not this machine.** Play can reset an upload key on
  request, but a lost one still costs a support round-trip.
- `apps/mobile/src-tauri/gen/android/keystore.properties` (gitignored) points
  Gradle at it; a release build with that file present is signed with the
  upload key, and without it is left unsigned, which is what CI produces.
- Build: `cd apps/mobile && npm run tauri -- android build --aab` →
  `src-tauri/gen/android/app/build/outputs/bundle/universalRelease/app-universal-release.aab`.
  Verify with `jarsigner -verify -verbose:summary <aab>`.

**You.**

1. A Play Console developer account for the `openphotoid@gmail.com`
   identity (one-time fee), identity verification, then *Create app*:
   name `OpenPhotoId`, default language `en-US`, app, free.
2. Under *Release → Setup → App signing* accept Play App Signing (Play holds
   the app signing key; ours is only the upload key).
3. *Store presence → Main store listing*: paste `listing/<lang>.json` per
   language (Play locale codes are in each file as `play_locale`); upload
   `screenshots/play-phone/<lang>/*.png`, `screenshots/play-tablet-10/<lang>/*.png`,
   `feature-graphic.png`, and `apps/mobile/src-tauri/icons/icon.png` (1024×1024) as the icon.
4. *Policy → App content*: privacy policy `https://openphotoid.com/privacy.html`;
   data safety and content rating answers are in `answers.md`.
5. *Release → Testing → Internal testing*: upload the `.aab`, add yourself
   as a tester, install from the opt-in link and run the app's *Device
   test* from the home page footer on a real phone. Then promote to
   Production and **stop at review**: publishing is a separate button and
   a separate decision.

## App Store

**Done.** The Xcode project archives unsigned today; signing is one setting.

- `apps/mobile/src-tauri/gen/apple/PrivacyInfo.xcprivacy`: no tracking, no
  collected data, the two required-reason APIs a WebView app touches.
- `Info.plist` (from `project.yml`): camera and photo-library usage strings,
  `ITSAppUsesNonExemptEncryption = false` so every build skips the export
  compliance question.
- `scripts/store/ios-upload.sh`: builds the App Store archive and uploads it
  with an App Store Connect API key, once the three values below exist.

**You.**

1. Apple Developer Program membership for the `openphotoid@gmail.com`
   Apple ID (annual fee; D-U-N-S is only needed for an organisation, an
   individual account is fine and lists the app under your name).
2. In App Store Connect → *Users and Access → Integrations → App Store
   Connect API*, create a key with the *App Manager* role; note the Key ID,
   Issuer ID, and download the `.p8` once. Put them in
   `~/.config/openphotoid/appstore.env` (mode 600):
   ```
   APPLE_DEVELOPMENT_TEAM=XXXXXXXXXX
   ASC_KEY_ID=XXXXXXXXXX
   ASC_ISSUER_ID=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
   ASC_KEY_PATH=/Users/.../AuthKey_XXXXXXXXXX.p8
   ```
3. Register the bundle ID `com.openapps.openphotoid` (Certificates,
   Identifiers & Profiles → Identifiers) and create the app record in App
   Store Connect: name `OpenPhotoId`, primary language English (U.S.),
   SKU `openphotoid`.
4. `bash scripts/store/ios-upload.sh` — Xcode's automatic signing creates
   the distribution certificate and profile on first run when signed into
   Xcode with that Apple ID (`Xcode → Settings → Accounts`). The build
   lands in TestFlight; install it there and run the *Device test*.
5. *App Store → iOS App → 1.0 Prepare for Submission*: paste
   `listing/<lang>.json` per language (`appstore_locale` in each file),
   upload `screenshots/appstore-iphone-6.9/<lang>/*.png` and
   `screenshots/appstore-ipad-13/<lang>/*.png`, privacy policy URL, the
   App Privacy answers and the age rating from `answers.md`, then *Add for
   Review* and **stop there**: submitting for review is your button.

## Both

- The RC on GitHub stays the sideload route; store builds come from the
  same commit. Bump `version` in `apps/mobile/src-tauri/tauri.conf.json`
  and `CFBundleShortVersionString` / `CFBundleVersion` in
  `gen/apple/project.yml` together (`xcodegen generate` after), and Play's
  `versionCode` from `tauri.properties`, for every upload: stores refuse a
  build number they have seen.
- Listing copy is written, not machine-translated, and each file was
  checked against the field limits (30 / 30 / 80 / 170 / 100 characters).
