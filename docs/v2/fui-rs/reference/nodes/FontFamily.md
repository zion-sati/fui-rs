# FontFamily

Maps requested weight and style to retained `FontStack` resources.

Use `with_regular_stack(...)` or `with_regular_face(...)` for a single-face
family, `regular_bold_stacks(...)` for the common regular/bold pair, or
`FontFamily::new(...)` when supplying the complete weight/style map. Missing
variants resolve to the nearest available stack.

Apply a family when text should respond to `font_weight(...)` and
`font_style(...)`. Apply a `FontStack` directly when no family mapping is
required.

See [Custom fonts](../../CUSTOM_FONTS.md).
