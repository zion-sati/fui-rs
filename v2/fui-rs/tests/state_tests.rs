use std::cell::RefCell;

use fui_rs::app::Application;
use fui_rs::component::Component;
use fui_rs::state::{derived, state, State};
use fui_rs::node::{flex_box, Node};

thread_local! {
    static COUNT_STATE: RefCell<Option<State<i32>>> = const { RefCell::new(None) };
    static DOUBLED_STATE: RefCell<Option<State<i32>>> = const { RefCell::new(None) };
}

struct DerivedComponent {
    count: State<i32>,
    doubled: State<i32>,
}

impl DerivedComponent {
    fn new() -> Self {
        let count = state(2_i32);
        let doubled = derived(&count, |value| value * 2);
        COUNT_STATE.with(|slot| slot.replace(Some(count.clone())));
        DOUBLED_STATE.with(|slot| slot.replace(Some(doubled.clone())));
        Self { count, doubled }
    }
}

impl Component for DerivedComponent {
    fn render(&self) -> Box<dyn Node> {
        let _ = self.count.get();
        let _ = self.doubled.get();
        Box::new(flex_box())
    }
}

#[test]
fn state_updates_derived_value() {
    Application::run(DerivedComponent::new);

    let count = COUNT_STATE.with(|slot| slot.borrow().as_ref().cloned().expect("count state"));
    let doubled =
        DOUBLED_STATE.with(|slot| slot.borrow().as_ref().cloned().expect("doubled state"));

    assert_eq!(doubled.get(), 4);
    count.set(6);
    assert_eq!(doubled.get(), 12);
}
