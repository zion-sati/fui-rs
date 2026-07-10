use super::core::*;
use super::*;
use crate::event::{SelectionChangedEventArgs, TextChangedEventArgs};
use crate::{FontFamily, FontStack, FontStyle, FontWeight};
use std::ops::Deref;

#[derive(Clone)]
pub struct TextCore {
    inner: TextNode,
}

impl TextCore {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            inner: TextNode::new_core(content),
        }
    }
}

impl Deref for TextCore {
    type Target = TextNode;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Node for TextCore {
    fn retained_node_ref(&self) -> NodeRef {
        self.inner.retained_node_ref()
    }

    fn build_self(&self) {
        self.inner.build_self();
    }
}

#[derive(Clone)]
pub struct TextNode {
    core: Rc<RefCell<NodeCore>>,
    props: Rc<RefCell<TextProps>>,
}

impl TextNode {
    pub fn new(content: impl Into<String>) -> Self {
        Self::new_with_defaults(content, true)
    }

    fn new_core(content: impl Into<String>) -> Self {
        Self::new_with_defaults(content, false)
    }

    fn new_with_defaults(content: impl Into<String>, selectable_by_default: bool) -> Self {
        let content = content.into();
        let core = Rc::new(RefCell::new(NodeCore::new(NodeKind::Text)));
        {
            let mut core_mut = core.borrow_mut();
            if selectable_by_default {
                core_mut.behavior.cursor = Some(CursorStyle::Text);
                core_mut.behavior.selectable_text = true;
            }
            core_mut.behavior.text_content = Some(content.clone());
        }
        let theme = theme::current_theme();
        let node = Self {
            core,
            props: Rc::new(RefCell::new(TextProps {
                content,
                font_size: theme.fonts.size_body,
                has_font: true,
                selectable: selectable_by_default.then_some((true, theme.colors.selection)),
                uses_theme_selection_color: selectable_by_default,
                ..TextProps::default()
            })),
        };
        node.bind_theme_defaults();
        node
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

    pub fn text(&self, content: impl Into<String>) -> &Self {
        let content = content.into();
        self.props.borrow_mut().content = content.clone();
        self.retained_node_ref()
            .set_text_content_for_routing(Some(content.clone()));
        if self.has_built_handle() {
            ui::set_text(self.handle().raw(), &content);
            self.notify_retained_layout_mutation();
        }
        self
    }

    pub fn text_color(&self, color: u32) -> &Self {
        self.props.borrow_mut().text_color = Some(color);
        if self.has_built_handle() {
            ui::set_text_color(self.handle().raw(), color);
            self.notify_retained_mutation();
        }
        self
    }

    pub fn style_runs(&self, words: Vec<u32>) -> &Self {
        {
            let mut props = self.props.borrow_mut();
            props.style_runs = words;
            props.has_style_runs = true;
        }
        if self.has_built_handle() {
            let props = self.props.borrow();
            ui::set_text_style_runs(self.handle().raw(), &props.style_runs);
            self.notify_retained_layout_mutation();
        }
        self
    }

    pub(crate) fn font_id(&self, font_id: u32, size: f32) -> &Self {
        let mut props = self.props.borrow_mut();
        props.uses_direct_font_id = true;
        props.font_family = None;
        props.font_id = font_id;
        props.font_size = size;
        props.has_font = true;
        drop(props);
        if self.has_built_handle() {
            self.apply_resolved_font();
        }
        self
    }

    pub fn font_stack(&self, stack: FontStack, size: f32) -> &Self {
        self.font_id(stack.id(), size)
    }

    pub fn font_family(&self, family: FontFamily) -> &Self {
        let mut props = self.props.borrow_mut();
        props.font_family = Some(family);
        props.uses_direct_font_id = false;
        drop(props);
        if self.has_built_handle() && self.props.borrow().has_font {
            self.apply_resolved_font();
        }
        self
    }

    pub fn font_weight(&self, weight: FontWeight) -> &Self {
        self.props.borrow_mut().font_weight = weight;
        if self.has_built_handle() && self.props.borrow().has_font {
            self.apply_resolved_font();
        }
        self
    }

    pub fn font_style(&self, style: FontStyle) -> &Self {
        self.props.borrow_mut().font_style = style;
        if self.has_built_handle() && self.props.borrow().has_font {
            self.apply_resolved_font();
        }
        self
    }

    pub fn font_size(&self, size: f32) -> &Self {
        let mut props = self.props.borrow_mut();
        props.font_size = size;
        props.has_font = true;
        drop(props);
        if self.has_built_handle() {
            self.apply_resolved_font();
        }
        self
    }

    pub fn line_height(&self, line_height: f32) -> &Self {
        self.props.borrow_mut().line_height = Some(line_height);
        if self.has_built_handle() {
            ui::set_line_height(self.handle().raw(), line_height);
            self.notify_retained_layout_mutation();
        }
        self
    }

    pub fn text_align(&self, align: TextAlign) -> &Self {
        self.props.borrow_mut().text_align = Some(align);
        self
    }

    pub fn text_vertical_align(&self, align: TextVerticalAlign) -> &Self {
        self.props.borrow_mut().text_vertical_align = Some(align);
        self
    }

    pub fn text_limits(&self, max_chars: i32, max_lines: i32) -> &Self {
        self.props.borrow_mut().text_limits = Some((max_chars, max_lines));
        if self.has_built_handle() {
            ui::set_text_limits(self.handle().raw(), max_chars, max_lines);
            self.notify_retained_layout_mutation();
        }
        self
    }

    pub fn wrapping(&self, wrap: bool) -> &Self {
        self.props.borrow_mut().wrapping = Some(wrap);
        if self.has_built_handle() {
            ui::set_text_wrapping(self.handle().raw(), wrap);
            self.notify_retained_layout_mutation();
        }
        self
    }

    pub fn text_overflow(&self, overflow: TextOverflow) -> &Self {
        self.props.borrow_mut().overflow = Some(overflow);
        self
    }

    pub fn text_overflow_fade(&self, horizontal: bool, vertical: bool) -> &Self {
        self.props.borrow_mut().overflow_fade = Some((horizontal, vertical));
        self
    }

    pub fn selectable(&self, selectable: bool, selection_color: u32) -> &Self {
        let resolved_selection_color = if selection_color == 0 {
            theme::current_theme().colors.selection
        } else {
            selection_color
        };
        let mut props = self.props.borrow_mut();
        props.selectable = Some((selectable, resolved_selection_color));
        props.uses_theme_selection_color = selection_color == 0;
        drop(props);
        self.core.borrow_mut().behavior.selectable_text = selectable;
        if self.has_built_handle() {
            ui::set_selectable(self.handle().raw(), selectable, resolved_selection_color);
            self.notify_retained_mutation();
        }
        self
    }

    pub fn editable(&self, editable: bool) -> &Self {
        if editable {
            let selection_color = self
                .props
                .borrow()
                .selectable
                .map(|(_, color)| color)
                .unwrap_or(theme::current_theme().colors.selection);
            self.props.borrow_mut().selectable = Some((true, selection_color));
            self.core.borrow_mut().behavior.selectable_text = true;
        }
        self.props.borrow_mut().editable = Some(editable);
        self.core.borrow_mut().behavior.editable_text = editable;
        if self.has_built_handle() {
            ui::set_editable(self.handle().raw(), editable);
            self.notify_retained_mutation();
        }
        self
    }

    pub(crate) fn editor_command_keys(&self, enabled: bool) -> &Self {
        self.props.borrow_mut().editor_command_keys = Some(enabled);
        if self.has_built_handle() {
            ui::set_editor_command_keys(self.handle().raw(), enabled);
            self.notify_retained_mutation();
        }
        self
    }

    pub(crate) fn editor_accepts_tab(&self, enabled: bool) -> &Self {
        self.props.borrow_mut().editor_accepts_tab = Some(enabled);
        if self.has_built_handle() {
            ui::set_editor_accepts_tab(self.handle().raw(), enabled);
            self.notify_retained_mutation();
        }
        self
    }

    pub fn obscured(&self, obscured: bool) -> &Self {
        self.props.borrow_mut().obscured = Some(obscured);
        if self.has_built_handle() {
            ui::set_text_obscured(self.handle().raw(), obscured);
            self.notify_retained_mutation();
        }
        self
    }

    pub fn caret_color(&self, color: u32) -> &Self {
        self.props.borrow_mut().caret_color = Some(color);
        if self.has_built_handle() {
            ui::set_caret_color(self.handle().raw(), color);
            self.notify_retained_mutation();
        }
        self
    }

    pub fn selection_range(&self, start: u32, end: u32) -> &Self {
        if self.has_built_handle() {
            ui::set_text_selection_range(self.handle().raw(), start, end);
            self.notify_retained_mutation();
        }
        self
    }

    pub fn focus_now(&self) -> &Self {
        if self.has_built_handle() {
            ui::request_focus(self.handle().raw());
        }
        self
    }

    pub(crate) fn default_semantic_label(&self, label: impl Into<String>) -> &Self {
        let label = label.into();
        let mut core = self.core.borrow_mut();
        core.behavior.default_semantic_label = Some(label.clone());
        if core.behavior.semantic_label.is_none() && core.handle != NodeHandle::INVALID {
            ui::set_semantic_label(core.handle.raw(), &label);
            drop(core);
            self.notify_retained_mutation();
        }
        self
    }

    pub fn interactive(&self, interactive: bool) -> &Self {
        self.core.borrow_mut().behavior.interactive = interactive;
        self
    }

    pub fn cursor(&self, style: CursorStyle) -> &Self {
        self.core.borrow_mut().behavior.cursor = Some(style);
        crate::event::handle_cursor_style_changed(self.handle());
        self
    }

    pub fn focusable(&self, enabled: bool, tab_index: i32) -> &Self {
        let mut core = self.core.borrow_mut();
        core.behavior.focusable = Some((enabled, tab_index));
        let interactive = core.behavior.enabled && core.behavior.inherited_enabled;
        let handle = core.handle;
        drop(core);
        if handle != NodeHandle::INVALID {
            ui::set_focusable(handle.raw(), interactive && enabled, tab_index);
            self.notify_retained_mutation();
        }
        self
    }

    pub(crate) fn reflect_semantic_disabled_from_enabled(&self) -> &Self {
        let mut core = self.core.borrow_mut();
        core.behavior.track_semantic_disabled_from_enabled = true;
        let effective_enabled = core.behavior.enabled && core.behavior.inherited_enabled;
        let handle = core.handle;
        drop(core);
        if handle != NodeHandle::INVALID {
            ui::set_semantic_disabled(handle.raw(), true, !effective_enabled);
            self.notify_retained_mutation();
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

    pub fn on_pointer_up(&self, handler: impl Fn(&mut PointerEventArgs) + 'static) -> &Self {
        self.core.borrow_mut().handlers.pointer_up = Some(Rc::new(handler));
        self.retained_node_ref().require_interactive();
        self
    }

    pub fn on_focus_changed(&self, handler: impl Fn(FocusChangedEventArgs) + 'static) -> &Self {
        self.core.borrow_mut().handlers.focus_changed = Some(Rc::new(handler));
        self
    }

    pub fn on_text_changed(&self, handler: impl Fn(TextChangedEventArgs) + 'static) -> &Self {
        let node = self.clone();
        self.core.borrow_mut().handlers.text_changed = Some(Rc::new(move |event| {
            node.sync_text_from_runtime(event.text.clone());
            handler(event);
        }));
        self
    }

    pub(crate) fn on_text_replaced(&self, handler: impl Fn(u32, u32, String) + 'static) -> &Self {
        let node = self.clone();
        self.core.borrow_mut().handlers.text_replaced = Some(Rc::new(move |start, end, text| {
            node.apply_text_replacement(start, end, &text);
            handler(start, end, text);
        }));
        self
    }

    pub fn on_selection_changed(
        &self,
        handler: impl Fn(SelectionChangedEventArgs) + 'static,
    ) -> &Self {
        self.core.borrow_mut().handlers.selection_changed = Some(Rc::new(handler));
        self
    }

    pub fn text_value(&self) -> String {
        self.props.borrow().content.clone()
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

    pub(crate) fn required_font_ids(&self) -> Vec<u32> {
        let props = self.props.borrow();
        let mut font_ids = Vec::new();
        if props.has_font && props.font_id != 0 {
            font_ids.push(props.font_id);
        }
        for chunk in props.style_runs.chunks_exact(7) {
            let font_id = chunk[2];
            if !font_ids.contains(&font_id) {
                font_ids.push(font_id);
            }
        }
        font_ids
    }

    fn bind_theme_defaults(&self) {
        let weak_core = Rc::downgrade(&self.core);
        let weak_props = Rc::downgrade(&self.props);
        let guard = theme::subscribe(move |theme| {
            let Some(core) = weak_core.upgrade() else {
                return;
            };
            let Some(props) = weak_props.upgrade() else {
                return;
            };
            TextNode::handle_theme_changed(&core, &props, &theme);
        });
        self.retained_node_ref().retain_attachment(Rc::new(guard));
    }

    fn handle_theme_changed(
        core: &Rc<RefCell<NodeCore>>,
        props: &Rc<RefCell<TextProps>>,
        theme: &crate::theme::Theme,
    ) {
        let handle = core.borrow().handle;
        if handle == NodeHandle::INVALID {
            if props.borrow().has_font
                && !props.borrow().uses_direct_font_id
                && props.borrow().font_family.is_none()
            {
                let resolved_font_id = theme
                    .fonts
                    .body_family
                    .resolve(props.borrow().font_weight, props.borrow().font_style);
                props.borrow_mut().font_id = resolved_font_id;
            }
            return;
        }

        let mut should_request_render = false;
        {
            let mut text_props = props.borrow_mut();
            if text_props.text_color.is_none() {
                ui::set_text_color(handle.raw(), theme.colors.text_primary);
                should_request_render = true;
            }
            if text_props.has_font
                && !text_props.uses_direct_font_id
                && text_props.font_family.is_none()
            {
                text_props.font_id = theme
                    .fonts
                    .body_family
                    .resolve(text_props.font_weight, text_props.font_style);
                ui::set_font(handle.raw(), text_props.font_id, text_props.font_size);
                should_request_render = true;
            }
            if text_props.uses_theme_selection_color {
                if let Some((selectable, _)) = text_props.selectable {
                    text_props.selectable = Some((selectable, theme.colors.selection));
                    ui::set_selectable(handle.raw(), selectable, theme.colors.selection);
                    should_request_render = true;
                }
            }
        }
        if should_request_render {
            crate::frame_scheduler::mark_needs_commit();
        }
    }

    fn resolve_font_id(&self) -> u32 {
        let mut props = self.props.borrow_mut();
        if props.uses_direct_font_id {
            return props.font_id;
        }
        let family = props
            .font_family
            .clone()
            .unwrap_or_else(|| theme::current_theme().fonts.body_family.clone());
        props.font_id = family.resolve(props.font_weight, props.font_style);
        props.font_id
    }

    fn apply_resolved_font(&self) {
        let font_id = self.resolve_font_id();
        let font_size = self.props.borrow().font_size;
        ui::set_font(self.handle().raw(), font_id, font_size);
        self.notify_retained_layout_mutation();
    }

    fn sync_text_from_runtime(&self, content: String) {
        self.props.borrow_mut().content = content.clone();
        self.retained_node_ref()
            .set_text_content_for_routing(Some(content));
    }

    fn apply_text_replacement(&self, start: u32, end: u32, replacement: &str) {
        let mut props = self.props.borrow_mut();
        let start = start.min(props.content.len() as u32) as usize;
        let end = end.min(props.content.len() as u32) as usize;
        if start <= end
            && props.content.is_char_boundary(start)
            && props.content.is_char_boundary(end)
        {
            props.content.replace_range(start..end, replacement);
        }
        let content = props.content.clone();
        drop(props);
        self.retained_node_ref()
            .set_text_content_for_routing(Some(content));
    }
}

impl Node for TextNode {
    fn retained_node_ref(&self) -> NodeRef {
        NodeRef::from_node(self.core.clone(), self.clone())
    }

    fn build_self(&self) {
        apply_text_props(
            self.handle(),
            &self.props.borrow(),
            self.core.borrow().behavior.clone(),
        );
    }
}
