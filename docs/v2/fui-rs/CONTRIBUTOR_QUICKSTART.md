# FUI-RS Contributor Quickstart

This guide is for contributors working on the FUI-RS SDK, generated ABI,
browser integration, tests, or repository demos. Application developers should
use the [FUI-RS developer quickstart](./QUICKSTART.md) instead.

## Prerequisites

- Node.js 24 or newer and npm
- Stable Rust and Cargo
- `wasm32-unknown-unknown` Rust target
- Binaryen for optimized WASM builds
- Playwright browser binaries for integration tests

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustup target add wasm32-unknown-unknown
npm install
npx playwright install chromium
```

Install Binaryen with `brew install binaryen` on macOS or your distribution's
Binaryen package on Linux.

## Build

From the standalone FUI-RS repository root:

```bash
npm run build
```

This runs TypeScript linting/typechecking and builds the FUI-RS package and its
WASM outputs against the packaged EffinDom runtime dependency.

## Test and lint

```bash
npm test
npm --workspace v2/fui-rs run lint:rust
```

For the browser integration lane:

```bash
npm --workspace v2/fui-rs run test:integration
```

## Contribution rules

- FUI-RS is retained mode. Construct retained controls once and mutate them;
  do not introduce a recurring render/rebuild model.
- Preserve public behavior and ABI parity with the EffinDom runtime.
- Use Rust ownership tools such as `Rc`, `Weak`, `RefCell`, and RAII without
  introducing strong reference cycles.
- Add behavior-focused tests for every SDK change.
- Run the unit suite, Clippy with warnings denied, and the affected build before
  opening a pull request.

## Documentation

- [SDK docs index](./SDK_INDEX.md)
- [API reference](./API_REFERENCE.md)
- [Controls and nodes](./CONTROLS_AND_NODES.md)
- [Events and callbacks](./EVENTS_AND_CALLBACKS.md)
