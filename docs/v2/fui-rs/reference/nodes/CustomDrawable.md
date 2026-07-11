# CustomDrawable

Retained custom drawing surface.

## Constructor

- `custom_drawable(|ctx| ...)`, `CustomDrawable::new(...)`

## Key APIs

- `DrawContext` drawing commands, `mark_dirty`, inherited box styling.

## Notes

- This is retained SDK state or a retained runtime resource.
- Prefer public constructors/helpers from `fui::prelude::*`.
- Avoid raw runtime handles in app code; use public node/resource APIs.

## See also

- [Per-type reference index](../README.md)
- [Controls and nodes](../../CONTROLS_AND_NODES.md)
- [API reference](../../API_REFERENCE.md)
