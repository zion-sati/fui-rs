use crate::bindings::ui;
use std::cell::{Cell, RefCell};

type LoadedCallback = Box<dyn FnOnce(LoadedEventArgs)>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LoadedEventArgs;

impl LoadedEventArgs {
    pub const EMPTY: Self = Self;
}

thread_local! {
    static NEEDS_COMMIT: Cell<bool> = const { Cell::new(false) };
    static FLUSH_SCHEDULED: Cell<bool> = const { Cell::new(false) };
    static DID_FIRST_COMMIT: Cell<bool> = const { Cell::new(false) };
    static LOADED_CALLBACKS: RefCell<Vec<LoadedCallback>> = const { RefCell::new(Vec::new()) };
}

pub fn mark_needs_commit() {
    NEEDS_COMMIT.with(|needs_commit| needs_commit.set(true));
    let already_scheduled = FLUSH_SCHEDULED.with(|scheduled| {
        let already_scheduled = scheduled.get();
        if !already_scheduled {
            scheduled.set(true);
        }
        already_scheduled
    });
    if !already_scheduled {
        ui::request_render();
    }
}

pub fn on_loaded(callback: impl FnOnce(LoadedEventArgs) + 'static) {
    if DID_FIRST_COMMIT.with(Cell::get) {
        callback(LoadedEventArgs::EMPTY);
        return;
    }
    LOADED_CALLBACKS.with(|callbacks| callbacks.borrow_mut().push(Box::new(callback)));
}

pub(crate) fn fire_loaded_callbacks() {
    if DID_FIRST_COMMIT.with(Cell::get) {
        return;
    }
    DID_FIRST_COMMIT.with(|did_first_commit| did_first_commit.set(true));
    let callbacks = LOADED_CALLBACKS.with(|callbacks| std::mem::take(&mut *callbacks.borrow_mut()));
    for callback in callbacks {
        callback(LoadedEventArgs::EMPTY);
    }
}

pub(crate) fn flush_commit() -> bool {
    FLUSH_SCHEDULED.with(|scheduled| scheduled.set(false));
    let needs_commit = NEEDS_COMMIT.with(|slot| {
        let needs_commit = slot.get();
        if needs_commit {
            slot.set(false);
        }
        needs_commit
    });
    if !needs_commit {
        return false;
    }
    ui::commit_frame();
    true
}

pub(crate) fn reset_commit_state() {
    NEEDS_COMMIT.with(|needs_commit| needs_commit.set(false));
    FLUSH_SCHEDULED.with(|scheduled| scheduled.set(false));
    DID_FIRST_COMMIT.with(|did_first_commit| did_first_commit.set(false));
    LOADED_CALLBACKS.with(|callbacks| callbacks.borrow_mut().clear());
}

pub(crate) fn reset_commit_state_preserving_loaded_callbacks() {
    NEEDS_COMMIT.with(|needs_commit| needs_commit.set(false));
    FLUSH_SCHEDULED.with(|scheduled| scheduled.set(false));
    DID_FIRST_COMMIT.with(|did_first_commit| did_first_commit.set(false));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::{self, Call};
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn on_loaded_fires_once_and_late_registration_fires_immediately() {
        reset_commit_state();
        let count = Rc::new(Cell::new(0));
        on_loaded({
            let count = count.clone();
            move |_| count.set(count.get() + 1)
        });

        fire_loaded_callbacks();
        fire_loaded_callbacks();
        assert_eq!(count.get(), 1);

        on_loaded({
            let count = count.clone();
            move |_| count.set(count.get() + 1)
        });
        assert_eq!(count.get(), 2);
    }

    #[test]
    fn mark_needs_commit_coalesces_render_requests_until_flush() {
        reset_commit_state();
        ffi::test::reset();

        mark_needs_commit();
        mark_needs_commit();
        let calls = ffi::test::take_calls();
        assert_eq!(
            calls
                .iter()
                .filter(|call| matches!(call, Call::RequestRender))
                .count(),
            1
        );

        assert!(flush_commit());
        let _ = ffi::test::take_calls();
        mark_needs_commit();
        let calls = ffi::test::take_calls();
        assert_eq!(
            calls
                .iter()
                .filter(|call| matches!(call, Call::RequestRender))
                .count(),
            1
        );
    }

    #[test]
    fn flush_commit_noops_without_pending_work() {
        reset_commit_state();
        ffi::test::reset();

        assert!(!flush_commit());
        assert!(ffi::test::take_calls()
            .iter()
            .all(|call| !matches!(call, Call::CommitFrame)));

        mark_needs_commit();
        assert!(flush_commit());
        assert!(ffi::test::take_calls()
            .iter()
            .any(|call| matches!(call, Call::CommitFrame)));

        assert!(!flush_commit());
        assert!(ffi::test::take_calls()
            .iter()
            .all(|call| !matches!(call, Call::CommitFrame)));
    }
}
