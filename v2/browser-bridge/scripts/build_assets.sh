#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
OUT_DIR="${REPO_ROOT}/public/v2/browser-bridge"
UI_OUT_DIR="${REPO_ROOT}/public/v2/ui"
source "${REPO_ROOT}/v2/browser-bridge/scripts/font_assets.sh"
MANIFEST_FILE="${OUT_DIR}/effindom.v2.manifest.json"
RUNTIME_DIR="${OUT_DIR}/runtime"

mkdir -p "${OUT_DIR}" "${UI_OUT_DIR}"

if [ ! -f "${MANIFEST_FILE}" ] || [ ! -d "${RUNTIME_DIR}" ]; then
  echo "Missing staged browser-bridge runtime assets in ${OUT_DIR}." >&2
  echo "Run 'npm run build:v2:browser-bridge' first, then use this lightweight asset build." >&2
  exit 1
fi

find "${OUT_DIR}" -maxdepth 1 -type f \( -name '*.ttf' -o -name '*.otf' -o -name '*.woff' -o -name '*.woff2' \) -delete
rm -rf "${OUT_DIR}/fonts"

npx esbuild "${REPO_ROOT}/v2/browser-bridge/src/bridge.ts" \
  --bundle \
  --format=iife \
  --platform=browser \
  --target=es2020 \
  --minify \
  --outfile="${OUT_DIR}/bridge.js" \
  --sourcemap

npx esbuild "${REPO_ROOT}/v2/browser-bridge/src/harness.ts" \
  --bundle \
  --format=iife \
  --platform=browser \
  --target=es2020 \
  --minify \
  --outfile="${OUT_DIR}/harness.js" \
  --sourcemap

npx esbuild "${REPO_ROOT}/v2/ui/browser/bridge-harness.ts" \
  --bundle \
  --format=iife \
  --platform=browser \
  --target=es2020 \
  --minify \
  --outfile="${UI_OUT_DIR}/bridge-harness.js" \
  --sourcemap

cp "${REPO_ROOT}/v2/browser-bridge/index.html" "${OUT_DIR}/index.html"
copy_bridge_font_assets "${REPO_ROOT}/public/v2/fonts"
node "${REPO_ROOT}/v2/browser-bridge/scripts/finalize_runtime_manifest.mjs" \
  "${OUT_DIR}" \
  "${REPO_ROOT}/public/v2/fonts" \
  "../fonts"
