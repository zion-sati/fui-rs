use super::internal::text_input_core::TextInputCore;
use super::text_editor_surface::impl_text_editor_surface;
use crate::event::FocusChangedEventArgs;
use crate::node::{FlexBox, HasFlexBoxRoot, Node, NodeRef, ScrollBarVisibility};
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

impl_text_editor_surface!(TextArea);

impl HasFlexBoxRoot for TextArea {
    fn flex_box_root(&self) -> &FlexBox {
        self.core.flex_box_root()
    }
}

impl crate::ThemeBindable for TextArea {
    fn theme_binding_node(&self) -> NodeRef {
        self.core.flex_box_root().retained_node_ref()
    }

    fn weak_theme_target(&self) -> Box<dyn Fn() -> Option<Self>> {
        let weak_core = Rc::downgrade(&self.core);
        Box::new(move || {
            Some(TextArea {
                core: weak_core.upgrade()?,
            })
        })
    }
}

impl Node for TextArea {
    fn apply_node_id(&self, node_id: String) {
        self.core.node_id(node_id);
    }

    fn apply_semantic_label(&self, label: String) {
        self.core.semantic_label(label);
    }

    fn apply_focusable(&self, enabled: bool, tab_index: i32) {
        self.core.focusable(enabled, tab_index);
    }

    fn apply_focus_now(&self) {
        self.core.focus_now();
    }

    fn apply_enabled(&self, enabled: bool) {
        self.core.enabled(enabled);
    }

    fn apply_focus_changed_handler(&self, handler: Rc<dyn Fn(FocusChangedEventArgs)>) {
        self.core.set_focus_changed_callback(handler);
    }

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
