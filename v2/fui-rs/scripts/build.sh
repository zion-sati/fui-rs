#!/usr/bin/env bash

set -euo pipefail

PACKAGE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO_ROOT="$(cd "${PACKAGE_DIR}/../.." && pwd)"
OUT_DIR="${REPO_ROOT}/public/v2/fui-rs"
BRIDGE_DIR="${REPO_ROOT}/public/v2/browser-bridge"

mkdir -p "${PACKAGE_DIR}/build" "${OUT_DIR}"

cd "${PACKAGE_DIR}"
if ! command -v cargo >/dev/null 2>&1 && [ -f "${HOME}/.cargo/env" ]; then
  # shellcheck disable=SC1090
  source "${HOME}/.cargo/env"
fi
if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not found. Install Rust and ensure cargo is on PATH." >&2
  exit 1
fi
cargo build --target wasm32-unknown-unknown --release

cp "${PACKAGE_DIR}/target/wasm32-unknown-unknown/release/fui_rs.wasm" "${PACKAGE_DIR}/build/app.wasm"
cp "${PACKAGE_DIR}/build/app.wasm" "${OUT_DIR}/app.wasm"

npx esbuild "${PACKAGE_DIR}/browser/harness.ts" \
  --bundle \
  --format=esm \
  --platform=browser \
  --target=es2020 \
  --minify \
  --outfile="${OUT_DIR}/harness.js" \
  --sourcemap

cp "${PACKAGE_DIR}/browser/index.html" "${OUT_DIR}/index.html"
cat > "${OUT_DIR}/effindom-runtime-config.js" << 'RUNTIMECFG'
window.__effindomRuntime = Object.assign({}, window.__effindomRuntime, {
  manifestUrl: './effindom.v2.manifest.json',
});
RUNTIMECFG
cp "${BRIDGE_DIR}/bridge.js" "${OUT_DIR}/bridge.js"
cp "${BRIDGE_DIR}/bridge.js.map" "${OUT_DIR}/bridge.js.map"
cp "${BRIDGE_DIR}/effindom.v2.manifest.json" "${OUT_DIR}/effindom.v2.manifest.json"
if [ -f "${BRIDGE_DIR}/icu-asset.json" ]; then
  cp "${BRIDGE_DIR}/icu-asset.json" "${OUT_DIR}/icu-asset.json"
fi
rm -rf "${OUT_DIR}/runtime"
cp -R "${BRIDGE_DIR}/runtime" "${OUT_DIR}/runtime"
