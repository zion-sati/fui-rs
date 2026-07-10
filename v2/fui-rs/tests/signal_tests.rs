use fui_rs::signal::Signal;

#[test]
fn signal_returns_current_value() {
    let signal = Signal::new(7_i32);
    assert_eq!(signal.get(), 7);
}

#[test]
fn signal_notifies_subscribers_when_changed() {
    let mut signal = Signal::new(3_i32);
    let observed = std::rc::Rc::new(std::cell::Cell::new(0_i32));
    let observed_clone = observed.clone();
    let _subscription = signal.subscribe(std::rc::Rc::new(move || {
        observed_clone.set(observed_clone.get() + 1);
    }));

    let listeners = signal.set(9).expect("signal should change");
    for listener in listeners {
        listener();
    }
    assert_eq!(observed.get(), 1);
}
