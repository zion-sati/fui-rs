# TextArea

Multiline retained text editor.

## Constructor

- `text_area()`, `TextArea::new()`

## Key APIs

- `text`, `placeholder`, `wrapping`, scrollbar visibility, `accepts_tab`, text/selection/focus callbacks.

## Notes

- This is a retained control. Clone values are cheap handles to the same control.
- Store the control in a page/controller field when callbacks need to mutate it later.
- Use `use fui::prelude::*;` in app code.

## See also

- [Per-type reference index](../README.md)
- [Controls and nodes](../../CONTROLS_AND_NODES.md)
- [Events and callbacks](../../EVENTS_AND_CALLBACKS.md)
