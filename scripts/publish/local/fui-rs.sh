#!/usr/bin/env bash

set -euo pipefail

source "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/lib/common.sh"

PACKAGE_DIR="${REPO_ROOT}/v2/fui-rs"

log_step "Running @effindomv2/fui-rs publish checks"
ensure_npm_deps "${REPO_ROOT}" "npm install --legacy-peer-deps --silent"
ensure_npm_deps "${PACKAGE_DIR}" "npm install --legacy-peer-deps --silent"
run_in_dir "${PACKAGE_DIR}" npm run lint
run_in_dir "${PACKAGE_DIR}" npm run typecheck
run_in_dir "${PACKAGE_DIR}" npm run test:unit
run_in_dir "${PACKAGE_DIR}" npm run build
run_in_dir "${PACKAGE_DIR}" npm pack --dry-run >/dev/null
