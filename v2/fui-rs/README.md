# FUI-RS - Rust SDK for EffinDom v2

FUI-RS is the Rust retained-mode SDK for building EffinDom v2 WebAssembly UI
apps. It provides retained controls, layout nodes, text input, overlays,
custom drawing, host services, workers, routing support, and app lifecycle
macros for browser-hosted Rust WASM apps.

## Quickstart

Create a FUI-RS application with the published scaffolder:

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

For a routed application with one independently built WASM module per route:

```bash
npx @effindomv2/create-fui-rs-app my-routed-app -- --template routed
cd my-routed-app
npm install
npm run dev
```

Install [Binaryen](https://github.com/WebAssembly/binaryen) to make release
builds run `wasm-opt`. Development builds do not require it.

Application setup, retained-mode guidance, and entrypoint examples are covered
in the [FUI-RS developer quickstart](../../docs/v2/fui-rs/QUICKSTART.md).

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

## Rich text

Use `rich_text!` to create retained rich text without manually constructing a
span vector. String literals become spans, braced expressions provide dynamic
text, and `span => expression` accepts an existing `RichTextSpan`:

```rust
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

- [SDK docs index](../../docs/v2/fui-rs/SDK_INDEX.md)
- [API reference](../../docs/v2/fui-rs/API_REFERENCE.md)
- [Controls and nodes](../../docs/v2/fui-rs/CONTROLS_AND_NODES.md)
- [Events and callbacks](../../docs/v2/fui-rs/EVENTS_AND_CALLBACKS.md)
- [Text input reference](../../docs/v2/fui-rs/TEXT_INPUT_REFERENCE.md)
- [Forms and autofill](../../docs/v2/fui-rs/FORMS_AND_AUTOFILL.md)
- [Theming and style matrix](../../docs/v2/fui-rs/THEMING_STYLE_MATRIX.md)

## Contributing to FUI-RS

The commands above are for developers building applications with the published
FUI-RS SDK. Contributors working on the SDK, EffinDom runtime, browser bridge,
or repository demos should follow the
[FUI-RS contributor quickstart](../../docs/v2/fui-rs/CONTRIBUTOR_QUICKSTART.md).
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
| Custom drawing and text layouts | Available |
| Browser file/fetch/worker bridges | Available |
| Host services/events generator support | Available |

## Recycled virtual-list rows

`VirtualList` creates a fixed retained row pool. Use `item_template` once to
construct typed row state, then update that state from `on_bind_item` whenever a
pool slot is assigned a new item index:

```rust
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

```rust
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

## Architecture

FUI-RS builds Rust retained UI objects into the EffinDom v2 runtime through the
browser bridge. Rust app WASM and the UI runtime WASM are separate modules;
strings and command data cross the bridge through explicit UTF-8/runtime ABI
calls.

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

## License

AGPL-3.0-only, or commercial license. See [COMMERCIAL.md](COMMERCIAL.md).
