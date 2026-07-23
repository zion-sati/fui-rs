use super::core::*;
use super::*;
use crate::event::{SelectionChangedEventArgs, TextChangedEventArgs};
use crate::text_indices::{byte_to_scalar, scalar_count, scalar_to_byte};
use crate::{FontFamily, FontStack, FontStyle, FontWeight};

type TextChangedCallback = Rc<dyn Fn(TextChangedEventArgs)>;
type TextReplacedCallback = Rc<dyn Fn(u32, u32, String)>;
type SelectionChangedCallback = Rc<dyn Fn(SelectionChangedEventArgs)>;

#[derive(Clone)]
pub struct TextNode {
    core: Rc<RefCell<NodeCore>>,
    props: Rc<RefCell<TextProps>>,
    text_changed_callback: Rc<RefCell<Option<TextChangedCallback>>>,
    text_replaced_callback: Rc<RefCell<Option<TextReplacedCallback>>>,
    selection_changed_callback: Rc<RefCell<Option<SelectionChangedCallback>>>,
}

impl TextNode {
    pub fn new(content: impl Into<String>) -> Self {
        Self::new_with_defaults(content, true)
    }

    pub(crate) fn new_core(content: impl Into<String>) -> Self {
        Self::new_with_defaults(content, false)
    }

    fn new_with_defaults(content: impl Into<String>, selectable_by_default: bool) -> Self {
        let content = content.into();
        let core = Rc::new(RefCell::new(NodeCore::new(NodeKind::Text)));
        let theme = theme::current_theme();
        let props = Rc::new(RefCell::new(TextProps {
            content: content.clone(),
            font_size: theme.fonts.size_body,
            has_font: true,
            selectable: selectable_by_default.then_some((true, theme.colors.selection)),
            uses_theme_selection_color: selectable_by_default,
            ..TextProps::default()
        }));
        {
            let mut core_mut = core.borrow_mut();
            if selectable_by_default {
                core_mut.behavior.cursor = Some(CursorStyle::Text);
                core_mut.behavior.selectable_text = true;
            }
            core_mut.behavior.text_content = Some(content.clone());
            let weak_props = Rc::downgrade(&props);
            core_mut.behavior.text_selection_range_bytes = Some(Rc::new(move || {
                weak_props
                    .upgrade()
                    .and_then(|props| props.borrow().selection_range_bytes)
                    .unwrap_or((0, 0))
            }));
        }
        let node = Self {
            core,
            props,
            text_changed_callback: Rc::new(RefCell::new(None)),
            text_replaced_callback: Rc::new(RefCell::new(None)),
            selection_changed_callback: Rc::new(RefCell::new(None)),
        };
        node.install_runtime_state_handlers();
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

    pub fn min_width(&self, value: f32, unit: Unit) -> &Self {
        self.props.borrow_mut().min_width = Some((value, unit));
        if self.has_built_handle() {
            ui::set_min_width(self.handle().raw(), value, unit as u32);
            self.notify_retained_layout_mutation();
        }
        self
    }

    pub fn max_width(&self, value: f32, unit: Unit) -> &Self {
        self.props.borrow_mut().max_width = Some((value, unit));
        if self.has_built_handle() {
            ui::set_max_width(self.handle().raw(), value, unit as u32);
            self.notify_retained_layout_mutation();
        }
        self
    }

    pub fn min_height(&self, value: f32, unit: Unit) -> &Self {
        self.props.borrow_mut().min_height = Some((value, unit));
        if self.has_built_handle() {
            ui::set_min_height(self.handle().raw(), value, unit as u32);
            self.notify_retained_layout_mutation();
        }
        self
    }

    pub fn max_height(&self, value: f32, unit: Unit) -> &Self {
        self.props.borrow_mut().max_height = Some((value, unit));
        if self.has_built_handle() {
            ui::set_max_height(self.handle().raw(), value, unit as u32);
            self.notify_retained_layout_mutation();
        }
        self
    }

    pub fn text(&self, content: impl Into<String>) -> &Self {
        let content = content.into();
        Self::sync_content_state(&self.core, &self.props, content.clone());
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
        if self.has_built_handle() {
            ui::set_text_align(self.handle().raw(), align as u32);
            self.notify_retained_layout_mutation();
        }
        self
    }

    pub fn text_vertical_align(&self, align: TextVerticalAlign) -> &Self {
        self.props.borrow_mut().text_vertical_align = Some(align);
        if self.has_built_handle() {
            ui::set_text_vertical_align(self.handle().raw(), align as u32);
            self.notify_retained_layout_mutation();
        }
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

    pub fn max_lines(&self, max_lines: i32) -> &Self {
        self.text_limits(i32::MAX, max_lines)
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
        if self.has_built_handle() {
            ui::set_text_overflow(self.handle().raw(), overflow as u32);
            self.notify_retained_layout_mutation();
        }
        self
    }

    pub fn text_overflow_fade(&self, horizontal: bool, vertical: bool) -> &Self {
        self.props.borrow_mut().overflow_fade = Some((horizontal, vertical));
        if self.has_built_handle() {
            ui::set_text_overflow_fade(self.handle().raw(), horizontal, vertical);
            self.notify_retained_layout_mutation();
        }
        self
    }

    pub fn selectable(&self, selectable: bool) -> &Self {
        let mut props = self.props.borrow_mut();
        let resolved_selection_color = props
            .selectable
            .map(|(_, color)| color)
            .unwrap_or_else(|| theme::current_theme().colors.selection);
        props.selectable = Some((selectable, resolved_selection_color));
        drop(props);
        {
            let mut core = self.core.borrow_mut();
            core.behavior.selectable_text = selectable;
            if selectable {
                core.behavior.cursor = Some(CursorStyle::Text);
            } else if core.behavior.cursor == Some(CursorStyle::Text) {
                core.behavior.cursor = Some(CursorStyle::Default);
            }
        }
        crate::event::handle_cursor_style_changed(self.handle());
        if self.has_built_handle() {
            ui::set_selectable(self.handle().raw(), selectable, resolved_selection_color);
            self.notify_retained_mutation();
        }
        self
    }

    pub fn selection_color(&self, color: u32) -> &Self {
        let mut props = self.props.borrow_mut();
        let selectable = props.selectable.map(|(value, _)| value).unwrap_or(false);
        props.selectable = Some((selectable, color));
        props.uses_theme_selection_color = false;
        drop(props);
        if self.has_built_handle() {
            ui::set_selectable(self.handle().raw(), selectable, color);
            self.notify_retained_mutation();
        }
        self
    }

    pub fn editable(&self, editable: bool) -> &Self {
        if editable && self.props.borrow().selectable.is_none() {
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
        let mut props = self.props.borrow_mut();
        let start = start.min(scalar_count(&props.content));
        let end = end.min(scalar_count(&props.content));
        let start_byte = scalar_to_byte(&props.content, start);
        let end_byte = scalar_to_byte(&props.content, end);
        props.selection_start = start;
        props.selection_end = end;
        props.selection_range_bytes = Some((start_byte, end_byte));
        drop(props);
        if self.has_built_handle() {
            ui::set_text_selection_range(self.handle().raw(), start_byte, end_byte);
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
        if enabled {
            self.retained_node_ref().require_interactive();
        }
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

    pub fn on_pointer_click(&self, handler: impl Fn(&mut PointerEventArgs) + 'static) -> &Self {
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
        *self.text_changed_callback.borrow_mut() = Some(Rc::new(handler));
        self
    }

    pub(crate) fn on_text_replaced(&self, handler: impl Fn(u32, u32, String) + 'static) -> &Self {
        *self.text_replaced_callback.borrow_mut() = Some(Rc::new(handler));
        self
    }

    pub fn on_selection_changed(
        &self,
        handler: impl Fn(SelectionChangedEventArgs) + 'static,
    ) -> &Self {
        *self.selection_changed_callback.borrow_mut() = Some(Rc::new(handler));
        self
    }

    pub fn content(&self) -> String {
        self.props.borrow().content.clone()
    }

    pub fn uses_default_selection_behavior(&self) -> bool {
        self.props.borrow().selectable.is_none()
    }

    pub fn is_editable_text(&self) -> bool {
        self.props.borrow().editable.unwrap_or(false)
    }

    pub fn is_selectable_text(&self) -> bool {
        self.props
            .borrow()
            .selectable
            .map(|(selectable, _)| selectable)
            .unwrap_or(false)
    }

    pub fn selection_start(&self) -> u32 {
        self.props.borrow().selection_start
    }

    pub fn selection_end(&self) -> u32 {
        self.props.borrow().selection_end
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

    fn install_runtime_state_handlers(&self) {
        let weak_core = Rc::downgrade(&self.core);
        let weak_props = Rc::downgrade(&self.props);
        let weak_callback = Rc::downgrade(&self.text_changed_callback);
        self.core.borrow_mut().handlers.text_changed = Some(Rc::new(move |event| {
            let (Some(core), Some(props)) = (weak_core.upgrade(), weak_props.upgrade()) else {
                return;
            };
            Self::sync_content_state(&core, &props, event.text.clone());
            if let Some(callbacks) = weak_callback.upgrade() {
                let callback = callbacks.borrow().clone();
                if let Some(callback) = callback {
                    callback(event);
                }
            }
        }));

        let weak_core = Rc::downgrade(&self.core);
        let weak_props = Rc::downgrade(&self.props);
        let weak_changed_callback = Rc::downgrade(&self.text_changed_callback);
        let weak_replaced_callback = Rc::downgrade(&self.text_replaced_callback);
        self.core.borrow_mut().handlers.text_replaced =
            Some(Rc::new(move |start, end, replacement| {
                let (Some(core), Some(props)) = (weak_core.upgrade(), weak_props.upgrade()) else {
                    return;
                };
                let content = {
                    let current = props.borrow().content.clone();
                    let start_byte = scalar_to_byte(&current, byte_to_scalar(&current, start));
                    let end_byte =
                        scalar_to_byte(&current, byte_to_scalar(&current, start.max(end)));
                    let mut updated = current;
                    updated.replace_range(start_byte as usize..end_byte as usize, &replacement);
                    updated
                };
                Self::sync_content_state(&core, &props, content.clone());
                if let Some(callbacks) = weak_changed_callback.upgrade() {
                    let callback = callbacks.borrow().clone();
                    if let Some(callback) = callback {
                        callback(TextChangedEventArgs {
                            text: content.clone(),
                        });
                    }
                }
                if let Some(callbacks) = weak_replaced_callback.upgrade() {
                    let callback = callbacks.borrow().clone();
                    if let Some(callback) = callback {
                        callback(start, end, replacement);
                    }
                }
            }));

        let weak_props = Rc::downgrade(&self.props);
        let weak_callback = Rc::downgrade(&self.selection_changed_callback);
        self.core.borrow_mut().handlers.selection_changed = Some(Rc::new(move |event| {
            let Some(props) = weak_props.upgrade() else {
                return;
            };
            let (start, end) = {
                let mut props = props.borrow_mut();
                let start = byte_to_scalar(&props.content, event.start);
                let end = byte_to_scalar(&props.content, event.end);
                let start_byte = scalar_to_byte(&props.content, start);
                let end_byte = scalar_to_byte(&props.content, end);
                props.selection_start = start;
                props.selection_end = end;
                props.selection_range_bytes = Some((start_byte, end_byte));
                (start, end)
            };
            if let Some(callbacks) = weak_callback.upgrade() {
                let callback = callbacks.borrow().clone();
                if let Some(callback) = callback {
                    callback(SelectionChangedEventArgs { start, end });
                }
            }
        }));
    }

    fn sync_content_state(
        core: &Rc<RefCell<NodeCore>>,
        props: &Rc<RefCell<TextProps>>,
        content: String,
    ) {
        {
            let mut props = props.borrow_mut();
            let length = scalar_count(&content);
            props.selection_start = props.selection_start.min(length);
            props.selection_end = props.selection_end.min(length);
            if props.selection_range_bytes.is_some() {
                props.selection_range_bytes = Some((
                    scalar_to_byte(&content, props.selection_start),
                    scalar_to_byte(&content, props.selection_end),
                ));
            }
            props.content = content.clone();
        }
        core.borrow_mut().behavior.text_content = Some(content);
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
}

impl Node for TextNode {
    fn retained_node_ref(&self) -> NodeRef {
        NodeRef::from_node(self.core.clone(), self.clone())
    }

    fn build_self(&self) {
        if self.props.borrow().has_font {
            self.resolve_font_id();
        }
        // Host setters may synchronously report text or selection state back into
        // this retained node. Never hold a RefCell borrow across that boundary.
        let props = self.props.borrow().clone();
        apply_text_props(self.handle(), &props, self.core.borrow().behavior.clone());
    }

    fn required_font_ids_for_preparation(&self) -> Vec<u32> {
        self.required_font_ids()
    }
}

impl super::ThemeBindable for TextNode {
    fn theme_binding_node(&self) -> NodeRef {
        self.retained_node_ref()
    }

    fn weak_theme_target(&self) -> Box<dyn Fn() -> Option<Self>> {
        let weak_core = Rc::downgrade(&self.core);
        let weak_props = Rc::downgrade(&self.props);
        let weak_text_changed_callback = Rc::downgrade(&self.text_changed_callback);
        let weak_text_replaced_callback = Rc::downgrade(&self.text_replaced_callback);
        let weak_selection_changed_callback = Rc::downgrade(&self.selection_changed_callback);
        Box::new(move || {
            Some(TextNode {
                core: weak_core.upgrade()?,
                props: weak_props.upgrade()?,
                text_changed_callback: weak_text_changed_callback.upgrade()?,
                text_replaced_callback: weak_text_replaced_callback.upgrade()?,
                selection_changed_callback: weak_selection_changed_callback.upgrade()?,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::{self, Call};
    use crate::Application;

    #[test]
    fn prebuild_font_family_weight_and_style_are_resolved_during_build() {
        ffi::test::reset();
        let theme = theme::current_theme();
        let label = TextNode::new_core("Bold before build");
        label
            .font_family(theme.fonts.body_family)
            .font_weight(FontWeight::Bold)
            .font_style(FontStyle::Normal)
            .font_size(17.0);

        Application::mount(label.clone());
        let handle = label.handle().raw();
        let calls = ffi::test::take_calls();
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::SetFont {
                handle: call_handle,
                font_id: 2,
                size,
            } if *call_handle == handle && (*size - 17.0).abs() < f32::EPSILON
        )));
        Application::unmount();
    }

    #[test]
    fn text_layout_surface_replays_before_build_and_mutates_after_build() {
        ffi::test::reset();
        let label = TextNode::new("Layout");
        label
            .fill_width_percent(75.0)
            .fill_height_percent(50.0)
            .min_width(12.0, Unit::Pixel)
            .max_width(80.0, Unit::Percent)
            .min_height(14.0, Unit::Pixel)
            .max_height(90.0, Unit::Pixel)
            .text_align(TextAlign::Center)
            .text_vertical_align(TextVerticalAlign::Bottom)
            .text_overflow(TextOverflow::Ellipsis)
            .text_overflow_fade(true, false);

        Application::mount(label.clone());
        let calls = ffi::test::take_calls();
        assert!(calls.iter().any(|call| matches!(call, Call::SetFillWidthPercent { percent, .. } if (*percent - 75.0).abs() < f32::EPSILON)));
        assert!(calls.iter().any(|call| matches!(call, Call::SetFillHeightPercent { percent, .. } if (*percent - 50.0).abs() < f32::EPSILON)));
        assert!(calls.iter().any(|call| matches!(call, Call::SetMinWidth { value, unit_enum, .. } if (*value - 12.0).abs() < f32::EPSILON && *unit_enum == Unit::Pixel as u32)));
        assert!(calls.iter().any(|call| matches!(call, Call::SetMaxWidth { value, unit_enum, .. } if (*value - 80.0).abs() < f32::EPSILON && *unit_enum == Unit::Percent as u32)));
        assert!(calls.iter().any(|call| matches!(call, Call::SetMinHeight { value, .. } if (*value - 14.0).abs() < f32::EPSILON)));
        assert!(calls.iter().any(|call| matches!(call, Call::SetMaxHeight { value, .. } if (*value - 90.0).abs() < f32::EPSILON)));

        label
            .text_align(TextAlign::Right)
            .text_vertical_align(TextVerticalAlign::Center)
            .text_overflow(TextOverflow::Clip)
            .text_overflow_fade(false, true);
        let calls = ffi::test::take_calls();
        assert!(calls.iter().any(|call| matches!(call, Call::SetTextAlign { align_enum, .. } if *align_enum == TextAlign::Right as u32)));
        assert!(calls.iter().any(|call| matches!(call, Call::SetTextVerticalAlign { align_enum, .. } if *align_enum == TextVerticalAlign::Center as u32)));
        assert!(calls.iter().any(|call| matches!(call, Call::SetTextOverflow { overflow_enum, .. } if *overflow_enum == TextOverflow::Clip as u32)));
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::SetTextOverflowFade {
                horizontal: false,
                vertical: true,
                ..
            }
        )));
        Application::unmount();
    }

    #[test]
    fn public_selection_uses_scalar_indices_at_the_utf8_boundary() {
        ffi::test::reset();
        let label = TextNode::new("A你😀Z");
        Application::mount(label.clone());
        ffi::test::take_calls();

        label.selection_range(2, 3);
        assert_eq!(label.selection_start(), 2);
        assert_eq!(label.selection_end(), 3);
        assert!(ffi::test::take_calls().iter().any(|call| matches!(
            call,
            Call::SetTextSelectionRange {
                start: 4,
                end: 8,
                ..
            }
        )));

        let observed = Rc::new(RefCell::new(None));
        label.on_selection_changed({
            let observed = observed.clone();
            move |event| *observed.borrow_mut() = Some((event.start, event.end))
        });
        crate::event::__fui_on_selection_changed(label.handle().raw(), 1, 8);
        assert_eq!(label.selection_start(), 1);
        assert_eq!(label.selection_end(), 3);
        assert_eq!(*observed.borrow(), Some((1, 3)));
        Application::unmount();
    }

    #[test]
    fn runtime_text_state_stays_synchronized_without_user_callbacks() {
        ffi::test::reset();
        let label = TextNode::new("A你😀Z");
        Application::mount(label.clone());
        let replacement = "界";
        unsafe {
            crate::event::__fui_on_text_replaced(
                label.handle().raw(),
                1,
                4,
                replacement.as_ptr(),
                replacement.len() as u32,
            );
        }
        assert_eq!(label.content(), "A界😀Z");

        let changed = "你好😀";
        unsafe {
            crate::event::__fui_on_text_changed(
                label.handle().raw(),
                changed.as_ptr(),
                changed.len() as u32,
            );
        }
        assert_eq!(label.content(), changed);
        Application::unmount();
    }
}
