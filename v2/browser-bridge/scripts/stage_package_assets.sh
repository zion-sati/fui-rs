#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
PACKAGE_DIR="${REPO_ROOT}/v2/browser-bridge"
BRIDGE_PUBLIC_DIR="${REPO_ROOT}/public/v2/browser-bridge"
PACKAGE_DIST_DIR="${PACKAGE_DIR}/dist"
source "${REPO_ROOT}/v2/browser-bridge/scripts/font_assets.sh"

require_path() {
  local path="$1"
  local label="$2"
  if [ ! -e "${path}" ]; then
    echo "Missing ${label}: ${path}" >&2
    echo "Run 'npm run build:v2:browser-bridge' (or at least 'npm run build:v2:browser-bridge-assets') first." >&2
    exit 1
  fi
}

require_path "${BRIDGE_PUBLIC_DIR}/bridge.js" "bridge bundle"
require_path "${BRIDGE_PUBLIC_DIR}/harness.js" "harness bundle"
require_path "${BRIDGE_PUBLIC_DIR}/effindom.v2.manifest.json" "runtime manifest"
require_path "${BRIDGE_PUBLIC_DIR}/runtime" "runtime artifact directory"

rm -rf "${PACKAGE_DIST_DIR}"
mkdir -p "${PACKAGE_DIST_DIR}"

cp "${BRIDGE_PUBLIC_DIR}/bridge.js" "${PACKAGE_DIST_DIR}/bridge.js"
cp "${BRIDGE_PUBLIC_DIR}/bridge.js.map" "${PACKAGE_DIST_DIR}/bridge.js.map"
cp "${BRIDGE_PUBLIC_DIR}/harness.js" "${PACKAGE_DIST_DIR}/harness.js"
cp "${BRIDGE_PUBLIC_DIR}/harness.js.map" "${PACKAGE_DIST_DIR}/harness.js.map"
cp "${BRIDGE_PUBLIC_DIR}/effindom.v2.manifest.json" "${PACKAGE_DIST_DIR}/effindom.v2.manifest.json"
cp "${BRIDGE_PUBLIC_DIR}/index.html" "${PACKAGE_DIST_DIR}/index.html"
cp -R "${BRIDGE_PUBLIC_DIR}/runtime" "${PACKAGE_DIST_DIR}/runtime"

if [ -f "${BRIDGE_PUBLIC_DIR}/icu-asset.json" ]; then
  cp "${BRIDGE_PUBLIC_DIR}/icu-asset.json" "${PACKAGE_DIST_DIR}/icu-asset.json"
fi

rm -rf "${PACKAGE_DIST_DIR}/fonts"
copy_runtime_package_font_assets "${PACKAGE_DIST_DIR}/fonts"
