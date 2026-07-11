# ProgressBar

Determinate progress visualization.

## Constructor

- `progress_bar()`, `ProgressBar::new()`

## Key APIs

- `value`, `min`, `max`, `length`, `thickness`, `track_color`, `fill_color`, inherited node/layout APIs.

## Notes

- This is a retained control. Clone values are cheap handles to the same control.
- Store the control in a page/controller field when callbacks need to mutate it later.
- Use `use fui::prelude::*;` in app code.

## See also

- [Per-type reference index](../README.md)
- [Controls and nodes](../../CONTROLS_AND_NODES.md)
- [Events and callbacks](../../EVENTS_AND_CALLBACKS.md)
