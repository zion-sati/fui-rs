#!/usr/bin/env python3

from __future__ import annotations

import json
import shutil
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(REPO_ROOT / "scripts"))

from content_hash import short_content_hash, standard_content_hash  # noqa: E402

MANIFEST_FILENAME = "effindom.v2.manifest.json"
MANIFEST_VERSION = "1.0"
ARCHITECTURES = ("wasm64-simd", "wasm64", "wasm32-simd", "wasm32")
BUNDLES = ("core", "ui")


def stable_json_bytes(value: object) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode("utf-8")


def content_integrity(data: bytes) -> str:
    return f"sha256-{standard_content_hash(data)}"


def stage_versioned_copy(source: Path, runtime_dir: Path, stem: str, suffix: str) -> dict[str, str]:
    data = source.read_bytes()
    hashed_name = f"{stem}.{short_content_hash(data)}{suffix}"
    destination = runtime_dir / hashed_name
    destination.write_bytes(data)
    return {
        "url": f"./runtime/{hashed_name}",
        "integrity": content_integrity(data),
    }


def stage_bundle(stage_dir: Path, runtime_dir: Path, architecture: str, bundle_name: str) -> dict[str, str]:
    bundle_dir = stage_dir / architecture
    js_asset = stage_versioned_copy(
        bundle_dir / f"{bundle_name}.js",
        runtime_dir,
        f"effindom-{bundle_name}-v2.{architecture}",
        ".js",
    )
    wasm_asset = stage_versioned_copy(
        bundle_dir / f"{bundle_name}.wasm",
        runtime_dir,
        f"effindom-{bundle_name}-v2.{architecture}",
        ".wasm",
    )

    symbols_source = bundle_dir / f"{bundle_name}.js.symbols"
    if symbols_source.exists():
        stage_versioned_copy(
            symbols_source,
            runtime_dir,
            f"effindom-{bundle_name}-v2.{architecture}",
            ".js.symbols",
        )

    return {
        "js": js_asset["url"],
        "js_integrity": js_asset["integrity"],
        "wasm": wasm_asset["url"],
        "wasm_integrity": wasm_asset["integrity"],
    }


def main() -> int:
    if len(sys.argv) != 4:
        print(
            f"Usage: {Path(sys.argv[0]).name} <out-dir> <stage-dir> <icu-source>",
            file=sys.stderr,
        )
        return 1

    out_dir = Path(sys.argv[1]).resolve()
    stage_dir = Path(sys.argv[2]).resolve()
    icu_source = Path(sys.argv[3]).resolve()
    runtime_dir = out_dir / "runtime"

    if runtime_dir.exists():
        shutil.rmtree(runtime_dir)
    runtime_dir.mkdir(parents=True, exist_ok=True)

    architectures: dict[str, dict[str, dict[str, str]]] = {}
    for architecture in ARCHITECTURES:
        architectures[architecture] = {}
        for bundle_name in BUNDLES:
            architectures[architecture][bundle_name] = stage_bundle(stage_dir, runtime_dir, architecture, bundle_name)

    icu_asset = stage_versioned_copy(icu_source, runtime_dir, "icudt_minimal", ".dat")
    assets = {
        "icu": {
            "url": icu_asset["url"],
            "integrity": icu_asset["integrity"],
        }
    }

    manifest_payload = {
        "architectures": architectures,
        "assets": assets,
    }
    manifest = {
        "version": MANIFEST_VERSION,
        "manifest_hash": short_content_hash(stable_json_bytes(manifest_payload)),
        **manifest_payload,
    }

    manifest_path = out_dir / MANIFEST_FILENAME
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print(f"Generated: {manifest_path}")
    print(json.dumps(manifest, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
