# FUI-RS SDK Docs Index (v2)

This is the primary SDK navigation page for `v2/fui-rs`.

FUI-RS is the Rust SDK for EffinDom retained UI apps. It uses Rust
ownership, cheap retained-control clones, RAII guards, and macros for app
entrypoints and mixed child trees.

## Start here

- [Quickstart](./QUICKSTART.md)
- [API reference](./API_REFERENCE.md)
- [Controls and nodes overview](./CONTROLS_AND_NODES.md)
- [Text input reference](./TEXT_INPUT_REFERENCE.md)

## Accessibility

- [Accessibility and semantics](./ACCESSIBILITY_AND_SEMANTICS.md)
- [Text input reference](./TEXT_INPUT_REFERENCE.md)
- [Events and callbacks](./EVENTS_AND_CALLBACKS.md)

## Controls

- [Controls and nodes overview](./CONTROLS_AND_NODES.md#controls)
- [Per-type reference](./reference/README.md)
- [Keyboard policy](./KEYBOARD_POLICY.md)
- [Control customization and templating](./CONTROL_CUSTOMIZATION.md)
- [Overlays and portals](./OVERLAYS_AND_PORTALS.md)
- [Forms and autofill](./FORMS_AND_AUTOFILL.md)

## Nodes

- [Controls and nodes overview](./CONTROLS_AND_NODES.md#nodes)
- [Per-type reference](./reference/README.md#nodes-and-resources)
- [Core layout concept (`width(100%)` vs `fill_width()`)](./CONTROLS_AND_NODES.md#core-layout-concept)
- [Layout sizing guide](./CONTROLS_AND_NODES.md#layout-sizing-guide-fill-vs-percent)
- [Events and callbacks](./EVENTS_AND_CALLBACKS.md#node-level-events)

## Core

- [API reference](./API_REFERENCE.md)
- [Application lifecycle macros](./API_REFERENCE.md#application-lifecycle)
- [Browser file bridge](./API_REFERENCE.md#browser-file-bridge)
- [Browser fetch bridge](./API_REFERENCE.md#browser-fetch-bridge)
- [Workers](./API_REFERENCE.md#workers)
- [Timers](./API_REFERENCE.md#timers)
- [Platform helpers](./API_REFERENCE.md#platform-and-shortcuts)
- [DevTools DOM Mirror](https://github.com/zion-sati/EffinDOM/blob/main/docs/v2/browser-bridge/DEVTOOLS_DOM_MIRROR.md)

## Theme

- [Theming and style matrix](./THEMING_STYLE_MATRIX.md)
- [Control customization and templating](./CONTROL_CUSTOMIZATION.md)
- [Controls and nodes](./CONTROLS_AND_NODES.md)

## Rust-specific DX

- [Retained-mode app construction](./QUICKSTART.md#retained-mode-model)
- [`ui!` mixed child tree macro](./QUICKSTART.md#mixed-child-trees-with-ui)
- [`rich_text!` retained rich-text macro](./QUICKSTART.md#rich-text-with-rich_text)
- [`fui_app!` and `fui_managed_app!`](./QUICKSTART.md#app-entrypoint-macros)
- [Rust SDK conventions](./API_REFERENCE.md#rust-sdk-conventions)

## Contributing

- [FUI-RS contributor quickstart](./CONTRIBUTOR_QUICKSTART.md)
- [EffinDom Browser Bridge documentation](https://github.com/zion-sati/EffinDOM/tree/main/docs/v2/browser-bridge)
