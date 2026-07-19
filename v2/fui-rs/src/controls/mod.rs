use crate::bindings::ui;
use crate::event::{KeyEventArgs, PointerEventArgs};
use crate::ffi::{
    AlignItems, CursorStyle, FlexDirection, JustifyContent, Orientation, PositionType,
    SemanticCheckedState, SemanticRole, Unit,
};
use crate::node::{
    flex_box, row, FlexBox, HasFlexBoxRoot, Node, NodeRef, TextNode, ThemeBindable, WeakNodeRef,
};
use crate::theme::{current_theme, subscribe};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

pub(crate) mod internal;
mod shared;
use internal::pressable_labeled_control::PressableLabeledControl;
use radio_group::WeakRadioGroupEventTarget;
pub(crate) use shared::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClickEventArgs;

/// Semantic click activation for controls whose primary action can be invoked
/// by pointer, keyboard, or another supported activation source.
///
/// Use [`Node::on_pointer_click`] when raw routed pointer data is required.
pub trait Clickable: Sized {
    fn on_click(&self, handler: impl Fn(ClickEventArgs) + 'static) -> &Self;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckState {
    False,
    True,
    Mixed,
}

impl CheckState {
    fn semantic(self) -> SemanticCheckedState {
        match self {
            Self::False => SemanticCheckedState::False,
            Self::True => SemanticCheckedState::True,
            Self::Mixed => SemanticCheckedState::Mixed,
        }
    }

    pub fn is_checked(self) -> bool {
        self == Self::True
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckboxChangedEventArgs {
    pub state: CheckState,
    pub checked: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RadioButtonChangedEventArgs {
    pub checked: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RadioGroupChangedEventArgs {
    pub value: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SwitchChangedEventArgs {
    pub checked: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SliderChangedEventArgs {
    pub value: f32,
}

pub trait LabeledControlTextStyle: Sized {
    #[doc(hidden)]
    fn set_label_font_family(&self, family: crate::FontFamily);
    #[doc(hidden)]
    fn set_label_font_size(&self, size: f32);
    #[doc(hidden)]
    fn set_label_text_color(&self, color: u32);

    fn font_family(&self, family: crate::FontFamily) -> &Self {
        self.set_label_font_family(family);
        self
    }

    fn font_size(&self, size: f32) -> &Self {
        self.set_label_font_size(size);
        self
    }

    fn text_color(&self, color: u32) -> &Self {
        self.set_label_text_color(color);
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DropdownChangedEventArgs<T> {
    pub item: T,
    pub selected_index: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComboBoxChangedEventArgs<T> {
    pub item: T,
    pub selected_index: i32,
}

pub mod anti_selection_area;
mod appearance;
pub mod button;
pub mod checkbox;
pub mod combobox;
pub mod context_menu;
pub mod control_template_set;
pub mod control_tokens;
pub mod dialog;
pub mod dropdown;
pub mod form;
pub mod nav_link;
pub mod popup;
pub mod progress_bar;
pub mod radio_button;
pub mod radio_group;
pub mod selection_area;
pub mod slider;
pub mod switch;
pub mod templating;
#[cfg(test)]
mod tests;
pub mod text_area;
mod text_editor_surface;
pub mod text_input;

pub use anti_selection_area::AntiSelectionArea;
pub use appearance::{
    ContextMenuAppearance, ContextMenuItemAppearance, DialogAppearance, OverlayBackdropAppearance,
    PopupAppearance, SurfaceAppearance,
};
pub use button::Button;
pub use checkbox::Checkbox;
pub use combobox::{ComboBox, ComboBoxCommitMode, ComboBoxFilterMode, ComboBoxItem};
pub use context_menu::{
    run_context_menu_action, ContextMenu, ContextMenuAction, ContextMenuVisibilityChangedEventArgs,
    MenuItem,
};
pub use control_template_set::{
    clear_control_templates, get_control_templates, use_control_templates, ControlTemplateSet,
};
pub use control_tokens::{
    ButtonColors, DropdownColors, DropdownSizing, LabeledControlColors, LabeledControlSizing,
    ProgressBarColors, ProgressBarSizing, SliderColors, SliderSizing, TextInputColors,
};
pub use dialog::{Dialog, DialogShownEventArgs};
pub use dropdown::{Dropdown, DropdownItem};
pub use form::Form;
pub use nav_link::{NavLink, NavigateEventArgs};
pub use popup::Popup;
pub use progress_bar::ProgressBar;
pub use radio_button::RadioButton;
pub use radio_group::RadioGroup;
pub use selection_area::SelectionArea;
pub use slider::Slider;
pub use switch::Switch;
pub use templating::{
    create_default_button_presenter, create_default_checkbox_indicator_presenter,
    create_default_dropdown_chevron_presenter, create_default_dropdown_field_presenter,
    create_default_dropdown_option_row_presenter, create_default_radio_indicator_presenter,
    create_default_slider_presenter, create_default_switch_indicator_presenter,
    create_default_text_input_presenter, ButtonPresenter, ButtonTemplate, ButtonVisualState,
    CheckboxIndicatorPresenter, CheckboxIndicatorTemplate, CheckboxIndicatorVisualState,
    DefaultButtonTemplate, DefaultCheckboxIndicatorTemplate, DefaultDropdownChevronTemplate,
    DefaultDropdownFieldTemplate, DefaultDropdownOptionRowTemplate, DefaultRadioIndicatorTemplate,
    DefaultSliderTemplate, DefaultSwitchIndicatorTemplate, DefaultTextInputTemplate,
    DropdownChevronMetrics, DropdownChevronPresenter, DropdownChevronTemplate,
    DropdownChevronVisualState, DropdownFieldMetrics, DropdownFieldPresenter,
    DropdownFieldTemplate, DropdownFieldVisualState, DropdownOptionRowMetrics,
    DropdownOptionRowPresenter, DropdownOptionRowTemplate, DropdownOptionRowVisualState,
    PressableIndicatorMetrics, PressableIndicatorPresenter, PressableIndicatorVisualState,
    RadioIndicatorPresenter, RadioIndicatorTemplate, RadioIndicatorVisualState, SliderPresenter,
    SliderPresenterMetrics, SliderTemplate, SliderVisualState, SwitchIndicatorPresenter,
    SwitchIndicatorTemplate, SwitchIndicatorVisualState, TextInputPresenter, TextInputTemplate,
    TextInputVisualState, DEFAULT_BUTTON_TEMPLATE, DEFAULT_CHECKBOX_INDICATOR_TEMPLATE,
    DEFAULT_DROPDOWN_CHEVRON_TEMPLATE, DEFAULT_DROPDOWN_FIELD_TEMPLATE,
    DEFAULT_DROPDOWN_OPTION_ROW_TEMPLATE, DEFAULT_RADIO_INDICATOR_TEMPLATE,
    DEFAULT_SLIDER_TEMPLATE, DEFAULT_SWITCH_INDICATOR_TEMPLATE, DEFAULT_TEXT_INPUT_TEMPLATE,
};
pub use text_area::TextArea;
pub use text_editor_surface::TextEditorSurface;
pub use text_input::TextInput;

pub fn button(label: impl Into<String>) -> Button {
    Button::new(label)
}

pub fn selection_area() -> SelectionArea {
    SelectionArea::new()
}

pub fn anti_selection_area() -> AntiSelectionArea {
    AntiSelectionArea::new()
}

pub fn checkbox(label: impl Into<String>) -> Checkbox {
    Checkbox::new(label)
}

pub fn combo_box() -> ComboBox {
    ComboBox::new()
}

pub fn context_menu<I>(items: I) -> ContextMenu
where
    I: IntoIterator<Item = MenuItem>,
{
    let menu = ContextMenu::new();
    menu.items(items);
    menu
}

pub fn popup() -> Popup {
    Popup::new()
}

pub fn dialog(title: impl Into<String>, body: impl Into<String>) -> Dialog {
    Dialog::new(title, body)
}

pub fn dropdown() -> Dropdown {
    Dropdown::new()
}

pub fn form() -> Form {
    Form::new()
}

pub fn nav_link(href: impl Into<String>) -> NavLink {
    NavLink::new(href)
}

pub fn radio_button(label: impl Into<String>) -> RadioButton {
    RadioButton::new(label)
}

pub fn radio_group() -> RadioGroup {
    RadioGroup::new()
}

pub fn switch(label: impl Into<String>) -> Switch {
    Switch::new(label)
}

pub fn progress_bar() -> ProgressBar {
    ProgressBar::new()
}

pub fn slider() -> Slider {
    Slider::new()
}

pub fn text_input() -> TextInput {
    TextInput::new()
}

pub fn text_area() -> TextArea {
    TextArea::new()
}
