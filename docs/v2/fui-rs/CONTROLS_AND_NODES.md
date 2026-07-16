# FUI-RS Controls and Nodes (v2)

This page is the practical guide to the retained UI building blocks exported by
`fui::prelude::*`.

For the complete export list, see:

- [API reference](./API_REFERENCE.md)
- [SDK docs index](./SDK_INDEX.md)
- [Per-type reference](./reference/README.md)
- [Forms and autofill guide](./FORMS_AND_AUTOFILL.md)

## Controls

| Control | Purpose | Key APIs |
|---|---|---|
| `Button` | Theme-aware action control | `button(label)`, `on_click(...)`, `on_double_click(...)`, `on_triple_click(...)`, `template(...)`, `colors(...)`, typed `bind_theme(...)` |
| `Checkbox` | Boolean or tri-state check control | `checkbox(label)`, `check(...)`, `tri_state(...)`, `mixed(...)`, `on_changed(...)`, `template(...)`, `sizing(...)`, `colors(...)` |
| `Switch` | On/off toggle control | `switch(label)`, `check(...)`, `on_changed(...)`, `template(...)`, `sizing(...)`, `colors(...)` |
| `RadioButton` / `RadioGroup` | Single-choice grouped options | `radio_button(label)`, `radio_group()`, `add_option(...)`, `add_options(...)`, `select_index(...)`, `on_changed(...)` |
| `ProgressBar` | Determinate horizontal or vertical progress visualization | `value(...)`, `min(...)`, `max(...)`, `length(...)`, `thickness(...)`, `orientation(...)`, `sizing(...)`, `clear_sizing()`, `colors(...)`, `clear_colors()` |
| `Slider` | Single-value range control | `min(...)`, `max(...)`, `step(...)`, `orientation(...)`, `on_changed(...)`, `template(...)`, `sizing(...)`, `colors(...)` |
| `Dropdown` | Non-editable selection popup control | `items(...)`, `select_index(...)`, `on_changed(...)`, `max_visible_items(...)`, templates and colors |
| `ComboBox` | Editable filter/selection control | `items(...)`, `filter_mode(...)`, `commit_mode(...)`, `auto_complete(...)`, `on_changed(...)` |
| `TextInput` | Single-line editable text | `text(...)`, `placeholder(...)`, `read_only(...)`, `password(...)`, `host_autofill(...)`, `on_changed(...)` |
| `TextArea` | Multiline editable text | `text(...)`, `wrapping(...)`, scrollbar visibility, `accepts_tab(...)`, `on_changed(...)` |
| `Form` | Default/cancel action and autofill grouping host | `form()`, `default_action(...)`, `cancel_action(...)`, child fields |
| `Dialog` | Modal overlay with actions | `dialog(title, body)`, `show()`, `hide()`, `on_accept(...)`, `on_cancel(...)`, `appearance(...)`, `clear_appearance()` |
| `ContextMenu` / `MenuItem` | Retained context menu surface | `context_menu(items)`, `ContextMenu::new()`, `show(...)`, `hide()`, `appearance(...)`, `clear_appearance()` |

`Button`, `NavLink`, `Checkbox`, `RadioButton`, and `Switch` share
`LabeledControlTextStyle`, providing `font_family(...)`, `font_size(...)`, and
`text_color(...)` with explicit overrides that survive theme updates.

`Grid` uses typed `GridTrack` values: `GridTrack::px(...)`,
`GridTrack::star(...)`, and `GridTrack::auto()`.
| `Popup` | Generic popup overlay | `popup()`, child overlay content, show/hide behavior, `appearance(...)`, `clear_appearance()` |
| `NavLink` | Route/link control | `nav_link(href)`, `href_to(...)`, `on_navigate(...)` |
| `SelectionArea` | Cross-node text selection host | `selection_area()`, selected text callbacks and mobile selection affordances |
| `AntiSelectionArea` | Selection barrier island | `anti_selection_area()` prevents ancestor selection collection |
| `ToolTip` | Hover/focus tooltip behavior | attach via node tooltip APIs |

## Nodes

| Node | Purpose | Key APIs |
|---|---|---|
| `FlexBox` | Base retained layout node | flex layout, sizing, padding, margin, border, radius, gradient, blur, shadow, child composition |
| `Grid` | WPF-style retained grid layout | rows, columns, placements, shared-size scope/group helpers |
| `Text` / `TextNode` | Retained text rendering node | `text(...)`, font family/size/weight/style, text color, selection, alignment |
| `RichText` / `RichTextSpan` | Attributed inline rich text | `span(...)`, inline color/background/bold/italic styling |
| `Bitmap` | Retained pixel buffer backed by GPU texture | direct pixel access and retained bitmap text rendering |
| `TextLayout` | Immediate-mode formatted text resource | `TextLayout::text(...)`, `TextLayout::rich(...)`, drawing via `DrawContext` |
| `DynamicTextLayout` | Immediate-mode short label resource | fixed charset text layout for frequently changing labels |
| `Image` / `ImageNode` | Retained raster image node | URL/asset-backed images, object-fit, sampling, nine-patch behavior |
| `Svg` / `SvgNode` | Retained SVG node | URL/asset-backed SVG rendering, tinting, sampling |
| `Portal` | Overlay host node | detached overlay composition surfaces |
| `ScrollView` | Low-level retained viewport | scroll state/offset plumbing, animated scroll, transitions |
| `ScrollState` | Shared scroll metrics/state object | offsets, viewport size, content size |
| `ScrollBar` | Retained scrollbar chrome | axis-aware track/thumb style and geometry |
| `ScrollBox` | High-level scroll container | owned viewport and scrollbars, per-axis enable/visibility control |
| `VirtualList` | Pooled retained list surface | `virtual_list(total, item_height)`, `on_bind_item(...)`, recycled rows |
| `CustomDrawable` | Retained custom drawing surface | `custom_drawable(|ctx| ...)`, `DrawContext`, `mark_dirty()` |
| `GradientStop` | Linear gradient stop value | `GradientStop::new(offset, color)` |

## Helpers

- `row()`
- `column()`
- `flex_box()`
- `grid()`
- `text("...")`
- `px(...)`
- `pct(...)`
- `auto()`
- `fill()`
- `children![...]`
- `ui! { ... }`

## Core layout concept

This is the most important sizing distinction in the SDK:

- `width(100.0, Unit::Percent)` means: make my box 100% of the parent.
- `fill_width()` means: take the available inner width the parent layout offers.
- `fill_width_percent(50.0)` means: take 50% of the available inner width.

The same rule applies vertically:

- `height(100.0, Unit::Percent)` means: make my box 100% of the parent.
- `fill_height()` means: take the available inner height the parent layout offers.
- `fill_height_percent(50.0)` means: take 50% of the available inner height.

For normal stretch/fill layouts, prefer `fill_width()`, `fill_height()`, and
`fill_size()`. Use percent sizing when the size itself should be a literal ratio
of the parent.

## Layout sizing guide: fill vs percent

```rust
use fui::prelude::*;

let page = ui! {
    row().fill_size().gap(12.0) {
        column().fill_width_percent(35.0).fill_height() {
            text("Sidebar"),
        },
        column().fill_width().fill_height() {
            text("Main content"),
        },
    }
};
```

Use `align_items(...)` on the parent for cross-axis child alignment. Use
`align_self(...)` when one child should override the parent alignment policy.

`Unit::Auto` means intrinsic content size. Do not combine `fill_width()` with
`width(..., Unit::Auto)` on the same axis; those are contradictory layout asks.

## Child composition

Use `.child(&node)` for retained nodes you need to store and mutate later:

```rust
let status = text("Idle");
let root = column();
root.child(&status);
status.text("Ready");
```

Use `ui!` for static mixed child trees:

```rust
let root = ui! {
    column().gap(8.0) {
        text("Settings"),
        checkbox("Enable sync"),
        button("Save"),
    }
};
```

Fluent setters return borrowed handles, and `ui!` accepts those borrowed
expressions directly without cloning or rebuilding the retained node:

```rust
let root = ui! {
    column() {
        button("Save").margin(0.0, 8.0, 0.0, 0.0),
        text("Not selectable")
            .selectable(false)
            .selection_color(0x3B82F680),
    }
};
```

Use `fui_component!` for a retained wrapper with a designated layout root:

```rust
#[derive(Clone)]
struct SettingsHeader {
    root: FlexBox,
}

fui_component!(SettingsHeader => root);
```

The macro delegates `Node` and `HasFlexBoxRoot`; it does not create another UI
node. Keep an intermediate variable when callbacks or later mutations need the
control. Do not use `Deref` to imitate UI inheritance.

## Common node state APIs

Most retained nodes and controls expose common behavior through the `Node` trait
or through their retained root:

- `node_id(...)`
- `enabled(...)`
- `focusable(...)`
- `interactive(...)`
- `visibility(...)`
- `cursor(...)`
- `semantic_role(...)`
- `semantic_label(...)`
- `on_pointer_down(...)`, `on_pointer_up(...)`, `on_pointer_move(...)`
- `on_key_down(...)`, `on_key_up(...)`
- `on_focus_changed(...)`
- `on_wheel(...)`
- gesture callbacks
- context-menu callbacks
- geometry helpers

## See also

- [SDK docs index](./SDK_INDEX.md)
- [API reference](./API_REFERENCE.md)
- [Events and callbacks](./EVENTS_AND_CALLBACKS.md)
- [Theming and style matrix](./THEMING_STYLE_MATRIX.md)
