# AntiSelectionArea

Selection barrier island.

## Constructor

- `anti_selection_area()`, `AntiSelectionArea::new()`

## Key APIs

- child composition; prevents ancestor selection areas from collecting subtree text.

## Notes

- This is a retained control. Clone values are cheap handles to the same control.
- Store the control in a page/controller field when callbacks need to mutate it later.
- Use `use fui_rs::prelude::*;` in app code.

## See also

- [Per-type reference index](../README.md)
- [Controls and nodes](../../CONTROLS_AND_NODES.md)
- [Events and callbacks](../../EVENTS_AND_CALLBACKS.md)
