# FUI-RS — Rust SDK for EffinDom v2

FUI-RS is the Rust retained-mode UI SDK for EffinDom v2. It builds Rust UI
objects into WebAssembly and runs them through the shared EffinDom browser
runtime.

The SDK includes retained nodes, controls, themes, events, popups, dialogs,
text input, selection, custom drawing, workers, host services, routed app
helpers, and Rust-specific authoring macros.

## Quickstart

```bash
# Prerequisites: Rust + wasm32-unknown-unknown target + Binaryen
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown
brew install binaryen

# Clone and build
git clone https://github.com/zion-sati/fui-rs.git
cd fui-rs
npm ci
npm run build
npm run serve
```

Open:

```text
http://127.0.0.1:8080/v2/fui-rs/demo/index.html
```

Full quickstart: [docs/v2/fui-rs/QUICKSTART.md](docs/v2/fui-rs/QUICKSTART.md)

## Create a new app

For a single-page app:

```bash
npx @effindomv2/create-fui-rs-app my-app
cd my-app
npm install
npm run dev
```

For a routed MVC-style app with one separately built WASM per route:

```bash
npx @effindomv2/create-fui-rs-app my-routed-app -- --template mvc
cd my-routed-app
npm install
npm run dev
```

The generated app code uses `fui_app!` or `fui_managed_app!`; normal app code
does not hand-write browser lifecycle exports.

## Minimal app

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

## Architecture

FUI-RS is retained mode:

- Construct nodes and controls once.
- Store stateful controls as fields when callbacks need to mutate them later.
- Mutate retained objects from events, timers, host callbacks, or signals.
- Use `ui!` as syntax sugar for retained construction, not as a render loop.

The Rust app WASM talks to the shared EffinDom browser bridge and retained C++
UI runtime through generated ABI bindings. The public SDK keeps raw ABI details
out of normal app code.

## Documentation

- [SDK docs index](docs/v2/fui-rs/SDK_INDEX.md)
- [API reference](docs/v2/fui-rs/API_REFERENCE.md)
- [Controls and nodes](docs/v2/fui-rs/CONTROLS_AND_NODES.md)
- [Events and callbacks](docs/v2/fui-rs/EVENTS_AND_CALLBACKS.md)
- [Text input reference](docs/v2/fui-rs/TEXT_INPUT_REFERENCE.md)
- [Forms and autofill](docs/v2/fui-rs/FORMS_AND_AUTOFILL.md)
- [Theming and styles](docs/v2/fui-rs/THEMING_STYLE_MATRIX.md)

## License

AGPL-3.0-only, or commercial. See [COMMERCIAL.md](COMMERCIAL.md).
