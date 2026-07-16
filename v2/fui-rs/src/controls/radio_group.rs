use super::*;
use crate::logger;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

#[derive(Clone)]
pub(crate) struct RadioGroupEventTarget {
    radios: Rc<RefCell<Vec<RadioButton>>>,
    selected_index: Rc<Cell<i32>>,
    changed: Rc<RefCell<Option<RadioGroupChangedCallback>>>,
}

impl RadioGroupEventTarget {
    pub(crate) fn select_radio_handle(&self, radio_handle: u64, focus: bool) {
        let index = self.index_of_radio_handle(radio_handle);
        if index >= 0 {
            self.select_index_internal(index, focus, true, true);
        }
    }

    pub(crate) fn move_selection_from_handle(&self, radio_handle: u64, delta: i32) {
        let start_index = self.index_of_radio_handle(radio_handle);
        if start_index < 0 || self.radios.borrow().is_empty() {
            return;
        }
        let next_index = self.find_enabled_index(start_index, delta);
        if next_index >= 0 {
            self.select_index_internal(next_index, true, true, true);
        }
    }

    pub(crate) fn select_first_enabled(&self, focus: bool) {
        let next_index = self.find_boundary_index(true);
        if next_index >= 0 {
            self.select_index_internal(next_index, focus, true, true);
        }
    }

    pub(crate) fn select_last_enabled(&self, focus: bool) {
        let next_index = self.find_boundary_index(false);
        if next_index >= 0 {
            self.select_index_internal(next_index, focus, true, true);
        }
    }

    pub(crate) fn select_index(&self, index: i32) {
        if index == -1 {
            let previous_index = self.selected_index.get();
            let changed = previous_index != -1;
            if let Some(previous) = self.radio_at(previous_index) {
                previous.update_checked(false, true, false);
            }
            self.selected_index.set(-1);
            if changed {
                self.emit_changed(String::new());
            }
            return;
        }
        let radio_count = self.radios.borrow().len() as i32;
        if radio_count == 0 {
            if index != -1 {
                logger::warn(
                    "Layout",
                    &format!(
                        "RadioGroup.select_index() received {index} before any radios were added."
                    ),
                );
            }
            return;
        }
        let clamped_index = if index < 0 {
            0
        } else if index >= radio_count {
            radio_count - 1
        } else {
            index
        };
        if clamped_index != index {
            logger::warn(
                "Layout",
                &format!(
                    "RadioGroup.select_index() received {index}; clamping to {clamped_index}."
                ),
            );
        }
        self.select_index_internal(clamped_index, false, true, false);
    }

    fn select_index_internal(&self, index: i32, focus: bool, emit: bool, announce: bool) {
        let Some(radio) = self.radio_at(index) else {
            return;
        };
        if !radio.is_enabled() {
            return;
        }
        let previous_index = self.selected_index.get();
        if previous_index == index {
            if focus {
                radio.focus_now();
            }
            return;
        }
        if let Some(previous) = self.radio_at(previous_index) {
            previous.update_checked(false, emit, false);
        }
        self.selected_index.set(index);
        radio.update_checked(true, emit, announce);
        if focus {
            radio.focus_now();
        }
        if emit {
            self.emit_changed(radio.value().to_string());
        }
    }

    fn emit_changed(&self, value: String) {
        if let Some(callback) = self.changed.borrow().clone() {
            callback(RadioGroupChangedEventArgs { value });
        }
    }

    fn index_of_radio_handle(&self, target_handle: u64) -> i32 {
        self.radios
            .borrow()
            .iter()
            .position(|radio| radio.retained_node_ref().handle().raw() == target_handle)
            .map(|index| index as i32)
            .unwrap_or(-1)
    }

    fn find_enabled_index(&self, start_index: i32, delta: i32) -> i32 {
        let radios = self.radios.borrow();
        if radios.is_empty() {
            return -1;
        }
        let len = radios.len() as i32;
        let mut cursor = start_index;
        for _ in 0..len {
            cursor += delta;
            if cursor < 0 {
                cursor = len - 1;
            } else if cursor >= len {
                cursor = 0;
            }
            if radios[cursor as usize].is_enabled() {
                return cursor;
            }
        }
        -1
    }

    fn find_boundary_index(&self, first: bool) -> i32 {
        let radios = self.radios.borrow();
        if first {
            radios
                .iter()
                .position(|radio| radio.is_enabled())
                .map(|index| index as i32)
                .unwrap_or(-1)
        } else {
            radios
                .iter()
                .rposition(|radio| radio.is_enabled())
                .map(|index| index as i32)
                .unwrap_or(-1)
        }
    }

    fn radio_at(&self, index: i32) -> Option<RadioButton> {
        if index < 0 {
            return None;
        }
        self.radios.borrow().get(index as usize).cloned()
    }
}

#[derive(Clone)]
pub struct RadioGroup {
    root: FlexBox,
    event_target: Rc<RadioGroupEventTarget>,
}

impl RadioGroup {
    pub fn new() -> Self {
        let root = flex_box();
        root.semantic_role(SemanticRole::RadioGroup)
            .flex_direction(FlexDirection::Column);
        let event_target = Rc::new(RadioGroupEventTarget {
            radios: Rc::new(RefCell::new(Vec::new())),
            selected_index: Rc::new(Cell::new(-1)),
            changed: Rc::new(RefCell::new(None)),
        });
        root.retained_node_ref()
            .retain_attachment(event_target.clone());
        let control = Self { root, event_target };
        let selected_index = control.event_target.selected_index.clone();
        let restore_target = control.event_target.clone();
        control.persist_state(crate::persisted::persisted_value_adapter(
            "radio-group-selected-index",
            crate::persisted::PersistedInt32Codec,
            1,
            move || {
                let index = selected_index.get();
                if index >= 0 {
                    Some(index)
                } else {
                    None
                }
            },
            move |index| {
                restore_target.select_index(index);
            },
        ));
        control
    }

    pub fn add_radio(&self, radio: RadioButton) -> &Self {
        radio.bind_group(Rc::downgrade(&self.event_target));
        if self.selected_index() < 0 && radio.is_checked() {
            self.event_target
                .selected_index
                .set(self.event_target.radios.borrow().len() as i32);
        }
        self.root.child(&radio);
        self.event_target.radios.borrow_mut().push(radio);
        self
    }

    pub fn add_option(&self, value: impl Into<String>, label: impl Into<String>) -> RadioButton {
        let radio = RadioButton::with_label(value, label);
        self.add_radio(radio.clone());
        radio
    }

    pub fn add_options<I>(&self, radios: I) -> &Self
    where
        I: IntoIterator<Item = RadioButton>,
    {
        for radio in radios {
            self.add_radio(radio);
        }
        self
    }

    pub fn on_changed(&self, handler: impl Fn(RadioGroupChangedEventArgs) + 'static) -> &Self {
        *self.event_target.changed.borrow_mut() = Some(Rc::new(handler));
        self
    }

    pub fn selected_index(&self) -> i32 {
        self.event_target.selected_index.get()
    }

    pub fn selected_value(&self) -> String {
        self.event_target
            .radio_at(self.selected_index())
            .map(|radio| radio.value().to_string())
            .unwrap_or_default()
    }

    pub fn select_index(&self, index: i32) -> &Self {
        self.event_target.select_index(index);
        self
    }
}

impl Default for RadioGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl Node for RadioGroup {
    fn retained_node_ref(&self) -> NodeRef {
        self.root.retained_node_ref()
    }

    fn build_self(&self) {
        self.root.build_self();
    }
}

impl crate::node::HasFlexBoxRoot for RadioGroup {
    fn flex_box_root(&self) -> &FlexBox {
        &self.root
    }
}

pub(crate) type WeakRadioGroupEventTarget = Weak<RadioGroupEventTarget>;
