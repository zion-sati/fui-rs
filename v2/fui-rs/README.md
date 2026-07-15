# FUI-RS - Rust SDK for EffinDom v2

FUI-RS is the Rust retained-mode SDK for building EffinDom v2 WebAssembly UI
apps. It provides retained controls, layout nodes, text input, overlays,
custom drawing, host services, workers, routing support, and app lifecycle
macros for browser-hosted Rust WASM apps.

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

Full quickstart: [docs/v2/fui-rs/QUICKSTART.md](../../docs/v2/fui-rs/QUICKSTART.md)

## Minimal app

```rust
use fui::prelude::*;

fn build_page() -> FlexBox {
    ui! {
        column().fill_size().padding(24.0, 24.0, 24.0, 24.0) {
            text("Hello from Rust"),
            button("Click me").on_click(|_| logger::info("App", "clicked")),
        }
    }
}

fui_app!(FlexBox, build_page);
```

## SDK docs

- [SDK docs index](../../docs/v2/fui-rs/SDK_INDEX.md)
- [API reference](../../docs/v2/fui-rs/API_REFERENCE.md)
- [Controls and nodes](../../docs/v2/fui-rs/CONTROLS_AND_NODES.md)
- [Events and callbacks](../../docs/v2/fui-rs/EVENTS_AND_CALLBACKS.md)
- [Text input reference](../../docs/v2/fui-rs/TEXT_INPUT_REFERENCE.md)
- [Forms and autofill](../../docs/v2/fui-rs/FORMS_AND_AUTOFILL.md)
- [Theming and style matrix](../../docs/v2/fui-rs/THEMING_STYLE_MATRIX.md)

## What is included

| Area | Status |
|---|---|
| Retained app lifecycle macros | Available |
| `ui!` mixed child tree macro | Available |
| `fui_component!` retained component delegation | Available |
| Flex/Grid layout nodes | Available |
| Text, rich text, image, SVG | Available |
| Buttons, toggles, slider, dropdown, combobox | Available |
| TextInput/TextArea | Available |
| Context menu, popup, dialog, tooltip | Available |
| Selection, mobile text handles, context toolbar | Available |
| ScrollView, ScrollBox, VirtualList | Available |
| Custom drawing and text layouts | Available |
| Browser file/fetch/worker bridges | Available |
| Host services/events generator support | Available |

## Architecture

FUI-RS builds Rust retained UI objects into the EffinDom v2 runtime through the
browser bridge. Rust app WASM and the UI runtime WASM are separate modules;
strings and command data cross the bridge through explicit UTF-8/runtime ABI
calls.

Retained controls are cheap clone handles. Cloning a control gives another Rust
handle to the same retained UI object.

## License

AGPL-3.0-only, or commercial license. See [COMMERCIAL.md](COMMERCIAL.md).
