# ScrollView

Low-level retained viewport.

## Constructor

- `scroll_view()`, `ScrollView::new()`

## Key APIs

- scroll offset, scroll content size, animated scrolling, scroll changed callbacks, child composition
- `smooth_scrolling(true)` is the default; rapid wheel events accumulate into
  one smooth target without debounce. Pass `false` for immediate wheel steps.
  Touch, scrollbar, and explicit programmatic scrolling remain direct.
- `wheel_latching(true)` is the default: a mouse-wheel run that starts in this
  scroll view stays in it at its edges, as browsers latch wheel scrolling, and
  only a wheel event after a pause (150 ms) scrolls an ancestor. Pass `false`
  to hand the rest of a run to the nearest ancestor that can move. Trackpad
  gestures keep their scroll view until they end, unless it does not latch.

## Notes

- This is retained SDK state or a retained runtime resource.
- Prefer public constructors/helpers from `fui::prelude::*`.
- Avoid raw runtime handles in app code; use public node/resource APIs.

## See also

- [Per-type reference index](../README.md)
- [Controls and nodes](../../CONTROLS_AND_NODES.md)
- [API reference](../../API_REFERENCE.md)
