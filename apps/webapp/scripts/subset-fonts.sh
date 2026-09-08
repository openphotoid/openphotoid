#!/usr/bin/env bash
# Subsets the four Geist faces the page uses into WOFF2 under src/fonts/.
#
# The shared token kit ships full OTFs (~160 KB each, every script Geist
# covers). This page renders English and Simplified Chinese; Geist has no
# CJK glyphs at all, so those fall through to the system font whatever we
# ship, and the Latin the page does use fits in a fraction of the file.
# Latin + Latin Extended-A/B (country names: Türkiye, São Tomé, Côte
# d'Ivoire), general punctuation, currency, a few arrows and math signs.
#
# Re-run after a token-kit font update; the outputs are committed so the
# build needs no Python.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WEBAPP_DIR="$(dirname "$SCRIPT_DIR")"
SRC="$WEBAPP_DIR/../desktop/src/design/assets/fonts"
OUT="$WEBAPP_DIR/src/fonts"
RANGES="U+0000-024F,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+2000-206F,U+20A0-20CF,U+2113,U+2122,U+2190-2199,U+2212,U+2215,U+2264-2265,U+25A0-25FF,U+FEFF,U+FFFD"
command -v pyftsubset >/dev/null || { echo "pyftsubset missing: pip3 install fonttools brotli" >&2; exit 1; }
mkdir -p "$OUT"
for face in Geist-Regular Geist-Medium Geist-Bold GeistMono-Regular; do
  pyftsubset "$SRC/$face.otf" --unicodes="$RANGES" --flavor=woff2 \
    --layout-features='kern,liga,calt,tnum,ss01' --no-hinting --desubroutinize \
    --output-file="$OUT/$face.woff2"
  printf '  %-22s %7d -> %6d bytes\n' "$face" "$(stat -f%z "$SRC/$face.otf")" "$(stat -f%z "$OUT/$face.woff2")"
done
