#!/usr/bin/env bash
# Install the built phone apps on the booted iOS simulator and the running
# Android emulator, launch each straight into the device test, and capture
# the screen. Nothing here needs a tap.
#   scripts/run-simulators.sh <ios-udid> <android-serial> <outdir>
set -euo pipefail
UDID="$1"; SERIAL="$2"; OUT="${3:-/tmp}"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GEN="$HERE/../src-tauri/gen"
ADB=/opt/homebrew/share/android-commandlinetools/platform-tools/adb
mkdir -p "$OUT"

app=$(find "$GEN/apple/build" -name "OpenPhotoId.app" -path "*iphonesimulator*" | head -1)
if [ -n "$app" ]; then
  echo "iOS: $app"
  xcrun simctl install "$UDID" "$app"
  xcrun simctl terminate "$UDID" com.openapps.openphotoid 2>/dev/null || true
  xcrun simctl launch "$UDID" com.openapps.openphotoid -diag
  sleep 45
  xcrun simctl io "$UDID" screenshot "$OUT/ios-native-diag.png" >/dev/null
  echo "  captured $OUT/ios-native-diag.png"
else
  echo "iOS: no simulator .app under $GEN/apple/build"
fi

apk=$(find "$GEN/android" -name "*.apk" | grep -v unaligned | head -1)
if [ -n "$apk" ]; then
  echo "Android: $apk"
  "$ADB" -s "$SERIAL" install -r "$apk" >/dev/null
  "$ADB" -s "$SERIAL" shell am force-stop com.openapps.openphotoid || true
  # `--es` extras do not reach std::env::args; the device test is reached by
  # its footer link instead, tapped by coordinates read from the screen.
  "$ADB" -s "$SERIAL" shell am start -n com.openapps.openphotoid/.MainActivity >/dev/null
  sleep 8
  "$ADB" -s "$SERIAL" exec-out screencap -p > "$OUT/android-native-home.png"
  echo "  captured $OUT/android-native-home.png"
else
  echo "Android: no APK under $GEN/android"
fi
