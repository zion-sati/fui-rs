use super::internal::text_input_core::TextInputCore;
use super::text_editor_surface::impl_text_editor_surface;
use crate::event::FocusChangedEventArgs;
use crate::node::{FlexBox, HasFlexBoxRoot, Node, NodeRef, TextNode};
use std::any::Any;
use std::rc::Rc;

#[derive(Clone)]
pub struct TextInput {
    core: Rc<TextInputCore>,
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

impl TextInput {
    pub fn new() -> Self {
        let core = Rc::new(TextInputCore::new());
        core.finish_init(Rc::downgrade(&core));
        Self { core }
    }

    pub fn password(&self, flag: bool) -> &Self {
        self.core.password(flag);
        self
    }

    pub fn host_autofill(&self, hint: impl AsRef<str>) -> &Self {
        self.core.host_autofill(Some(hint.as_ref()));
        self
    }

    pub fn clear_host_autofill(&self) -> &Self {
        self.core.host_autofill(None);
        self
    }

    pub(crate) fn editor_node(&self) -> TextNode {
        self.core.editor_node()
    }
}

impl_text_editor_surface!(TextInput);

impl HasFlexBoxRoot for TextInput {
    fn flex_box_root(&self) -> &FlexBox {
        self.core.flex_box_root()
    }
}

impl crate::ThemeBindable for TextInput {
    fn theme_binding_node(&self) -> NodeRef {
        self.core.flex_box_root().retained_node_ref()
    }

    fn weak_theme_target(&self) -> Box<dyn Fn() -> Option<Self>> {
        let weak_core = Rc::downgrade(&self.core);
        Box::new(move || {
            Some(TextInput {
                core: weak_core.upgrade()?,
            })
        })
    }
}

impl Node for TextInput {
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
