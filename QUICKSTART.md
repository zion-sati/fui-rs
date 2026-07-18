# FUI-RS Quickstart

This guide is for developers building applications with the published FUI-RS
SDK. Contributors working on the SDK itself should read
[CONTRIBUTING.md](CONTRIBUTING.md).

## Prerequisites

- Node.js 18 or newer and npm
- Stable Rust and Cargo
- The `wasm32-unknown-unknown` Rust target
- Binaryen optionally, for optimized release WASM

Install Rust and its WebAssembly target once:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustup target add wasm32-unknown-unknown
```

Install Binaryen with `brew install binaryen` on macOS or your distribution's
Binaryen package on Linux. Development builds work without it.

## Create a simple app

```bash
npx @effindomv2/create-fui-rs-app my-app
cd my-app
npm install
npm run dev
```

The development server watches Rust and host source files and rebuilds fast
debug WASM automatically.

## Create a routed app

```bash
npx @effindomv2/create-fui-rs-app my-routed-app -- --template routed
cd my-routed-app
npm install
npm run dev
```

The routed template builds each route as a separate WASM module. Routes can be
deployed independently while sharing the same browser shell and EffinDom
runtime assets.

## Build and publish static assets

Create an optimized build:

```bash
npm run build
```

Stage deployable static assets in `published/`:

```bash
npm run publish
```

## Minimal retained app

```rust
use fui::prelude::*;

fn build_page() -> FlexBox {
    ui! {
        column().fill_size().padding(24.0, 24.0, 24.0, 24.0) {
            text("Hello from FUI-RS").font_size(28.0),
            button("Click me").on_click(|_| {
                logger::info("App", "Button clicked");
            }),
        }
    }
}

fui_app!(FlexBox, build_page);
```

FUI-RS is retained mode. Construct controls once and mutate retained controls
from callbacks; do not recreate the UI tree in a recurring render loop.

Use `ui!` for mixed retained child trees and `rich_text!` for fluent attributed
text:

```rust
let label = rich_text![
    "Status: ".italic(),
    "Ready".bold().text_color(rgb(0x3a, 0xc5, 0x6c)),
];
```

## Documentation

- [SDK docs index](docs/v2/fui-rs/SDK_INDEX.md)
- [Full developer quickstart](docs/v2/fui-rs/QUICKSTART.md)
- [API reference](docs/v2/fui-rs/API_REFERENCE.md)
- [Controls and nodes](docs/v2/fui-rs/CONTROLS_AND_NODES.md)
- [Events and callbacks](docs/v2/fui-rs/EVENTS_AND_CALLBACKS.md)
- [Theming and styles](docs/v2/fui-rs/THEMING_STYLE_MATRIX.md)
