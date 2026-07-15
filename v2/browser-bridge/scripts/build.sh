#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
OUT_DIR="${REPO_ROOT}/public/v2/browser-bridge"
source "${REPO_ROOT}/v2/browser-bridge/scripts/font_assets.sh"
MANIFEST_SCRIPT="${REPO_ROOT}/v2/browser-bridge/scripts/generate_manifest.py"
MANIFEST_FILE="${OUT_DIR}/effindom.v2.manifest.json"

ICU_ROOT=""
ICU_ROOT_CANDIDATES=(
  "${REPO_ROOT}/build/build-v2-ui/_deps/effindom_skia_pinned_icu-src"
  "${REPO_ROOT}/build/build-v2-ui-wasm32/_deps/effindom_skia_pinned_icu-src"
  "${REPO_ROOT}/build/build-v2-core/_deps/effindom_skia_pinned_icu-src"
  "${REPO_ROOT}/build/build-v2-core-wasm32/_deps/effindom_skia_pinned_icu-src"
)
ICU_BUILD_DIR="${REPO_ROOT}/build/build-v2-browser-bridge-icu"
ICU_FILTER="${REPO_ROOT}/v2/browser-bridge/icu-filter.json"
ICU_CONFIG_STAMP="${ICU_BUILD_DIR}/.effindom-icu-config"
ICU_CONFIG_STATUS="${ICU_BUILD_DIR}/config.status"
ICU_JOBS="${ICU_JOBS:-4}"

rm -rf "${OUT_DIR}"
mkdir -p "${REPO_ROOT}/build"
STAGE_DIR="$(mktemp -d "${REPO_ROOT}/build/build-v2-browser-bridge-stage-XXXXXX")"

if ! command -v emcmake >/dev/null 2>&1 || ! command -v emcc >/dev/null 2>&1; then
  if [ -f "${HOME}/emsdk/emsdk_env.sh" ]; then
    # shellcheck disable=SC1091
    source "${HOME}/emsdk/emsdk_env.sh" >/dev/null
  fi
fi

mkdir -p "${OUT_DIR}"

SKIA_WORKDIR_BASE="${SKIA_BUILD_WORKDIR:-${HOME}/.cache/effindom-skia-build}"
parallel_variant_pids=()
parallel_variant_names=()
parallel_variant_logs=()
SKIA_SOURCE_SEED_WORKDIR="${SKIA_WORKDIR_BASE}/seed"

bold()  { printf '\033[1m%s\033[0m\n' "$*"; }
green() { printf '\033[32m%s\033[0m\n' "$*"; }
yellow(){ printf '\033[33m%s\033[0m\n' "$*"; }
red()   { printf '\033[31m%s\033[0m\n' "$*"; }

collect_descendant_pids() {
  local frontier=("$@")
  local next=()
  local pid=""
  local ppid=""

  while [ "${#frontier[@]}" -gt 0 ]; do
    next=()
    while read -r pid ppid; do
      [ -n "${pid}" ] || continue
      [ -n "${ppid}" ] || continue
      for parent in "${frontier[@]}"; do
        if [ "${ppid}" = "${parent}" ]; then
          next+=("${pid}")
          break
        fi
      done
    done < <(ps -axo pid=,ppid=)

    if [ "${#next[@]}" -eq 0 ]; then
      break
    fi

    printf '%s\n' "${next[@]}"
    frontier=("${next[@]}")
  done
}

kill_process_tree() {
  local roots=("$@")
  local descendants=()
  local pid=""

  while IFS= read -r pid; do
    [ -n "${pid}" ] && descendants+=("${pid}")
  done < <(collect_descendant_pids "${roots[@]}")

  for pid in "${descendants[@]}"; do
    kill -TERM "${pid}" >/dev/null 2>&1 || true
  done
  for pid in "${roots[@]}"; do
    kill -TERM "${pid}" >/dev/null 2>&1 || true
  done

  sleep 2

  descendants=()
  while IFS= read -r pid; do
    [ -n "${pid}" ] && descendants+=("${pid}")
  done < <(collect_descendant_pids "${roots[@]}")

  for pid in "${descendants[@]}" "${roots[@]}"; do
    kill -KILL "${pid}" >/dev/null 2>&1 || true
  done
}

cleanup() {
  rm -rf "${STAGE_DIR}"
}

trap cleanup EXIT

cleanup_on_signal() {
  trap - EXIT INT TERM HUP
  if [ "${#parallel_variant_pids[@]}" -gt 0 ]; then
    kill_process_tree "${parallel_variant_pids[@]}"
  fi
  exit 130
}

trap cleanup_on_signal INT TERM HUP

launch_variant_build() {
  local architecture_name="$1"
  local wasm_arch="$2"
  local simd_mode="$3"
  local log_file

  log_file="$(mktemp "${STAGE_DIR}/${architecture_name}.XXXXXX.log")"
  (
    stage_variant "${architecture_name}" "${wasm_arch}" "${simd_mode}"
  ) >"${log_file}" 2>&1 &
  parallel_variant_pids+=("$!")
  parallel_variant_names+=("${architecture_name}")
  parallel_variant_logs+=("${log_file}")
}

prepare_skia_seed() {
  if [ -d "${SKIA_SOURCE_SEED_WORKDIR}/skia" ] && [ -d "${SKIA_SOURCE_SEED_WORKDIR}/depot_tools" ]; then
    bold "-- Reusing seeded Skia checkout at ${SKIA_SOURCE_SEED_WORKDIR}"
    return
  fi

  bold "-- Seeding shared Skia checkout at ${SKIA_SOURCE_SEED_WORKDIR}"
  rm -rf "${SKIA_SOURCE_SEED_WORKDIR}"
  mkdir -p "${SKIA_SOURCE_SEED_WORKDIR}"
  cd "${REPO_ROOT}"
  SKIA_BUILD_WORKDIR="${SKIA_SOURCE_SEED_WORKDIR}" \
  EFFINDOM_WASM_ARCH="wasm32" \
  EFFINDOM_SIMD="off" \
  SKIA_PREP_ONLY=1 \
  ./scripts/build_skia_wasm.sh
}

copy_skia_seed_to_workdir() {
  local target_workdir="$1"
  local existing_entries=""

  existing_entries="$(find "${target_workdir}" -mindepth 1 -maxdepth 1 -print -quit 2>/dev/null || true)"
  if [ -n "${existing_entries}" ] && [ -f "${target_workdir}/depot_tools/python3_bin_reldir.txt" ]; then
    bold "   Reusing existing workdir ${target_workdir}"
    return
  fi

  bold "   Copying Skia seed into ${target_workdir}"
  mkdir -p "${target_workdir}"
  mkdir -p "${target_workdir}/skia"
  tar -cf - -C "${SKIA_SOURCE_SEED_WORKDIR}/skia" . | tar -xf - -C "${target_workdir}/skia"
  rm -rf "${target_workdir}/depot_tools"
  ln -s "${SKIA_SOURCE_SEED_WORKDIR}/depot_tools" "${target_workdir}/depot_tools"
  bold "   Seed copied into ${target_workdir}"
}

wait_for_variant_builds() {
  local status=0
  local index=0
  local pid=""
  local name=""
  local log_file=""

  for index in "${!parallel_variant_pids[@]}"; do
    pid="${parallel_variant_pids[$index]}"
    name="${parallel_variant_names[$index]}"
    log_file="${parallel_variant_logs[$index]}"
    if wait "${pid}"; then
      cat "${log_file}"
      green "=== ${name} variant complete ==="
    else
      cat "${log_file}"
      printf '=== %s variant failed ===\n' "${name}" >&2
      status=1
      kill_process_tree "${parallel_variant_pids[@]}"
      break
    fi
    rm -f "${log_file}" >/dev/null 2>&1 || true
  done

  for log_file in "${parallel_variant_logs[@]}"; do
    rm -f "${log_file}" >/dev/null 2>&1 || true
  done
  parallel_variant_pids=()
  parallel_variant_names=()
  parallel_variant_logs=()
  return "${status}"
}

icu_config_is_stale() {
  if [ ! -f "${ICU_CONFIG_STATUS}" ]; then
    return 0
  fi

  local input=""
  for input in \
    "${ICU_ROOT}/source/configure" \
    "${ICU_ROOT}/source/common/unicode/uvernum.h" \
    "${ICU_FILTER}"; do
    if [ "${input}" -nt "${ICU_CONFIG_STATUS}" ]; then
      return 0
    fi
  done

  return 1
}

prepare_icu_data() {
  local candidate=""
  local expected_config=""
  local current_config=""

  for candidate in "${ICU_ROOT_CANDIDATES[@]}"; do
    if [ -d "${candidate}" ]; then
      ICU_ROOT="${candidate}"
      break
    fi
  done

  if [ -z "${ICU_ROOT}" ]; then
    echo "Could not find ICU source tree after building v2/core and v2/ui lanes." >&2
    exit 1
  fi

  mkdir -p "${ICU_BUILD_DIR}"
  expected_config=$(printf '%s\n%s\n' "${ICU_ROOT}" "${ICU_FILTER}")
  if [ -f "${ICU_CONFIG_STAMP}" ]; then
    current_config="$(cat "${ICU_CONFIG_STAMP}")"
  fi

  if [ ! -f "${ICU_BUILD_DIR}/Makefile" ] || [ "${current_config}" != "${expected_config}" ] || icu_config_is_stale; then
    rm -rf "${ICU_BUILD_DIR}"
    mkdir -p "${ICU_BUILD_DIR}"
    pushd "${ICU_BUILD_DIR}" >/dev/null
    ICU_DATA_FILTER_FILE="${ICU_FILTER}" \
      "${ICU_ROOT}/source/runConfigureICU" \
      --enable-debug \
      --disable-release \
      Linux/gcc \
      --disable-tests \
      --disable-layoutex \
      --enable-rpath \
      --prefix="${ICU_BUILD_DIR}"
    popd >/dev/null
    printf '%s' "${expected_config}" > "${ICU_CONFIG_STAMP}"
  fi

  pushd "${ICU_BUILD_DIR}" >/dev/null
  make -j"${ICU_JOBS}"
  ICU_VERSION="$(egrep '^SO_TARGET.*MAJOR' icudefs.mk | awk '{print $3}')"
  ICU_SOURCE="${ICU_BUILD_DIR}/data/out/tmp/icudt${ICU_VERSION}l.dat"
  popd >/dev/null

  if [ ! -f "${ICU_SOURCE}" ]; then
    echo "Filtered ICU data build did not produce ${ICU_SOURCE}." >&2
    exit 1
  fi
}

stage_variant() {
  local architecture_name="$1"
  local wasm_arch="$2"
  local simd_mode="$3"
  local variant_dir="${STAGE_DIR}/${architecture_name}"
  local skia_workdir="${SKIA_WORKDIR_BASE}/${architecture_name}"

  mkdir -p "${variant_dir}"

  cd "${REPO_ROOT}"
  SKIA_BUILD_WORKDIR="${skia_workdir}" \
  EFFINDOM_WASM_ARCH="${wasm_arch}" \
  EFFINDOM_SIMD="${simd_mode}" \
  SKIA_SKIP_SOURCE_PREP=1 \
  EFFINDOM_BROWSER_OUTPUT_DIR="${variant_dir}/core-out" \
  EFFINDOM_TEMP_JS_OUTPUT="${variant_dir}/core.js" \
  EFFINDOM_TEMP_WASM_OUTPUT="${variant_dir}/core.wasm" \
  EFFINDOM_TEMP_SYMBOLS_OUTPUT="${variant_dir}/core.js.symbols" \
  bash v2/core/scripts/build_wasm_arch.sh

  cd "${REPO_ROOT}"
  SKIA_BUILD_WORKDIR="${skia_workdir}" \
  EFFINDOM_WASM_ARCH="${wasm_arch}" \
  EFFINDOM_SIMD="${simd_mode}" \
  SKIA_SKIP_SOURCE_PREP=1 \
  EFFINDOM_BROWSER_OUTPUT_DIR="${variant_dir}/ui-out" \
  EFFINDOM_SKIP_BRIDGE_HARNESS=1 \
  EFFINDOM_TEMP_JS_OUTPUT="${variant_dir}/ui.js" \
  EFFINDOM_TEMP_WASM_OUTPUT="${variant_dir}/ui.wasm" \
  EFFINDOM_TEMP_SYMBOLS_OUTPUT="${variant_dir}/ui.js.symbols" \
  bash v2/ui/scripts/build_wasm_arch.sh
}

prepare_skia_seed

for variant_workdir in \
  "${SKIA_WORKDIR_BASE}/wasm32" \
  "${SKIA_WORKDIR_BASE}/wasm32-simd" \
  "${SKIA_WORKDIR_BASE}/wasm64" \
  "${SKIA_WORKDIR_BASE}/wasm64-simd"; do
  copy_skia_seed_to_workdir "${variant_workdir}"
done

for variant_args in \
  "wasm32 wasm32 off" \
  "wasm32-simd wasm32 on" \
  "wasm64 wasm64 off" \
  "wasm64-simd wasm64 on"; do
  # shellcheck disable=SC2086
  launch_variant_build ${variant_args}
done

if ! wait_for_variant_builds; then
  exit 1
fi

prepare_icu_data

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

cp "${REPO_ROOT}/v2/browser-bridge/index.html" "${OUT_DIR}/index.html"
copy_bridge_font_assets "${REPO_ROOT}/public/v2/fonts"

rm -rf "${OUT_DIR}/runtime"
rm -f "${MANIFEST_FILE}" "${OUT_DIR}/icu-asset.json"
python3 "${MANIFEST_SCRIPT}" "${OUT_DIR}" "${STAGE_DIR}" "${ICU_SOURCE}"
node "${REPO_ROOT}/v2/browser-bridge/scripts/finalize_runtime_manifest.mjs" \
  "${OUT_DIR}" \
  "${REPO_ROOT}/public/v2/fonts" \
  "../fonts"

# Leave the canonical v2/core and v2/ui browser outputs on the safest baseline lane.
cp "${REPO_ROOT}/v2/core/browser/index.html" "${REPO_ROOT}/public/v2/core/index.html"
cp "${STAGE_DIR}/wasm32/core.js" "${REPO_ROOT}/public/v2/core/effindom-core-v2.js"
cp "${STAGE_DIR}/wasm32/core.wasm" "${REPO_ROOT}/public/v2/core/effindom-core-v2.wasm"
if [ -f "${STAGE_DIR}/wasm32/core.js.symbols" ]; then
  cp "${STAGE_DIR}/wasm32/core.js.symbols" "${REPO_ROOT}/public/v2/core/effindom-core-v2.js.symbols"
fi

cp "${REPO_ROOT}/v2/ui/browser/index.html" "${REPO_ROOT}/public/v2/ui/index.html"
cp "${STAGE_DIR}/wasm32/ui.js" "${REPO_ROOT}/public/v2/ui/effindom-ui-v2.js"
cp "${STAGE_DIR}/wasm32/ui.wasm" "${REPO_ROOT}/public/v2/ui/effindom-ui-v2.wasm"
if [ -f "${STAGE_DIR}/wasm32/ui.js.symbols" ]; then
  cp "${STAGE_DIR}/wasm32/ui.js.symbols" "${REPO_ROOT}/public/v2/ui/effindom-ui-v2.js.symbols"
fi

mkdir -p "${REPO_ROOT}/public/v2/ui"
npx esbuild "${REPO_ROOT}/v2/ui/browser/bridge-harness.ts" \
  --bundle \
  --format=iife \
  --platform=browser \
  --target=es2020 \
  --minify \
  --outfile="${REPO_ROOT}/public/v2/ui/bridge-harness.js" \
  --sourcemap
