# Releasing OpenPhotoId

This is a runbook, not automation — parts of it need credentials nobody
but the project owner should hold (Apple/Microsoft signing identities,
the update-signing private key). Everything that *can* be automated
without those credentials already is (see `.github/workflows/ci.yml`).

## 1. What's already in place

- **Bundling**: `apps/desktop/src-tauri/tauri.conf.json` has
  `bundle.active = true`, targets `["dmg", "app", "msi", "nsis"]`, and a
  full icon set (`icons/*.icns/.ico/.png`, source at
  `icons/app-icon-source.png` — regenerate the whole set from a new
  design with `npx tauri icon path/to/new-source.png`).
- **Updater plugin**: `tauri-plugin-updater` + `tauri-plugin-process` are
  wired into `main.rs`, the frontend has a working "Check for updates"
  button (`App.svelte`), and `capabilities/default.json` grants the
  permissions both plugins need.
- **Update-signing keypair**: generated this session via
  `tauri signer generate`. **The private key
  (`~/.cache/openphotoid-updater-key.pem` on the machine that generated it)
  is a scratch artifact and was never committed anywhere** — before the
  first real release, generate a fresh keypair, store the private key in
  a real secret manager (GitHub Actions secret, 1Password, etc.), and put
  only the public key in `tauri.conf.json`'s `plugins.updater.pubkey`
  (already populated with this session's public key as a placeholder —
  replace it if you regenerate).
- **Third-party licenses**: `THIRD-PARTY-LICENSES.html`, generated via
  `cargo about generate about.hbs -o THIRD-PARTY-LICENSES.html`.
- **License policy enforcement**: `cargo deny check` (licenses,
  advisories, banned sources) runs in CI on every PR.

## 2. What still needs the project owner's action

Nothing below can be done by an agent without real credentials — these
are exactly the steps that require a human with signing authority.

### macOS: Developer ID signing + notarization

1. Enroll in the Apple Developer Program, create a **Developer ID
   Application** certificate in Xcode/Keychain Access.
2. Set these when running `tauri build` (or in CI secrets):
   - `APPLE_SIGNING_IDENTITY` — e.g. `"Developer ID Application: Your Name (TEAMID)"`
   - `APPLE_CERTIFICATE` / `APPLE_CERTIFICATE_PASSWORD` — base64 `.p12` + its password, for CI runners with no local keychain
   - `APPLE_ID`, `APPLE_PASSWORD` (an app-specific password, not the Apple ID password), `APPLE_TEAM_ID` — for `notarytool` submission
3. Tauri's bundler signs and notarizes automatically when these env vars
   are present (`tauri build` on macOS). Verify with:
   ```sh
   codesign -dv --verbose=4 path/to/OpenPhotoId.app
   spctl -a -vv path/to/OpenPhotoId.app   # should say "accepted", not "rejected"
   ```
   (Without these, `codesign -dv` on the .app in this repo's build output
   shows `Signature=adhoc` — fine for local testing, not for distribution.)

### Windows: code signing

1. Obtain an EV (or OV) code-signing certificate from a CA (DigiCert,
   SSL.com, etc.) — EV avoids SmartScreen warnings on first install.
2. Configure via `tauri.conf.json`'s `bundle.windows.certificateThumbprint`
   (cert installed in the Windows cert store) or, for CI, an Azure Key
   Vault-backed `signtool` invocation (see Tauri's Windows signing docs —
   the exact flow depends on which CA/HSM you use).
3. Verify with `signtool verify /pa path\to\OpenPhotoId.msi`.

### Update-signing key rotation

If the placeholder key from this session was ever used for a real
release, rotate it: `tauri signer generate -w <path>`, update
`plugins.updater.pubkey` in `tauri.conf.json`, and set
`TAURI_SIGNING_PRIVATE_KEY` (or `_PATH` + `_PASSWORD`) as a CI secret —
never commit the private key.

## 3. Version bump checklist

Three files currently carry the version independently (no sync script
exists yet — a `scripts/check-versions.mjs` like the sibling `openpdfedit`
project has would be a good follow-up):

- `Cargo.toml` → `[workspace.package] version`
- `apps/desktop/src-tauri/tauri.conf.json` → `"version"`
- `apps/desktop/package.json` → `"version"`

Also update `CHANGELOG.md` (create one at the repo root if this is the
first release — following Keep a Changelog format is a reasonable
default) before tagging.

## 4. Cutting a release (once signing is configured)

```sh
# 1. Bump versions (see §3), update CHANGELOG.md, commit.
# 2. Tag and push — this is the point of no return; only the project
#    owner should run this.
git tag v0.1.0
git push origin v0.1.0

# 3. Build signed bundles per-platform (run on the respective OS, or in
#    CI with the secrets from §2 configured):
cd apps/desktop
npm run tauri build

# 4. Bundles + updater artifacts land in:
#    src-tauri/target/release/bundle/{dmg,msi,nsis}/
#    plus a `latest.json` + `.sig` files if `createUpdaterArtifacts` is
#    enabled in tauri.conf.json's bundle config (not yet enabled — add it
#    once you're ready to actually ship auto-updates: it needs the real
#    signing key from §2 present at build time).

# 5. Create a GitHub release for the tag, upload the bundles + latest.json.
#    The updater endpoint in tauri.conf.json already points at
#    `.../releases/latest/download/latest.json` for this repo.
```

## 5. Models

Models are fetched directly from their upstream sources at runtime
(HivisionIDPhotos' GitHub releases, an ONNX-community HuggingFace repo,
OpenCV Zoo's GitHub) with SHA-256 pinning — see `MODELS.md` and
`crates/frame-engine/src/registry.rs`. There is no OpenPhotoId-operated
model CDN and none is currently needed; if an upstream source ever
disappears, the fix is re-hosting that one model (e.g. as a GitHub
release asset on this repo) and updating its URL in `registry.rs`, not
building new infrastructure.

## 6. Crash reporting

Not implemented. If added, prefer an opt-in, privacy-respecting option
(e.g. local crash-dump directory the user can voluntarily attach to a
bug report) over a third-party telemetry SDK by default, consistent with
the project's local-first privacy positioning (`PLAN.md` §1).
