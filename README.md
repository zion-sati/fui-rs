# FUI-RS — Rust SDK for EffinDom v2

FUI-RS is the Rust retained-mode UI SDK for EffinDom v2. It builds Rust UI
objects into WebAssembly and runs them through the shared EffinDom browser
runtime.

The SDK includes retained nodes, controls, themes, events, popups, dialogs,
text input, selection, custom drawing, workers, host services, routed app
helpers, and Rust-specific authoring macros.

## Quickstart

Create an application with the published FUI-RS scaffolder:

```bash
# Install Rust and the WebAssembly target once
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustup target add wasm32-unknown-unknown

# Create and run an app
npx @effindomv2/create-fui-rs-app my-app
cd my-app
npm install
npm run dev
```

For a routed app with one separately built WASM module per route:

```bash
npx @effindomv2/create-fui-rs-app my-routed-app -- --template routed
cd my-routed-app
npm install
npm run dev
```

Binaryen is optional for application development and optimizes release WASM
when `wasm-opt` is available. See the
[FUI-RS developer quickstart](QUICKSTART.md) for setup and retained-mode
guidance.

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

## Community projects

- [galaga-rs](https://github.com/jatm80/galaga-rs) by
  [jatm80](https://github.com/jatm80) — a Galaga-style space shooter written in
  Rust with FUI-RS, and the first known community-built FUI-RS project.
  [Play the live demo](https://jatm80.github.io/galaga-rs/). Its repository also
  includes a
  [community-maintained FUI-RS skill](https://github.com/jatm80/galaga-rs/tree/main/.claude/skills/fui-rs)
  with retained-mode guidance, API reference notes, and worked examples.

## Documentation

- [SDK docs index](docs/v2/fui-rs/SDK_INDEX.md)
- [API reference](docs/v2/fui-rs/API_REFERENCE.md)
- [Controls and nodes](docs/v2/fui-rs/CONTROLS_AND_NODES.md)
- [Events and callbacks](docs/v2/fui-rs/EVENTS_AND_CALLBACKS.md)
- [Text input reference](docs/v2/fui-rs/TEXT_INPUT_REFERENCE.md)
- [Forms and autofill](docs/v2/fui-rs/FORMS_AND_AUTOFILL.md)
- [Theming and styles](docs/v2/fui-rs/THEMING_STYLE_MATRIX.md)

## Contributing

The quickstart above is for developers consuming the published SDK. To work on
FUI-RS itself, follow the
[FUI-RS contributor quickstart](docs/v2/fui-rs/CONTRIBUTOR_QUICKSTART.md).

## License

AGPL-3.0-only, or commercial. See
[the commercial licensing terms](COMMERCIAL.md).
