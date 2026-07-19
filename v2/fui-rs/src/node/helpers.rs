use super::core::*;
use super::*;

pub type Length = (f32, Unit);

pub fn flex_box() -> FlexBox {
    FlexBox::default()
}

pub fn text(content: impl Into<String>) -> TextNode {
    TextNode::new(content)
}

pub fn grid() -> Grid {
    Grid::default()
}

pub fn image(texture_id: u32) -> ImageNode {
    ImageNode::new(texture_id)
}

pub fn svg(svg_id: u32) -> SvgNode {
    SvgNode::new(svg_id)
}

pub fn scroll_view() -> ScrollView {
    ScrollView::new()
}

pub fn scroll_box() -> ScrollBox {
    ScrollBox::new()
}

pub fn virtual_list(total_items: i32, item_height: f32) -> VirtualList<FlexBox> {
    VirtualList::new(total_items, item_height)
}

pub fn row() -> FlexBox {
    let root = FlexBox::default();
    root.flex_direction(FlexDirection::Row);
    root
}

pub fn column() -> FlexBox {
    let root = FlexBox::default();
    root.flex_direction(FlexDirection::Column);
    root
}

pub fn portal() -> Portal {
    let root = FlexBox::default();
    root.clip_to_bounds(false).portal(true);
    root
}

pub fn custom_drawable(handler: impl Fn(&mut DrawContext) + 'static) -> CustomDrawable {
    CustomDrawable::new(handler)
}

pub fn viewport_width() -> f32 {
    ui::get_viewport_width()
}

pub fn viewport_height() -> f32 {
    ui::get_viewport_height()
}

pub fn px(value: f32) -> Length {
    (value, Unit::Pixel)
}

pub fn pct(value: f32) -> Length {
    (value, Unit::Percent)
}

pub fn auto() -> Length {
    (0.0, Unit::Auto)
}

pub fn fill() -> Length {
    (100.0, Unit::Percent)
}

pub(crate) fn apply_flex_box_props(
    handle: NodeHandle,
    props: &FlexBoxProps,
    behavior: NodeBehavior,
) {
    ui::set_fill_width(handle.raw(), behavior.fill_width);
    ui::set_fill_height(handle.raw(), behavior.fill_height);
    if let Some(percent) = behavior.fill_width_percent {
        ui::set_fill_width_percent(handle.raw(), percent);
    }
    if let Some(percent) = behavior.fill_height_percent {
        ui::set_fill_height_percent(handle.raw(), percent);
    }
    if let Some((width, unit)) = props.width {
        ui::set_width(handle.raw(), width, unit as u32);
    }
    if let Some((height, unit)) = props.height {
        ui::set_height(handle.raw(), height, unit as u32);
    }
    if let Some((value, unit)) = props.min_width {
        ui::set_min_width(handle.raw(), value, unit as u32);
    }
    if let Some((value, unit)) = props.max_width {
        ui::set_max_width(handle.raw(), value, unit as u32);
    }
    if let Some((value, unit)) = props.min_height {
        ui::set_min_height(handle.raw(), value, unit as u32);
    }
    if let Some((value, unit)) = props.max_height {
        ui::set_max_height(handle.raw(), value, unit as u32);
    }
    if let Some(style) = props.box_style {
        ui::set_box_style(
            handle.raw(),
            props.bg_color.unwrap_or(0),
            style.radius_tl,
            style.radius_tr,
            style.radius_br,
            style.radius_bl,
            style.border_width,
            style.border_color,
            style.border_style as u32,
            style.border_dash_on,
            style.border_dash_off,
        );
    } else if let Some(color) = props.bg_color {
        ui::set_bg_color(handle.raw(), color);
    }
    if let Some((left, top, right, bottom)) = props.padding {
        ui::set_padding(handle.raw(), left, top, right, bottom);
    }
    if let Some(direction) = props.flex_direction {
        ui::set_flex_direction(handle.raw(), direction as u32);
    }
    if props.opacity.is_some() || props.blur_sigma.is_some() {
        ui::set_layer_effect(
            handle.raw(),
            props.opacity.unwrap_or(1.0),
            props.blur_sigma.unwrap_or(0.0),
            0,
        );
    }
    if let Some(shadow) = props.drop_shadow {
        ui::set_drop_shadow(
            handle.raw(),
            shadow.color,
            shadow.offset_x,
            shadow.offset_y,
            shadow.blur_sigma,
            shadow.spread,
        );
    }
    if let Some(sigma) = props.background_blur_sigma {
        ui::set_background_blur(handle.raw(), sigma);
    }
    if let Some(gradient) = props.linear_gradient.as_ref() {
        ui::set_linear_gradient(
            handle.raw(),
            gradient.sx,
            gradient.sy,
            gradient.ex,
            gradient.ey,
            &gradient.offsets,
            &gradient.colors,
        );
    }
    apply_behavior(handle, behavior);
}

pub(crate) fn apply_text_props(handle: NodeHandle, props: &TextProps, behavior: NodeBehavior) {
    ui::set_text(handle.raw(), &props.content);
    ui::set_fill_width(handle.raw(), behavior.fill_width);
    ui::set_fill_height(handle.raw(), behavior.fill_height);
    if let Some(percent) = behavior.fill_width_percent {
        ui::set_fill_width_percent(handle.raw(), percent);
    }
    if let Some(percent) = behavior.fill_height_percent {
        ui::set_fill_height_percent(handle.raw(), percent);
    }
    apply_behavior(handle, behavior);
    if let Some((width, unit)) = props.width {
        ui::set_width(handle.raw(), width, unit as u32);
    }
    if let Some((height, unit)) = props.height {
        ui::set_height(handle.raw(), height, unit as u32);
    }
    if let Some((value, unit)) = props.min_width {
        ui::set_min_width(handle.raw(), value, unit as u32);
    }
    if let Some((value, unit)) = props.max_width {
        ui::set_max_width(handle.raw(), value, unit as u32);
    }
    if let Some((value, unit)) = props.min_height {
        ui::set_min_height(handle.raw(), value, unit as u32);
    }
    if let Some((value, unit)) = props.max_height {
        ui::set_max_height(handle.raw(), value, unit as u32);
    }
    if props.has_font {
        ui::set_font(handle.raw(), props.font_id, props.font_size);
    }
    if props.has_style_runs {
        ui::set_text_style_runs(handle.raw(), &props.style_runs);
    }
    ui::set_text_color(
        handle.raw(),
        props
            .text_color
            .unwrap_or_else(|| crate::theme::current_theme().colors.text_primary),
    );
    if let Some(line_height) = props.line_height {
        ui::set_line_height(handle.raw(), line_height);
    }
    if let Some(text_align) = props.text_align {
        ui::set_text_align(handle.raw(), text_align as u32);
    }
    if let Some(text_vertical_align) = props.text_vertical_align {
        ui::set_text_vertical_align(handle.raw(), text_vertical_align as u32);
    }
    if let Some((max_chars, max_lines)) = props.text_limits {
        ui::set_text_limits(handle.raw(), max_chars, max_lines);
    }
    if let Some(wrapping) = props.wrapping {
        ui::set_text_wrapping(handle.raw(), wrapping);
    }
    if let Some(overflow) = props.overflow {
        ui::set_text_overflow(handle.raw(), overflow as u32);
    }
    if let Some((horizontal, vertical)) = props.overflow_fade {
        ui::set_text_overflow_fade(handle.raw(), horizontal, vertical);
    }
    if let Some((selectable, selection_color)) = props.selectable {
        ui::set_selectable(handle.raw(), selectable, selection_color);
    }
    if let Some(editable) = props.editable {
        ui::set_editable(handle.raw(), editable);
    }
    if let Some(enabled) = props.editor_command_keys {
        ui::set_editor_command_keys(handle.raw(), enabled);
    }
    if let Some(enabled) = props.editor_accepts_tab {
        ui::set_editor_accepts_tab(handle.raw(), enabled);
    }
    if let Some(obscured) = props.obscured {
        ui::set_text_obscured(handle.raw(), obscured);
    }
    if let Some(caret_color) = props.caret_color {
        ui::set_caret_color(handle.raw(), caret_color);
    }
    if let Some((start, end)) = props.selection_range_bytes {
        ui::set_text_selection_range(handle.raw(), start, end);
    }
}

pub(crate) fn apply_grid_props(handle: NodeHandle, props: &GridProps) {
    ui::grid_set_columns(handle.raw(), &props.columns, &props.column_types);
    ui::grid_set_rows(handle.raw(), &props.rows, &props.row_types);
    for (index, group) in &props.column_shared_size_groups {
        ui::grid_set_column_shared_size_group(handle.raw(), *index, group);
    }
    for (index, group) in &props.row_shared_size_groups {
        ui::grid_set_row_shared_size_group(handle.raw(), *index, group);
    }
}

pub(crate) fn apply_image_props(handle: NodeHandle, props: &ImageProps) {
    if let Some((left, top, right, bottom)) = props.image_nine {
        ui::set_image_nine(
            handle.raw(),
            props.texture_id,
            left,
            top,
            right,
            bottom,
            props.sampling_kind,
            props.max_aniso,
        );
    } else {
        ui::set_image(
            handle.raw(),
            props.texture_id,
            props.object_fit as u32,
            props.sampling_kind,
            props.max_aniso,
        );
    }
}

pub(crate) fn apply_svg_props(handle: NodeHandle, props: &SvgProps) {
    ui::set_svg(
        handle.raw(),
        props.svg_id,
        props.tint_color,
        props.sampling_kind,
        props.max_aniso,
    );
}

pub(crate) fn apply_scroll_view_props(
    handle: NodeHandle,
    props: &ScrollViewProps,
    behavior: NodeBehavior,
) {
    ui::set_fill_width(handle.raw(), behavior.fill_width);
    ui::set_fill_height(handle.raw(), behavior.fill_height);
    if let Some(percent) = behavior.fill_width_percent {
        ui::set_fill_width_percent(handle.raw(), percent);
    }
    if let Some(percent) = behavior.fill_height_percent {
        ui::set_fill_height_percent(handle.raw(), percent);
    }
    if let Some((width, unit)) = props.width {
        ui::set_width(handle.raw(), width, unit as u32);
    }
    if let Some((height, unit)) = props.height {
        ui::set_height(handle.raw(), height, unit as u32);
    }
    ui::set_scroll_enabled(handle.raw(), props.enable_scroll_x, props.enable_scroll_y);
    ui::set_smooth_scrolling(handle.raw(), props.smooth_scrolling);
    if let Some(friction) = props.friction {
        ui::set_scroll_friction(handle.raw(), friction);
    }
    if let Some((offset_x, offset_y)) = props.scroll_offset {
        ui::set_scroll_offset(handle.raw(), offset_x, offset_y);
    }
    if let Some((content_width, content_height)) = props.content_size {
        ui::set_scroll_content_size(handle.raw(), content_width, content_height);
    }
    apply_behavior(handle, behavior);
}

pub(crate) fn apply_behavior(handle: NodeHandle, behavior: NodeBehavior) {
    let effective_enabled = behavior.enabled && behavior.inherited_enabled;
    if let Some(node_id) = behavior.node_id.as_deref() {
        ui::set_node_id(handle.raw(), node_id);
    }
    if let Some(role) = behavior.semantic_role {
        ui::set_semantic_role(handle.raw(), role as u32);
    }
    if let Some(label) = behavior
        .semantic_label
        .as_deref()
        .or(behavior.default_semantic_label.as_deref())
    {
        ui::set_semantic_label(handle.raw(), label);
    }
    if let Some(disabled) = behavior.semantic_disabled {
        ui::set_semantic_disabled(handle.raw(), true, disabled);
    }
    if let Some(state) = behavior.semantic_checked {
        ui::set_semantic_checked(handle.raw(), state as u32);
    }
    if let Some(selected) = behavior.semantic_selected {
        ui::set_semantic_selected(handle.raw(), true, selected);
    }
    if let Some(expanded) = behavior.semantic_expanded {
        ui::set_semantic_expanded(handle.raw(), true, expanded);
    }
    if let Some((value_now, value_min, value_max)) = behavior.semantic_value_range {
        ui::set_semantic_value_range(handle.raw(), true, value_now, value_min, value_max);
    }
    if let Some(orientation) = behavior.semantic_orientation {
        ui::set_semantic_orientation(handle.raw(), orientation as u32);
    }
    if let Some(visibility) = behavior.visibility {
        ui::set_visibility(handle.raw(), visibility as u32);
    }
    ui::set_is_portal(handle.raw(), behavior.is_portal);
    if let Some((value, unit)) = behavior.min_width {
        ui::set_min_width(handle.raw(), value, unit as u32);
    }
    if let Some((value, unit)) = behavior.max_width {
        ui::set_max_width(handle.raw(), value, unit as u32);
    }
    if let Some((value, unit)) = behavior.min_height {
        ui::set_min_height(handle.raw(), value, unit as u32);
    }
    if let Some((value, unit)) = behavior.max_height {
        ui::set_max_height(handle.raw(), value, unit as u32);
    }
    if let Some(basis) = behavior.flex_basis {
        ui::set_flex_basis(handle.raw(), basis);
    }
    if let Some(justify) = behavior.justify_content {
        ui::set_justify_content(handle.raw(), justify as u32);
    }
    if let Some(align) = behavior.align_items {
        ui::set_align_items(handle.raw(), align as u32);
    }
    if let Some(align) = behavior.align_self {
        ui::set_align_self(handle.raw(), align as u32);
    }
    if let Some((left, top, right, bottom)) = behavior.margin {
        ui::set_margin(handle.raw(), left, top, right, bottom);
    }
    if let Some(position_type) = behavior.position_type {
        ui::set_position_type(handle.raw(), position_type as u32);
    }
    if let Some((left, top, right, bottom)) = behavior.position {
        ui::set_position(handle.raw(), left, top, right, bottom);
    }
    ui::set_is_shared_size_scope(handle.raw(), behavior.is_shared_size_scope);
    ui::set_custom_drawable(handle.raw(), behavior.custom_drawable);
    if let Some(wrap) = behavior.flex_wrap {
        ui::set_flex_wrap(handle.raw(), wrap as u32);
    }
    if let Some(clip) = behavior.clip_to_bounds {
        ui::set_clip_to_bounds(handle.raw(), clip);
    }
    if behavior.selection_area {
        ui::set_selection_area(handle.raw(), true);
    }
    if behavior.selection_area_barrier {
        ui::set_selection_area_barrier(handle.raw(), true);
    }
    if behavior.preserve_selection_on_pointer_down {
        ui::set_preserve_selection_on_pointer_down(handle.raw(), true);
    }
    if let Some(scroll_handle) = behavior.scroll_proxy_target {
        ui::set_scroll_proxy_target(handle.raw(), scroll_handle);
    }
    ui::set_interactive(handle.raw(), effective_enabled && behavior.interactive);
    if let Some((enabled, tab_index)) = behavior.focusable {
        ui::set_focusable(handle.raw(), effective_enabled && enabled, tab_index);
    }
    if behavior.track_semantic_disabled_from_enabled {
        ui::set_semantic_disabled(handle.raw(), true, !effective_enabled);
    }
}
