#!/usr/bin/env bash

BRIDGE_FONT_SOURCE_DIR="${REPO_ROOT}/v2/fonts"
BRIDGE_FONT_ASSETS=(
  "DejaVuSans.ttf"
  "DejaVuSans-Bold.ttf"
  "NotoSans-Regular.ttf"
  "NotoSans-Bold.ttf"
  "NotoSans-Italic.ttf"
  "NotoSans-BoldItalic.ttf"
  "NotoSansMono-Regular.ttf"
  "NotoSansMono-Bold.ttf"
  "NotoSansSymbols2-Regular.ttf"
  "NotoEmoji-Regular.ttf"
  "NotoColorEmoji.ttf"
  "NotoSansThai-Regular.ttf"
  "NotoNaskhArabic-Variable.ttf"
)

RUNTIME_PACKAGE_FONT_ASSETS=(
  "NotoSans-Regular.ttf"
  "NotoSans-Bold.ttf"
  "NotoSans-Italic.ttf"
  "NotoSans-BoldItalic.ttf"
  "NotoSansMono-Regular.ttf"
  "NotoSansMono-Bold.ttf"
  "NotoSansSymbols2-Regular.ttf"
  "NotoEmoji-Regular.ttf"
  "NotoColorEmoji.ttf"
)

copy_bridge_font_assets() {
  local out_dir="$1"
  local font_asset=""
  mkdir -p "${out_dir}"
  for font_asset in "${BRIDGE_FONT_ASSETS[@]}"; do
    if [ -f "${BRIDGE_FONT_SOURCE_DIR}/${font_asset}" ]; then
      cp "${BRIDGE_FONT_SOURCE_DIR}/${font_asset}" "${out_dir}/${font_asset}"
    fi
  done
}

copy_runtime_package_font_assets() {
  local out_dir="$1"
  local font_asset=""
  mkdir -p "${out_dir}"
  for font_asset in "${RUNTIME_PACKAGE_FONT_ASSETS[@]}"; do
    if [ -f "${BRIDGE_FONT_SOURCE_DIR}/${font_asset}" ]; then
      cp "${BRIDGE_FONT_SOURCE_DIR}/${font_asset}" "${out_dir}/${font_asset}"
    fi
  done
}
