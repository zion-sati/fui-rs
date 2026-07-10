# ScrollBox

High-level scroll container with owned scrollbars.

## Constructor

- `scroll_box()`, `ScrollBox::new()`

## Key APIs

- per-axis enablement, scrollbar visibility, scroll offsets, animated scrolling, child composition.

## Notes

- This is retained SDK state or a retained runtime resource.
- Prefer public constructors/helpers from `fui_rs::prelude::*`.
- Avoid raw runtime handles in app code; use public node/resource APIs.

## See also

- [Per-type reference index](../README.md)
- [Controls and nodes](../../CONTROLS_AND_NODES.md)
- [API reference](../../API_REFERENCE.md)
