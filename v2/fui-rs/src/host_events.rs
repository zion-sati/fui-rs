#[must_use = "the host-event handler is removed when the subscription is dropped"]
pub struct HostEventSubscription {
    unsubscribe: Option<Box<dyn FnOnce()>>,
}

impl HostEventSubscription {
    #[doc(hidden)]
    pub fn new(unsubscribe: impl FnOnce() + 'static) -> Self {
        Self {
            unsubscribe: Some(Box::new(unsubscribe)),
        }
    }
}

impl Drop for HostEventSubscription {
    fn drop(&mut self) {
        if let Some(unsubscribe) = self.unsubscribe.take() {
            unsubscribe();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn dropping_host_event_subscription_runs_unsubscribe_once() {
        let unsubscribe_count = Rc::new(Cell::new(0));
        let captured_count = unsubscribe_count.clone();
        let subscription = HostEventSubscription::new(move || {
            captured_count.set(captured_count.get() + 1);
        });

        drop(subscription);

        assert_eq!(unsubscribe_count.get(), 1);
    }
}
