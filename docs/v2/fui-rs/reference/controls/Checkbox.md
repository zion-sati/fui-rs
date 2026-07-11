# Checkbox

Boolean or tri-state check control.

## Constructor

- `checkbox(label)`, `Checkbox::new(label)`

## Key APIs

- `check`, `tri_state`, `mixed`, `on_changed`, `template`, `sizing`, `colors`, inherited node/layout APIs.

## Notes

- This is a retained control. Clone values are cheap handles to the same control.
- Store the control in a page/controller field when callbacks need to mutate it later.
- Use `use fui::prelude::*;` in app code.

## See also

- [Per-type reference index](../README.md)
- [Controls and nodes](../../CONTROLS_AND_NODES.md)
- [Events and callbacks](../../EVENTS_AND_CALLBACKS.md)
