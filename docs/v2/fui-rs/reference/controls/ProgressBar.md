# ProgressBar

Determinate progress visualization.

## Constructor

- `progress_bar()`, `ProgressBar::new()`

## Key APIs

- `value`, `min`, `max`, `length`, `thickness`, `orientation`,
  `sizing(ProgressBarSizing)`, `clear_sizing`,
  `colors(ProgressBarColors)`, `clear_colors`, inherited node/layout APIs.

## Orientation and geometry

`ProgressBar` defaults to horizontal. `length(...)` controls the orientation
axis and `thickness(...)` controls the cross axis. Horizontal bars fill
left-to-right; vertical bars fill bottom-to-top.

```rust
use fui::prelude::*;

let progress = progress_bar();
progress
    .value(40.0)
    .orientation(Orientation::Vertical)
    .length(180.0)
    .thickness(14.0);
```

The semantic buffer receives both the value range and selected orientation.
Unlike `Slider`, `ProgressBar` is not interactive.

## Notes

- This is a retained control. Clone values are cheap handles to the same control.
- Store the control in a page/controller field when callbacks need to mutate it later.
- Use `use fui::prelude::*;` in app code.

## See also

- [Per-type reference index](../README.md)
- [Controls and nodes](../../CONTROLS_AND_NODES.md)
- [Events and callbacks](../../EVENTS_AND_CALLBACKS.md)
