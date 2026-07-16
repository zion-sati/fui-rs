use super::*;
use crate::signal::{Callback, Signal, Subscription};

#[derive(Clone)]
pub struct SelectionArea {
    root: FlexBox,
    selected_text_signal: Rc<RefCell<Signal<String>>>,
}

impl SelectionArea {
    pub fn new() -> Self {
        let root = flex_box();
        root.selection_area(true);
        let selected_text_signal = Rc::new(RefCell::new(Signal::new(String::new())));
        let signal = selected_text_signal.clone();
        root.core.borrow_mut().handlers.cross_selection_changed = Some(Rc::new(move |text| {
            let callbacks = signal.borrow_mut().set(text);
            if let Some(callbacks) = callbacks {
                for callback in callbacks {
                    callback();
                }
            }
        }));
        Self {
            root,
            selected_text_signal,
        }
    }

    pub fn child<T: Node>(&self, node: &T) -> &Self {
        self.root.child(node);
        self
    }

    pub fn children<I, C>(&self, nodes: I) -> &Self
    where
        I: IntoIterator<Item = C>,
        C: Into<Child>,
    {
        for node in nodes {
            self.root
                .retained_node_ref()
                .append_child_ref(&node.into().node_ref);
        }
        self
    }

    pub fn selected_text(&self) -> String {
        self.selected_text_signal.borrow().get()
    }

    pub fn subscribe_selected_text(&self, handler: impl Fn(String) + 'static) -> Subscription {
        let signal = self.selected_text_signal.clone();
        let callback: Callback = Rc::new(move || handler(signal.borrow().get()));
        self.selected_text_signal.borrow_mut().subscribe(callback)
    }
}

impl Default for SelectionArea {
    fn default() -> Self {
        Self::new()
    }
}

impl Node for SelectionArea {
    fn retained_node_ref(&self) -> NodeRef {
        let selection_area = self.clone();
        self.root
            .retained_node_ref()
            .with_build_callback(move || selection_area.build())
    }

    fn build_self(&self) {
        self.root.build_self();
    }
}

impl HasFlexBoxRoot for SelectionArea {
    fn flex_box_root(&self) -> &FlexBox {
        &self.root
    }
}

impl ThemeBindable for SelectionArea {
    fn theme_binding_node(&self) -> NodeRef {
        self.root.retained_node_ref()
    }

    fn weak_theme_target(&self) -> Box<dyn Fn() -> Option<Self>> {
        let weak_root = self.root.downgrade();
        let signal = self.selected_text_signal.clone();
        Box::new(move || {
            Some(SelectionArea {
                root: weak_root.upgrade()?,
                selected_text_signal: signal.clone(),
            })
        })
    }
}
