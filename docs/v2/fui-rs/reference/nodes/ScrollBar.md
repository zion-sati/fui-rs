# ScrollBar

Helper that composes retained scrollbar chrome.

## Constructor

- constructed by scroll surfaces or directly where needed
- `render()` returns the retained `FlexBox` chrome to attach

## Key APIs

- axis-aware track/thumb geometry, colors, drag and track-click behavior.

## Notes

- `ScrollBar` itself is not a `Node`; its rendered `FlexBox` is retained UI.
- It follows the active theme internally. Bind a deliberate per-instance theme
  override to the rendered `FlexBox` when needed.
- Prefer public constructors/helpers from `fui::prelude::*`.
- Avoid raw runtime handles in app code; use public node/resource APIs.

## See also

- [Per-type reference index](../README.md)
- [Controls and nodes](../../CONTROLS_AND_NODES.md)
- [API reference](../../API_REFERENCE.md)
