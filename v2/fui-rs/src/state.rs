use crate::component::{current_owner, OwnerHandle};
use crate::signal::{Callback, Signal};
use std::cell::RefCell;
use std::rc::Rc;

struct StateInner<T> {
    signal: RefCell<Signal<T>>,
    owner: Rc<OwnerHandle>,
}

pub struct State<T> {
    inner: Rc<StateInner<T>>,
}

impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> State<T> {
    fn new(initial: T, owner: Rc<OwnerHandle>) -> Self {
        Self {
            inner: Rc::new(StateInner {
                signal: RefCell::new(Signal::new(initial)),
                owner,
            }),
        }
    }

    fn set_without_dirty(&self, next: T)
    where
        T: PartialEq,
    {
        let listeners = self.inner.signal.borrow_mut().set(next);
        if let Some(listeners) = listeners {
            for listener in listeners {
                listener();
            }
        }
    }

    pub(crate) fn subscribe(&self, callback: Callback) {
        self.inner.signal.borrow_mut().subscribe(callback);
    }
}

impl<T: Clone> State<T> {
    pub fn get(&self) -> T {
        self.inner.signal.borrow().get()
    }
}

impl<T: PartialEq> State<T> {
    pub fn set(&self, next: T) {
        let listeners = self.inner.signal.borrow_mut().set(next);
        if let Some(listeners) = listeners {
            for listener in listeners {
                listener();
            }
            self.inner.owner.notify_dirty();
        }
    }
}

pub fn state<T: Clone + PartialEq + 'static>(initial: T) -> State<T> {
    State::new(initial, current_owner())
}

pub fn derived<T, U, F>(source: &State<U>, compute: F) -> State<T>
where
    T: Clone + PartialEq + 'static,
    U: Clone + PartialEq + 'static,
    F: Fn(U) -> T + 'static,
{
    let derived_state = State::new(compute(source.get()), current_owner());
    let source_state = source.clone();
    let derived_clone = derived_state.clone();
    let callback: Callback = Rc::new(move || {
        derived_clone.set_without_dirty(compute(source_state.get()));
    });
    source.subscribe(callback);
    derived_state
}
