# Slider

Single-value range input.

## Constructor

- `slider()`, `Slider::new()`

## Key APIs

- `min`, `max`, `step`, `value`, `orientation`, `length`, `fill_length`, `on_changed`, `template`, `sizing`, `colors`,
  inherited node/layout APIs.

## Length

`length(...)` sets the track's length along the slider's orientation in pixels; the focus chrome adds a few pixels
around it. `fill_length()` makes the slider fill its container along its orientation instead, as `fill_width` or
`fill_height` does, so it follows the container on any screen. Calling `length(...)` again returns to a fixed length.

```rust
use fui::prelude::*;

let volume = slider();
volume.min(0.0).max(100.0).value(40.0).fill_length();
```

A template's `SliderPresenter` lays out relative to its root, which the slider sizes to its track area; `layout`
receives the visual state but no length.

## Notes

- This is a retained control. Clone values are cheap handles to the same control.
- Store the control in a page/controller field when callbacks need to mutate it later.
- Use `use fui::prelude::*;` in app code.

## See also

- [Per-type reference index](../README.md)
- [Controls and nodes](../../CONTROLS_AND_NODES.md)
- [Events and callbacks](../../EVENTS_AND_CALLBACKS.md)
