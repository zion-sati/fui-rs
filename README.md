# FUI-RS — retained Rust UI for WebAssembly and native desktop

[![FUI-RS CI](https://github.com/zion-sati/fui-rs/actions/workflows/fui-rs-ci.yml/badge.svg)](https://github.com/zion-sati/fui-rs/actions/workflows/fui-rs-ci.yml)
[![crates.io](https://img.shields.io/crates/v/fui-rs)](https://crates.io/crates/fui-rs)
[![docs.rs](https://img.shields.io/docsrs/fui-rs)](https://docs.rs/fui-rs)
[![npm](https://img.shields.io/npm/v/@effindomv2/fui-rs?label=web%20tooling)](https://www.npmjs.com/package/@effindomv2/fui-rs)
[![License: AGPL-3.0 or commercial](https://img.shields.io/badge/license-AGPL--3.0%20or%20commercial-green.svg)](LICENSE.md)

FUI-RS is a web-born retained-mode Rust UI SDK. It runs application UI in the
browser through WebAssembly and carries the same retained application model to
native macOS, Windows, and Linux. Native applications do not use Electron or a
WebView.

The browser is a primary target, not a compatibility layer. FUI-RS supports the
web platform facilities application UI depends on, including accessibility,
IME and text editing, password-manager integration, workers, history, files,
touch input, and asynchronous assets. Its native hosts reuse that architecture
rather than defining a separate framework.

The SDK includes retained nodes, controls, layout, themes, events, popups,
dialogs, editable text, selection, custom drawing, workers, host services,
accessibility semantics, and Rust-specific authoring macros.

- [Run the browser demo](https://fui-rs-demo.effindom.dev/)
- [Play Galaga-RS](https://jatm80.github.io/galaga-rs/), a Galaga-style space
  shooter and the first known community-built FUI-RS application

https://github.com/user-attachments/assets/75815f18-8476-4882-b290-6a6bd6a9b0e7

<details>

<summary>More demo videos, including the Galaga-style space shooter</summary>

https://github.com/user-attachments/assets/198f1c7f-3d92-4acb-b2b3-53fd72898ed0

https://github.com/user-attachments/assets/22e2c40f-a762-4a05-b164-4499cca0bf27

https://github.com/user-attachments/assets/f4b94bad-b93d-486e-8ed4-43482990d324

https://github.com/user-attachments/assets/ac497b4a-cc36-4f4f-862d-005ca706edbc

</details>

## Start a project

For a new native, web, or universal application, use
[`cargo-fui`](https://github.com/zion-sati/cargo-fui):

Install the platform compiler and linker first: Xcode Command Line Tools on
macOS; Visual Studio 2022 Build Tools with Desktop development with C++ and a
Windows SDK on Windows; or a C/C++ toolchain and the required native development
libraries on Linux. Windows ARM64 also requires the ARM64 C++ build tools.

Then install stable Rust and Cargo through [rustup](https://rustup.rs/):

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
 browser/WebAssembly     native host
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

FUI-RS is feature-rich early access. It already supports substantial retained
UI and native packaging, but breaking API and generated-project changes remain
possible before 1.0 when correctness or Rust ergonomics require them.

Current boundaries:

- Native support is desktop macOS, Windows, and Linux. iOS and Android are not
  currently supported.
- Accessibility projection is implemented through DOM/ARIA on the web,
  `NSAccessibility` on macOS, Microsoft UI Automation on Windows, and AT-SPI on
  Linux. Broad compatibility testing across screen readers, browser
  combinations, and Linux desktop environments remains early.
- Browser-native find-on-page cannot be reproduced perfectly for all mirrored,
  hidden, and virtualised content.
- Native and web targets intentionally expose different platform-service
  adapters where the underlying capability differs.
- Browser-only routed applications use separate WASM modules and a shared
  browser shell; native applications do not use that routing model.
- Native packaging does not supply an application's production signing identity,
  store account, notarization credentials, or distribution policy.
- The third-party control and integration ecosystem is new.

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
