# TextLayout

Immediate-mode formatted text resource.

## Constructor

- `TextLayout::text(...)`, `TextLayout::rich(...)`

## Key APIs

- ready callbacks, text metrics, drawing via `DrawContext`.

## Notes

- This is retained SDK state or a retained runtime resource.
- Prefer public constructors/helpers from `fui_rs::prelude::*`.
- Avoid raw runtime handles in app code; use public node/resource APIs.

## See also

- [Per-type reference index](../README.md)
- [Controls and nodes](../../CONTROLS_AND_NODES.md)
- [API reference](../../API_REFERENCE.md)
