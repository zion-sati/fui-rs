#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"

log_step() {
  echo "==> $*"
}

run_in_dir() {
  local directory="$1"
  shift
  (
    cd "${directory}"
    "$@"
  )
}

ensure_npm_deps() {
  local package_dir="$1"
  local install_cmd="$2"
  if [ ! -d "${package_dir}/node_modules" ]; then
    (cd "${package_dir}" && eval "${install_cmd}")
  fi
}

ensure_npm_path() {
  local package_dir="$1"
  local relative_path="$2"
  local install_cmd="$3"
  if [ ! -e "${package_dir}/${relative_path}" ]; then
    (cd "${package_dir}" && eval "${install_cmd}")
  fi
}

package_json_field() {
  local package_dir="$1"
  local field="$2"
  node --input-type=module -e '
import { readFileSync } from "node:fs";
const [packageJsonPath, key] = process.argv.slice(1);
const packageJson = JSON.parse(readFileSync(packageJsonPath, "utf8"));
const value = packageJson[key];
if (value === undefined) {
  process.exit(2);
}
if (typeof value === "string") {
  process.stdout.write(value);
} else {
  process.stdout.write(String(value));
}
' "${package_dir}/package.json" "${field}"
}

package_json_set_name() {
local package_dir="$1"
local package_name="$2"
node --input-type=module -e '
import { readFileSync, writeFileSync } from "node:fs";
const [packageJsonPath, nextName] = process.argv.slice(1);
const packageJson = JSON.parse(readFileSync(packageJsonPath, "utf8"));
packageJson.name = nextName;
writeFileSync(packageJsonPath, JSON.stringify(packageJson, null, 2) + "\n");
' "${package_dir}/package.json" "${package_name}"
}

effective_package_name() {
local package_dir="$1"
local package_name="${EFFINDOM_PACKAGE_NAME:-}"
if [ -n "${package_name}" ]; then
  printf '%s\n' "${package_name}"
  return
fi
package_json_field "${package_dir}" "name"
}

assert_publishable_package() {
  local package_dir="$1"
  local package_name
  local is_private
  package_name="$(package_json_field "${package_dir}" "name")"
  is_private="$(package_json_field "${package_dir}" "private" || true)"
  if [ "${is_private}" = "true" ]; then
    echo "Package ${package_name} is marked private=true and cannot be published." >&2
    exit 1
  fi
}

ensure_npm_logged_in() {
  if ! npm whoami >/dev/null 2>&1; then
    echo "Not logged in to npm. Run 'npm login' first." >&2
    exit 1
  fi
}

publish_package() {
  local package_dir="$1"
  shift
  assert_publishable_package "${package_dir}"
  ensure_npm_logged_in
  local package_name
  local package_version
  local publish_dir="${package_dir}"
  local publish_temp_dir=""
  local publish_name
  publish_name="$(effective_package_name "${package_dir}")"
  package_name="$(package_json_field "${package_dir}" "name")"
  package_version="$(package_json_field "${package_dir}" "version")"
  if [ "${publish_name}" != "${package_name}" ]; then
    publish_temp_dir="$(mktemp -d "${REPO_ROOT}/build/.npm-publish-XXXXXX")"
    cp -R "${package_dir}/." "${publish_temp_dir}/"
    package_json_set_name "${publish_temp_dir}" "${publish_name}"
    publish_dir="${publish_temp_dir}"
  fi
  log_step "Publishing ${publish_name}@${package_version}"
  run_in_dir "${publish_dir}" npm publish --access public "$@"
  if [ -n "${publish_temp_dir}" ]; then
    rm -rf "${publish_temp_dir}" >/dev/null 2>&1 || true
  fi
}

stage_local_package() {
  local package_dir="$1"
  local published_root="${2:-${REPO_ROOT}/published}"
  assert_publishable_package "${package_dir}"

  mkdir -p "${REPO_ROOT}/build"
  mkdir -p "${published_root}"

  local pack_dir="${package_dir}"
  local pack_temp_dir=""
  local package_name
  local package_version
  local publish_name
  publish_name="$(effective_package_name "${package_dir}")"
  package_name="$(package_json_field "${package_dir}" "name")"
  package_version="$(package_json_field "${package_dir}" "version")"
  if [ "${publish_name}" != "${package_name}" ]; then
    pack_temp_dir="$(mktemp -d "${REPO_ROOT}/build/.npm-pack-src-XXXXXX")"
    cp -R "${package_dir}/." "${pack_temp_dir}/"
    package_json_set_name "${pack_temp_dir}" "${publish_name}"
    pack_dir="${pack_temp_dir}"
  fi

  local pack_output_file
  pack_output_file="$(mktemp "${REPO_ROOT}/build/.npm-pack.XXXXXX")"

  run_in_dir "${pack_dir}" npm pack --json --ignore-scripts > "${pack_output_file}"

  local pack_info
  pack_info="$(node --input-type=module -e '
import { readFileSync } from "node:fs";
const items = JSON.parse(readFileSync(process.argv[1], "utf8"));
const item = items[0];
if (!item?.filename || !item?.name || !item?.version) {
  process.exit(1);
}
process.stdout.write(item.filename + "\n" + item.name + "\n" + item.version + "\n");
' "${pack_output_file}")"
  rm -f "${pack_output_file}" >/dev/null 2>&1 || true

  local tarball_name
  local package_name
  local package_version
  tarball_name="$(printf '%s' "${pack_info}" | sed -n '1p')"
  package_name="$(printf '%s' "${pack_info}" | sed -n '2p')"
  package_version="$(printf '%s' "${pack_info}" | sed -n '3p')"

  local sanitized_package_name
  sanitized_package_name="${package_name//@/}"
  sanitized_package_name="${sanitized_package_name//\//-}"

  local package_output_dir="${published_root}/${sanitized_package_name}-${package_version}"
  local tarball_path="${pack_dir}/${tarball_name}"
  local unpack_dir
  unpack_dir="$(mktemp -d "${REPO_ROOT}/build/.npm-pack-unpack-XXXXXX")"

  rm -rf "${package_output_dir}"
  mkdir -p "${package_output_dir}"
  tar -xzf "${tarball_path}" -C "${unpack_dir}"
  if [ -d "${unpack_dir}/package" ]; then
    cp -R "${unpack_dir}/package/." "${package_output_dir}/"
  else
    cp -R "${unpack_dir}/." "${package_output_dir}/"
  fi
  rm -rf "${unpack_dir}"
  mv "${tarball_path}" "${published_root}/${tarball_name}"
  if [ -n "${pack_temp_dir}" ]; then
    rm -rf "${pack_temp_dir}" >/dev/null 2>&1 || true
  fi

  log_step "Staged ${publish_name}@${package_version} into ${package_output_dir}"
  log_step "Wrote tarball ${published_root}/${tarball_name}"
}
