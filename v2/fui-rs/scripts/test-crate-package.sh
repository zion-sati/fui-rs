#!/usr/bin/env bash

set -euo pipefail

PACKAGE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_ROOT="${CARGO_TARGET_DIR:-${PACKAGE_ROOT}/target}"
VERSION="$(cargo metadata --manifest-path "${PACKAGE_ROOT}/Cargo.toml" --no-deps --format-version 1 | node -e 'let value=""; process.stdin.on("data", chunk => value += chunk); process.stdin.on("end", () => process.stdout.write(JSON.parse(value).packages[0].version));')"
ARCHIVE="${TARGET_ROOT}/package/fui-rs-${VERSION}.crate"
WORK_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/fui-rs-crate-package.XXXXXX")"

cleanup() {
  rm -rf "${WORK_ROOT}"
}
trap cleanup EXIT

cargo package --manifest-path "${PACKAGE_ROOT}/Cargo.toml" --allow-dirty
test -f "${ARCHIVE}"

while IFS= read -r entry; do
  relative="${entry#fui-rs-${VERSION}/}"
  case "${relative}" in
    .cargo_vcs_info.json|Cargo.lock|Cargo.toml|Cargo.toml.orig|README.md|LICENSE.md|COMMERCIAL.md|src/*|tests/*|LICENSES/AGPL-3.0-only.md)
      ;;
    *)
      echo "Unexpected file in fui-rs crate: ${entry}" >&2
      exit 1
      ;;
  esac
done < <(tar -tzf "${ARCHIVE}" | sed '/\/$/d')

tar -xzf "${ARCHIVE}" -C "${WORK_ROOT}"
CRATE_ROOT="${WORK_ROOT}/fui-rs-${VERSION}"
CONSUMER_ROOT="${WORK_ROOT}/consumer"
mkdir -p "${CONSUMER_ROOT}/src"
cat > "${CONSUMER_ROOT}/Cargo.toml" <<EOF
[package]
name = "fui-rs-package-consumer"
version = "0.0.0"
edition = "2021"
publish = false

[dependencies]
fui = { package = "fui-rs", path = "${CRATE_ROOT}" }
EOF
cat > "${CONSUMER_ROOT}/src/main.rs" <<'EOF'
use fui::prelude::*;

fn main() {
    let _label = text("packaged FUI-RS dependency");
}
EOF

cargo check --manifest-path "${CONSUMER_ROOT}/Cargo.toml"
cargo check --manifest-path "${CONSUMER_ROOT}/Cargo.toml" --features fui/native-runtime

echo "FUI-RS crate package ${VERSION} passed external-consumer validation."
