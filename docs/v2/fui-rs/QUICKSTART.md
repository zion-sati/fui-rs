# FUI-RS Quickstart

FUI-RS is the Rust SDK for EffinDom v2. It builds retained Rust UI objects into
WebAssembly and runs them through the shared EffinDom browser bridge/runtime.

Use this page to get running quickly, then use the [SDK docs index](./SDK_INDEX.md)
for the full public docs map.

## Prerequisites

Install the shared v2 toolchain first:

- [Top-level v2 quickstart](../../QUICKSTART.md)

Then install Rust/Cargo, the wasm target, and Binaryen:

### macOS

```bash
brew install binaryen
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustup target add wasm32-unknown-unknown
```

### Linux (Debian / Ubuntu)

```bash
sudo apt-get install -y binaryen
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustup target add wasm32-unknown-unknown
```

`binaryen` provides `wasm-opt`. FUI-RS builds run `wasm-opt -O3` when available,
preserving speed-oriented output while eliminating additional dead code.

## Create a new app

```bash
npx @effindomv2/create-fui-rs-app my-app
cd my-app
npm install
npm run dev
```

For the routed starter:

```bash
npx @effindomv2/create-fui-rs-app my-routed-app -- --template routed
cd my-routed-app
npm install
npm run dev
```

The routed template builds one WASM per route so route pages can be shipped as
separate micro-frontends. Route crates use retained components and explicit
presentation state rather than an object-oriented MVC ownership graph.

## Build the repo demo

From the repository root:

```bash
npm run build:v2:fui-rs
npm run serve
```

Open:

```text
http://127.0.0.1:8080/v2/fui-rs/demo/index.html
```

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

`fui_app!` emits the browser lifecycle exports required by the harness. Normal
app code should not hand-write `#[no_mangle] pub extern "C" fn __runApp()`.

## Retained-mode model

FUI-RS is retained mode. Construct controls once, store stateful controls as
fields when you need to mutate them later, and mutate retained objects in event
callbacks.

Correct retained shape:

```rust
use fui::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

#[derive(Clone)]
struct CounterPage {
    root: FlexBox,
    count_label: Text,
}

fui_component!(CounterPage => root);

impl CounterPage {
    fn new() -> Self {
        let count_label = text("Count: 0");
        let button = button("Increment");
        let count = Rc::new(Cell::new(0));

        button.on_click({
            let count_label = count_label.clone();
            let count = count.clone();
            move |_| {
                let next = count.get() + 1;
                count.set(next);
                count_label.text(format!("Count: {next}"));
            }
        });

        let root = ui! {
            column().padding(16.0, 16.0, 16.0, 16.0) {
                count_label.clone(),
                button,
            }
        };

        Self { root, count_label }
    }
}

fui_managed_app!(CounterPage, CounterPage::new, |page: &CounterPage| page.clone());
```

Do not recreate retained controls in a render loop. That loses identity, focus,
scroll state, subscriptions, overlay state, and persisted control state.

## Mixed child trees with `ui!`

Rust `Vec<T>` requires one concrete item type. `ui!` is syntax sugar over
retained builders so mixed trees do not need repeated `.child(...)` or `.into()`
noise.

```rust
let page = ui! {
    column().gap(12.0).fill_width() {
        text("Profile"),
        row().gap(8.0) {
            text_input().configure(|input| {
                input.placeholder("Name").fill_width();
            }),
            button("Save"),
        },
    }
};
```

Most retained setters return `&Self` because controls are cheap cloned handles
with interior retained state. `ui!` accepts those borrowed fluent expressions
directly and preserves the original retained identity.

Stateful controls still need explicit variables when callbacks or later methods
must mutate them.

## App entrypoint macros

Use `fui_app!` for simple pages:

```rust
fn build_page() -> FlexBox { column() }
fui_app!(FlexBox, build_page);
```

Use `fui_managed_app!` for retained page/controller ownership:

```rust
#[derive(Clone)]
struct Page { root: FlexBox }

impl Page {
    fn new() -> Self { Self { root: column() } }
}

fui_managed_app!(Page, Page::new, |page: &Page| page.root.clone());
```

Optional `mount:` and `dispose:` callbacks are available for route pages that
need to attach host subscriptions or release route-scoped resources.

## Common imports

Application code should normally use:

```rust
use fui::prelude::*;
```

Avoid importing from `bindings`, `generated`, internal control modules, or raw
FFI modules in app code.

## Next docs

- [SDK docs index](./SDK_INDEX.md)
- [API reference](./API_REFERENCE.md)
- [Controls and nodes](./CONTROLS_AND_NODES.md)
- [Events and callbacks](./EVENTS_AND_CALLBACKS.md)
