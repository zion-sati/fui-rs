# FUI-RS — Rust bindings for EffinDom v2

> **⚠️ Early stage.** FUI-RS is a thin Rust binding over the shared C++ ABI.
> It currently provides a bare-bones smoke app and a fluent node builder.
> Controls, theming, signals, and component reconciliation are planned for
> future slices.  Expect breaking changes.

## Quickstart

```bash
# Prerequisites: Rust + wasm32-unknown-unknown target
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown

# Clone and build
git clone https://github.com/zion-sati/EffinDOM.git
cd EffinDOM
npm ci
npm run build:v2:browser-bridge
npm run build:v2:fui-rs
npm run serve
```

Open `http://127.0.0.1:8080/v2/fui-rs/index.html`.

Full quickstart: [docs/v2/fui-rs/QUICKSTART.md](docs/v2/fui-rs/QUICKSTART.md)

## What's here (Slice 1)

| Primitive | Status |
|---|---|
| `FlexBox` / `TextNode` builders | ✅ |
| `Component` + `Application::run` | ✅ |
| `state` / `derived` reactivity | ✅ |
| Destroy-and-remount reconciliation | ✅ |
| Controls (Button, Slider, etc.) | 🔜 Slice 2 |
| Theming / styles | 🔜 Slice 2 |
| Signal-based reactivity | 🔜 Slice 2 |

## Architecture

```
Rust (wasm32-unknown-unknown)        C++ UI Runtime
┌──────────────────────┐            ┌────────────────┐
│  ffi::ui_set_text()  │──imports──▶│  _ui_set_text() │
│  Node::build()       │            │  _ui_commit_frame()
│  Component::render() │            │                 │
└──────────────────────┘            └────────────────┘
         │                                    │
         └──────────── JS bridge ──────────────┘
              (harness.ts wires both sides)
```

FUI-RS calls the same C ABI as fui-as and fui-kt — the binding layer is the
only difference.

## License

AGPL-3.0-only (or commercial — see [COMMERCIAL.md](COMMERCIAL.md)).
