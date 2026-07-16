use super::Button;
use crate::event;
use crate::ffi::{FlexDirection, KeyEventType, SemanticRole};
use crate::node::{flex_box, Child, FlexBox, Node, NodeRef};
use std::any::Any;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

#[derive(Clone)]
pub struct Form {
    shared: Rc<FormShared>,
}

struct FormShared {
    root: FlexBox,
    default_button: RefCell<Option<Button>>,
    cancel_button: RefCell<Option<Button>>,
    key_filter_token: Cell<u32>,
    armed_key: RefCell<Option<String>>,
    armed_button: RefCell<Option<Button>>,
}

impl Default for Form {
    fn default() -> Self {
        Self::new()
    }
}

impl Form {
    pub fn new() -> Self {
        let root = flex_box();
        root.flex_direction(FlexDirection::Column)
            .semantic_role(SemanticRole::Form);
        let form = Self {
            shared: Rc::new(FormShared {
                root,
                default_button: RefCell::new(None),
                cancel_button: RefCell::new(None),
                key_filter_token: Cell::new(0),
                armed_key: RefCell::new(None),
                armed_button: RefCell::new(None),
            }),
        };
        form.bind_events();
        form
    }

    pub fn default_btn(&self, button: &Button) -> &Self {
        self.shared
            .default_button
            .borrow_mut()
            .replace(button.clone());
        self
    }

    pub fn cancel_btn(&self, button: &Button) -> &Self {
        self.shared
            .cancel_button
            .borrow_mut()
            .replace(button.clone());
        self
    }

    pub fn child<T: Node>(&self, node: &T) -> &Self {
        self.shared.root.child(node);
        self
    }

    pub fn children<I, C>(&self, nodes: I) -> &Self
    where
        I: IntoIterator<Item = C>,
        C: Into<Child>,
    {
        for node in nodes {
            self.shared
                .root
                .retained_node_ref()
                .append_child_ref(&node.into().node_ref);
        }
        self
    }

    pub fn activate(&self) {
        if self.shared.key_filter_token.get() != 0
            || (self.shared.default_button.borrow().is_none()
                && self.shared.cancel_button.borrow().is_none())
        {
            return;
        }
        let weak = Rc::downgrade(&self.shared);
        let token = event::push_key_filter(move |event_type, key, modifiers| {
            let Some(shared) = weak.upgrade() else {
                return false;
            };
            shared.handle_global_key_event(event_type, key, modifiers)
        });
        self.shared.key_filter_token.set(token);
    }

    pub fn deactivate(&self) {
        self.shared.deactivate();
    }

    fn bind_events(&self) {
        let weak = Rc::downgrade(&self.shared);
        self.shared.root.on_key_down(move |event| {
            let Some(shared) = weak.upgrade() else {
                return;
            };
            if shared.handle_global_key_event(
                KeyEventType::Down,
                event.key.as_str(),
                event.modifiers,
            ) {
                event.handled = true;
            }
        });
        let weak = Rc::downgrade(&self.shared);
        self.shared.root.on_key_up(move |event| {
            let Some(shared) = weak.upgrade() else {
                return;
            };
            if shared.handle_global_key_event(KeyEventType::Up, event.key.as_str(), event.modifiers)
            {
                event.handled = true;
            }
        });
    }
}

impl Drop for Form {
    fn drop(&mut self) {
        if Rc::strong_count(&self.shared) == 1 {
            self.shared.deactivate();
        }
    }
}

impl FormShared {
    fn deactivate(&self) {
        self.cancel_armed_button();
        let token = self.key_filter_token.replace(0);
        if token != 0 {
            event::remove_key_filter(token);
        }
    }

    fn handle_global_key_event(&self, event_type: KeyEventType, key: &str, modifiers: u32) -> bool {
        if key == "Enter" && self.should_defer_enter_to_focused_button(modifiers) {
            return false;
        }
        match event_type {
            KeyEventType::Down => self.handle_key_down(key),
            KeyEventType::Up => self.handle_key_up(key),
        }
    }

    fn handle_key_down(&self, key: &str) -> bool {
        let Some(button) = self.resolve_button_for_key(key) else {
            return false;
        };
        if let Some(armed_key) = self.armed_key.borrow().as_ref() {
            return armed_key == key;
        }
        self.armed_key.borrow_mut().replace(key.to_string());
        self.armed_button.borrow_mut().replace(button.clone());
        button.begin_press();
        true
    }

    fn handle_key_up(&self, key: &str) -> bool {
        let armed_matches = self
            .armed_key
            .borrow()
            .as_ref()
            .is_some_and(|armed_key| armed_key == key);
        if !armed_matches {
            return false;
        }
        let button = self.armed_button.borrow_mut().take();
        self.armed_key.borrow_mut().take();
        if let Some(button) = button {
            button.end_press(true);
        }
        true
    }

    fn cancel_armed_button(&self) {
        let button = self.armed_button.borrow_mut().take();
        self.armed_key.borrow_mut().take();
        if let Some(button) = button {
            button.cancel_press();
        }
    }

    fn resolve_button_for_key(&self, key: &str) -> Option<Button> {
        match key {
            "Enter" => self.default_button.borrow().clone(),
            "Escape" => self.cancel_button.borrow().clone(),
            _ => None,
        }
    }

    fn should_defer_enter_to_focused_button(&self, modifiers: u32) -> bool {
        if modifiers != 0 {
            return false;
        }
        event::focused_node_is_enabled_button()
    }
}

impl Node for Form {
    fn retained_node_ref(&self) -> NodeRef {
        self.shared.root.retained_node_ref()
    }

    fn retained_owner_attachment(&self) -> Option<Rc<dyn Any>> {
        Some(self.shared.clone())
    }

    fn build_self(&self) {
        self.shared.root.build_self();
    }

    fn dispose(&self) {
        self.deactivate();
        self.shared.root.dispose();
    }
}

impl crate::node::HasFlexBoxRoot for Form {
    fn flex_box_root(&self) -> &FlexBox {
        &self.shared.root
    }
}
