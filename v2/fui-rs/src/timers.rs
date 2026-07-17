use crate::ffi;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;

type TimerCallback = Box<dyn Fn()>;

thread_local! {
    static NEXT_TIMER_ID: Cell<u32> = const { Cell::new(1) };
    static ACTIVE_TIMERS: RefCell<HashMap<u32, TimerCallback>> = RefCell::new(HashMap::new());
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TimerHandle(u32);

impl TimerHandle {
    pub fn raw(self) -> u32 {
        self.0
    }
}

pub fn set_timeout(delay_ms: i32, callback: impl Fn() + 'static) -> TimerHandle {
    let timer_id = NEXT_TIMER_ID.with(|next| {
        let timer_id = next.get();
        next.set(timer_id.saturating_add(1));
        timer_id
    });
    ACTIVE_TIMERS.with(|timers| {
        timers.borrow_mut().insert(timer_id, Box::new(callback));
    });
    unsafe { ffi::fui_start_timer(timer_id, delay_ms) };
    TimerHandle(timer_id)
}

pub(crate) fn schedule_internal_timer(timer_id: u32, delay_ms: i32, callback: impl Fn() + 'static) {
    ACTIVE_TIMERS.with(|timers| {
        timers.borrow_mut().insert(timer_id, Box::new(callback));
    });
    unsafe { ffi::fui_start_timer(timer_id, delay_ms) };
}

pub(crate) fn cancel_internal_timer(timer_id: u32) -> bool {
    let removed = ACTIVE_TIMERS.with(|timers| timers.borrow_mut().remove(&timer_id).is_some());
    if removed {
        unsafe { ffi::fui_cancel_timer(timer_id) };
    }
    removed
}

pub fn cancel_timeout(handle: TimerHandle) -> bool {
    let removed = ACTIVE_TIMERS.with(|timers| timers.borrow_mut().remove(&handle.0).is_some());
    if removed {
        unsafe { ffi::fui_cancel_timer(handle.0) };
    }
    removed
}

pub fn cancel_all_timers() {
    ACTIVE_TIMERS.with(|timers| timers.borrow_mut().clear());
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
pub extern "C" fn __fui_on_timer(timer_id: u32) {
    let callback = ACTIVE_TIMERS.with(|timers| timers.borrow_mut().remove(&timer_id));
    if let Some(callback) = callback {
        callback();
    }
}

#[cfg(test)]
mod tests {
    use crate::ffi::{self, Call};
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn timer_callback_fires_once() {
        ffi::test::reset();
        let fired = Rc::new(Cell::new(0));
        let handle = super::set_timeout(25, || {});
        let timer_id = handle.raw();
        let calls = ffi::test::take_calls();
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::StartTimer { timer_id: captured, delay_ms } if *captured == timer_id && *delay_ms == 25
        )));

        let fired_clone = fired.clone();
        let handle = super::set_timeout(10, move || fired_clone.set(fired_clone.get() + 1));
        super::__fui_on_timer(handle.raw());
        super::__fui_on_timer(handle.raw());
        assert_eq!(fired.get(), 1);
    }

    #[test]
    fn cancel_timeout_emits_host_cancel() {
        ffi::test::reset();
        let handle = super::set_timeout(100, || {});
        ffi::test::take_calls();
        assert!(super::cancel_timeout(handle));
        let calls = ffi::test::take_calls();
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::CancelTimer { timer_id } if *timer_id == handle.raw()
        )));
    }
}
