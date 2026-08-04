use crate::node::{Node, NodeRef};
use std::any::Any;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

type LifecycleHandler = Box<dyn Fn(&RetainedView)>;
type DisposeHandler = Box<dyn FnOnce()>;

struct RetainedViewInner {
    root: NodeRef,
    active: Cell<bool>,
    disposed: Cell<bool>,
    resources: RefCell<Vec<Box<dyn Any>>>,
    activation_handlers: RefCell<Vec<LifecycleHandler>>,
    deactivation_handlers: RefCell<Vec<LifecycleHandler>>,
    disposal_handlers: RefCell<Vec<DisposeHandler>>,
}

impl Drop for RetainedViewInner {
    fn drop(&mut self) {
        if !self.disposed.get() {
            self.disposed.set(true);
            for handler in self.disposal_handlers.get_mut().drain(..) {
                handler();
            }
            self.resources.get_mut().clear();
            self.root.dispose();
        }
    }
}

#[derive(Clone)]
pub struct RetainedView {
    inner: Rc<RetainedViewInner>,
}

impl RetainedView {
    pub fn new<T: Node + 'static>(root: &T) -> Self {
        Self {
            inner: Rc::new(RetainedViewInner {
                root: root.node_ref(),
                active: Cell::new(false),
                disposed: Cell::new(false),
                resources: RefCell::new(vec![Box::new(root.clone())]),
                activation_handlers: RefCell::new(Vec::new()),
                deactivation_handlers: RefCell::new(Vec::new()),
                disposal_handlers: RefCell::new(Vec::new()),
            }),
        }
    }

    pub fn root(&self) -> NodeRef {
        self.inner.root.clone()
    }
    pub fn is_active(&self) -> bool {
        self.inner.active.get()
    }
    pub fn is_disposed(&self) -> bool {
        self.inner.disposed.get()
    }

    pub fn keep_alive<T: 'static>(self, resource: T) -> Self {
        if !self.inner.disposed.get() {
            self.inner.resources.borrow_mut().push(Box::new(resource));
        }
        self
    }

    pub fn on_activate(self, handler: impl Fn(&RetainedView) + 'static) -> Self {
        self.inner
            .activation_handlers
            .borrow_mut()
            .push(Box::new(handler));
        self
    }

    pub fn on_deactivate(self, handler: impl Fn(&RetainedView) + 'static) -> Self {
        self.inner
            .deactivation_handlers
            .borrow_mut()
            .push(Box::new(handler));
        self
    }

    pub fn on_dispose(self, handler: impl FnOnce() + 'static) -> Self {
        self.inner
            .disposal_handlers
            .borrow_mut()
            .push(Box::new(handler));
        self
    }

    pub fn activate(&self) {
        if self.inner.disposed.get() || self.inner.active.replace(true) {
            return;
        }
        for handler in self.inner.activation_handlers.borrow().iter() {
            handler(self);
        }
    }

    pub fn deactivate(&self) {
        if self.inner.disposed.get() || !self.inner.active.replace(false) {
            return;
        }
        crate::event::deactivate_subtree(&self.inner.root);
        for handler in self.inner.deactivation_handlers.borrow().iter() {
            handler(self);
        }
    }

    pub fn dispose(&self) {
        if self.inner.disposed.replace(true) {
            return;
        }
        if self.inner.active.replace(false) {
            crate::event::deactivate_subtree(&self.inner.root);
            for handler in self.inner.deactivation_handlers.borrow().iter() {
                handler(self);
            }
        }
        for handler in std::mem::take(&mut *self.inner.disposal_handlers.borrow_mut()) {
            handler();
        }
        self.inner.resources.borrow_mut().clear();
        self.inner.root.dispose();
        self.inner.activation_handlers.borrow_mut().clear();
        self.inner.deactivation_handlers.borrow_mut().clear();
    }
}

pub fn retained_view<T: Node + 'static>(root: &T) -> RetainedView {
    RetainedView::new(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::{self, Call};
    use crate::node::{flex_box, Node, ThemeBindable};
    use crate::theme::{current_theme, generate_theme, use_custom_theme};
    use std::cell::RefCell;

    struct DropProbe(Rc<Cell<u32>>);
    impl Drop for DropProbe {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }

    #[test]
    fn retained_view_orders_lifecycle_retains_state_and_disposes_once() {
        let root = flex_box();
        let events = Rc::new(RefCell::new(Vec::new()));
        let drops = Rc::new(Cell::new(0));
        let view = retained_view(&root)
            .keep_alive(DropProbe(drops.clone()))
            .on_activate({
                let events = events.clone();
                move |_| events.borrow_mut().push("activate")
            })
            .on_deactivate({
                let events = events.clone();
                move |_| events.borrow_mut().push("deactivate")
            })
            .on_dispose({
                let events = events.clone();
                move || events.borrow_mut().push("dispose")
            });

        view.activate();
        view.activate();
        view.deactivate();
        view.activate();
        view.dispose();
        view.dispose();

        assert_eq!(
            &*events.borrow(),
            &[
                "activate",
                "deactivate",
                "activate",
                "deactivate",
                "dispose"
            ]
        );
        assert_eq!(drops.get(), 1);
        assert!(view.is_disposed());
        assert!(!view.is_active());
        assert!(!root.has_built_handle());
    }

    #[test]
    fn final_owner_drop_disposes_retained_resources_and_root() {
        let root = flex_box();
        root.build();
        let drops = Rc::new(Cell::new(0));
        {
            let _view = retained_view(&root).keep_alive(DropProbe(drops.clone()));
        }
        assert_eq!(drops.get(), 1);
        assert!(!root.has_built_handle());
    }

    #[test]
    fn retained_view_keeps_concrete_root_theme_binding_alive() {
        ffi::test::reset();
        let previous_theme = current_theme();
        let root = flex_box();
        root.bind_theme(|root, theme| {
            root.bg_color(theme.colors.surface);
        });
        root.build();
        let handle = root.handle().raw();
        let view = retained_view(&root);
        drop(root);
        ffi::test::take_calls();

        let changed = generate_theme(false, 0x2468ACFF);
        use_custom_theme(changed.clone());
        let calls = ffi::test::take_calls();
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::SetBgColor {
                handle: actual,
                color,
            } if *actual == handle && *color == changed.colors.surface
        )));

        view.dispose();
        use_custom_theme(previous_theme);
    }
}
