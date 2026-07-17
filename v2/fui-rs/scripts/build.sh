#!/usr/bin/env bash

set -euo pipefail

PACKAGE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO_ROOT="$(cd "${PACKAGE_DIR}/../.." && pwd)"
OUT_DIR="${REPO_ROOT}/public/v2/fui-rs"
DEMO_OUT_DIR="${OUT_DIR}/demo"
PUBLIC_BRIDGE_DIR="${REPO_ROOT}/public/v2/browser-bridge"
PACKAGE_BRIDGE_DIR="${REPO_ROOT}/v2/browser-bridge/dist"
SHARED_FONTS_DIR="${REPO_ROOT}/public/v2/fonts"
SHARED_DEMO_TEXTURE="${REPO_ROOT}/v2/fui-as/demo/demo-texture.png"
HOST_SERVICE_GENERATOR_BUILD="${PACKAGE_DIR}/build/generate-host-services.mjs"
HOST_EVENT_GENERATOR_BUILD="${PACKAGE_DIR}/build/generate-host-events.mjs"
WORKER_BOOTSTRAP_BUILD="${PACKAGE_DIR}/build/worker-bootstrap.js"
WORKER_BOOTSTRAP_MAP_BUILD="${PACKAGE_DIR}/build/worker-bootstrap.js.map"

mkdir -p "${PACKAGE_DIR}/build" "${OUT_DIR}" "${DEMO_OUT_DIR}" "${DEMO_OUT_DIR}/workbench" "${DEMO_OUT_DIR}/stage4" "${DEMO_OUT_DIR}/stage5" "${DEMO_OUT_DIR}/immediate-drawing"

cd "${PACKAGE_DIR}"
if ! command -v cargo >/dev/null 2>&1 && [ -f "${HOME}/.cargo/env" ]; then
  # shellcheck disable=SC1090
  source "${HOME}/.cargo/env"
fi
if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not found. Install Rust and ensure cargo is on PATH." >&2
  exit 1
fi

resolve_runtime_dist_dir() {
  local candidate=""
  local public_manifest="${PUBLIC_BRIDGE_DIR}/effindom.v2.manifest.json"
  local package_manifest="${PACKAGE_BRIDGE_DIR}/effindom.v2.manifest.json"
  local candidates=()

  if [ -n "${EFFINDOM_RUNTIME_DIST_DIR:-}" ]; then
    candidates+=("${EFFINDOM_RUNTIME_DIST_DIR}")
  fi

  candidates+=(
    "${PUBLIC_BRIDGE_DIR}"
    "${PACKAGE_BRIDGE_DIR}"
    "${PACKAGE_DIR}/node_modules/@effindomv2/runtime/dist"
    "${REPO_ROOT}/node_modules/@effindomv2/runtime/dist"
  )

  if [ -f "${public_manifest}" ] && [ -f "${package_manifest}" ] && ! cmp -s "${public_manifest}" "${package_manifest}"; then
    echo "Note: using ${PUBLIC_BRIDGE_DIR} for local runtime assets." >&2
    echo "      ${PACKAGE_BRIDGE_DIR} is a staged package copy and may be stale." >&2
    echo "      For ABI changes, run repo-root ./build.sh (or npm run build:v2:abi)." >&2
  fi

  for candidate in "${candidates[@]}"; do
    if [ -f "${candidate}/bridge.js" ] && [ -f "${candidate}/effindom.v2.manifest.json" ] && [ -d "${candidate}/runtime" ]; then
      printf '%s\n' "${candidate}"
      return 0
    fi
  done

  echo "Could not locate runtime dist assets." >&2
  echo "Expected one of:" >&2
  echo "  - \$EFFINDOM_RUNTIME_DIST_DIR" >&2
  echo "  - ${PACKAGE_DIR}/node_modules/@effindomv2/runtime/dist" >&2
  echo "  - ${REPO_ROOT}/node_modules/@effindomv2/runtime/dist" >&2
  echo "  - ${REPO_ROOT}/public/v2/browser-bridge" >&2
  echo "  - ${REPO_ROOT}/v2/browser-bridge/dist" >&2
  echo "Install @effindomv2/runtime or build runtime assets first." >&2
  exit 1
}

RUNTIME_DIST_DIR="$(resolve_runtime_dist_dir)"
optimize_wasm() {
  local wasm_file="$1"

  if command -v wasm-opt >/dev/null 2>&1; then
    wasm-opt -O3 --strip-debug --strip-producers "${wasm_file}" -o "${wasm_file}"
  else
    echo "wasm-opt not found; skipping Binaryen speed optimization for ${wasm_file}." >&2
  fi
}

generate_host_services() {
  local definition_file="$1"
  local export_name="$2"
  local output_file="$3"
  local runtime_path="${4:-}"
  local host_import_module="${5:-}"

  npx esbuild "${PACKAGE_DIR}/scripts/generate-host-services.ts" \
    --bundle \
    --format=esm \
    --platform=node \
    --target=node20 \
    --packages=external \
    --outfile="${HOST_SERVICE_GENERATOR_BUILD}"

  if [ -n "${runtime_path}" ] && [ -n "${host_import_module}" ]; then
    node "${HOST_SERVICE_GENERATOR_BUILD}" \
      "${definition_file}" "${export_name}" "${output_file}" "${runtime_path}" "${host_import_module}"
  elif [ -n "${runtime_path}" ]; then
    node "${HOST_SERVICE_GENERATOR_BUILD}" "${definition_file}" "${export_name}" "${output_file}" "${runtime_path}"
  elif [ -n "${host_import_module}" ]; then
    node "${HOST_SERVICE_GENERATOR_BUILD}" "${definition_file}" "${export_name}" "${output_file}" "" "${host_import_module}"
  else
    node "${HOST_SERVICE_GENERATOR_BUILD}" "${definition_file}" "${export_name}" "${output_file}"
  fi
}

generate_host_events() {
  local definition_file="$1"
  local export_name="$2"
  local output_file="$3"

  npx esbuild "${PACKAGE_DIR}/scripts/generate-host-events.ts" \
    --bundle \
    --format=esm \
    --platform=node \
    --target=node20 \
    --packages=external \
    --outfile="${HOST_EVENT_GENERATOR_BUILD}"

  node "${HOST_EVENT_GENERATOR_BUILD}" "${definition_file}" "${export_name}" "${output_file}"
}

generate_host_services "demo/src/host-services.ts" "demoHostServices" "crates/demo-shared/src/generated/host_services.rs" "fui::host_services"
generate_host_events "demo/src/host-events.ts" "demoHostEvents" "crates/demo-home/src/generated/host_events.rs"
generate_host_events "demo/src/host-events.ts" "demoHostEvents" "crates/demo-workbench/src/generated/host_events.rs"
generate_host_events "demo/src/host-events.ts" "demoHostEvents" "crates/demo-stage4/src/generated/host_events.rs"
generate_host_events "demo/src/host-events.ts" "demoHostEvents" "crates/demo-stage5/src/generated/host_events.rs"
generate_host_events "demo/src/host-events.ts" "demoHostEvents" "crates/demo-immediate-drawing/src/generated/host_events.rs"
generate_host_services "demo/src/worker-host-services.ts" "demoWorkerHostServices" "crates/demo-shared/src/generated/worker_host_services.rs" "fui::host_services" "fui_host_service"
generate_host_services "demo/src/worker-host-services.ts" "demoWorkerHostServices" "crates/demo-worker/src/generated/worker_host_services.rs" "" "fui_host_service"
generate_host_services "scripts/framework-host-services.ts" "frameworkHostServices" "src/generated/framework_host_services.rs" "crate::host_services" "fui_host"

rm -f "${OUT_DIR}/app.wasm" "${OUT_DIR}/harness.js" "${OUT_DIR}/harness.js.map" "${DEMO_OUT_DIR}/demo.wasm"

build_demo_route() {
  local package="$1"
  local wasm_name="$2"
  local wasm_out="$3"

  cargo build --manifest-path "${PACKAGE_DIR}/crates/Cargo.toml" --package "${package}" --target wasm32-unknown-unknown --target-dir "${PACKAGE_DIR}/target" --release
  cp "${PACKAGE_DIR}/target/wasm32-unknown-unknown/release/${wasm_name}.wasm" "${wasm_out}"
  optimize_wasm "${wasm_out}"
}

build_demo_route "fui-rs-demo-home" "fui_rs_demo_home" "${DEMO_OUT_DIR}/home.wasm"
build_demo_route "fui-rs-demo-workbench" "fui_rs_demo_workbench" "${DEMO_OUT_DIR}/workbench.wasm"
build_demo_route "fui-rs-demo-stage4" "fui_rs_demo_stage4" "${DEMO_OUT_DIR}/stage4.wasm"
build_demo_route "fui-rs-demo-stage5" "fui_rs_demo_stage5" "${DEMO_OUT_DIR}/stage5.wasm"
build_demo_route "fui-rs-demo-immediate-drawing" "fui_rs_demo_immediate_drawing" "${DEMO_OUT_DIR}/immediate-drawing.wasm"
build_demo_route "fui-rs-demo-worker" "fui_rs_demo_worker" "${DEMO_OUT_DIR}/workers.wasm"
cp "${DEMO_OUT_DIR}/workers.wasm" "${DEMO_OUT_DIR}/workbench/workers.wasm"
cp "${DEMO_OUT_DIR}/workers.wasm" "${DEMO_OUT_DIR}/stage4/workers.wasm"
cp "${DEMO_OUT_DIR}/workers.wasm" "${DEMO_OUT_DIR}/stage5/workers.wasm"
cp "${DEMO_OUT_DIR}/workers.wasm" "${DEMO_OUT_DIR}/immediate-drawing/workers.wasm"

npx esbuild "${PACKAGE_DIR}/demo/harness.ts" \
  --bundle \
  --format=esm \
  --platform=browser \
  --target=es2020 \
  --minify \
  --outfile="${DEMO_OUT_DIR}/harness.js" \
  --sourcemap

npx esbuild "${PACKAGE_DIR}/demo/worker-host-services.ts" \
  --bundle \
  --format=iife \
  --platform=browser \
  --target=es2020 \
  --minify \
  --outfile="${DEMO_OUT_DIR}/worker-host-services.js" \
  --sourcemap

npx esbuild "${REPO_ROOT}/v2/browser-bridge/src/managed-harness/worker-bootstrap.ts" \
  --bundle \
  --format=iife \
  --platform=browser \
  --target=es2020 \
  --minify \
  --outfile="${WORKER_BOOTSTRAP_BUILD}" \
  --sourcemap

cp "${WORKER_BOOTSTRAP_BUILD}" "${DEMO_OUT_DIR}/worker-bootstrap.js"
cp "${WORKER_BOOTSTRAP_MAP_BUILD}" "${DEMO_OUT_DIR}/worker-bootstrap.js.map"

cp "${PACKAGE_DIR}/browser/index.html" "${OUT_DIR}/index.html"
cp "${PACKAGE_DIR}/demo/index.html" "${DEMO_OUT_DIR}/index.html"
cp "${PACKAGE_DIR}/demo/route-shell.html" "${DEMO_OUT_DIR}/workbench/index.html"
cp "${PACKAGE_DIR}/demo/route-shell.html" "${DEMO_OUT_DIR}/stage4/index.html"
cp "${PACKAGE_DIR}/demo/route-shell.html" "${DEMO_OUT_DIR}/stage5/index.html"
cp "${PACKAGE_DIR}/demo/route-shell.html" "${DEMO_OUT_DIR}/immediate-drawing/index.html"
cp "${PACKAGE_DIR}/demo/routes.json" "${DEMO_OUT_DIR}/routes.json"
rm -f "${DEMO_OUT_DIR}/worker-manifest.json"
if [ -f "${SHARED_DEMO_TEXTURE}" ]; then
  cp "${SHARED_DEMO_TEXTURE}" "${OUT_DIR}/demo-texture.png"
  cp "${SHARED_DEMO_TEXTURE}" "${DEMO_OUT_DIR}/demo-texture.png"
  cp "${SHARED_DEMO_TEXTURE}" "${DEMO_OUT_DIR}/workbench/demo-texture.png"
  cp "${SHARED_DEMO_TEXTURE}" "${DEMO_OUT_DIR}/stage4/demo-texture.png"
  cp "${SHARED_DEMO_TEXTURE}" "${DEMO_OUT_DIR}/stage5/demo-texture.png"
  cp "${SHARED_DEMO_TEXTURE}" "${DEMO_OUT_DIR}/immediate-drawing/demo-texture.png"
fi
RUNTIME_SET_HASH="$(node -e 'const fs=require("fs"); const value=JSON.parse(fs.readFileSync(process.argv[1], "utf8")).runtime_set_hash; if (typeof value !== "string" || value.length === 0) process.exit(1); process.stdout.write(value);' "${RUNTIME_DIST_DIR}/effindom.v2.manifest.json")"
cat > "${OUT_DIR}/effindom-runtime-config.js" << RUNTIMECFG
window.__effindomRuntime = Object.assign({}, window.__effindomRuntime, {
  manifestUrls: [
    'https://runtimes.effindom.dev/v2/manifests/${RUNTIME_SET_HASH}.json',
    './effindom.v2.manifest.json',
  ],
  expectedRuntimeSetHash: '${RUNTIME_SET_HASH}',
});
RUNTIMECFG
cp "${OUT_DIR}/effindom-runtime-config.js" "${DEMO_OUT_DIR}/effindom-runtime-config.js"
cp "${RUNTIME_DIST_DIR}/bridge.js" "${OUT_DIR}/bridge.js"
cp "${RUNTIME_DIST_DIR}/bridge.js.map" "${OUT_DIR}/bridge.js.map"
cp "${RUNTIME_DIST_DIR}/effindom.v2.manifest.json" "${OUT_DIR}/effindom.v2.manifest.json"
cp "${OUT_DIR}/bridge.js" "${DEMO_OUT_DIR}/bridge.js"
cp "${OUT_DIR}/bridge.js.map" "${DEMO_OUT_DIR}/bridge.js.map"
cp "${OUT_DIR}/effindom.v2.manifest.json" "${DEMO_OUT_DIR}/effindom.v2.manifest.json"
if [ -f "${RUNTIME_DIST_DIR}/icu-asset.json" ]; then
  cp "${RUNTIME_DIST_DIR}/icu-asset.json" "${OUT_DIR}/icu-asset.json"
  cp "${OUT_DIR}/icu-asset.json" "${DEMO_OUT_DIR}/icu-asset.json"
fi
rm -rf "${OUT_DIR}/runtime"
cp -R "${RUNTIME_DIST_DIR}/runtime" "${OUT_DIR}/runtime"
rm -rf "${OUT_DIR}/fonts"
rm -rf "${DEMO_OUT_DIR}/fonts"
if [ -d "${SHARED_FONTS_DIR}" ]; then
  cp -R "${SHARED_FONTS_DIR}" "${OUT_DIR}/fonts"
  cp -R "${SHARED_FONTS_DIR}" "${DEMO_OUT_DIR}/fonts"
elif [ -d "${RUNTIME_DIST_DIR}/fonts" ]; then
  cp -R "${RUNTIME_DIST_DIR}/fonts" "${OUT_DIR}/fonts"
  cp -R "${RUNTIME_DIST_DIR}/fonts" "${DEMO_OUT_DIR}/fonts"
fi
rm -rf "${DEMO_OUT_DIR}/runtime"
cp -R "${RUNTIME_DIST_DIR}/runtime" "${DEMO_OUT_DIR}/runtime"
