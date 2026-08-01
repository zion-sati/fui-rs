# FUI-RS - retained Rust UI for WebAssembly and native desktop

FUI-RS is the web-born retained-mode Rust UI SDK for EffinDOM. One application
model runs in the browser through WebAssembly and as a real native macOS,
Windows, or Linux desktop application. Native applications embed neither
Chromium nor a system WebView.

The SDK provides retained controls, layout nodes, text input, overlays, custom
drawing, host services, workers, accessibility semantics, routing support, and
application lifecycle macros.

The Cargo package is named `fui-rs`, while application code imports its
library as `fui`:

```bash
cargo add fui-rs --rename fui
```

## Quickstart

Create a native, web, or universal application with `cargo-fui`:

```bash
# Install stable Rust and Cargo once
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

cargo install --locked cargo-fui
cargo fui new my-app --target universal
cd my-app
cargo fui dev
```

Choose `native` for desktop only, `web` for browser only, or `universal` for a
shared retained UI with explicit native and web adapters. Native-only projects
do not require Node.js. `cargo fui build --release` creates optimised output and
`cargo fui package` emits DMG, MSIX, or AppImage packages.

For a browser-only routed application with one independently built WASM module
per route, use the npm scaffolder:

```bash
npx @effindomv2/create-fui-rs-app my-routed-app -- --template routed
cd my-routed-app
npm install
npm run dev
```

Install [Binaryen](https://github.com/WebAssembly/binaryen) to make release
builds run `wasm-opt`. Development builds do not require it.

Application setup, retained-mode guidance, and entrypoint examples are covered
in the [FUI-RS developer quickstart](https://github.com/zion-sati/fui-rs/blob/main/docs/v2/fui-rs/QUICKSTART.md).

## Minimal app

```rust
use fui::prelude::*;

fn build_page() -> FlexBox {
    ui! {
        column().fill_size().padding(24.0, 24.0, 24.0, 24.0) {
            text("Hello from Rust"),
            button("Click me").on_click(|_| {}),
        }
    }
}

fui_app!(FlexBox, build_page);
```

## Documentation

- [SDK index](https://github.com/zion-sati/fui-rs/blob/main/docs/v2/fui-rs/SDK_INDEX.md)
- [Quickstart](https://github.com/zion-sati/fui-rs/blob/main/docs/v2/fui-rs/QUICKSTART.md)
- [API reference](https://github.com/zion-sati/fui-rs/blob/main/docs/v2/fui-rs/API_REFERENCE.md)
- [Custom fonts](https://github.com/zion-sati/fui-rs/blob/main/docs/v2/fui-rs/CUSTOM_FONTS.md)
- [Custom drawing and bitmaps](https://github.com/zion-sati/fui-rs/blob/main/docs/v2/fui-rs/CUSTOM_DRAWING_AND_BITMAPS.md)
- [Host services, host events, and workers](https://github.com/zion-sati/fui-rs/blob/main/docs/v2/fui-rs/HOST_SERVICES_AND_WORKERS.md)
- [Accessibility and semantics](https://github.com/zion-sati/fui-rs/blob/main/docs/v2/fui-rs/ACCESSIBILITY_AND_SEMANTICS.md)

The [live routed demo](https://fui-rs-demo.effindom.dev/) demonstrates the
browser/npm workflow. Native and packaging claims are validated separately by
`cargo-fui` fixtures and native platform tests.

## Rich text

Use `rich_text!` to create retained rich text without manually constructing a
span vector. String literals become spans, braced expressions provide dynamic
text, and `span => expression` accepts an existing `RichTextSpan`:

```rust
use fui::prelude::*;

let value = 42;
let suffix = span("!").underline();
let label = rich_text![
    "Current value: ".italic(),
    { format!("{value}") }.bold().text_color(rgb(0x3a, 0xc5, 0x6c)),
    span => suffix,
]
.font_size(18.0);
```

## SDK docs

- [SDK docs index](https://github.com/zion-sati/fui-rs/blob/main/docs/v2/fui-rs/SDK_INDEX.md)
- [API reference](https://github.com/zion-sati/fui-rs/blob/main/docs/v2/fui-rs/API_REFERENCE.md)
- [Controls and nodes](https://github.com/zion-sati/fui-rs/blob/main/docs/v2/fui-rs/CONTROLS_AND_NODES.md)
- [Events and callbacks](https://github.com/zion-sati/fui-rs/blob/main/docs/v2/fui-rs/EVENTS_AND_CALLBACKS.md)
- [Text input reference](https://github.com/zion-sati/fui-rs/blob/main/docs/v2/fui-rs/TEXT_INPUT_REFERENCE.md)
- [Forms and autofill](https://github.com/zion-sati/fui-rs/blob/main/docs/v2/fui-rs/FORMS_AND_AUTOFILL.md)
- [Theming and style matrix](https://github.com/zion-sati/fui-rs/blob/main/docs/v2/fui-rs/THEMING_STYLE_MATRIX.md)

## Contributing to FUI-RS

The commands above are for developers building applications with the published
FUI-RS SDK. Contributors working on the SDK, EffinDom runtime, browser bridge,
or repository demos should follow the
[FUI-RS contributor quickstart](https://github.com/zion-sati/fui-rs/blob/main/docs/v2/fui-rs/CONTRIBUTOR_QUICKSTART.md).
It covers the standalone repository toolchain, SDK build, lint, and test lanes.

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
| Custom drawing, paths, dynamic bitmaps, offscreen composition, retained rasterization, and timers | Available on web and native |
| Native and browser platform adapters | Available |
| Browser file/fetch bridges | Available in browser applications |
| First-party background workers | Available on web and native |
| Host services/events generator support | Available |

## Recycled virtual-list rows

`VirtualList` creates a fixed retained row pool. Use `item_template` once to
construct typed row state, then update that state from `on_bind_item` whenever a
pool slot is assigned a new item index:

```rust
use fui::prelude::*;

struct ContactRow {
    name: TextNode,
}

let contacts = virtual_list(10_000, 28.0)
    .item_template(|container| {
        let name = text("");
        container.child(&name);
        ContactRow { name }
    });
contacts.on_bind_item(|row, index| {
    row.name.text(format!("Contact {index}"));
});
```

The template is not rerun while scrolling. Do not key recycled rows by pointer
or create controls inside `on_bind_item`.

## Scrollbar styling

Apply common scrollbar chrome without leaving fluent `ScrollBox` construction:

```rust
use fui::prelude::*;

let content = scroll_box().scrollbar_style(
    ScrollBarStyle::new()
        .track_width(10.0)
        .thumb_width(7.0)
        .thumb_corner_radius(3.5),
);
```

Use `vertical_scrollbar()` or `horizontal_scrollbar()` afterward for an
axis-specific override.

## Host-event lifetime

Generated `on_*` host-event functions return `HostEventSubscription`. Retain
the guard for exactly as long as the handler should remain active; dropping it
unsubscribes automatically. Replacing a handler is generation-safe, so dropping
an older guard cannot remove its replacement.

## Worker entrypoints

Enable the `worker-runtime` feature, implement `Default + WorkerJob`, and let
the SDK emit resumable entries plus the shared callback-buffer ABI:

```rust,ignore
use fui::prelude::*;

#[derive(Default)]
struct PrimeJob {
    state: WorkerJobState,
}

impl WorkerJob for PrimeJob {
    fn state(&mut self) -> &mut WorkerJobState { &mut self.state }
    fn run(&mut self) { self.complete("done"); }
}

fui_worker!(primeWorker => PrimeJob);
```

Declare each Worker artifact, native Cargo manifest, and exported entry in
`fui.toml`. Application code then uses the same identity on every target:

```rust,ignore
let worker = Worker::new("./workers.wasm", "primeWorker")
    .on_complete(|event| println!("{}", event.result))
    .start("input");
```

On the web, `cargo-fui` compiles the worker crate to the declared Worker WASM
artifact and runs it in a browser Worker. On native desktop, the same worker
crate is linked into the application and the artifact/entry pair resolves
through a generated registry onto a dedicated thread; native does not load
`workers.wasm` from disk. Cancellation is cooperative, so long-running jobs
must yield or check cancellation regularly. Worker callbacks are delivered on
the application UI thread.

## Architecture

FUI-RS builds retained Rust UI objects against the shared EffinDom runtime.
Native hosts execute that runtime directly through platform adapters. On the
web, Rust app WASM and the UI runtime WASM are separate modules; strings and
command data cross the browser bridge through explicit UTF-8/runtime ABI calls.

Retained controls are cheap clone handles. Cloning a control gives another Rust
handle to the same retained UI object.

FUI-RS maps retained inheritance to capability traits. `Node` supplies the
universal retained/event surface; FlexBox-derived visuals additionally expose
`LayoutSurface`, `BoxStyleSurface`, `FlexLayoutSurface`, and
`ChildContainerSurface`. `TextSurface` covers `Text` and `RichText`, while
`TextEditorSurface` covers `TextInput` and `TextArea`.

Use `on_pointer_click(...)` for raw routed pointer input. Use
`on_pointer_double_click(...)` and `on_pointer_triple_click(...)` for exact raw
multi-click gestures. `Button`, `Checkbox`, `RadioButton`, and `Switch` expose
count-free `on_click(...)` semantic activation for supported pointer and
keyboard input.

## Project status

FUI-RS is feature-rich early access. Its retained SDK, web and native hosts,
controls, text editing, accessibility projection, custom drawing, and packaging
workflow are usable today, but public APIs remain pre-1.0 and may change
incompatibly.

Current platform limits:

- Accessibility projection is implemented through DOM/ARIA on the web,
  `NSAccessibility` on macOS, Microsoft UI Automation on Windows, and AT-SPI on
  Linux. Broad compatibility testing across screen readers, browser
  combinations, and Linux desktop environments remains early.
- iOS and Android are not currently supported.
- Find-on-page is implemented through retained find for the normal desktop
  shortcut and projected semantic text for mobile or explicitly invoked
  browser-native find. Browser-native highlights can render with a slightly
  different DOM font from the canvas text.
- The third-party control and integration ecosystem is new.

Try the [live demo](https://fui-rs-demo.effindom.dev/), then open a discussion
or issue if a real application is blocked by a missing capability.

## License

AGPL-3.0-only, or commercial license. See [COMMERCIAL.md](COMMERCIAL.md).
