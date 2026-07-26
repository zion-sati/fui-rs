# FUI-RS — retained Rust UI for native desktop and WebAssembly

[![FUI-RS CI](https://github.com/zion-sati/fui-rs/actions/workflows/fui-rs-ci.yml/badge.svg)](https://github.com/zion-sati/fui-rs/actions/workflows/fui-rs-ci.yml)
[![crates.io](https://img.shields.io/crates/v/fui-rs)](https://crates.io/crates/fui-rs)
[![docs.rs](https://img.shields.io/docsrs/fui-rs)](https://docs.rs/fui-rs)
[![npm](https://img.shields.io/npm/v/@effindomv2/fui-rs?label=web%20tooling)](https://www.npmjs.com/package/@effindomv2/fui-rs)
[![License: AGPL-3.0 or commercial](https://img.shields.io/badge/license-AGPL--3.0%20or%20commercial-green.svg)](LICENSE.md)

FUI-RS is a retained-mode Rust UI SDK that runs the same application UI as a
real native macOS, Windows, or Linux desktop application and in the browser
through WebAssembly. Native applications do not use Electron or a WebView.

The SDK includes retained nodes, controls, layout, themes, events, popups,
dialogs, editable text, selection, custom drawing, workers, host services,
accessibility semantics, and Rust-specific authoring macros.

- [Run the browser demo](https://fui-rs-demo.effindom.dev/)
- [Play Galaga-RS](https://jatm80.github.io/galaga-rs/), the first known
  community-built FUI-RS application

## Start a project

For a new native, web, or universal application, use
[`cargo-fui`](https://github.com/zion-sati/cargo-fui):

Install stable Rust and Cargo through [rustup](https://rustup.rs/), then:

```bash
rustc --version
cargo --version
cargo install --locked cargo-fui
cargo fui new my-app --target universal
cd my-app
cargo fui dev
```

Choose `native` for desktop only, `web` for browser only, or `universal` for a
shared retained UI with explicit native and web adapters.

For a browser-only application that needs the npm-oriented simple or routed/MVC
scaffolds, use `create-fui-rs-app`:

```bash
npx @effindomv2/create-fui-rs-app my-app
cd my-app
npm install
npm run dev
```

See the [FUI-RS quickstart](QUICKSTART.md) for prerequisites and the exact
differences between these entry points.

## How the stack fits together

```text
Your retained Rust UI
        │
        ▼
      FUI-RS          controls, themes, events, application APIs
        │
        ▼
     EffinDOM         layout, text, rendering, input, semantics
       ╱   ╲
      ▼     ▼
 native     browser/WebAssembly
  host
```

`cargo-fui` sits around this stack as project/build/package tooling. FUI-RS
package metadata identifies the compatible EffinDOM runtime, so application
developers do not manually synchronize runtime versions.

## Minimal retained UI

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

FUI-RS is retained mode:

- Construct controls once.
- Keep stateful controls when callbacks need to mutate them.
- Mutate retained objects from events, timers, host callbacks, or signals.
- Use `ui!` as construction syntax, not as a recurring render function.

FUI-RS maps retained inheritance to capability traits. `Node` supplies common
retained state and raw events, while visual controls expose layout, style,
flex-layout, child-container, text, focus, and interaction capabilities as
appropriate.

## Current status

FUI-RS is an early release. It already supports substantial retained UI and
native packaging, but breaking API and generated-project changes remain possible
before 1.0 when correctness or Rust ergonomics require them.

Current boundaries:

- Native support is desktop macOS, Windows, and Linux. iOS and Android are not
  currently supported.
- Native and web targets intentionally expose different platform-service
  adapters where the underlying capability differs.
- Browser-only routed applications use separate WASM modules and a shared
  browser shell; native applications do not use that routing model.
- Native packaging does not supply an application's production signing identity,
  store account, notarization credentials, or distribution policy.
- The supported CI matrix cannot represent every Linux desktop environment,
  browser, GPU, or driver.

Open a discussion when evaluating FUI-RS for an application whose requirements
touch one of these boundaries.

## Community projects

- [galaga-rs](https://github.com/jatm80/galaga-rs) by
  [jatm80](https://github.com/jatm80) is a Galaga-style space shooter and the
  first known community-built FUI-RS project.
  [Play it](https://jatm80.github.io/galaga-rs/) or read its
  [community-maintained FUI-RS skill](https://github.com/jatm80/galaga-rs/tree/main/.claude/skills/fui-rs).

## Documentation

- [Developer quickstart](QUICKSTART.md)
- [SDK docs index](docs/v2/fui-rs/SDK_INDEX.md)
- [API reference](docs/v2/fui-rs/API_REFERENCE.md)
- [Controls and nodes](docs/v2/fui-rs/CONTROLS_AND_NODES.md)
- [Events and callbacks](docs/v2/fui-rs/EVENTS_AND_CALLBACKS.md)
- [Text input reference](docs/v2/fui-rs/TEXT_INPUT_REFERENCE.md)
- [Forms and autofill](docs/v2/fui-rs/FORMS_AND_AUTOFILL.md)
- [Theming and styles](docs/v2/fui-rs/THEMING_STYLE_MATRIX.md)

## Contributing

Application developers should begin with the quickstart. Contributors working
on the SDK should follow the
[FUI-RS contributor quickstart](docs/v2/fui-rs/CONTRIBUTOR_QUICKSTART.md).

## License

FUI-RS is AGPL-3.0-only or commercially licensed. See
[the commercial licensing terms](COMMERCIAL.md). EffinDOM runtime components are
distributed under their own terms; FUI-RS's AGPL licence should not be inferred
to apply to the runtime itself.
