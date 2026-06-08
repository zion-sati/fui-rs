# Contributing to FUI-RS

> **⚠️ Early stage.** FUI-RS is a thin Rust binding over the shared C++ ABI.
> It's under active development — controls, theming, and signal-based
> reactivity are planned for future slices.

This guide is for people working **on the SDK itself** — fixing bugs, adding
bindings, or improving the runtime integration.

---

## Prerequisites

- **Rust** (stable) with `wasm32-unknown-unknown` target
- **Node.js 24+** and npm
- **`@effindomv2/runtime@0.1.15+`** — fetched via the monorepo or npm

If developing against a **local runtime checkout**, install from the
[EffinDOM repo](https://github.com/zion-sati/EffinDOM) first.

---

## Clone and build

```bash
git clone https://github.com/zion-sati/EffinDOM.git
cd EffinDOM
npm ci
npm run build:v2:browser-bridge
npm run build:v2:fui-rs
```

## Run tests

```bash
npm run test:v2:fui-rs:integration
```

The integration test loads the smoke app through Playwright with the shared
v2 browser bridge runtime.

## Run the smoke app

```bash
npm run serve
```

Open `http://127.0.0.1:8080/v2/fui-rs/index.html`.

## Repo structure

```
src/
  ffi.rs        — C ABI declarations (wasm import stubs)
  bindings/ui.rs — Safe wrappers over the FFI
  node.rs       — Fluent node builders (FlexBox, TextNode)
  component.rs  — Component trait + reconciliation
  app.rs        — Application::run
  state.rs      — Reactive state primitives
  signal.rs     — Signal base types
  smoke.rs      — Smoke app (compiled into library for testability)
```

## Docs

- **[FUI-RS Quickstart](https://github.com/zion-sati/EffinDOM/blob/main/docs/v2/fui-rs/QUICKSTART.md)**
- **[FUI-AS Docs Index](https://github.com/zion-sati/EffinDOM/blob/main/docs/v2/fui-as/SDK_INDEX.md)** (C ABI docs apply to all SDKs)

## Getting in touch

This is a solo project. If you're thinking about contributing, please open an
issue or start a discussion before writing code.

For anything else: **zionsatidev@gmail.com**
