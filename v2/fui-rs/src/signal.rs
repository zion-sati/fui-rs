use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub type Callback = Rc<dyn Fn()>;

struct ListenerEntry {
    id: usize,
    callback: Callback,
}

struct ListenerStore {
    next_id: usize,
    listeners: Vec<ListenerEntry>,
}

impl ListenerStore {
    fn new() -> Self {
        Self {
            next_id: 1,
            listeners: Vec::new(),
        }
    }

    fn subscribe(&mut self, callback: Callback) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.listeners.push(ListenerEntry { id, callback });
        id
    }

    fn unsubscribe(&mut self, id: usize) {
        self.listeners.retain(|listener| listener.id != id);
    }

    fn callbacks(&self) -> Vec<Callback> {
        self.listeners
            .iter()
            .map(|listener| listener.callback.clone())
            .collect()
    }
}

#[must_use = "subscription is removed when the guard is dropped"]
pub struct SubscriptionGuard {
    store: Weak<RefCell<ListenerStore>>,
    id: usize,
}

pub type Subscription = SubscriptionGuard;

impl SubscriptionGuard {
    fn new(store: &Rc<RefCell<ListenerStore>>, id: usize) -> Self {
        Self {
            store: Rc::downgrade(store),
            id,
        }
    }
}

impl Drop for SubscriptionGuard {
    fn drop(&mut self) {
        if let Some(store) = self.store.upgrade() {
            if let Ok(mut store) = store.try_borrow_mut() {
                store.unsubscribe(self.id);
            }
        }
    }
}

pub struct Signal<T> {
    value: T,
    listeners: Rc<RefCell<ListenerStore>>,
}

impl<T> Signal<T> {
    pub fn new(initial: T) -> Self {
        Self {
            value: initial,
            listeners: Rc::new(RefCell::new(ListenerStore::new())),
        }
    }

    pub fn subscribe(&mut self, callback: Callback) -> SubscriptionGuard {
        let id = self.listeners.borrow_mut().subscribe(callback);
        SubscriptionGuard::new(&self.listeners, id)
    }
}

impl<T: Clone> Signal<T> {
    pub fn get(&self) -> T {
        self.value.clone()
    }
}

impl<T: PartialEq> Signal<T> {
    pub fn set(&mut self, next: T) -> Option<Vec<Callback>> {
        if self.value == next {
            return None;
        }

        self.value = next;
        Some(self.listeners.borrow().callbacks())
    }
}

#[cfg(test)]
mod tests {
    use super::{Callback, Signal};
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn dropping_subscription_guard_unsubscribes() {
        let fired = Rc::new(Cell::new(0));
        let fired_clone = fired.clone();
        let callback: Callback = Rc::new(move || {
            fired_clone.set(fired_clone.get() + 1);
        });

        let mut signal = Signal::new(1);
        let guard = signal.subscribe(callback);
        let listeners = signal.set(2).unwrap();
        for listener in listeners {
            listener();
        }
        assert_eq!(fired.get(), 1);

        drop(guard);

        let listeners = signal.set(3).unwrap();
        assert!(listeners.is_empty());
        assert_eq!(fired.get(), 1);
    }
}
