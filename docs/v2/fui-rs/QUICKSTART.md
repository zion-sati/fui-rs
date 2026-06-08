# FUI Rust Quickstart

> **⚠️ Early stage.** FUI-RS is under active development. The current slice
> provides a working smoke app, fluent node builders, and basic component
> reactivity. Controls, theming, and signal-based reactivity are planned for
> future slices. Expect breaking changes between slices.

## Prerequisites

Install the shared v2 toolchain first:

- [docs/QUICKSTART.md](../../QUICKSTART.md)

Then install Rust/Cargo (required for this guide) and the wasm target:

### macOS

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustup target add wasm32-unknown-unknown
```

### Linux (Debian / Ubuntu)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustup target add wasm32-unknown-unknown
```

## Build and run the Slice 1 smoke app

From the repository root:

```bash
npm run build:v2:fui-rs
npm run test:v2:fui-rs:integration
```

The build stages a self-contained smoke page under `public/v2/fui-rs/` and the integration test loads it through Playwright beside the existing v2 browser bridge runtime.

## Manually test the smoke app in a browser

From the repository root:

```bash
npm run build:v2:browser-bridge
npm run build:v2:fui-rs
npm run serve
```

Then open:

```text
http://127.0.0.1:8080/v2/fui-rs/index.html
```

If port `8080` is busy, `npm run serve` prints the fallback port it chose. The page should render the blue-box smoke scene and the console should log readiness without errors.

## Write a component

```rust
use fui_rs::prelude::*;

struct BlueBox {
    color: State<u32>,
}

impl BlueBox {
    fn new() -> Self {
        Self {
            color: state(0x006CFFFF),
        }
    }
}

impl Component for BlueBox {
    fn render(&self) -> Box<dyn Node> {
        Box::new(
            flex_box()
                .width(120.0, Unit::Pixel)
                .height(96.0, Unit::Pixel)
                .bg_color(self.color.get()),
        )
    }
}
```

## Reactive state

- `state(initial)` creates component-owned writable state.
- `derived(&source, |value| ...)` creates a read-only state derived from an explicit source state.
- Mutating state marks the owning component dirty and rebuilds the widget tree.

## Available Slice 1 primitives

- `Component` with `render() -> Box<dyn Node>`
- `Application::run(...)`
- `flex_box()` and `text(...)`
- `State`, `state`, and `derived`

Slice 1 uses destroy-and-remount reconciliation, so every dirty render rebuilds a fresh built-node subtree through the v2 UI ABI.

The public SDK barrel is `v2/fui-rs/src/lib.rs`. The browser smoke app used by this slice is compiled from `v2/fui-rs/src/smoke.rs` so the crate root stays SDK-only.
