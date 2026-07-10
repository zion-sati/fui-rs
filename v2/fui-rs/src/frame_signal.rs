use crate::signal::{Callback, Signal, SubscriptionGuard};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Copy)]
pub struct FrameTimeSignalHandle;

thread_local! {
    static FRAME_TIME_SIGNAL: RefCell<Signal<f64>> = RefCell::new(Signal::new(0.0));
}

impl FrameTimeSignalHandle {
    pub fn value(&self) -> f64 {
        FRAME_TIME_SIGNAL.with(|slot| slot.borrow().get())
    }

    pub fn subscribe(&self, handler: impl Fn(f64) + 'static) -> SubscriptionGuard {
        handler(self.value());
        FRAME_TIME_SIGNAL.with(|slot| {
            let callback: Callback = Rc::new(move || handler(frame_time_signal().value()));
            slot.borrow_mut().subscribe(callback)
        })
    }
}

pub fn frame_time_signal() -> FrameTimeSignalHandle {
    FrameTimeSignalHandle
}

pub(crate) fn set_frame_time(timestamp_ms: f64) {
    FRAME_TIME_SIGNAL.with(|slot| {
        let callbacks = slot.borrow_mut().set(timestamp_ms);
        if let Some(callbacks) = callbacks {
            for callback in callbacks {
                callback();
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{frame_time_signal, set_frame_time};
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn updates_frame_time_signal_and_notifies_subscribers() {
        let count = Rc::new(Cell::new(0));
        let seen = Rc::new(Cell::new(0.0));
        let count_for_handler = count.clone();
        let seen_for_handler = seen.clone();
        let _guard = frame_time_signal().subscribe(move |value| {
            count_for_handler.set(count_for_handler.get() + 1);
            seen_for_handler.set(value);
        });

        set_frame_time(1234.5);

        assert_eq!(frame_time_signal().value(), 1234.5);
        assert_eq!(seen.get(), 1234.5);
        assert!(count.get() >= 2);
    }
}
