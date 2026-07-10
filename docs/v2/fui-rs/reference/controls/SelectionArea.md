# SelectionArea

Cross-node text selection host.

## Constructor

- `selection_area()`, `SelectionArea::new()`

## Key APIs

- selected text tracking, child composition, retained selection behavior.

## Notes

- This is a retained control. Clone values are cheap handles to the same control.
- Store the control in a page/controller field when callbacks need to mutate it later.
- Use `use fui_rs::prelude::*;` in app code.

## See also

- [Per-type reference index](../README.md)
- [Controls and nodes](../../CONTROLS_AND_NODES.md)
- [Events and callbacks](../../EVENTS_AND_CALLBACKS.md)
