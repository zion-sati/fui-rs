use crate::bindings::ui;
use crate::signal::{Callback, Signal, SubscriptionGuard};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Copy)]
enum ViewportAxis {
    Width,
    Height,
}

#[derive(Clone, Copy)]
pub struct ViewportSignalHandle {
    axis: ViewportAxis,
}

thread_local! {
    static VIEWPORT_WIDTH_SIGNAL: RefCell<Signal<f32>> = RefCell::new(Signal::new(ui::get_viewport_width()));
    static VIEWPORT_HEIGHT_SIGNAL: RefCell<Signal<f32>> = RefCell::new(Signal::new(ui::get_viewport_height()));
}

fn update_signal(signal: &RefCell<Signal<f32>>, next: f32) {
    let callbacks = signal.borrow_mut().set(next);
    if let Some(callbacks) = callbacks {
        for callback in callbacks {
            callback();
        }
    }
}

impl ViewportSignalHandle {
    pub fn value(&self) -> f32 {
        match self.axis {
            ViewportAxis::Width => VIEWPORT_WIDTH_SIGNAL.with(|slot| slot.borrow().get()),
            ViewportAxis::Height => VIEWPORT_HEIGHT_SIGNAL.with(|slot| slot.borrow().get()),
        }
    }

    pub fn subscribe(&self, handler: impl Fn(f32) + 'static) -> SubscriptionGuard {
        handler(self.value());
        match self.axis {
            ViewportAxis::Width => VIEWPORT_WIDTH_SIGNAL.with(|slot| {
                let callback: Callback = Rc::new(move || handler(viewport_width_signal().value()));
                slot.borrow_mut().subscribe(callback)
            }),
            ViewportAxis::Height => VIEWPORT_HEIGHT_SIGNAL.with(|slot| {
                let callback: Callback = Rc::new(move || handler(viewport_height_signal().value()));
                slot.borrow_mut().subscribe(callback)
            }),
        }
    }
}

pub fn viewport_width_signal() -> ViewportSignalHandle {
    ViewportSignalHandle {
        axis: ViewportAxis::Width,
    }
}

pub fn viewport_height_signal() -> ViewportSignalHandle {
    ViewportSignalHandle {
        axis: ViewportAxis::Height,
    }
}

pub(crate) fn set_viewport_size(width: f32, height: f32) {
    VIEWPORT_WIDTH_SIGNAL.with(|slot| update_signal(slot, width));
    VIEWPORT_HEIGHT_SIGNAL.with(|slot| update_signal(slot, height));
}

#[cfg(test)]
mod tests {
    use super::{set_viewport_size, viewport_height_signal, viewport_width_signal};
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn viewport_signals_update_and_notify() {
        let width_values = Rc::new(Cell::new(0));
        let height_values = Rc::new(Cell::new(0));

        let width_seen = Rc::new(Cell::new(0.0));
        let height_seen = Rc::new(Cell::new(0.0));

        let width_values_clone = width_values.clone();
        let width_seen_clone = width_seen.clone();
        let _width_guard = viewport_width_signal().subscribe(move |value| {
            width_values_clone.set(width_values_clone.get() + 1);
            width_seen_clone.set(value);
        });

        let height_values_clone = height_values.clone();
        let height_seen_clone = height_seen.clone();
        let _height_guard = viewport_height_signal().subscribe(move |value| {
            height_values_clone.set(height_values_clone.get() + 1);
            height_seen_clone.set(value);
        });

        set_viewport_size(777.0, 555.0);

        assert_eq!(viewport_width_signal().value(), 777.0);
        assert_eq!(viewport_height_signal().value(), 555.0);
        assert_eq!(width_seen.get(), 777.0);
        assert_eq!(height_seen.get(), 555.0);
        assert!(width_values.get() >= 2);
        assert!(height_values.get() >= 2);
    }
}
