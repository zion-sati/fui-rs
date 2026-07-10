#!/usr/bin/env bash

set -euo pipefail

PACKAGE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPO_ROOT="$(cd "${PACKAGE_DIR}/../.." && pwd)"
PACKAGE_JSON="${PACKAGE_DIR}/package.json"
RUNTIME_PACKAGE_JSON="${REPO_ROOT}/v2/browser-bridge/package.json"

runtime_spec="$(
  node -e '
    const fs = require("node:fs");
    const pkg = JSON.parse(fs.readFileSync(process.argv[1], "utf8"));
    process.stdout.write(String(pkg.dependencies?.["@effindomv2/runtime"] ?? ""));
  ' "${PACKAGE_JSON}"
)"

if [ -z "${runtime_spec}" ]; then
  echo "@effindomv2/runtime must be declared in dependencies." >&2
  exit 1
fi

if ! printf '%s' "${runtime_spec}" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+([-.][0-9A-Za-z.]+)?$'; then
  echo "@effindomv2/runtime must be pinned to an exact version, found: ${runtime_spec}" >&2
  exit 1
fi

if [ -f "${RUNTIME_PACKAGE_JSON}" ]; then
  runtime_version="$(
    node -e '
      const fs = require("node:fs");
      const pkg = JSON.parse(fs.readFileSync(process.argv[1], "utf8"));
      process.stdout.write(String(pkg.version ?? ""));
    ' "${RUNTIME_PACKAGE_JSON}"
  )"

  if [ -n "${runtime_version}" ] && [ "${runtime_spec}" != "${runtime_version}" ]; then
    echo "@effindomv2/fui-rs depends on @effindomv2/runtime@${runtime_spec}, but the local runtime is ${runtime_version}." >&2
    exit 1
  fi
fi
