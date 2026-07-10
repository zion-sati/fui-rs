use super::internal::text_input_core::TextInputCore;
use super::internal::text_input_presenter::TextInputTemplate;
use super::TextInputColors;
use crate::event::{FocusChangedEventArgs, SelectionChangedEventArgs, TextChangedEventArgs};
use crate::node::{FlexBox, HasFlexBoxRoot, Node, NodeRef, ScrollBarVisibility};
use crate::FontFamily;
use std::any::Any;
use std::rc::Rc;

#[derive(Clone)]
pub struct TextArea {
    core: Rc<TextInputCore>,
}

impl Default for TextArea {
    fn default() -> Self {
        Self::new()
    }
}

impl TextArea {
    pub fn new() -> Self {
        let core = Rc::new(TextInputCore::multiline());
        core.finish_init(Rc::downgrade(&core));
        Self { core }
    }

    pub fn text(&self, value: impl Into<String>) -> &Self {
        self.core.text(value);
        self
    }

    pub fn value(&self) -> String {
        self.core.value()
    }

    pub fn selection_start(&self) -> u32 {
        self.core.selection_start()
    }

    pub fn selection_end(&self) -> u32 {
        self.core.selection_end()
    }

    pub fn selection_start_byte_offset(&self) -> u32 {
        self.core.selection_start_byte_offset()
    }

    pub fn selection_end_byte_offset(&self) -> u32 {
        self.core.selection_end_byte_offset()
    }

    pub fn placeholder(&self, value: impl Into<String>) -> &Self {
        self.core.placeholder(value);
        self
    }

    pub fn max_chars(&self, limit: i32) -> &Self {
        self.core.max_chars(limit);
        self
    }

    pub fn read_only(&self, flag: bool) -> &Self {
        self.core.read_only(flag);
        self
    }

    pub fn accepts_tab(&self, flag: bool) -> &Self {
        self.core.accepts_tab(flag);
        self
    }

    pub fn selection_range(&self, start: u32, end: u32) -> &Self {
        self.core.selection_range(start, end);
        self
    }

    pub fn caret(&self, position: u32) -> &Self {
        self.core.caret(position);
        self
    }

    pub fn caret_to_end(&self) -> &Self {
        self.core.caret_to_end();
        self
    }

    pub fn colors(&self, colors: Option<TextInputColors>) -> &Self {
        self.core.colors(colors);
        self
    }

    pub fn template(&self, template: Option<Rc<dyn TextInputTemplate>>) -> &Self {
        self.core.template(template);
        self
    }

    pub fn enabled(&self, enabled: bool) -> &Self {
        self.core.enabled(enabled);
        self
    }

    pub fn focusable(&self, enabled: bool, tab_index: i32) -> &Self {
        self.core.focusable(enabled, tab_index);
        self
    }

    pub fn node_id(&self, id: impl Into<String>) -> &Self {
        self.core.node_id(id);
        self
    }

    pub fn line_height(&self, value: f32) -> &Self {
        self.core.line_height(value);
        self
    }

    pub fn font_family(&self, family: FontFamily) -> &Self {
        self.core.font_family(family);
        self
    }

    pub fn font_size(&self, size: f32) -> &Self {
        self.core.font_size(size);
        self
    }

    pub fn wrapping(&self, flag: bool) -> &Self {
        self.core.wrapping(flag);
        self
    }

    pub fn vertical_scrollbar_visibility(&self, mode: ScrollBarVisibility) -> &Self {
        self.core.vertical_scrollbar_visibility(mode);
        self
    }

    pub fn horizontal_scrollbar_visibility(&self, mode: ScrollBarVisibility) -> &Self {
        self.core.horizontal_scrollbar_visibility(mode);
        self
    }

    pub fn on_changed(&self, handler: impl Fn(TextChangedEventArgs) + 'static) -> &Self {
        self.core.on_changed(handler);
        self
    }

    pub fn on_text_changed(&self, handler: impl Fn(TextChangedEventArgs) + 'static) -> &Self {
        self.core.on_text_changed(handler);
        self
    }

    pub fn on_selection_changed(
        &self,
        handler: impl Fn(SelectionChangedEventArgs) + 'static,
    ) -> &Self {
        self.core.on_selection_changed(handler);
        self
    }

    pub fn on_focus_changed(&self, handler: impl Fn(FocusChangedEventArgs) + 'static) -> &Self {
        self.core.on_focus_changed(handler);
        self
    }

    pub fn focus_now(&self) -> &Self {
        self.core.focus_now();
        self
    }

    pub fn scroll_offset_x(&self) -> f32 {
        self.core.scroll_offset_x()
    }

    pub fn scroll_offset_y(&self) -> f32 {
        self.core.scroll_offset_y()
    }

    pub fn scroll_to(&self, x: f32, y: f32) -> &Self {
        self.core.scroll_to(x, y);
        self
    }
}

impl HasFlexBoxRoot for TextArea {
    fn flex_box_root(&self) -> &FlexBox {
        self.core.flex_box_root()
    }
}

impl Node for TextArea {
    fn retained_node_ref(&self) -> NodeRef {
        let core = self.core.clone();
        self.core
            .flex_box_root()
            .retained_node_ref()
            .with_build_callback(move || core.build_control())
    }

    fn retained_owner_attachment(&self) -> Option<Rc<dyn Any>> {
        Some(self.core.clone())
    }

    fn build_self(&self) {
        self.core.build_control();
    }
}
