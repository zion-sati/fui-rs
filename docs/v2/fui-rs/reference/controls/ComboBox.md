# ComboBox

Editable filter and selection control.

## Constructor

- `combo_box()`, `ComboBox::new()`

## Key APIs

- `items`, `text`, `filter_mode`, `commit_mode`, `auto_complete`, `on_changed`, popup and text-input styling APIs.

## Notes

- This is a retained control. Clone values are cheap handles to the same control.
- Store the control in a page/controller field when callbacks need to mutate it later.
- Use `use fui::prelude::*;` in app code.

## See also

- [Per-type reference index](../README.md)
- [Controls and nodes](../../CONTROLS_AND_NODES.md)
- [Events and callbacks](../../EVENTS_AND_CALLBACKS.md)
