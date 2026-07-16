use super::internal::button_presenter::ButtonTemplate;
use super::internal::checkbox_indicator_presenter::CheckboxIndicatorTemplate;
use super::internal::dropdown_chevron_presenter::DropdownChevronTemplate;
use super::internal::dropdown_field_presenter::DropdownFieldTemplate;
use super::internal::dropdown_option_row_presenter::DropdownOptionRowTemplate;
use super::internal::radio_indicator_presenter::RadioIndicatorTemplate;
use super::internal::slider_presenter::SliderTemplate;
use super::internal::switch_indicator_presenter::SwitchIndicatorTemplate;
use super::internal::text_input_presenter::TextInputTemplate;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Default)]
pub struct ControlTemplateSet {
    pub button: Option<Rc<dyn ButtonTemplate>>,
    pub checkbox_indicator: Option<Rc<dyn CheckboxIndicatorTemplate>>,
    pub dropdown_chevron: Option<Rc<dyn DropdownChevronTemplate>>,
    pub dropdown_field: Option<Rc<dyn DropdownFieldTemplate>>,
    pub dropdown_option_row: Option<Rc<dyn DropdownOptionRowTemplate>>,
    pub radio_indicator: Option<Rc<dyn RadioIndicatorTemplate>>,
    pub slider: Option<Rc<dyn SliderTemplate>>,
    pub switch_indicator: Option<Rc<dyn SwitchIndicatorTemplate>>,
    pub text_input: Option<Rc<dyn TextInputTemplate>>,
    pub text_area: Option<Rc<dyn TextInputTemplate>>,
}

thread_local! {
    static ACTIVE_CONTROL_TEMPLATES: RefCell<Option<ControlTemplateSet>> = const { RefCell::new(None) };
}

pub fn get_control_templates() -> Option<ControlTemplateSet> {
    ACTIVE_CONTROL_TEMPLATES.with(|slot| slot.borrow().clone())
}

pub fn use_control_templates(templates: ControlTemplateSet) {
    ACTIVE_CONTROL_TEMPLATES.with(|slot| {
        *slot.borrow_mut() = Some(templates);
    })
}

pub fn clear_control_templates() {
    ACTIVE_CONTROL_TEMPLATES.with(|slot| {
        *slot.borrow_mut() = None;
    });
}
