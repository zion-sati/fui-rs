# FUI-RS API Reference (v2)

This page documents the public Rust SDK surface exported by `v2/fui-rs/src/lib.rs`
and `fui::prelude::*`.

For practical control guidance, see [Controls and nodes](./CONTROLS_AND_NODES.md).
For navigation across the full doc set, see [SDK docs index](./SDK_INDEX.md).

## Public import policy

Use this in app code:

```rust
use fui::prelude::*;
```

The crate root also re-exports the public surface, but the prelude is the
recommended app-facing import. Avoid direct app imports from `bindings`,
`generated`, raw `ffi`, `popup_presenter`, and control `internal` modules.

## Application lifecycle

Public lifecycle types and macros:

- `Application`
- `ApplicationRegistration`
- `ManagedApplication<TPage>`
- `fui_app!(PageType, build_page)`
- `fui_managed_app!(PageType, build_page, get_root)`
- `fui_managed_app!(PageType, build_page, get_root, mount: mount_page)`
- `fui_managed_app!(PageType, build_page, get_root, dispose: dispose_page)`
- `fui_managed_app!(PageType, build_page, get_root, mount: mount_page, dispose: dispose_page)`
- `fui_component!(ComponentType => root_field)`
- `fui_component!(ComponentType => root_field, owner: state_field)`
- `fui_component!(ComponentType => root_field, owners: [state_field, guard_field])`

Normal apps and route pages should use the macros. They emit the browser harness
`__runApp` / `__disposeApp` exports and keep ABI details out of user code.

```rust
use fui::prelude::*;

fn build_page() -> FlexBox {
    ui! { column().fill_size() { text("Hello") } }
}

fui_app!(FlexBox, build_page);
```

## Retained node lifecycle

`Node` objects and retained runtime handles have different lifetimes.

- Controls and nodes can be constructed and configured before they are built.
- Runtime handles are assigned when the tree is built into the UI runtime.
- Route changes, app reset, and dispose invalidate runtime handles.
- Public app code should keep source-of-truth state in Rust controls/fields, not raw handles.
- Use public node methods such as `get_bounds()`, `absolute_to_local_position(...)`, and `local_to_absolute_position(...)` rather than caching handles.

## Rust ownership model

Retained controls are cheap clone handles to shared retained state. Cloning a
`Button`, `Text`, `FlexBox`, or `TextInput` does not duplicate the UI control;
it gives another Rust handle to the same retained object.

```rust
let label = text("Idle");
button("Run").on_click({
    let label = label.clone();
    move |_| label.text("Running")
});
```

Subscriptions and guards use RAII: dropping the guard unregisters or cancels the
operation where the API exposes a guard.

## Rust SDK conventions

FUI-RS follows Rust conventions while keeping the UI model retained and explicit:

- Use `fui_app!` and `fui_managed_app!` so app authors do not write browser lifecycle exports manually.
- Use `ui!` for static mixed child trees when Rust's concrete collection types would otherwise require noisy conversions.
- Use `rich_text!` for typed fluent spans without manually constructing a span vector.
- Fluent borrowed expressions can be placed directly in `ui!`; they preserve the original retained identity.
- Use `fui_component!` instead of handwritten `Node`/`HasFlexBoxRoot` forwarding for retained wrappers. Declare `owner:` or `owners:` when weak callbacks or RAII guards depend on component-owned state.
- Use ordinary Rust closure capture for event handlers and callbacks.
- Keep RAII guards alive for subscriptions, pending file requests, timers, workers, and similar resources when the API returns one.
- Treat cloned controls as cheap handles to the same retained UI object, not as duplicated controls.
- Public editable-text positions are Unicode scalar-value character positions. Internally FUI-RS converts to UTF-8 byte offsets for the runtime boundary.
- Prefer `Text`, `Image`, and `Svg` in public app code; lower-level node names such as `TextNode`, `ImageNode`, and `SvgNode` are also available where surfaced.

## Layout and node helpers

Constructors and helpers:

- `flex_box()`, `row()`, `column()`
- `grid()`
- `text(...)`, `Text`, `TextNode`, `TextCore`
- `image(...)`, `Image`, `ImageNode`, `ImageSampling`
- `svg(...)`, `Svg`, `SvgNode`
- `portal()`
- `scroll_view()`, `scroll_box()`, `ScrollState`, `ScrollBar`, `ScrollBarVisibility`
- `virtual_list(total_items, item_height)`
- `custom_drawable(handler)`
- `px(value)`, `pct(value)`, `auto()`, `fill()`
- `viewport_width()`, `viewport_height()`
- `children![...]`, `ui! { ... }`, `rich_text![...]`
- `fui_component!(Type => root)` for stateless wrappers
- `fui_component!(Type => root, owner: state)` for one retained owner
- `fui_component!(Type => root, owners: [state, subscriptions])` for multiple retained owners

Common public style/layout types:

- `Length`
- `Border`
- `GradientStop`
- `Unit`
- `FlexDirection`
- `FlexWrap`
- `AlignItems`, `AlignSelf`, `JustifyContent`
- `PositionType`
- `Visibility`
- `BorderStyle`
- `CursorStyle`
- `Orientation`
- `ObjectFit`
- `TextAlign`, `TextVerticalAlign`, `TextOverflow`

## Controls

Constructors and types:

- `button(label)`, `Button`, `ClickEventArgs`: click callbacks, templates/colors, and typed `bind_theme`
- `checkbox(label)`, `Checkbox`, `CheckState`, `CheckboxChangedEventArgs`
- `switch(label)`, `Switch`, `SwitchChangedEventArgs`
- `radio_button(label)`, `RadioButton`, `RadioButtonChangedEventArgs`
- `radio_group()`, `RadioGroup`, `RadioGroupChangedEventArgs`
- `progress_bar()`, `ProgressBar`: `min`, `max`, `value`, `length`, `thickness`, `orientation`, `sizing`, `colors`
- `slider()`, `Slider`, `SliderChangedEventArgs`
- `dropdown()`, `Dropdown`, `DropdownItem`, `DropdownChangedEventArgs<T>`
- `combo_box()`, `ComboBox`, `ComboBoxItem`, `ComboBoxChangedEventArgs<T>`, `ComboBoxFilterMode`, `ComboBoxCommitMode`
- `text_input()`, `TextInput`
- `text_area()`, `TextArea`
- `form()`, `Form`
- `dialog(title, body)`, `Dialog`, `DialogShownEventArgs`
- `context_menu(items)`, `ContextMenu`, `MenuItem`, `ContextMenuAction`
- `popup()`, `Popup`
- `nav_link(href)`, `NavLink`, `NavigateEventArgs`
- `selection_area()`, `SelectionArea`
- `anti_selection_area()`, `AntiSelectionArea`
- `ToolTip`

## Control templates and style tokens

Public template/style types:

- `ControlTemplateSet`
- `use_control_templates(...)`, `get_control_templates()`, `clear_control_templates()`
- `PresenterHostStyle`, `SurfaceAppearance`, `OverlayBackdropAppearance`
- `ButtonTemplate`, `ButtonPresenter`, `ButtonVisualState`, `ButtonColors`
- `CheckboxIndicatorTemplate`, `CheckboxIndicatorPresenter`, `CheckboxIndicatorVisualState`
- `RadioIndicatorTemplate`, `RadioIndicatorPresenter`, `RadioIndicatorVisualState`
- `SwitchIndicatorTemplate`, `SwitchIndicatorPresenter`, `SwitchIndicatorVisualState`
- `SliderTemplate`, `SliderPresenter`, `SliderVisualState`, `SliderSizing`, `SliderColors`
- `DropdownFieldTemplate`, `DropdownChevronTemplate`, `DropdownOptionRowTemplate`
- `TextInputTemplate`, `TextInputPresenter`, `TextInputVisualState`, `TextInputColors`
- `LabeledControlColors`, `LabeledControlSizing`, `DropdownColors`, `DropdownSizing`
- `ProgressBarColors`, `ProgressBarSizing`
- `PopupAppearance`, `DialogAppearance`, `ContextMenuAppearance`, `ContextMenuItemAppearance`
- `LabeledControlTextStyle` for shared `font_family`, `font_size`, and `text_color`
- `DEFAULT_*_TEMPLATE` constants and `create_default_*_presenter(...)` helpers

See [Control customization and templating](./CONTROL_CUSTOMIZATION.md).

Configuration setters take direct values. Use explicit `clear_colors()`,
`clear_sizing()`, `clear_template()`, and `clear_appearance()` methods where
available; there are no compatibility `Option<T>` setters.

## Events

Event args and enums:

- `PointerEventArgs`, `PointerType`, `PointerEventType`
- `WheelEventArgs`
- `KeyEventArgs`, `KeyEventType`, `KeyModifier`
- `FocusChangedEventArgs`
- `TextChangedEventArgs`, `SelectionChangedEventArgs`
- `GestureEventArgs`, `GestureEventKind`, `GestureEventPhase`, `GestureIntent`
- `LongPressEventArgs`
- `ContextMenuEventArgs`
- drag/drop event args under `drag_drop` and `external_drop`

Event handlers generally receive typed EventArgs structs. Set `event.handled = true`
for routed mutable event args to stop bubbling and suppress framework defaults.

The universal Node click event is `on_pointer_click(...)`; it reports only raw
pointer input through mutable `PointerEventArgs`. Exact raw multi-click gestures
use `on_pointer_double_click(...)` and `on_pointer_triple_click(...)`.
`Button`, `Checkbox`, `RadioButton`, and `Switch` expose a separate count-free
`on_click(...)` semantic activation API covering supported pointer and keyboard
activation, while `NavLink` uses `on_navigate(...)`.

## Public capability traits

- `Node`: universal retained identity, state, semantics, focus, routed input,
  gestures, context menus, geometry, and child inspection/removal.
- `LayoutSurface`: retained sizing, fill, constraints, margin, and positioning.
- `BoxStyleSurface`: box appearance, clipping, gradients, blur, shadow, opacity,
  transitions, and padding.
- `FlexLayoutSurface`: flex direction, wrapping, basis, justification, and
  alignment.
- `ChildContainerSurface`: fluent retained child composition.
- `FlexBoxSurface`: the complete composite FlexBox capability bound.
- `TextSurface`: retained text layout, content, typography, selection, editing,
  and text events for `Text` and `RichText`.
- `TextEditorSurface`: shared editable-control contract for `TextInput` and
  `TextArea`.
- `ThemeBindable`: cycle-safe retained theme subscription for visual types.

See [Controls and nodes](./CONTROLS_AND_NODES.md#capability-traits) for the type
matrix and the deliberate `ScrollView` boundary.

## Theme

Theme APIs:

- The active theme defaults to the host system theme. The application shell and built-in controls track it automatically.
- Standard built-in visuals do not require an explicit `bind_theme(...)`; use it for custom surfaces and intentional overrides.

- `use_system_theme()`
- `use_custom_theme(theme)`
- `set_accent_color(color)`
- `current_theme()`
- `node.bind_theme(handler)` for node-owned, cycle-safe styling
- FlexBox-backed controls expose typed `bind_theme(handler)` callbacks that preserve their concrete APIs; this includes Button, NavLink, selection controls, range controls, dropdown/editable controls, and text editors
- `subscribe(handler)` for advanced non-node lifetimes where the caller retains the RAII guard
- `is_dark_mode()`
- `is_using_system_theme()`
- `default_light_theme()`, `default_dark_theme()`, `generate_theme(...)`
- `Theme`, `Colors`, `Spacing`, `Fonts`, `ContextMenuTheme`, `ContextMenuItemTheme`, `ToolTipTheme`

## Typography and text resources

Typography and text layout APIs:

- `FontFace`, `FontStack`, `FontFamily`
- `FontStyle`, `FontWeight`
- font loaded event args
- `RichText`, `RichTextSpan`, `span(...)`, `rich_text![...]`
- `TextLayout`, `DynamicTextLayout`, `DynamicTextOverflow`, `TextMetrics`

Font IDs are not the primary public authoring surface. Prefer `FontFace`,
`FontStack`, and `FontFamily`.

## Browser file bridge

Public file APIs:

- `File`
- `FileOpenRequest`, `FileOpenEventArgs`
- `FileSaveRequest`, `FileSaveResult`, `FileSaveMode`
- `FileRequestGuard`
- `BrowserFile`, `BrowserFileWriter`
- `FileReadChunk`, `FileWriteProgress`
- `FileCapabilities`, `FileErrorEventArgs`
- worker-assisted file processing types

Keep returned guards alive while a pending request is active. Dropping the guard
cancels/unregisters pending callbacks.

## Browser fetch bridge

Public fetch APIs:

- `Fetch`
- `FetchRequest`
- `FetchResponse`
- `FetchErrorEventArgs`

Example:

```rust
Fetch::get("/api/status")
    .on_complete(|response| logger::info("Fetch", response.status.to_string()))
    .on_error(|error| logger::warn("Fetch", &error.message))
    .start();
```

## Workers

Public worker APIs:

- `Worker`
- `WorkerProgressEventArgs`
- `WorkerCompletedEventArgs`
- `WorkerErrorEventArgs`
- `WorkerRuntime` and worker-job APIs under the `worker-runtime` feature

Example:

```rust
let worker = Worker::new("/workers/report.wasm", "run_report")
    .on_progress(|event| logger::info("Worker", &event.message))
    .on_complete(|event| logger::info("Worker", &event.result))
    .on_error(|event| logger::warn("Worker", &event.message))
    .start("input payload");
```

Keep the `Worker` value alive while work is running. Dropping it cancels the
worker and unregisters callbacks.

## Timers

- `set_timeout(delay_ms, callback) -> TimerHandle`
- `cancel_timeout(handle)`
- `TimerHandle`

Timer IDs are implementation details. Keep the handle if you may need to cancel.

## Platform and shortcuts

Platform APIs:

- `platform::device_pixel_ratio()`
- `platform::platform_family()`
- `platform::is_coarse_pointer()`
- `platform::primary_shortcut_modifier()`
- shortcut matching and formatting helpers such as `is_undo_shortcut(...)`, `is_redo_shortcut(...)`, `format_primary_shortcut_label(...)`
- `show_keyboard_focus_for_key_event(...)`

## Navigation

- `navigation::can_navigate_back()`
- `navigation::can_navigate_forward()`
- `navigation::navigate_back()`
- `navigation::navigate_forward()`
- `navigation::navigate_to(target, open_in_new_tab)`
- `current_route()`
- `NavLink`

## Persisted state

The `persisted` module exposes persisted state adapters/codecs used by controls
such as scroll views, text inputs, and selection surfaces. Use high-level control
APIs such as `node_id(...)`, `persist_scroll(...)`, and relevant control state
persistence methods before reaching for low-level adapters.

## Rust construction helpers

- `ui! { ... }` builds mixed retained child trees without `.into()` noise.
- `children![...]` creates child vectors for APIs that accept child lists.
- `rich_text![...]` creates `RichText` from fluent literal, dynamic, or prebuilt spans.
- `Configure::configure(...)` lets retained controls receive multiple `&self`
  setters while still returning the owned handle:

```rust
let input = text_input().configure(|input| {
    input.placeholder("Email").host_autofill("email").fill_width();
});
```

## Grid tracks

Use typed tracks instead of parallel values/type arrays:

```rust
grid()
    .columns([GridTrack::star(1.0), GridTrack::auto()])
    .rows([GridTrack::px(48.0), GridTrack::star(1.0)]);
```

## See also

- [SDK docs index](./SDK_INDEX.md)
- [Controls and nodes](./CONTROLS_AND_NODES.md)
- [Events and callbacks](./EVENTS_AND_CALLBACKS.md)
