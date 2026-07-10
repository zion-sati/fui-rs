use crate::signal::{Callback, Signal, Subscription};
use std::cell::RefCell;
use std::rc::Rc;

fn set_signal(signal: &Rc<RefCell<Signal<f32>>>, next: f32) {
    let callbacks = signal.borrow_mut().set(next);
    if let Some(callbacks) = callbacks {
        for callback in callbacks {
            callback();
        }
    }
}

fn subscribe_signal(
    signal: &Rc<RefCell<Signal<f32>>>,
    handler: impl Fn() + 'static,
) -> Subscription {
    let callback: Callback = Rc::new(handler);
    signal.borrow_mut().subscribe(callback)
}

#[derive(Clone)]
pub struct ScrollState {
    offset_x: Rc<RefCell<Signal<f32>>>,
    offset_y: Rc<RefCell<Signal<f32>>>,
    content_width: Rc<RefCell<Signal<f32>>>,
    content_height: Rc<RefCell<Signal<f32>>>,
    viewport_width: Rc<RefCell<Signal<f32>>>,
    viewport_height: Rc<RefCell<Signal<f32>>>,
}

impl Default for ScrollState {
    fn default() -> Self {
        Self::new()
    }
}

impl ScrollState {
    pub fn new() -> Self {
        Self {
            offset_x: Rc::new(RefCell::new(Signal::new(0.0))),
            offset_y: Rc::new(RefCell::new(Signal::new(0.0))),
            content_width: Rc::new(RefCell::new(Signal::new(0.0))),
            content_height: Rc::new(RefCell::new(Signal::new(0.0))),
            viewport_width: Rc::new(RefCell::new(Signal::new(0.0))),
            viewport_height: Rc::new(RefCell::new(Signal::new(0.0))),
        }
    }

    pub fn offset_x(&self) -> f32 {
        self.offset_x.borrow().get()
    }

    pub fn offset_y(&self) -> f32 {
        self.offset_y.borrow().get()
    }

    pub fn content_width(&self) -> f32 {
        self.content_width.borrow().get()
    }

    pub fn content_height(&self) -> f32 {
        self.content_height.borrow().get()
    }

    pub fn viewport_width(&self) -> f32 {
        self.viewport_width.borrow().get()
    }

    pub fn viewport_height(&self) -> f32 {
        self.viewport_height.borrow().get()
    }

    pub fn set_offset_x(&self, next: f32) {
        set_signal(&self.offset_x, next);
    }

    pub fn set_offset_y(&self, next: f32) {
        set_signal(&self.offset_y, next);
    }

    pub fn set_content_width(&self, next: f32) {
        set_signal(&self.content_width, next);
    }

    pub fn set_content_height(&self, next: f32) {
        set_signal(&self.content_height, next);
    }

    pub fn set_viewport_width(&self, next: f32) {
        set_signal(&self.viewport_width, next);
    }

    pub fn set_viewport_height(&self, next: f32) {
        set_signal(&self.viewport_height, next);
    }

    pub fn subscribe_offset_x(&self, handler: impl Fn() + 'static) -> Subscription {
        subscribe_signal(&self.offset_x, handler)
    }

    pub fn subscribe_offset_y(&self, handler: impl Fn() + 'static) -> Subscription {
        subscribe_signal(&self.offset_y, handler)
    }

    pub fn subscribe_content_width(&self, handler: impl Fn() + 'static) -> Subscription {
        subscribe_signal(&self.content_width, handler)
    }

    pub fn subscribe_content_height(&self, handler: impl Fn() + 'static) -> Subscription {
        subscribe_signal(&self.content_height, handler)
    }

    pub fn subscribe_viewport_width(&self, handler: impl Fn() + 'static) -> Subscription {
        subscribe_signal(&self.viewport_width, handler)
    }

    pub fn subscribe_viewport_height(&self, handler: impl Fn() + 'static) -> Subscription {
        subscribe_signal(&self.viewport_height, handler)
    }
}
