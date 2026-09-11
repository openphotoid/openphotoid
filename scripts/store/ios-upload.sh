#!/usr/bin/env bash
# Build the App Store archive and upload it to App Store Connect.
#
# Needs, in ~/.config/openphotoid/appstore.env (mode 600):
#   APPLE_DEVELOPMENT_TEAM  the 10-character team id
#   ASC_KEY_ID, ASC_ISSUER_ID, ASC_KEY_PATH   an App Store Connect API key
# and Xcode signed into the Apple ID that owns the team, so automatic
# signing can mint the distribution certificate and profile on first run.
#
# The upload lands in TestFlight. Submitting for review is a separate step
# in App Store Connect and is never done from here.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MOBILE="$HERE/../../apps/mobile"
ENV="${APPSTORE_ENV:-$HOME/.config/openphotoid/appstore.env}"
[ -f "$ENV" ] || { echo "missing $ENV — see store/README.md" >&2; exit 1; }
set -a; . "$ENV"; set +a
for v in APPLE_DEVELOPMENT_TEAM ASC_KEY_ID ASC_ISSUER_ID ASC_KEY_PATH; do
  [ -n "${!v:-}" ] || { echo "$v is not set in $ENV" >&2; exit 1; }
done
[ -f "$ASC_KEY_PATH" ] || { echo "no key at $ASC_KEY_PATH" >&2; exit 1; }
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/.cache/openphotoid-target}"

cd "$MOBILE"
# The team id reaches Xcode through Tauri's config; the CLI archives and
# exports with the app-store-connect method, signed automatically.
node_modules/.bin/tauri ios build --target aarch64 --export-method app-store-connect \
  --config "{\"bundle\":{\"iOS\":{\"developmentTeam\":\"$APPLE_DEVELOPMENT_TEAM\"}}}"

IPA=$(find src-tauri/gen/apple/build -name "*.ipa" -newer src-tauri/tauri.conf.json | head -1)
[ -n "$IPA" ] || { echo "no .ipa produced under src-tauri/gen/apple/build" >&2; exit 1; }
echo "uploading $IPA"
xcrun altool --upload-app -f "$IPA" -t ios \
  --apiKey "$ASC_KEY_ID" --apiIssuer "$ASC_ISSUER_ID" \
  --private-key-path "$ASC_KEY_PATH"
echo "done: it appears in App Store Connect → TestFlight in a few minutes"
