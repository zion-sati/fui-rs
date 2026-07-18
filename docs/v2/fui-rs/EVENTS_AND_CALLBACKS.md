# FUI-RS Events and Callbacks (v2)

FUI-RS events use typed EventArgs structs and ordinary Rust closures.

```rust
button("Save").on_click(|event| {
    logger::info("Button", format!("click count {}", event.click_count));
});
```

## Callback ownership

Closures are `'static` because retained UI objects outlive the stack frame that
created them. Clone only the retained handles or state values you need:

```rust
let status = text("Idle");
button("Run").on_click({
    let status = status.clone();
    move |_| status.text("Running")
});
```

A cloned retained control is a cheap handle to the same UI object.

## EventArgs policy

Public UI events use EventArgs structs rather than flattened callback arguments.
This keeps callbacks stable as the event surface grows.

Common event args:

- `ClickEventArgs`
- `PointerEventArgs`
- `WheelEventArgs`
- `KeyEventArgs`
- `FocusChangedEventArgs`
- `TextChangedEventArgs`
- `SelectionChangedEventArgs`
- `GestureEventArgs`
- `LongPressEventArgs`
- control-specific changed args such as `SliderChangedEventArgs`

## Handled events

Routed mutable event args include `handled` where user code can claim the event.
Set `event.handled = true` to stop bubbling and suppress framework defaults.

```rust
flex_box().on_wheel(|event| {
    if should_keep_wheel_here(event.delta_y) {
        event.handled = true;
    }
});
```

If an event is not handled, framework defaults still run. For example, wheel
input can scroll a `ScrollView`, and keyboard shortcuts can pass through to the
browser unless a focused control or user handler handles them.

## Node-level events

Common node event APIs:

- `on_click(...)`
- `on_double_click(...)`
- `on_triple_click(...)`
- `on_pointer_down(...)`
- `on_pointer_move(...)`
- `on_pointer_up(...)`
- `on_pointer_enter(...)`
- `on_pointer_leave(...)`
- `on_pointer_cancel(...)`
- `on_wheel(...)`
- `on_key_down(...)`
- `on_key_up(...)`
- `on_focus_changed(...)`
- `on_pan_gesture(...)`
- `on_pinch_gesture(...)`
- `on_long_press(...)`
- `on_context_menu(...)`

## Pointer events

`PointerEventArgs` includes:

- local coordinates: `x`, `y`
- scene coordinates: `scene_x`, `scene_y`
- `pointer_id`
- `pointer_type`
- `button`, `buttons`
- `modifiers`
- `pressure`, `width`, `height`
- `click_count`
- `handled`

Pointer events route to the deepest enabled target first, then bubble through
enabled ancestors. Disabled nodes are skipped during hit testing.

## Wheel events

`WheelEventArgs` includes local/scene coordinates, `delta_x`, `delta_y`,
`delta_mode`, modifiers, and `handled`.

## Keyboard events

`KeyEventArgs` includes key identity, event type, modifiers, repeat state, and
handled state. Built-in controls consume the keys they own. If an app handler
marks a key handled, browser default handling is suppressed.

## Gesture events

Gesture APIs are explicit opt-in for custom controls:

```rust
custom_surface
    .on_pan_gesture(|event| {
        if event.phase == GestureEventPhase::Update {
            event.handled = true;
        }
    })
    .on_pinch_gesture(|event| {
        if event.phase == GestureEventPhase::Update {
            event.handled = true;
        }
    });
```

`LongPressEventArgs` is used for long-press gestures and context-menu-like
behavior on coarse pointer devices.

## Changed events

Stateful controls emit typed changed events when state changes:

- `CheckboxChangedEventArgs`
- `SwitchChangedEventArgs`
- `RadioButtonChangedEventArgs`
- `RadioGroupChangedEventArgs`
- `SliderChangedEventArgs`
- `DropdownChangedEventArgs<T>`
- `ComboBoxChangedEventArgs<T>`
- `TextChangedEventArgs`
- `SelectionChangedEventArgs`

Programmatic and user-driven state changes both update control state. User-visible
controls may also request semantic announcements for active focused state changes.

## See also

- [SDK docs index](./SDK_INDEX.md)
- [Keyboard policy](./KEYBOARD_POLICY.md)
- [Controls and nodes](./CONTROLS_AND_NODES.md)
