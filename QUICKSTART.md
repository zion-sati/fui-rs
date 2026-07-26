# FUI-RS Quickstart

This guide is for developers building applications with the published FUI-RS
SDK. Contributors working on the SDK itself should read
[CONTRIBUTING.md](CONTRIBUTING.md).

## Choose the entry point

Use `cargo-fui` for new native, web, or universal projects. Use
`create-fui-rs-app` when you specifically want the npm-oriented browser-only
simple or routed/MVC scaffold.

| Need | Start with |
| --- | --- |
| Native macOS, Windows, or Linux application | `cargo fui new --target native` |
| Browser/WASM application | `cargo fui new --target web` |
| Shared UI for native desktop and browser | `cargo fui new --target universal` |
| Browser-only simple or routed/MVC npm project | `npx @effindomv2/create-fui-rs-app` |

## Cargo workflow

All targets require stable Rust and Cargo. Install both through
[rustup](https://rustup.rs/), then verify the toolchain and install `cargo-fui`:

```bash
rustc --version
cargo --version
cargo install --locked cargo-fui
```

Create and run a universal application:

```bash
cargo fui new my-app --target universal
cd my-app
cargo fui dev
```

Use `--target native` or `--target web` for a single target.

Native prerequisites:

- macOS: Xcode command-line tools.
- Windows: Visual Studio Build Tools and the Windows SDK.
- Linux: a C++ compiler and the native libraries described by the generated
  project README.

Web and universal prerequisites:

- Node.js 24.
- The `wasm32-unknown-unknown` Rust target:

```bash
rustup target add wasm32-unknown-unknown
```

Build optimized output or a native package:

```bash
cargo fui build --release
cargo fui package
```

Native-only projects do not contain or require Node.js. Universal projects keep
shared retained UI in a target-independent crate and place platform capabilities
behind explicit native and web adapters.

## Browser-only npm workflow

Install Rust, Node.js 24, and the WebAssembly target, then create a simple app:

```bash
rustup target add wasm32-unknown-unknown
npx @effindomv2/create-fui-rs-app my-app
cd my-app
npm install
npm run dev
```

Create a routed/MVC app with one separately built WASM module per route:

```bash
npx @effindomv2/create-fui-rs-app my-routed-app -- --template routed
cd my-routed-app
npm install
npm run dev
```

The development server watches Rust and host source files. Binaryen is optional
for development and optimizes release WASM when `wasm-opt` is available.

Create optimized and deployable static assets:

```bash
npm run build
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

Construct retained controls once and mutate them from callbacks. Do not recreate
the UI tree in a recurring render loop.

`Button::on_click(...)` is the normal high-level action API and includes
supported keyboard activation. Use raw pointer handlers only when the control
needs routed pointer data.

Use `ui!` for retained child trees and `rich_text!` for attributed text:

```rust
let label = rich_text![
    "Status: ".italic(),
    "Ready".bold().text_color(rgb(0x3a, 0xc5, 0x6c)),
];
```

## Next references

- [Full developer guide](docs/v2/fui-rs/QUICKSTART.md)
- [API reference](docs/v2/fui-rs/API_REFERENCE.md)
- [Controls and nodes](docs/v2/fui-rs/CONTROLS_AND_NODES.md)
- [Events and callbacks](docs/v2/fui-rs/EVENTS_AND_CALLBACKS.md)
- [Theming and styles](docs/v2/fui-rs/THEMING_STYLE_MATRIX.md)
