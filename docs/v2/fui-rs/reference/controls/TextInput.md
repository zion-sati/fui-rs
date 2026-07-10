# TextInput

Single-line retained text editor.

## Constructor

- `text_input()`, `TextInput::new()`

## Key APIs

- `text`, `placeholder`, `read_only`, `password`, `host_autofill`, `max_chars`, `focus_now`, text/selection/focus callbacks.

## Notes

- This is a retained control. Clone values are cheap handles to the same control.
- Store the control in a page/controller field when callbacks need to mutate it later.
- Use `use fui_rs::prelude::*;` in app code.

## See also

- [Per-type reference index](../README.md)
- [Controls and nodes](../../CONTROLS_AND_NODES.md)
- [Events and callbacks](../../EVENTS_AND_CALLBACKS.md)
