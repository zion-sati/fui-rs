# `v2/abi`

ABI code generation for `fui-as` and `fui-rs`.

## Layout

- `generate.ts`: thin CLI entrypoint
- `strategies.ts`: strategy registry
- `shared/`: canonical ABI model, shared policies, and cross-package helpers
- `fui-as/strategies/`: `fui-as`-only generators and specs
- `fui-rs/strategies/`: `fui-rs` orchestration entrypoints
- `fui-rs/render/`: Rust-only projection/render helpers

## Design rules

- Keep canonical ABI truth in `shared/`.
- Keep package-specific language projection logic inside that package folder.
- One SDK generator must not import canonical ABI model/policy from another SDK folder.
- Prefer one strategy class per file.
- Extract reusable emit/parse logic instead of copying between strategies.
- Root `v2/abi/` should stay small and orchestration-focused.

## Architectural boundary

- `shared/*` owns the language-neutral ABI model:
  - header parsing
  - host import metadata
  - canonical enum specs
  - shared selection/value-resolution policy
- `fui-*/*` owns language projection only:
  - naming
  - type mapping
  - file layout
  - idiomatic code generation for that SDK

## Current shared modules

- `shared/core.ts`: CLI context, repo-root discovery, generated-file writing
- `shared/c-header.ts`: C header parsing
- `shared/fui-host.ts`: host ABI metadata
- `shared/model/enum-specs.ts`: canonical SDK-facing ABI enum set
- `shared/assemblyscript.ts`: AssemblyScript-specific emit helpers
- `shared/enum-generation.ts`: shared enum value resolution and TS/Rust enum emission

## Adding a new strategy

1. Put the strategy under the owning package folder:
   - `fui-as/strategies/`
   - `fui-rs/strategies/`
2. Implement `GenerateStrategy`.
3. Add it to `strategies.ts`.
4. Keep any package-only helper/spec files next to that strategy.
5. Move code into `shared/` only if another package genuinely needs it.

## Existing strategy names

- `fui-as-ui`
- `fui-as-host`
- `fui-as-enums`
- `fui-rs-ffi`
