use std::rc::Rc;

pub type Callback = Rc<dyn Fn()>;

pub struct Signal<T> {
    value: T,
    listeners: Vec<Callback>,
}

impl<T> Signal<T> {
    pub fn new(initial: T) -> Self {
        Self {
            value: initial,
            listeners: Vec::new(),
        }
    }

    pub fn subscribe(&mut self, callback: Callback) {
        self.listeners.push(callback);
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
        Some(self.listeners.clone())
    }
}
