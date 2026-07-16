# FUI-RS Control Customization and Templating (v2)

FUI-RS controls expose two customization levels:

1. Style token APIs for common visual changes.
2. Template/presenter APIs for full retained visual replacement.

Prefer style tokens first. Use templates when the control's retained visual tree
needs to change.

## Style token examples

```rust
let primary = button("Save");
primary.colors(
    ButtonColors::new()
        .background(0x2563EBFF)
        .background_hover(0x1D4ED8FF)
        .background_pressed(0x1E40AFFF)
        .text_primary(0xFFFFFFFF)
        .border(0x1D4ED8FF),
);
```

Labeled controls share sizing and color token concepts:

- `LabeledControlColors`
- `LabeledControlSizing`
- `SliderColors`
- `SliderSizing`
- `DropdownColors`
- `DropdownSizing`
- `TextInputColors`
- `ProgressBarColors`
- `ProgressBarSizing`

Configuration setters take direct values. Clear an override explicitly with its
`clear_*` method, for example `button.clear_colors()`,
`text_input.clear_template()`, or `dialog.clear_appearance()`. FUI-RS does not
use public `Option<T>` setters for this surface.

For theme-responsive Button overrides, use the typed control binding so the
callback retains the Button surface:

```rust
button("Save").bind_theme(|button, theme| {
    button.colors(
        ButtonColors::new()
            .background(theme.colors.accent)
            .text_primary(theme.colors.text_on_accent),
    );
});
```

Typed `bind_theme(...)` is also available on every public FlexBox-backed
control: `Button`, `NavLink`, `Checkbox`, `RadioButton`, `Switch`, `Slider`,
`ProgressBar`, `Dropdown`, `ComboBox`, `TextInput`, `TextArea`, `SelectionArea`,
and `AntiSelectionArea`. Each callback receives that concrete control type, so
control-specific APIs remain available without calling `flex_box_root()`.
The same typed contract applies to FlexBox-backed retained nodes such as
`ScrollBox` and `VirtualList`; `FlexBox` and `TextNode` also implement the
shared `ThemeBindable` trait directly.

The retained control owns these subscriptions and callback targets are weak,
avoiding `Rc` self-cycles.

## Host style precedence

Controls resolve two host-style layers independently:

1. Presenter style provides the control or template default.
2. Local node style, such as `background(...)`, `border_config(...)`,
   `corners(...)`, `padding(...)`, `shadow(...)`, and `opacity(...)`, wins per
   property.

Clearing a local property reveals the current presenter value. Theme changes,
hover/pressed transitions, and template replacement therefore cannot overwrite
an explicit local host property.

Presenter implementations return `PresenterHostStyle`; they do not mutate the
control host directly:

```rust
fn present(
    &self,
    theme: Theme,
    state: ButtonVisualState,
    _colors: Option<ButtonColors>,
) -> PresenterHostStyle {
    self.label.text_color(theme.colors.text_primary);
    PresenterHostStyle::new()
        .background(theme.colors.accent)
        .border(Border::solid(1.0, theme.colors.border))
        .corners(Corners::all(if state.pressed { 6.0 } else { 8.0 }))
}
```

## Overlay appearance recipes

Use one recipe per overlay instead of setting unrelated visual fields:

```rust
dialog("Sign in", "Enter your credentials").appearance(
    DialogAppearance::new()
        .backdrop(OverlayBackdropAppearance::new().background(0x00000080))
        .card(
            SurfaceAppearance::new()
                .background(0xFFFFFFFF)
                .border(Border::solid(1.0, 0xCBD5E1FF))
                .corners(Corners::all(16.0)),
        ),
);
```

`PopupAppearance`, `DialogAppearance`, and `ContextMenuAppearance` preserve
theme defaults for omitted fields. `clear_appearance()` restores the complete
live theme recipe atomically.

## Templates

Templates create presenter instances for controls. Presenters own the retained
visual subtree for that template.

Public template families:

- `ButtonTemplate` / `ButtonPresenter`
- `CheckboxIndicatorTemplate` / `CheckboxIndicatorPresenter`
- `RadioIndicatorTemplate` / `RadioIndicatorPresenter`
- `SwitchIndicatorTemplate` / `SwitchIndicatorPresenter`
- `SliderTemplate` / `SliderPresenter`
- `DropdownFieldTemplate` / `DropdownFieldPresenter`
- `DropdownChevronTemplate` / `DropdownChevronPresenter`
- `DropdownOptionRowTemplate` / `DropdownOptionRowPresenter`
- `TextInputTemplate` / `TextInputPresenter`

Default template constants and presenter helpers are exported for composition:

- `DEFAULT_BUTTON_TEMPLATE`
- `DEFAULT_CHECKBOX_INDICATOR_TEMPLATE`
- `DEFAULT_RADIO_INDICATOR_TEMPLATE`
- `DEFAULT_SWITCH_INDICATOR_TEMPLATE`
- `DEFAULT_SLIDER_TEMPLATE`
- `DEFAULT_DROPDOWN_FIELD_TEMPLATE`
- `DEFAULT_DROPDOWN_CHEVRON_TEMPLATE`
- `DEFAULT_DROPDOWN_OPTION_ROW_TEMPLATE`
- `DEFAULT_TEXT_INPUT_TEMPLATE`
- `create_default_*_presenter(...)`

## Global control template set

Use `ControlTemplateSet` to replace defaults app-wide:

```rust
let templates = ControlTemplateSet::new()
    .button(DEFAULT_BUTTON_TEMPLATE.clone())
    .text_input(DEFAULT_TEXT_INPUT_TEMPLATE.clone());

use_control_templates(templates);
```

Use `clear_control_templates()` to return to defaults.

## Presenter boundaries

A presenter owns visual composition and returns host defaults, not interaction
semantics. Built-in controls own enabled state, focus state, keyboard behavior,
events, and semantic state. Custom presenters should render the visual state
they are given and avoid taking over control logic unless the template contract
explicitly asks for it.

## Retained lifecycle guidance

Templates and presenters are retained objects. Node-owned theme subscriptions
are retained for the node lifetime and unsubscribe on unmount. Do not create
`Rc` self-cycles: subscription closures should capture weak targets or only the
retained fields they need. Use RAII guards for subscriber-owned non-node
subscriptions.

## See also

- [Theming and style matrix](./THEMING_STYLE_MATRIX.md)
- [Controls and nodes](./CONTROLS_AND_NODES.md)
- [API reference](./API_REFERENCE.md)
