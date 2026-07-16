use super::core::*;
use super::*;
use crate::animation::{animate_color, animate_float, Animation};
use crate::transitions::NodeTransitions;

#[derive(Default)]
pub(crate) struct FlexBoxAnimations {
    pub(crate) opacity: Option<Animation>,
    pub(crate) background_color: Option<Animation>,
}

#[derive(Clone)]
pub struct FlexBox {
    pub(crate) core: Rc<RefCell<NodeCore>>,
    pub(crate) props: Rc<RefCell<FlexBoxProps>>,
    pub(crate) active_animations: Rc<RefCell<FlexBoxAnimations>>,
    pub(crate) host_style_layers: Rc<RefCell<HostStyleLayers>>,
}

impl Default for FlexBox {
    fn default() -> Self {
        let mut core = NodeCore::new(NodeKind::FlexBox);
        core.behavior.clip_to_bounds = Some(true);
        Self {
            core: Rc::new(RefCell::new(core)),
            props: Rc::new(RefCell::new(FlexBoxProps::default())),
            active_animations: Rc::new(RefCell::new(FlexBoxAnimations::default())),
            host_style_layers: Rc::new(RefCell::new(HostStyleLayers::default())),
        }
    }
}

impl Node for FlexBox {
    fn retained_node_ref(&self) -> NodeRef {
        NodeRef::from_node(self.core.clone(), self.clone())
    }

    fn build_self(&self) {
        let props = self.props.borrow().clone();
        let behavior = self.core.borrow().behavior.clone();
        apply_flex_box_props(self.handle(), &props, behavior);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Border {
    pub width: f32,
    pub color: u32,
    pub style: BorderStyle,
    pub dash_on: f32,
    pub dash_off: f32,
}

impl Border {
    pub fn solid(width: f32, color: u32) -> Self {
        Self {
            width,
            color,
            style: BorderStyle::Solid,
            dash_on: 0.0,
            dash_off: 0.0,
        }
    }

    pub fn dashed(width: f32, color: u32, dash_on: f32, dash_off: f32) -> Self {
        Self {
            width,
            color,
            style: BorderStyle::Dashed,
            dash_on,
            dash_off,
        }
    }

    pub fn dotted(width: f32, color: u32, dash_on: f32, dash_off: f32) -> Self {
        Self {
            width,
            color,
            style: BorderStyle::Dotted,
            dash_on,
            dash_off,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GradientStop {
    pub offset: f32,
    pub color: u32,
}

impl GradientStop {
    pub fn new(offset: f32, color: u32) -> Self {
        Self { offset, color }
    }
}

impl FlexBox {
    pub(crate) fn downgrade(&self) -> WeakFlexBox {
        WeakFlexBox {
            core: Rc::downgrade(&self.core),
            props: Rc::downgrade(&self.props),
            active_animations: Rc::downgrade(&self.active_animations),
            host_style_layers: Rc::downgrade(&self.host_style_layers),
        }
    }

    pub fn key(&self, _key: u64) -> &Self {
        self
    }

    pub fn width(&self, width: f32, unit: Unit) -> &Self {
        self.props.borrow_mut().width = Some((width, unit));
        {
            let mut core = self.core.borrow_mut();
            core.behavior.fill_width = false;
            core.behavior.fill_width_percent = None;
        }
        if self.has_built_handle() {
            ui::set_width(self.handle().raw(), width, unit as u32);
            self.notify_retained_layout_mutation();
        }
        self
    }

    pub fn width_len(&self, length: Length) -> &Self {
        let (width, unit) = length;
        self.width(width, unit)
    }

    pub fn height(&self, height: f32, unit: Unit) -> &Self {
        self.props.borrow_mut().height = Some((height, unit));
        {
            let mut core = self.core.borrow_mut();
            core.behavior.fill_height = false;
            core.behavior.fill_height_percent = None;
        }
        if self.has_built_handle() {
            ui::set_height(self.handle().raw(), height, unit as u32);
            self.notify_retained_layout_mutation();
        }
        self
    }

    pub fn height_len(&self, length: Length) -> &Self {
        let (height, unit) = length;
        self.height(height, unit)
    }

    pub fn fill_width(&self) -> &Self {
        self.props.borrow_mut().width = None;
        {
            let mut core = self.core.borrow_mut();
            core.behavior.fill_width = true;
            core.behavior.fill_width_percent = None;
        }
        if self.has_built_handle() {
            ui::set_fill_width(self.handle().raw(), true);
            self.notify_retained_layout_mutation();
        }
        self
    }

    pub fn fill_height(&self) -> &Self {
        self.props.borrow_mut().height = None;
        {
            let mut core = self.core.borrow_mut();
            core.behavior.fill_height = true;
            core.behavior.fill_height_percent = None;
        }
        if self.has_built_handle() {
            ui::set_fill_height(self.handle().raw(), true);
            self.notify_retained_layout_mutation();
        }
        self
    }

    pub fn fill_size(&self) -> &Self {
        self.fill_width();
        self.fill_height();
        self
    }

    pub fn fill_width_percent(&self, percent: f32) -> &Self {
        self.props.borrow_mut().width = None;
        {
            let mut core = self.core.borrow_mut();
            core.behavior.fill_width = false;
            core.behavior.fill_width_percent = Some(percent);
        }
        if self.has_built_handle() {
            ui::set_fill_width_percent(self.handle().raw(), percent);
            self.notify_retained_layout_mutation();
        }
        self
    }

    pub fn fill_height_percent(&self, percent: f32) -> &Self {
        self.props.borrow_mut().height = None;
        {
            let mut core = self.core.borrow_mut();
            core.behavior.fill_height = false;
            core.behavior.fill_height_percent = Some(percent);
        }
        if self.has_built_handle() {
            ui::set_fill_height_percent(self.handle().raw(), percent);
            self.notify_retained_layout_mutation();
        }
        self
    }

    pub fn bg_color(&self, color: u32) -> &Self {
        self.host_style_layers.borrow_mut().local.background = Some(color);
        self.set_effective_background_color(color)
    }

    fn set_effective_background_color(&self, color: u32) -> &Self {
        self.cancel_background_color_transition();
        if self.should_animate_background_color(color) {
            let timing = {
                self.props
                    .borrow()
                    .transitions
                    .as_ref()
                    .and_then(NodeTransitions::background_color_timing)
            };
            if let Some(timing) = timing {
                let weak = self.downgrade();
                let from = self.current_background_color();
                let animation = animate_color(from, color, timing, move |value| {
                    if let Some(node) = weak.upgrade() {
                        node.apply_animated_background_color(value);
                    }
                });
                self.active_animations.borrow_mut().background_color = Some(animation);
                return self;
            }
        }
        self.apply_animated_background_color(color);
        self
    }

    pub fn padding(&self, left: f32, top: f32, right: f32, bottom: f32) -> &Self {
        self.host_style_layers.borrow_mut().local.padding =
            Some(EdgeInsets::new(left, top, right, bottom));
        self.props.borrow_mut().padding = Some((left, top, right, bottom));
        if self.has_built_handle() {
            ui::set_padding(self.handle().raw(), left, top, right, bottom);
            self.notify_retained_layout_mutation();
        }
        self
    }

    pub fn flex_direction(&self, direction: FlexDirection) -> &Self {
        self.host_style_layers.borrow_mut().local.flex_direction = Some(direction);
        self.props.borrow_mut().flex_direction = Some(direction);
        if self.has_built_handle() {
            ui::set_flex_direction(self.handle().raw(), direction as u32);
            self.notify_retained_layout_mutation();
        }
        self
    }

    pub fn corner_radius(&self, radius: f32) -> &Self {
        self.corners(radius, radius, radius, radius)
    }

    pub fn corners(&self, tl: f32, tr: f32, br: f32, bl: f32) -> &Self {
        self.host_style_layers.borrow_mut().local.corners = Some(Corners::new(tl, tr, br, bl));
        let existing = self.props.borrow().box_style.unwrap_or(BoxStyle {
            radius_tl: 0.0,
            radius_tr: 0.0,
            radius_br: 0.0,
            radius_bl: 0.0,
            border_width: 0.0,
            border_color: 0,
            border_style: BorderStyle::Solid,
            border_dash_on: 0.0,
            border_dash_off: 0.0,
        });
        self.props.borrow_mut().box_style = Some(BoxStyle {
            radius_tl: tl,
            radius_tr: tr,
            radius_br: br,
            radius_bl: bl,
            ..existing
        });
        if self.has_built_handle() {
            self.build_self();
            self.notify_retained_mutation();
        }
        self
    }

    pub fn border(&self, width: f32, color: u32) -> &Self {
        self.border_config(Border::solid(width, color))
    }

    pub fn border_config(&self, border: Border) -> &Self {
        self.host_style_layers.borrow_mut().local.border = Some(border);
        let existing = self.props.borrow().box_style.unwrap_or(BoxStyle {
            radius_tl: 0.0,
            radius_tr: 0.0,
            radius_br: 0.0,
            radius_bl: 0.0,
            border_width: 0.0,
            border_color: 0,
            border_style: BorderStyle::Solid,
            border_dash_on: 0.0,
            border_dash_off: 0.0,
        });
        self.props.borrow_mut().box_style = Some(BoxStyle {
            border_width: border.width,
            border_color: border.color,
            border_style: border.style,
            border_dash_on: border.dash_on,
            border_dash_off: border.dash_off,
            ..existing
        });
        if self.has_built_handle() {
            self.build_self();
            self.notify_retained_mutation();
        }
        self
    }

    pub fn opacity(&self, value: f32) -> &Self {
        let value = value.clamp(0.0, 1.0);
        self.host_style_layers.borrow_mut().local.opacity = Some(value);
        self.set_effective_opacity(value)
    }

    fn set_effective_opacity(&self, value: f32) -> &Self {
        self.cancel_opacity_transition();
        if self.should_animate_opacity(value) {
            let timing = {
                self.props
                    .borrow()
                    .transitions
                    .as_ref()
                    .and_then(NodeTransitions::opacity_timing)
            };
            if let Some(timing) = timing {
                let weak = self.downgrade();
                let from = self.current_opacity();
                let animation = animate_float(from, value, timing, move |next| {
                    if let Some(node) = weak.upgrade() {
                        node.apply_animated_opacity(next);
                    }
                });
                self.active_animations.borrow_mut().opacity = Some(animation);
                return self;
            }
        }
        self.apply_animated_opacity(value);
        self
    }

    pub fn transitions(&self, transitions: Option<NodeTransitions>) -> &Self {
        self.props.borrow_mut().transitions = transitions;
        self
    }

    pub fn blur(&self, sigma: f32) -> &Self {
        self.props.borrow_mut().blur_sigma = Some(sigma.max(0.0));
        if self.has_built_handle() {
            self.build_self();
            self.notify_retained_mutation();
        }
        self
    }

    pub fn drop_shadow(
        &self,
        color: u32,
        offset_x: f32,
        offset_y: f32,
        blur_sigma: f32,
        spread: f32,
    ) -> &Self {
        self.host_style_layers.borrow_mut().local.shadow =
            Some(Shadow::new(color, offset_x, offset_y, blur_sigma, spread));
        self.props.borrow_mut().drop_shadow = Some(DropShadow {
            color,
            offset_x,
            offset_y,
            blur_sigma: blur_sigma.max(0.0),
            spread,
        });
        if self.has_built_handle() {
            self.build_self();
            self.notify_retained_mutation();
        }
        self
    }

    pub fn background_blur(&self, sigma: f32) -> &Self {
        self.props.borrow_mut().background_blur_sigma = Some(sigma.max(0.0));
        if self.has_built_handle() {
            self.build_self();
            self.notify_retained_mutation();
        }
        self
    }

    pub fn linear_gradient(
        &self,
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        offsets: Vec<f32>,
        colors: Vec<u32>,
    ) -> &Self {
        self.props.borrow_mut().linear_gradient = Some(LinearGradient {
            sx,
            sy,
            ex,
            ey,
            offsets,
            colors,
        });
        if self.has_built_handle() {
            self.build_self();
            self.notify_retained_mutation();
        }
        self
    }

    pub fn linear_gradient_stops(
        &self,
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        stops: Vec<GradientStop>,
    ) -> &Self {
        let mut offsets = Vec::with_capacity(stops.len());
        let mut colors = Vec::with_capacity(stops.len());
        for stop in stops {
            offsets.push(stop.offset);
            colors.push(stop.color);
        }
        self.linear_gradient(sx, sy, ex, ey, offsets, colors)
    }

    pub fn interactive(&self, interactive: bool) -> &Self {
        let mut core = self.core.borrow_mut();
        core.behavior.interactive = interactive;
        let enabled = core.behavior.enabled && core.behavior.inherited_enabled;
        let has_built_handle = core.handle != NodeHandle::INVALID;
        drop(core);
        if has_built_handle {
            ui::set_interactive(self.handle().raw(), enabled && interactive);
            self.notify_retained_mutation();
        }
        self
    }

    pub fn enabled(&self, enabled: bool) -> &Self {
        self.retained_node_ref().set_own_enabled(enabled);
        self
    }

    pub(crate) fn reflect_semantic_disabled_from_enabled(&self) -> &Self {
        let mut core = self.core.borrow_mut();
        core.behavior.track_semantic_disabled_from_enabled = true;
        let enabled = core.behavior.enabled && core.behavior.inherited_enabled;
        let has_built_handle = core.handle != NodeHandle::INVALID;
        drop(core);
        if has_built_handle {
            ui::set_semantic_disabled(self.handle().raw(), true, !enabled);
            self.notify_retained_mutation();
        }
        self
    }

    pub fn focusable(&self, enabled: bool, tab_index: i32) -> &Self {
        let mut core = self.core.borrow_mut();
        core.behavior.focusable = Some((enabled, tab_index));
        let interactive = core.behavior.enabled && core.behavior.inherited_enabled;
        let has_built_handle = core.handle != NodeHandle::INVALID;
        drop(core);
        if has_built_handle {
            ui::set_focusable(self.handle().raw(), interactive && enabled, tab_index);
            self.notify_retained_mutation();
        }
        self
    }

    pub fn cursor(&self, style: CursorStyle) -> &Self {
        self.host_style_layers.borrow_mut().local.cursor = Some(style);
        self.core.borrow_mut().behavior.cursor = Some(style);
        crate::event::handle_cursor_style_changed(self.handle());
        self
    }

    fn current_background_color(&self) -> u32 {
        self.props.borrow().bg_color.unwrap_or(0)
    }

    fn current_opacity(&self) -> f32 {
        self.props.borrow().opacity.unwrap_or(1.0)
    }

    fn apply_animated_background_color(&self, color: u32) {
        self.props.borrow_mut().bg_color = Some(color);
        if self.has_built_handle() {
            ui::set_bg_color(self.handle().raw(), color);
            self.notify_retained_mutation();
        }
    }

    fn apply_animated_opacity(&self, value: f32) {
        let value = value.clamp(0.0, 1.0);
        self.props.borrow_mut().opacity = Some(value);
        if self.has_built_handle() {
            ui::set_layer_effect(
                self.handle().raw(),
                value,
                self.props.borrow().blur_sigma.unwrap_or(0.0),
                0,
            );
            self.notify_retained_mutation();
        }
    }

    fn should_animate_opacity(&self, next: f32) -> bool {
        self.has_built_handle()
            && self
                .props
                .borrow()
                .transitions
                .as_ref()
                .and_then(NodeTransitions::opacity_timing)
                .is_some()
            && (self.current_opacity() - next).abs() > f32::EPSILON
    }

    fn should_animate_background_color(&self, next: u32) -> bool {
        self.has_built_handle()
            && self
                .props
                .borrow()
                .transitions
                .as_ref()
                .and_then(NodeTransitions::background_color_timing)
                .is_some()
            && self.current_background_color() != next
    }

    fn cancel_opacity_transition(&self) {
        if let Some(animation) = self.active_animations.borrow_mut().opacity.take() {
            animation.cancel();
        }
    }

    fn cancel_background_color_transition(&self) {
        if let Some(animation) = self.active_animations.borrow_mut().background_color.take() {
            animation.cancel();
        }
    }

    pub fn node_id(&self, node_id: impl Into<String>) -> &Self {
        let node_id = node_id.into();
        self.core.borrow_mut().behavior.node_id = Some(node_id.clone());
        if self.has_built_handle() {
            ui::set_node_id(self.handle().raw(), &node_id);
            self.notify_retained_mutation();
        }
        self
    }

    pub fn semantic_role(&self, role: SemanticRole) -> &Self {
        self.core.borrow_mut().behavior.semantic_role = Some(role);
        if self.has_built_handle() {
            ui::set_semantic_role(self.handle().raw(), role as u32);
            self.notify_retained_mutation();
        }
        self
    }

    pub fn semantic_label(&self, label: impl Into<String>) -> &Self {
        let label = label.into();
        self.core.borrow_mut().behavior.semantic_label = Some(label.clone());
        if self.has_built_handle() {
            ui::set_semantic_label(self.handle().raw(), &label);
            self.notify_retained_mutation();
        }
        self
    }

    pub(crate) fn default_semantic_label(&self, label: impl Into<String>) -> &Self {
        let label = label.into();
        let should_emit = {
            let mut core = self.core.borrow_mut();
            core.behavior.default_semantic_label = Some(label.clone());
            core.behavior.semantic_label.is_none()
        };
        if should_emit && self.has_built_handle() {
            ui::set_semantic_label(self.handle().raw(), &label);
            self.notify_retained_mutation();
        }
        self
    }

    pub(crate) fn semantic_disabled(&self, disabled: bool) -> &Self {
        self.core.borrow_mut().behavior.semantic_disabled = Some(disabled);
        if self.has_built_handle() {
            ui::set_semantic_disabled(self.handle().raw(), true, disabled);
            self.notify_retained_mutation();
        }
        self
    }

    pub fn semantic_checked(&self, state: SemanticCheckedState) -> &Self {
        self.core.borrow_mut().behavior.semantic_checked = Some(state);
        self
    }

    pub(crate) fn semantic_selected(&self, selected: bool) -> &Self {
        self.core.borrow_mut().behavior.semantic_selected = Some(selected);
        if self.has_built_handle() {
            ui::set_semantic_selected(self.handle().raw(), true, selected);
            self.notify_retained_mutation();
        }
        self
    }

    pub fn semantic_value_range(&self, value_now: f32, value_min: f32, value_max: f32) -> &Self {
        self.core.borrow_mut().behavior.semantic_value_range =
            Some((value_now, value_min, value_max));
        self
    }

    pub fn semantic_orientation(&self, orientation: Orientation) -> &Self {
        self.core.borrow_mut().behavior.semantic_orientation = Some(orientation);
        self
    }

    pub fn request_semantic_announcement(&self) -> &Self {
        self.core
            .borrow_mut()
            .behavior
            .request_semantic_announcement = true;
        self
    }

    pub fn visibility(&self, visibility: Visibility) -> &Self {
        self.core.borrow_mut().behavior.visibility = Some(visibility);
        if self.has_built_handle() {
            ui::set_visibility(self.handle().raw(), visibility as u32);
            self.notify_retained_layout_mutation();
        }
        self
    }

    pub fn portal(&self, is_portal: bool) -> &Self {
        self.core.borrow_mut().behavior.is_portal = is_portal;
        if self.has_built_handle() {
            ui::set_is_portal(self.handle().raw(), is_portal);
            self.notify_retained_mutation();
        }
        self
    }

    pub fn min_width(&self, value: f32, unit: Unit) -> &Self {
        self.core.borrow_mut().behavior.min_width = Some((value, unit));
        if self.has_built_handle() {
            ui::set_min_width(self.handle().raw(), value, unit as u32);
            self.notify_retained_layout_mutation();
        }
        self
    }

    pub fn min_width_len(&self, length: Length) -> &Self {
        let (value, unit) = length;
        self.min_width(value, unit)
    }

    pub fn max_width(&self, value: f32, unit: Unit) -> &Self {
        self.core.borrow_mut().behavior.max_width = Some((value, unit));
        if self.has_built_handle() {
            ui::set_max_width(self.handle().raw(), value, unit as u32);
            self.notify_retained_layout_mutation();
        }
        self
    }

    pub fn max_width_len(&self, length: Length) -> &Self {
        let (value, unit) = length;
        self.max_width(value, unit)
    }

    pub fn min_height(&self, value: f32, unit: Unit) -> &Self {
        self.core.borrow_mut().behavior.min_height = Some((value, unit));
        if self.has_built_handle() {
            ui::set_min_height(self.handle().raw(), value, unit as u32);
            self.notify_retained_layout_mutation();
        }
        self
    }

    pub fn min_height_len(&self, length: Length) -> &Self {
        let (value, unit) = length;
        self.min_height(value, unit)
    }

    pub fn max_height(&self, value: f32, unit: Unit) -> &Self {
        self.core.borrow_mut().behavior.max_height = Some((value, unit));
        if self.has_built_handle() {
            ui::set_max_height(self.handle().raw(), value, unit as u32);
            self.notify_retained_layout_mutation();
        }
        self
    }

    pub fn max_height_len(&self, length: Length) -> &Self {
        let (value, unit) = length;
        self.max_height(value, unit)
    }

    pub fn flex_basis(&self, basis: f32) -> &Self {
        self.core.borrow_mut().behavior.flex_basis = Some(basis);
        self
    }

    pub fn justify_content(&self, justify: JustifyContent) -> &Self {
        self.host_style_layers.borrow_mut().local.justify_content = Some(justify);
        self.core.borrow_mut().behavior.justify_content = Some(justify);
        self
    }

    pub fn align_items(&self, align: AlignItems) -> &Self {
        self.host_style_layers.borrow_mut().local.align_items = Some(align);
        self.core.borrow_mut().behavior.align_items = Some(align);
        self
    }

    pub fn align_self(&self, align: AlignSelf) -> &Self {
        self.core.borrow_mut().behavior.align_self = Some(align);
        self
    }

    pub fn margin(&self, left: f32, top: f32, right: f32, bottom: f32) -> &Self {
        self.core.borrow_mut().behavior.margin = Some((left, top, right, bottom));
        self
    }

    pub fn apply_presenter_style(&self, style: PresenterHostStyle) -> &Self {
        let previous = self.host_style_layers.borrow().resolved();
        self.host_style_layers.borrow_mut().presenter = style;
        self.sync_resolved_host_style_if_changed(previous);
        self
    }

    pub fn clear_presenter_style(&self) -> &Self {
        self.apply_presenter_style(PresenterHostStyle::new())
    }

    pub fn clear_bg_color(&self) -> &Self {
        self.clear_local_host_style(|style| style.background = None)
    }

    pub fn clear_padding(&self) -> &Self {
        self.clear_local_host_style(|style| style.padding = None)
    }

    pub fn clear_corners(&self) -> &Self {
        self.clear_local_host_style(|style| style.corners = None)
    }

    pub fn clear_border(&self) -> &Self {
        self.clear_local_host_style(|style| style.border = None)
    }

    pub fn clear_drop_shadow(&self) -> &Self {
        self.clear_local_host_style(|style| style.shadow = None)
    }

    pub fn clear_opacity(&self) -> &Self {
        self.clear_local_host_style(|style| style.opacity = None)
    }

    pub fn clear_flex_direction(&self) -> &Self {
        self.clear_local_host_style(|style| style.flex_direction = None)
    }

    pub fn clear_justify_content(&self) -> &Self {
        self.clear_local_host_style(|style| style.justify_content = None)
    }

    pub fn clear_align_items(&self) -> &Self {
        self.clear_local_host_style(|style| style.align_items = None)
    }

    pub fn clear_cursor(&self) -> &Self {
        self.clear_local_host_style(|style| style.cursor = None)
    }

    pub fn resolved_host_style(&self) -> PresenterHostStyle {
        self.host_style_layers.borrow().resolved()
    }

    fn clear_local_host_style(&self, clear: impl FnOnce(&mut PresenterHostStyle)) -> &Self {
        let previous = self.host_style_layers.borrow().resolved();
        clear(&mut self.host_style_layers.borrow_mut().local);
        self.sync_resolved_host_style_if_changed(previous);
        self
    }

    fn sync_resolved_host_style_if_changed(&self, previous: PresenterHostStyle) {
        let resolved = self.host_style_layers.borrow().resolved();
        if resolved == previous {
            return;
        }
        self.sync_resolved_host_style(previous, resolved);
    }

    fn sync_resolved_host_style(&self, previous: PresenterHostStyle, resolved: PresenterHostStyle) {
        let background_changed = previous.background != resolved.background;
        let padding_changed = previous.padding != resolved.padding;
        let flex_direction_changed = previous.flex_direction != resolved.flex_direction;
        let justify_content_changed = previous.justify_content != resolved.justify_content;
        let align_items_changed = previous.align_items != resolved.align_items;
        let box_style_changed =
            previous.corners != resolved.corners || previous.border != resolved.border;
        let opacity_changed = previous.opacity != resolved.opacity;
        let shadow_changed = previous.shadow != resolved.shadow;
        let cursor_changed = previous.cursor != resolved.cursor;

        {
            let mut props = self.props.borrow_mut();
            props.padding = resolved
                .padding
                .map(|value| (value.left, value.top, value.right, value.bottom));
            props.flex_direction = resolved.flex_direction;
            props.opacity = resolved.opacity;
            props.drop_shadow = resolved.shadow.map(|value| DropShadow {
                color: value.color,
                offset_x: value.offset_x,
                offset_y: value.offset_y,
                blur_sigma: value.blur_sigma,
                spread: value.spread,
            });
            props.box_style = if resolved.corners.is_some() || resolved.border.is_some() {
                let corners = resolved.corners.unwrap_or_else(|| Corners::all(0.0));
                let border = resolved
                    .border
                    .unwrap_or_else(|| Border::solid(0.0, 0x00000000));
                Some(BoxStyle {
                    radius_tl: corners.top_left,
                    radius_tr: corners.top_right,
                    radius_br: corners.bottom_right,
                    radius_bl: corners.bottom_left,
                    border_width: border.width,
                    border_color: border.color,
                    border_style: border.style,
                    border_dash_on: border.dash_on,
                    border_dash_off: border.dash_off,
                })
            } else {
                None
            };
        }
        {
            let mut core = self.core.borrow_mut();
            core.behavior.justify_content = resolved.justify_content;
            core.behavior.align_items = resolved.align_items;
            core.behavior.cursor = resolved.cursor;
        }
        if !self.has_built_handle() {
            self.props.borrow_mut().bg_color = resolved.background;
            self.props.borrow_mut().opacity = resolved.opacity;
            return;
        }

        let handle = self.handle().raw();
        let mut direct_native_change = false;
        if box_style_changed {
            let corners = resolved.corners.unwrap_or_else(|| Corners::all(0.0));
            let border = resolved
                .border
                .unwrap_or_else(|| Border::solid(0.0, 0x00000000));
            ui::set_box_style(
                handle,
                if background_changed {
                    previous.background.unwrap_or(0x00000000)
                } else {
                    resolved.background.unwrap_or(0x00000000)
                },
                corners.top_left,
                corners.top_right,
                corners.bottom_right,
                corners.bottom_left,
                border.width,
                border.color,
                border.style as u32,
                border.dash_on,
                border.dash_off,
            );
            direct_native_change = true;
        }
        if padding_changed {
            let padding = resolved.padding.unwrap_or_else(|| EdgeInsets::all(0.0));
            ui::set_padding(
                handle,
                padding.left,
                padding.top,
                padding.right,
                padding.bottom,
            );
            direct_native_change = true;
        }
        if flex_direction_changed {
            ui::set_flex_direction(
                handle,
                resolved.flex_direction.unwrap_or(FlexDirection::Column) as u32,
            );
            direct_native_change = true;
        }
        if justify_content_changed {
            ui::set_justify_content(
                handle,
                resolved.justify_content.unwrap_or(JustifyContent::Start) as u32,
            );
            direct_native_change = true;
        }
        if align_items_changed {
            ui::set_align_items(
                handle,
                resolved.align_items.unwrap_or(AlignItems::Stretch) as u32,
            );
            direct_native_change = true;
        }
        if shadow_changed {
            let shadow = resolved
                .shadow
                .unwrap_or_else(|| Shadow::new(0x00000000, 0.0, 0.0, 0.0, 0.0));
            ui::set_drop_shadow(
                handle,
                shadow.color,
                shadow.offset_x,
                shadow.offset_y,
                shadow.blur_sigma,
                shadow.spread,
            );
            direct_native_change = true;
        }
        if cursor_changed {
            crate::event::handle_cursor_style_changed(self.handle());
        }
        if direct_native_change {
            self.notify_retained_mutation();
        }
        if background_changed {
            self.set_effective_background_color(resolved.background.unwrap_or(0x00000000));
        }
        if opacity_changed {
            self.set_effective_opacity(resolved.opacity.unwrap_or(1.0));
        }
    }

    pub fn position_type(&self, position_type: PositionType) -> &Self {
        self.core.borrow_mut().behavior.position_type = Some(position_type);
        if self.has_built_handle() {
            ui::set_position_type(self.handle().raw(), position_type as u32);
            self.notify_retained_layout_mutation();
        }
        self
    }

    pub fn position(&self, left: f32, top: f32) -> &Self {
        self.core.borrow_mut().behavior.position = Some((left, top, f32::NAN, f32::NAN));
        if self.has_built_handle() {
            ui::set_position(self.handle().raw(), left, top, f32::NAN, f32::NAN);
            self.notify_retained_layout_mutation();
        }
        self
    }

    pub fn custom_drawable(&self, enabled: bool) -> &Self {
        self.core.borrow_mut().behavior.custom_drawable = enabled;
        self
    }

    pub fn flex_wrap(&self, wrap: FlexWrap) -> &Self {
        self.core.borrow_mut().behavior.flex_wrap = Some(wrap);
        self
    }

    pub fn clip_to_bounds(&self, clip: bool) -> &Self {
        self.core.borrow_mut().behavior.clip_to_bounds = Some(clip);
        self
    }

    pub fn selection_area(&self, enabled: bool) -> &Self {
        self.core.borrow_mut().behavior.selection_area = enabled;
        self
    }

    pub fn selection_area_barrier(&self, enabled: bool) -> &Self {
        self.core.borrow_mut().behavior.selection_area_barrier = enabled;
        self
    }

    pub fn shared_size_scope(&self, enabled: bool) -> &Self {
        self.core.borrow_mut().behavior.is_shared_size_scope = enabled;
        if self.has_built_handle() {
            ui::set_is_shared_size_scope(self.handle().raw(), enabled);
            self.notify_retained_layout_mutation();
        }
        self
    }

    pub fn on_click(&self, handler: impl Fn(&mut PointerEventArgs) + 'static) -> &Self {
        self.core.borrow_mut().handlers.pointer_click = Some(Rc::new(handler));
        self.retained_node_ref().require_interactive();
        self
    }

    pub fn on_pointer_down(&self, handler: impl Fn(&mut PointerEventArgs) + 'static) -> &Self {
        self.core.borrow_mut().handlers.pointer_down = Some(Rc::new(handler));
        self.retained_node_ref().require_interactive();
        self
    }

    pub fn on_pointer_move(&self, handler: impl Fn(&mut PointerEventArgs) + 'static) -> &Self {
        self.core.borrow_mut().handlers.pointer_move = Some(Rc::new(handler));
        self.retained_node_ref().require_interactive();
        self
    }

    pub fn on_pointer_up(&self, handler: impl Fn(&mut PointerEventArgs) + 'static) -> &Self {
        self.core.borrow_mut().handlers.pointer_up = Some(Rc::new(handler));
        self.retained_node_ref().require_interactive();
        self
    }

    pub fn on_pointer_enter(&self, handler: impl Fn(&mut PointerEventArgs) + 'static) -> &Self {
        self.core.borrow_mut().handlers.pointer_enter = Some(Rc::new(handler));
        self.retained_node_ref().require_interactive();
        self
    }

    pub fn on_pointer_leave(&self, handler: impl Fn(&mut PointerEventArgs) + 'static) -> &Self {
        self.core.borrow_mut().handlers.pointer_leave = Some(Rc::new(handler));
        self.retained_node_ref().require_interactive();
        self
    }

    pub fn on_pointer_cancel(&self, handler: impl Fn(&mut PointerEventArgs) + 'static) -> &Self {
        self.core.borrow_mut().handlers.pointer_cancel = Some(Rc::new(handler));
        self.retained_node_ref().require_interactive();
        self
    }

    pub fn on_wheel(&self, handler: impl Fn(&mut WheelEventArgs) + 'static) -> &Self {
        self.core.borrow_mut().handlers.wheel = Some(Rc::new(handler));
        self.retained_node_ref().require_interactive();
        self
    }

    pub fn on_pan_gesture(&self, handler: impl Fn(&mut GestureEventArgs) + 'static) -> &Self {
        self.core.borrow_mut().handlers.pan_gesture = Some(Rc::new(handler));
        self
    }

    pub fn on_pinch_gesture(&self, handler: impl Fn(&mut GestureEventArgs) + 'static) -> &Self {
        self.core.borrow_mut().handlers.pinch_gesture = Some(Rc::new(handler));
        self
    }

    pub fn long_press_options(&self, minimum_duration_ms: i32, movement_tolerance: f32) -> &Self {
        let mut core = self.core.borrow_mut();
        core.handlers.long_press_minimum_duration_ms = minimum_duration_ms.max(0);
        core.handlers.long_press_movement_tolerance = movement_tolerance.max(0.0);
        self
    }

    pub fn on_long_press(&self, handler: impl Fn(&mut LongPressEventArgs) + 'static) -> &Self {
        self.core.borrow_mut().handlers.long_press = Some(Rc::new(handler));
        self
    }

    pub fn on_key_down(&self, handler: impl Fn(&mut KeyEventArgs) + 'static) -> &Self {
        self.core.borrow_mut().handlers.key_down = Some(Rc::new(handler));
        self
    }

    pub fn on_key_up(&self, handler: impl Fn(&mut KeyEventArgs) + 'static) -> &Self {
        self.core.borrow_mut().handlers.key_up = Some(Rc::new(handler));
        self
    }

    pub fn on_focus_changed(&self, handler: impl Fn(FocusChangedEventArgs) + 'static) -> &Self {
        self.core.borrow_mut().handlers.focus_changed = Some(Rc::new(handler));
        self
    }

    pub fn child<T: Node>(&self, child: &T) -> &Self {
        self.append_child(child);
        self
    }

    pub fn children<I, C>(&self, children: I) -> &Self
    where
        I: IntoIterator<Item = C>,
        C: Into<Child>,
    {
        for child in children {
            self.retained_node_ref()
                .append_child_ref(&child.into().node_ref);
        }
        self
    }
}
