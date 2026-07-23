# FUI-RS Events and Callbacks (v2)

FUI-RS events use typed EventArgs structs and ordinary Rust closures.

```rust
button("Save").on_click(|_| {
    logger::info("Button", "Save activated");
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

- `on_pointer_click(...)`
- `on_pointer_double_click(...)`
- `on_pointer_triple_click(...)`
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

`on_pointer_click(...)` is a low-level routed pointer event. It fires for every
click count and exposes mutable `PointerEventArgs` so handlers can consume the
event. For exact double or triple clicks, `on_pointer_double_click(...)` or
`on_pointer_triple_click(...)` then fires with the same event and handled state.

`Button::on_click(...)`, `Checkbox::on_click(...)`, `RadioButton::on_click(...)`,
and `Switch::on_click(...)` are semantic activation APIs. They fire for
supported pointer and keyboard activation and receive count-free
`ClickEventArgs`. Toggle controls emit `on_changed(...)` before `on_click(...)`
when user activation changes state. Programmatic and persisted state changes
emit `on_changed(...)` but never `on_click(...)`. `NavLink` uses
`on_navigate(...)` for navigation activation.

## Custom context menus and host capabilities

`on_context_menu(...)` receives the original descendant target, pointer
coordinates, and an immutable `event.host` snapshot. Gate menu operations by
capability instead of inferring browser or desktop behavior from the OS:

```rust
node.on_context_menu({
    let menu = menu.clone();
    move |event| {
        let mut items = Vec::new();
        if event.host.supports(HostCapability::NewBrowsingContext) {
            items.push(MenuItem::new("New Tab", ContextMenuAction::OpenLinkInNewTab)
                .payload(url.clone()));
        }
        if event.host.supports(HostCapability::OpenExternalUri) {
            items.push(MenuItem::new("Open", ContextMenuAction::OpenLink)
                .payload(url.clone()));
        }
        menu.items(items).show(event.x, event.y);
    }
});
```

Use `host_context()`, `host_environment()`, or `has_host_capability(...)` when a
menu factory is built outside the callback. `PlatformFamily` remains the OS
family and does not identify browser versus desktop execution.

On browser hosts, secondary click and coarse-pointer long press request the
retained menu. On desktop hosts, secondary click requests it through the native
input adapter; macOS also normalizes Control-click and supports `Shift+F10` for
the focused control. Marking secondary pointer input handled suppresses the
built-in fallback. Auxiliary/middle click remains a separate NavLink action and
never opens a context menu.

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
