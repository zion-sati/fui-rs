# FUI-RS Theming and Style Matrix (v2)

This page documents theme defaults and explicit style override behavior.

## Theme APIs

- `use_system_theme()` follows host dark/light mode and accent color where available.
- `use_custom_theme(theme)` applies an explicit `Theme`.
- `set_accent_color(color)` rebuilds the active theme with a custom accent.
- `current_theme()` returns the current effective theme.
- `node.bind_theme(|node, theme| ...)` stores a cycle-safe subscription on the retained node.
- `subscribe(handler)` returns a RAII subscription guard.

Core theme structs:

- `Theme`
- `Colors`
- `Spacing`
- `Fonts`
- `ContextMenuTheme`
- `ContextMenuItemTheme`
- `ToolTipTheme`

## Precedence rules

1. Explicit per-control style overrides win over theme defaults.
2. Theme updates re-apply non-overridden fields.
3. Controls with no explicit override for a field remain theme-driven for that field.

## Control style matrix

| Control | Theme-driven defaults | Explicit override examples |
|---|---|---|
| `Button` | accent/background/hover/pressed/border/radius/font/text | `colors(...)`, `template(...)`, `font_family(...)`, `font_size(...)`, `text_color(...)` |
| `Checkbox` / `Switch` / `RadioButton` | indicator/control colors and focus chrome | `colors(...)`, `sizing(...)`, `template(...)`, shared labeled-text styling |
| `Slider` | track/thumb/focus/value colors | `colors(...)`, `sizing(...)`, `template(...)` |
| `Dropdown` | trigger surface, popup border/shadow, option rows | `colors(...)`, `sizing(...)`, field/chevron/row templates |
| `ComboBox` | text editor surface plus popup chrome | text input colors/templates plus popup settings |
| `TextInput` / `TextArea` | surface, border, text, placeholder, caret, disabled opacity | `colors(...)`, `template(...)`, `font_family(...)`, `font_size(...)` |
| `ContextMenu` | panel/item/separator/shadow/theme metrics | item and panel styling APIs |
| `Dialog` | backdrop, card surface, border, radius, shadow, text styles | backdrop/card/action styling APIs |
| `ScrollBar` | track/thumb colors | track/thumb colors and geometry APIs |
| `NavLink` | link cursor, focus chrome, inherited text/box style | `font_family(...)`, `font_size(...)`, `text_color(...)`, inherited box styling |

## Node style matrix

| Node | Theme defaults | Explicit styling |
|---|---|---|
| `FlexBox`, `Grid`, `Portal` | none by default | background, border, radius, gradient, blur, shadow, opacity |
| `Text` | default theme typography and selection color | font, color, alignment, `selectable(...)`, `selection_color(...)` |
| `ScrollView`, `ScrollBox`, `VirtualList` | scrollbar chrome through `ScrollBar` | scrollbars and child surface styling |
| `Image`, `Svg` | none by default | tint, object fit, sampling, box styling |

## Example

```rust
let card = column();
card.corner_radius(14.0)
    .padding(16.0, 16.0, 16.0, 16.0)
    .bind_theme(|card, theme| {
        card.bg_color(theme.colors.surface)
            .border(1.0, theme.colors.border);
    });
```

The retained node owns the RAII guard and the signal callback holds only a weak
target. A parent-owned child therefore remains themed even if its configuring
wrapper is dropped, and dropping the retained node unsubscribes automatically.
Do not capture the themed node strongly inside its own callback; use the node
argument supplied to the callback.

## See also

- [Control customization and templating](./CONTROL_CUSTOMIZATION.md)
- [Controls and nodes](./CONTROLS_AND_NODES.md)
- [API reference](./API_REFERENCE.md)
