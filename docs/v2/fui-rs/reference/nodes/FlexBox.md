# FlexBox

Base retained layout and styling node.

## Constructor

- `flex_box()`, `row()`, `column()`, `FlexBox::default()`

## Key APIs

- sizing, fill, padding, margin, gap, flex direction/wrap, alignment, background, border, radius, gradient, blur, shadow, opacity, child composition.

## Notes

- This is retained SDK state or a retained runtime resource.
- Prefer public constructors/helpers from `fui_rs::prelude::*`.
- Avoid raw runtime handles in app code; use public node/resource APIs.

## See also

- [Per-type reference index](../README.md)
- [Controls and nodes](../../CONTROLS_AND_NODES.md)
- [API reference](../../API_REFERENCE.md)
