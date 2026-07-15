use super::*;
use crate::assets;
use crate::event::{self, PointerEventArgs};
use crate::ffi::{
    self, Call, KeyEventType, PointerEventType, SemanticCheckedState, SemanticRole,
    TextVerticalAlign, Unit,
};
use crate::focus_visibility;
use crate::generated::ffi::NodeType;
use crate::node::Node;
use crate::node::{column, text, Child, FlexBoxSurface};
use crate::theme::{current_theme, generate_theme, use_custom_theme, use_system_theme};
use crate::Application;
use crate::PointerType;
use crate::ScrollBarVisibility;
use crate::TextCore;
use std::cell::Cell;
use std::rc::Rc;

#[derive(Clone)]
struct TestButtonPresenter {
    content_root: FlexBox,
    label: TextCore,
    border_color: u32,
}

impl TestButtonPresenter {
    fn new(border_color: u32) -> Self {
        let label = TextCore::new("");
        let content_root = flex_box();
        content_root.child(&label);
        Self {
            content_root,
            label,
            border_color,
        }
    }
}

impl ButtonPresenter for TestButtonPresenter {
    fn content_root(&self) -> FlexBox {
        self.content_root.clone()
    }

    fn label_node(&self) -> TextCore {
        self.label.clone()
    }

    fn apply(
        &self,
        host: &FlexBox,
        _theme: crate::theme::Theme,
        _state: ButtonVisualState,
        _colors: Option<ButtonColors>,
    ) {
        host.border(3.0, self.border_color);
        self.label.text_color(0x112233FF);
    }
}

struct TestButtonTemplate {
    created: Rc<Cell<u32>>,
    border_color: u32,
}

impl ButtonTemplate for TestButtonTemplate {
    fn create(&self) -> Rc<dyn ButtonPresenter> {
        self.created.set(self.created.get() + 1);
        Rc::new(TestButtonPresenter::new(self.border_color))
    }
}

#[derive(Clone, Default)]
struct TestTextInputPresenter {
    last_state: Rc<RefCell<Option<TextInputVisualState>>>,
}

impl TextInputPresenter for TestTextInputPresenter {
    fn bind(&self, _host: FlexBox, _editor_host: TextCore, _placeholder_host: FlexBox) {}

    fn apply(
        &self,
        _theme: crate::theme::Theme,
        state: &TextInputVisualState,
        _colors: Option<TextInputColors>,
    ) {
        *self.last_state.borrow_mut() = Some(*state);
    }
}

struct TestTextInputTemplate {
    created: Rc<Cell<u32>>,
    last_state: Rc<RefCell<Option<TextInputVisualState>>>,
}

impl TextInputTemplate for TestTextInputTemplate {
    fn create(&self) -> Rc<dyn TextInputPresenter> {
        self.created.set(self.created.get() + 1);
        Rc::new(TestTextInputPresenter {
            last_state: self.last_state.clone(),
        })
    }
}

struct TestDropdownFieldTemplate {
    created: Rc<Cell<u32>>,
}

impl DropdownFieldTemplate for TestDropdownFieldTemplate {
    fn create(&self, _sizing: Option<DropdownSizing>) -> Rc<dyn DropdownFieldPresenter> {
        self.created.set(self.created.get() + 1);
        create_default_dropdown_field_presenter(None)
    }
}

struct TestDropdownChevronTemplate {
    created: Rc<Cell<u32>>,
}

impl DropdownChevronTemplate for TestDropdownChevronTemplate {
    fn create(&self, _sizing: Option<DropdownSizing>) -> Rc<dyn DropdownChevronPresenter> {
        self.created.set(self.created.get() + 1);
        create_default_dropdown_chevron_presenter(None)
    }
}

struct TestDropdownOptionRowTemplate {
    created: Rc<Cell<u32>>,
}

impl DropdownOptionRowTemplate for TestDropdownOptionRowTemplate {
    fn create(&self, _sizing: Option<DropdownSizing>) -> Rc<dyn DropdownOptionRowPresenter> {
        self.created.set(self.created.get() + 1);
        create_default_dropdown_option_row_presenter(None)
    }
}

fn press_down(node_ref: &crate::node::NodeRef, click_count: i32) {
    let mut event = PointerEventArgs::new(
        node_ref.handle(),
        PointerEventType::Down,
        10.0,
        10.0,
        0,
        1,
        PointerType::Mouse,
        0,
        1,
        0.0,
        0.0,
        0.0,
        click_count,
    );
    node_ref.handle_pointer_event(&mut event);
}

fn press_up(node_ref: &crate::node::NodeRef) {
    let mut event = PointerEventArgs::new(
        node_ref.handle(),
        PointerEventType::Up,
        10.0,
        10.0,
        0,
        1,
        PointerType::Mouse,
        0,
        0,
        0.0,
        0.0,
        0.0,
        0,
    );
    node_ref.handle_pointer_event(&mut event);
}

fn press_leave(node_ref: &crate::node::NodeRef) {
    let mut event = PointerEventArgs::new(
        node_ref.handle(),
        PointerEventType::Leave,
        10.0,
        10.0,
        0,
        1,
        PointerType::Mouse,
        0,
        0,
        0.0,
        0.0,
        0.0,
        0,
    );
    node_ref.handle_pointer_event(&mut event);
}

fn press_enter(node_ref: &crate::node::NodeRef) {
    let mut event = PointerEventArgs::new(
        node_ref.handle(),
        PointerEventType::Enter,
        10.0,
        10.0,
        0,
        1,
        PointerType::Mouse,
        0,
        0,
        0.0,
        0.0,
        0.0,
        0,
    );
    node_ref.handle_pointer_event(&mut event);
}

fn key_event(event_type: KeyEventType, key: &str, modifiers: u32) -> bool {
    event::__fui_on_key_event(event_type as u32, key.as_ptr(), key.len() as u32, modifiers)
}

fn blur(node_ref: &crate::node::NodeRef) {
    event::__fui_on_focus_changed(node_ref.handle().raw(), false);
}

fn focus(node_ref: &crate::node::NodeRef) {
    event::__fui_on_focus_changed(node_ref.handle().raw(), true);
}

fn child_handles_for_parent(calls: &[Call], parent: u64) -> Vec<u64> {
    calls
        .iter()
        .filter_map(|call| match call {
            Call::NodeAddChild {
                parent: call_parent,
                child,
            } if *call_parent == parent => Some(*child),
            _ => None,
        })
        .collect()
}

fn node_type_for_handle(calls: &[Call], handle: u64) -> Option<NodeType> {
    calls.iter().find_map(|call| match call {
        Call::CreateNode {
            handle: call_handle,
            node_type,
        } if *call_handle == handle => match *node_type {
            0 => Some(NodeType::FlexBox),
            1 => Some(NodeType::Text),
            2 => Some(NodeType::Image),
            3 => Some(NodeType::Svg),
            4 => Some(NodeType::ScrollView),
            5 => Some(NodeType::Grid),
            _ => None,
        },
        _ => None,
    })
}

#[test]
fn pressable_labeled_child_tree_order_matches_fui_as() {
    ffi::test::reset();
    let checkbox = checkbox("Agree");
    Application::mount(checkbox.clone());
    let calls = ffi::test::take_calls();
    let (root, indicator, gap_node, label_host) = checkbox.test_parts();
    let root_ref = root.retained_node_ref();
    let root_handle = root_ref.handle();
    let indicator_handle = indicator.retained_node_ref().handle().raw();
    let gap_handle = gap_node.retained_node_ref().handle().raw();
    let label_host_handle = label_host.retained_node_ref().handle().raw();

    let child_handles = child_handles_for_parent(&calls, root_handle.raw());
    assert_eq!(
        child_handles,
        vec![indicator_handle, gap_handle, label_host_handle]
    );

    let label_child_handles = child_handles_for_parent(&calls, label_host_handle);
    assert_eq!(label_child_handles.len(), 1);
}

#[test]
fn button_enabled_state_reflects_semantic_disabled_and_blocks_activation() {
    ffi::test::reset();
    let clicks = Rc::new(Cell::new(0));
    let clicks_clone = clicks.clone();
    let button = button("Action");
    button.on_click(move |_event| clicks_clone.set(clicks_clone.get() + 1));
    button.enabled(false);

    Application::mount(button.clone());
    let calls = ffi::test::take_calls();
    let button_ref = button.retained_node_ref();
    let button_handle = button_ref.handle();

    assert!(calls.iter().any(|call| matches!(
            call,
            Call::SetInteractive { handle, interactive } if *handle == button_handle.raw() && !*interactive
        )));
    assert!(calls.iter().any(|call| matches!(
            call,
            Call::SetFocusable { handle, focusable, .. } if *handle == button_handle.raw() && !*focusable
        )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticDisabled { handle, has_disabled, disabled }
            if *handle == button_handle.raw() && *has_disabled && *disabled
    )));

    press_down(&button_ref, 1);
    press_up(&button_ref);
    focus(&button_ref);
    key_event(KeyEventType::Down, "Enter", 0);
    key_event(KeyEventType::Down, " ", 0);
    assert_eq!(clicks.get(), 0);
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetLayerEffect { handle, opacity, .. }
            if *handle == button_handle.raw() && (*opacity - 0.38).abs() < f32::EPSILON
    )));
}

#[test]
fn dropdown_presenter_templates_create_once_and_are_stored_in_template_set() {
    let field_created = Rc::new(Cell::new(0));
    let chevron_created = Rc::new(Cell::new(0));
    let option_created = Rc::new(Cell::new(0));
    let templates = ControlTemplateSet {
        dropdown_field: Some(Rc::new(TestDropdownFieldTemplate {
            created: field_created.clone(),
        })),
        dropdown_chevron: Some(Rc::new(TestDropdownChevronTemplate {
            created: chevron_created.clone(),
        })),
        dropdown_option_row: Some(Rc::new(TestDropdownOptionRowTemplate {
            created: option_created.clone(),
        })),
        ..ControlTemplateSet::default()
    };
    use_control_templates(Some(templates));
    let active = get_control_templates().expect("templates should be installed");
    assert!(active.dropdown_field.is_some());
    assert!(active.dropdown_chevron.is_some());
    assert!(active.dropdown_option_row.is_some());
    active
        .dropdown_field
        .as_ref()
        .expect("field template")
        .create(None);
    active
        .dropdown_chevron
        .as_ref()
        .expect("chevron template")
        .create(None);
    active
        .dropdown_option_row
        .as_ref()
        .expect("option row template")
        .create(None);
    assert_eq!(field_created.get(), 1);
    assert_eq!(chevron_created.get(), 1);
    assert_eq!(option_created.get(), 1);
    clear_control_templates();
}

#[test]
fn default_dropdown_field_presenter_retained_tree_matches_fui_as_shape() {
    ffi::test::reset();
    let presenter = create_default_dropdown_field_presenter(None);
    Application::mount(presenter.root());
    let calls = ffi::test::take_calls();
    let root_handle = presenter.root().retained_node_ref().handle().raw();
    let value_host_handle = presenter.value_host().retained_node_ref().handle().raw();
    let value_node_handle = presenter.value_node().retained_node_ref().handle().raw();
    let chevron_host_handle = presenter.chevron_host().retained_node_ref().handle().raw();
    let child_handles = child_handles_for_parent(&calls, root_handle);
    assert_eq!(child_handles, vec![value_host_handle, chevron_host_handle]);
    let value_host_children = child_handles_for_parent(&calls, value_host_handle);
    assert_eq!(value_host_children, vec![value_node_handle]);
    assert_eq!(
        node_type_for_handle(&calls, root_handle),
        Some(NodeType::FlexBox)
    );
    assert_eq!(
        node_type_for_handle(&calls, value_node_handle),
        Some(NodeType::Text)
    );
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetFillWidth { handle, fill } if *handle == value_node_handle && *fill
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetFillHeight { handle, fill } if *handle == value_node_handle && *fill
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetTextLimits { handle, max_chars, max_lines }
            if *handle == value_node_handle && *max_chars == 0 && *max_lines == 1
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetTextWrapping { handle, wrap } if *handle == value_node_handle && !*wrap
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetTextOverflowFade { handle, horizontal, vertical }
            if *handle == value_node_handle && *horizontal && !*vertical
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetTextVerticalAlign { handle, align_enum }
            if *handle == value_node_handle && *align_enum == TextVerticalAlign::Center as u32
    )));
}

#[test]
fn default_dropdown_field_presenter_apply_matches_fui_as_style_contract() {
    ffi::test::reset();
    let presenter = create_default_dropdown_field_presenter(Some(
        DropdownSizing::new()
            .field_height(44.0)
            .field_font_size(18.0)
            .chevron_box_size(20.0),
    ));
    Application::mount(presenter.root());
    let _ = ffi::test::take_calls();
    let root_handle = presenter.root().retained_node_ref().handle().raw();
    let value_node_handle = presenter.value_node().retained_node_ref().handle().raw();
    let chevron_host_handle = presenter.chevron_host().retained_node_ref().handle().raw();
    let theme = current_theme();

    presenter.apply(
        theme.clone(),
        &DropdownFieldVisualState::new(true, true, true, true, "Selected"),
        Some(
            DropdownColors::new()
                .background(0x111111FF)
                .border(0x222222FF)
                .text_primary(0x333333FF),
        ),
    );
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetHeight { handle, value, unit_enum }
            if *handle == root_handle && (*value - 44.0).abs() < f32::EPSILON && *unit_enum == Unit::Pixel as u32
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetBoxStyle { handle, border_width, border_color, .. }
            if *handle == root_handle && (*border_width - 2.0).abs() < f32::EPSILON && *border_color == 0x222222FF
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetBgColor { handle, color } if *handle == root_handle && *color == 0x111111FF
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetLineHeight { handle, line_height }
            if *handle == value_node_handle && (*line_height - 44.0).abs() < f32::EPSILON
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetTextColor { handle, color } if *handle == value_node_handle && *color == 0x333333FF
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetWidth { handle, value, unit_enum }
            if *handle == chevron_host_handle && (*value - 20.0).abs() < f32::EPSILON && *unit_enum == Unit::Pixel as u32
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetHeight { handle, value, unit_enum }
            if *handle == chevron_host_handle && (*value - 20.0).abs() < f32::EPSILON && *unit_enum == Unit::Pixel as u32
    )));
    assert!(!calls.iter().any(|call| matches!(
        call,
        Call::SetText { handle, text } if *handle == value_node_handle && text == "Selected"
    )));
}

#[test]
fn default_dropdown_chevron_presenter_retained_tree_matches_fui_as_shape() {
    ffi::test::reset();
    let presenter = create_default_dropdown_chevron_presenter(None);
    Application::mount(presenter.root());
    let calls = ffi::test::take_calls();
    let root_handle = presenter.root().retained_node_ref().handle().raw();
    let child_handles = child_handles_for_parent(&calls, root_handle);
    assert_eq!(child_handles.len(), 1);
    assert_eq!(
        node_type_for_handle(&calls, child_handles[0]),
        Some(NodeType::Svg)
    );
}

#[test]
fn default_dropdown_chevron_presenter_apply_matches_fui_as_style_contract() {
    assets::test_reset();
    ffi::test::reset();
    let presenter = create_default_dropdown_chevron_presenter(Some(
        DropdownSizing::new().chevron_icon_size(18.0),
    ));
    Application::mount(presenter.root());
    let calls = ffi::test::take_calls();
    let root_handle = presenter.root().retained_node_ref().handle().raw();
    let icon_handle = child_handles_for_parent(&calls, root_handle)[0];
    let theme = current_theme();

    presenter.apply(
        theme.clone(),
        DropdownChevronVisualState::new(true, true, true),
    );
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetWidth { handle, value, unit_enum }
            if *handle == icon_handle && (*value - 18.0).abs() < f32::EPSILON && *unit_enum == Unit::Pixel as u32
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetHeight { handle, value, unit_enum }
            if *handle == icon_handle && (*value - 18.0).abs() < f32::EPSILON && *unit_enum == Unit::Pixel as u32
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSvg { handle, tint_color, .. }
            if *handle == icon_handle && *tint_color == theme.colors.text_primary
    )));
    drop(presenter);
    assets::test_reset();
}

#[test]
fn default_dropdown_option_row_presenter_retained_tree_matches_fui_as_shape() {
    ffi::test::reset();
    let presenter = create_default_dropdown_option_row_presenter(None);
    Application::mount(presenter.root());
    let calls = ffi::test::take_calls();
    let root_handle = presenter.root().retained_node_ref().handle().raw();
    let label_handle = presenter.label_node().retained_node_ref().handle().raw();
    let child_handles = child_handles_for_parent(&calls, root_handle);
    assert_eq!(child_handles, vec![label_handle]);
    assert_eq!(
        node_type_for_handle(&calls, label_handle),
        Some(NodeType::Text)
    );
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetFillWidth { handle, fill } if *handle == label_handle && *fill
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetFillHeight { handle, fill } if *handle == label_handle && *fill
    )));
}

#[test]
fn default_dropdown_option_row_presenter_apply_matches_fui_as_style_contract() {
    ffi::test::reset();
    let presenter = create_default_dropdown_option_row_presenter(Some(
        DropdownSizing::new()
            .option_height(42.0)
            .option_font_size(19.0),
    ));
    assert_eq!(presenter.metrics().height, 42.0);
    assert_eq!(presenter.metrics().font_size, 19.0);
    Application::mount(presenter.root());
    let _ = ffi::test::take_calls();
    let root_handle = presenter.root().retained_node_ref().handle().raw();
    let label_handle = presenter.label_node().retained_node_ref().handle().raw();
    let theme = current_theme();

    presenter.apply(
        theme.clone(),
        DropdownOptionRowVisualState::new(true, true, true),
        Some(
            DropdownColors::new()
                .accent(0xAABBCCFF)
                .text_primary(0x102030FF),
        ),
    );
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetBgColor { handle, color }
            if *handle == root_handle && *color == theme.context_menu.item.hover_background
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetTextColor { handle, color } if *handle == label_handle && *color == 0xAABBCCFF
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetTextColor { handle, color } if *handle == label_handle && *color != 0x102030FF
    )));
}

#[test]
fn button_space_activates_on_key_up_only_and_blur_cancels() {
    ffi::test::reset();
    let clicks = Rc::new(Cell::new(0));
    let clicks_clone = clicks.clone();
    let button = button("Action");
    button.on_click(move |_event| clicks_clone.set(clicks_clone.get() + 1));
    Application::mount(button.clone());
    let _ = ffi::test::take_calls();
    let node_ref = button.retained_node_ref();

    focus(&node_ref);
    key_event(KeyEventType::Down, "Space", 0);
    assert_eq!(clicks.get(), 0);
    key_event(KeyEventType::Up, "Space", 0);
    assert_eq!(clicks.get(), 1);

    focus(&node_ref);
    key_event(KeyEventType::Down, "Space", 0);
    focus(&node_ref);
    blur(&node_ref);
    key_event(KeyEventType::Up, "Space", 0);
    assert_eq!(clicks.get(), 1);
}

#[test]
fn button_repeated_key_down_arms_once_until_key_up() {
    ffi::test::reset();
    let clicks = Rc::new(Cell::new(0));
    let clicks_clone = clicks.clone();
    let button = button("Action");
    button.on_click(move |_event| clicks_clone.set(clicks_clone.get() + 1));
    Application::mount(button.clone());
    let _ = ffi::test::take_calls();
    let node_ref = button.retained_node_ref();

    focus(&node_ref);
    assert!(key_event(KeyEventType::Down, "Space", 0));
    assert!(key_event(KeyEventType::Down, "Space", 0));
    assert_eq!(clicks.get(), 0);
    assert!(key_event(KeyEventType::Up, "Space", 0));
    assert_eq!(clicks.get(), 1);
    assert!(!key_event(KeyEventType::Up, "Space", 0));
    assert_eq!(clicks.get(), 1);
}

#[test]
fn button_hover_and_pressed_visual_states_follow_fui_as_lifecycle() {
    ffi::test::reset();
    let theme = current_theme();
    let button = button("Action");
    Application::mount(button.clone());
    let _ = ffi::test::take_calls();
    let node_ref = button.retained_node_ref();
    let button_handle = node_ref.handle().raw();

    press_enter(&node_ref);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetBgColor { handle, color }
            if *handle == button_handle && *color == theme.colors.accent_hovered
    )));

    press_down(&node_ref, 1);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetBgColor { handle, color }
            if *handle == button_handle && *color == theme.colors.accent_pressed
    )));

    press_up(&node_ref);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetBgColor { handle, color }
            if *handle == button_handle && *color == theme.colors.accent_hovered
    )));

    press_leave(&node_ref);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetBgColor { handle, color }
            if *handle == button_handle && *color == theme.colors.accent
    )));
}

#[test]
fn button_uses_app_level_template_when_no_local_template_is_set() {
    ffi::test::reset();
    clear_control_templates();
    let created = Rc::new(Cell::new(0));
    use_control_templates(Some(ControlTemplateSet {
        button: Some(Rc::new(TestButtonTemplate {
            created: created.clone(),
            border_color: 0xAABBCCFF,
        })),
        ..Default::default()
    }));

    let button = button("Template action");
    Application::mount(button.clone());
    let calls = ffi::test::take_calls();
    let button_handle = button.retained_node_ref().handle().raw();
    assert_eq!(created.get(), 1);
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetBoxStyle { handle, border_width, border_color, .. }
            if *handle == button_handle
                && (*border_width - 3.0).abs() < f32::EPSILON
                && *border_color == 0xAABBCCFF
    )));

    clear_control_templates();
}

#[test]
fn button_local_template_replaces_presenter_tree_and_retains_label_updates() {
    ffi::test::reset();
    clear_control_templates();
    let button = button("Before");
    Application::mount(button.clone());
    let _ = ffi::test::take_calls();

    let created = Rc::new(Cell::new(0));
    button.template(Some(Rc::new(TestButtonTemplate {
        created: created.clone(),
        border_color: 0x445566FF,
    })));
    let calls = ffi::test::take_calls();
    let button_handle = button.retained_node_ref().handle().raw();
    assert_eq!(created.get(), 1);
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::NodeAddChild { parent, .. } if *parent == button_handle
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::NodeRemoveChild { parent, .. } if *parent == button_handle
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetBoxStyle { handle, border_width, border_color, .. }
            if *handle == button_handle
                && (*border_width - 3.0).abs() < f32::EPSILON
                && *border_color == 0x445566FF
    )));

    button.text("After");
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetText { text, .. } if text == "After"
    )));
}

#[test]
fn pressable_labeled_enabled_forwarding_blocks_activation() {
    ffi::test::reset();
    let checkbox_changes = Rc::new(Cell::new(0));
    let checkbox_changes_clone = checkbox_changes.clone();
    let checkbox = checkbox("Agree");
    checkbox.on_changed(move |_event| {
        checkbox_changes_clone.set(checkbox_changes_clone.get() + 1);
    });
    checkbox.enabled(false);
    Application::mount(checkbox.clone());
    let calls = ffi::test::take_calls();
    let checkbox_ref = checkbox.retained_node_ref();
    let checkbox_handle = checkbox_ref.handle();

    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticDisabled { handle, has_disabled, disabled }
            if *handle == checkbox_handle.raw() && *has_disabled && *disabled
    )));
    press_down(&checkbox_ref, 1);
    press_up(&checkbox_ref);
    focus(&checkbox_ref);
    key_event(KeyEventType::Down, "Space", 0);
    key_event(KeyEventType::Up, "Space", 0);
    assert_eq!(checkbox_changes.get(), 0);

    ffi::test::reset();
    let radio_changes = Rc::new(Cell::new(0));
    let radio_changes_clone = radio_changes.clone();
    let radio = radio_button("Option");
    radio.on_changed(move |_event| {
        radio_changes_clone.set(radio_changes_clone.get() + 1);
    });
    radio.enabled(false);
    Application::mount(radio.clone());
    let radio_ref = radio.retained_node_ref();
    press_down(&radio_ref, 1);
    press_up(&radio_ref);
    focus(&radio_ref);
    key_event(KeyEventType::Down, "Space", 0);
    key_event(KeyEventType::Up, "Space", 0);
    assert_eq!(radio_changes.get(), 0);

    ffi::test::reset();
    let switch_changes = Rc::new(Cell::new(0));
    let switch_changes_clone = switch_changes.clone();
    let switch = switch("Toggle");
    switch.on_changed(move |_event| {
        switch_changes_clone.set(switch_changes_clone.get() + 1);
    });
    switch.enabled(false);
    Application::mount(switch.clone());
    let switch_ref = switch.retained_node_ref();
    press_down(&switch_ref, 1);
    press_up(&switch_ref);
    focus(&switch_ref);
    key_event(KeyEventType::Down, "Space", 0);
    key_event(KeyEventType::Up, "Space", 0);
    assert_eq!(switch_changes.get(), 0);
}

#[test]
fn focus_chrome_tracks_theme_changes() {
    ffi::test::reset();
    use_custom_theme(generate_theme(true, 0x112233FF));
    let theme = current_theme();

    let button = button("Action");
    Application::mount(button.clone());
    let _ = ffi::test::take_calls();
    let button_handle = button.retained_node_ref().handle().raw();

    event::__fui_on_focus_changed(button_handle, true);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetBoxStyle { handle, border_width, border_color, .. }
            if *handle != button_handle
                && *border_width == 2.0
                && *border_color == theme.colors.focus_ring
    )));

    use_custom_theme(generate_theme(true, 0x445566FF));
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetBoxStyle { handle, border_width, border_color, .. }
            if *handle != button_handle
                && *border_width == 2.0
                && *border_color == 0x445566FF
    )));

    event::__fui_on_focus_changed(button_handle, false);
    let calls = ffi::test::take_calls();
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::NodeRemoveChild { .. })));

    use_system_theme();
}

#[test]
fn pointer_focus_hides_focus_adorner_until_keyboard_input() {
    ffi::test::reset();
    focus_visibility::reset_keyboard_focus_visibility();
    let button = button("Action");
    Application::mount(button.clone());
    let _ = ffi::test::take_calls();
    let button_handle = button.retained_node_ref().handle().raw();

    event::__fui_on_focus_changed(button_handle, true);
    let calls = ffi::test::take_calls();
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::NodeAddChild { .. })));

    event::__fui_on_pointer_event_with_metadata(
        PointerEventType::Down as u32,
        button_handle,
        10.0,
        10.0,
        0,
        1,
        PointerType::Mouse as u32,
        0,
        1,
        0.0,
        0.0,
        0.0,
        1,
    );
    let calls = ffi::test::take_calls();
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::NodeRemoveChild { .. })));

    key_event(KeyEventType::Down, "Tab", 0);
    let calls = ffi::test::take_calls();
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::NodeAddChild { .. })));
}

#[test]
fn pressable_focus_visibility_signal_re_shows_focus_adorner() {
    ffi::test::reset();
    focus_visibility::reset_keyboard_focus_visibility();
    let checkbox = checkbox("Agree");
    Application::mount(checkbox.clone());
    let _ = ffi::test::take_calls();
    let node_ref = checkbox.retained_node_ref();
    let handle = node_ref.handle().raw();

    event::__fui_on_focus_changed(handle, true);
    let calls = ffi::test::take_calls();
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::NodeAddChild { .. })));

    focus_visibility::show_keyboard_focus_for_pointer_event(PointerEventType::Down);
    let calls = ffi::test::take_calls();
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::NodeRemoveChild { .. })));

    focus_visibility::show_keyboard_focus_for_key_event(KeyEventType::Down, "Tab", 0);
    let calls = ffi::test::take_calls();
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::NodeAddChild { .. })));
}

#[test]
fn pointer_leave_clears_pending_activation() {
    ffi::test::reset();
    let checkbox = checkbox("Agree");
    let clicks = Rc::new(Cell::new(0));
    let clicks_clone = clicks.clone();
    checkbox.on_changed(move |_event| clicks_clone.set(clicks_clone.get() + 1));
    Application::mount(checkbox.clone());
    let node_ref = checkbox.retained_node_ref();

    press_down(&node_ref, 1);
    press_leave(&node_ref);
    press_up(&node_ref);
    assert_eq!(clicks.get(), 0);
}

#[test]
fn space_activates_on_key_up_only_and_blur_clears_key_state() {
    ffi::test::reset();
    let changes = Rc::new(Cell::new(0));
    let changes_clone = changes.clone();
    let checkbox = checkbox("Agree");
    checkbox.on_changed(move |_event| changes_clone.set(changes_clone.get() + 1));
    Application::mount(checkbox.clone());
    let calls = ffi::test::take_calls();
    let node_ref = checkbox.retained_node_ref();
    let handle = node_ref.handle();

    focus(&node_ref);
    key_event(KeyEventType::Down, "Space", 0);
    assert_eq!(changes.get(), 0);
    key_event(KeyEventType::Up, "Space", 0);
    assert_eq!(changes.get(), 1);

    focus(&node_ref);
    key_event(KeyEventType::Down, "Space", 0);
    blur(&node_ref);
    key_event(KeyEventType::Up, "Space", 0);
    assert_eq!(changes.get(), 1);

    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticRole { role_enum, .. } if *role_enum == SemanticRole::Checkbox as u32
    )));
    assert!(calls.iter().any(|call| matches!(
            call,
            Call::SetSemanticChecked { handle: checked_handle, checked_state_enum }
                if *checked_handle == handle.raw() && *checked_state_enum == SemanticCheckedState::False as u32
        )));
    Application::unmount();
}

#[test]
fn checkbox_tri_state_cycles_false_true_mixed_false() {
    ffi::test::reset();
    let last_state = Rc::new(Cell::new(CheckState::False));
    let last_checked = Rc::new(Cell::new(false));
    let last_state_clone = last_state.clone();
    let last_checked_clone = last_checked.clone();
    let checkbox = checkbox("Agree");
    checkbox.tri_state(true).on_changed(move |event| {
        last_state_clone.set(event.state);
        last_checked_clone.set(event.checked);
    });
    Application::mount(checkbox.clone());
    let node_ref = checkbox.retained_node_ref();

    press_down(&node_ref, 1);
    press_up(&node_ref);
    assert_eq!(last_state.get(), CheckState::True);
    assert!(last_checked.get());

    press_down(&node_ref, 1);
    press_up(&node_ref);
    assert_eq!(last_state.get(), CheckState::Mixed);
    assert!(!last_checked.get());

    press_down(&node_ref, 1);
    press_up(&node_ref);
    assert_eq!(last_state.get(), CheckState::False);
    assert!(!last_checked.get());
    Application::unmount();
}

#[test]
fn standalone_radio_sets_checked_once_and_does_not_toggle_back() {
    ffi::test::reset();
    let changes = Rc::new(Cell::new(0));
    let last_checked = Rc::new(Cell::new(false));
    let changes_clone = changes.clone();
    let last_checked_clone = last_checked.clone();
    let radio = radio_button("solo");
    radio.on_changed(move |event| {
        changes_clone.set(changes_clone.get() + 1);
        last_checked_clone.set(event.checked);
    });
    Application::mount(radio.clone());
    let node_ref = radio.retained_node_ref();

    press_down(&node_ref, 1);
    press_up(&node_ref);
    assert!(radio.is_checked());
    assert_eq!(changes.get(), 1);
    assert!(last_checked.get());

    press_down(&node_ref, 1);
    press_up(&node_ref);
    assert!(radio.is_checked());
    assert_eq!(changes.get(), 1);
}

#[test]
fn radio_group_add_option_and_selected_value_follow_selection() {
    ffi::test::reset();
    let group = radio_group();
    let alpha = group.add_option("alpha", "Alpha");
    let beta = group.add_option("beta", "Beta");
    Application::mount(group.clone());

    assert_eq!(group.selected_index(), -1);
    assert_eq!(group.selected_value(), "");

    let beta_ref = beta.retained_node_ref();
    press_down(&beta_ref, 1);
    press_up(&beta_ref);
    assert!(!alpha.is_checked());
    assert!(beta.is_checked());
    assert_eq!(group.selected_index(), 1);
    assert_eq!(group.selected_value(), "beta");
}

#[test]
fn radio_group_state_survives_after_wrapper_is_dropped() {
    ffi::test::reset();
    let last_value = Rc::new(RefCell::new(String::new()));
    let last_value_clone = last_value.clone();
    let alpha;
    let beta;

    {
        let group = radio_group();
        alpha = group.add_option("alpha", "Alpha");
        beta = group.add_option("beta", "Beta");
        group.select_index(0);
        group.on_changed(move |event| {
            *last_value_clone.borrow_mut() = event.value;
        });
        Application::mount(group.clone());
    }

    let beta_ref = beta.retained_node_ref();
    press_down(&beta_ref, 1);
    press_up(&beta_ref);

    assert!(!alpha.is_checked());
    assert!(beta.is_checked());
    assert_eq!(last_value.borrow().as_str(), "beta");
}

#[test]
fn radio_group_reselection_does_not_duplicate_events() {
    ffi::test::reset();
    let changes = Rc::new(Cell::new(0));
    let changes_clone = changes.clone();
    let group = radio_group();
    let alpha = group.add_option("alpha", "Alpha");
    group.add_option("beta", "Beta");
    group.on_changed(move |_event| {
        changes_clone.set(changes_clone.get() + 1);
    });
    Application::mount(group.clone());

    let alpha_ref = alpha.retained_node_ref();
    press_down(&alpha_ref, 1);
    press_up(&alpha_ref);
    press_down(&alpha_ref, 1);
    press_up(&alpha_ref);
    assert_eq!(changes.get(), 1);
    assert_eq!(group.selected_value(), "alpha");
}

#[test]
fn radio_group_arrow_keys_wrap_and_skip_disabled_radios() {
    ffi::test::reset();
    let group = radio_group();
    let alpha = group.add_option("alpha", "Alpha");
    let beta = group.add_option("beta", "Beta");
    let gamma = group.add_option("gamma", "Gamma");
    beta.enabled(false);
    group.select_index(0);
    Application::mount(group.clone());

    focus(&alpha.retained_node_ref());
    let gamma_handle = gamma.retained_node_ref().handle().raw();
    let _ = ffi::test::take_calls();
    assert!(key_event(KeyEventType::Down, "ArrowRight", 0));
    assert_eq!(group.selected_value(), "gamma");
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::RequestFocus { handle } if *handle == gamma_handle
    )));

    focus(&gamma.retained_node_ref());
    assert!(key_event(KeyEventType::Down, "ArrowRight", 0));
    assert_eq!(group.selected_value(), "alpha");

    focus(&alpha.retained_node_ref());
    assert!(key_event(KeyEventType::Down, "ArrowLeft", 0));
    assert_eq!(group.selected_value(), "gamma");
}

#[test]
fn radio_group_home_end_and_clear_follow_fui_as_behavior() {
    ffi::test::reset();
    let group = radio_group();
    let alpha = group.add_option("alpha", "Alpha");
    let beta = group.add_option("beta", "Beta");
    let gamma = group.add_option("gamma", "Gamma");
    beta.enabled(false);
    Application::mount(group.clone());

    focus(&gamma.retained_node_ref());
    let alpha_handle = alpha.retained_node_ref().handle().raw();
    let _ = ffi::test::take_calls();
    assert!(key_event(KeyEventType::Down, "Home", 0));
    assert_eq!(group.selected_value(), "alpha");
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::RequestFocus { handle } if *handle == alpha_handle
    )));

    focus(&alpha.retained_node_ref());
    let gamma_handle = gamma.retained_node_ref().handle().raw();
    let _ = ffi::test::take_calls();
    assert!(key_event(KeyEventType::Down, "End", 0));
    assert_eq!(group.selected_value(), "gamma");
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::RequestFocus { handle } if *handle == gamma_handle
    )));

    group.select_index(-1);
    assert_eq!(group.selected_index(), -1);
    assert_eq!(group.selected_value(), "");
    assert!(!alpha.is_checked());
    assert!(!gamma.is_checked());
}

#[test]
fn radio_group_changed_event_emits_selected_value() {
    ffi::test::reset();
    let last_value = Rc::new(RefCell::new(String::new()));
    let last_value_clone = last_value.clone();
    let group = radio_group();
    group.add_option("alpha", "Alpha");
    let beta = group.add_option("beta", "Beta");
    group.on_changed(move |event| {
        *last_value_clone.borrow_mut() = event.value;
    });
    Application::mount(group.clone());

    let beta_ref = beta.retained_node_ref();
    press_down(&beta_ref, 1);
    press_up(&beta_ref);
    assert_eq!(last_value.borrow().as_str(), "beta");

    group.select_index(-1);
    assert_eq!(last_value.borrow().as_str(), "");
}

#[test]
fn switch_pointer_activation_toggles_and_announces() {
    ffi::test::reset();
    let changes = Rc::new(Cell::new(0));
    let last_checked = Rc::new(Cell::new(false));
    let changes_clone = changes.clone();
    let last_checked_clone = last_checked.clone();
    let toggle = switch("Toggle");
    toggle.on_changed(move |event| {
        changes_clone.set(changes_clone.get() + 1);
        last_checked_clone.set(event.checked);
    });
    Application::mount(toggle.clone());
    let toggle_ref = toggle.retained_node_ref();
    let handle = toggle_ref.handle().raw();

    press_down(&toggle_ref, 1);
    press_up(&toggle_ref);
    assert!(toggle.is_checked());
    assert_eq!(changes.get(), 1);
    assert!(last_checked.get());

    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticChecked { handle: checked_handle, checked_state_enum }
            if *checked_handle == handle && *checked_state_enum == SemanticCheckedState::True as u32
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::RequestSemanticAnnouncement { handle: announced } if *announced == handle
    )));
}

#[test]
fn switch_space_key_activation_toggles() {
    ffi::test::reset();
    let changes = Rc::new(Cell::new(0));
    let changes_clone = changes.clone();
    let toggle = switch("Toggle");
    toggle.on_changed(move |_event| {
        changes_clone.set(changes_clone.get() + 1);
    });
    Application::mount(toggle.clone());
    let toggle_ref = toggle.retained_node_ref();

    focus(&toggle_ref);
    assert!(key_event(KeyEventType::Down, "Space", 0));
    assert!(key_event(KeyEventType::Up, "Space", 0));

    assert!(toggle.is_checked());
    assert_eq!(changes.get(), 1);
}

#[test]
fn switch_programmatic_check_emits_without_semantic_announcement() {
    ffi::test::reset();
    let changes = Rc::new(Cell::new(0));
    let changes_clone = changes.clone();
    let toggle = switch("Toggle");
    toggle.on_changed(move |_event| {
        changes_clone.set(changes_clone.get() + 1);
    });

    toggle.check(true);

    assert!(toggle.is_checked());
    assert_eq!(changes.get(), 1);
    let calls = ffi::test::take_calls();
    assert!(!calls
        .iter()
        .any(|call| matches!(call, Call::RequestSemanticAnnouncement { .. })));
}

#[test]
fn checkbox_mixed_is_ignored_or_coerced_when_tri_state_disabled() {
    ffi::test::reset();
    let changes = Rc::new(Cell::new(0));
    let changes_clone = changes.clone();
    let checkbox = checkbox("Agree");
    checkbox.on_changed(move |_event| changes_clone.set(changes_clone.get() + 1));

    checkbox.mixed(true);
    assert_eq!(checkbox.checked_state(), CheckState::False);
    checkbox.check_state(CheckState::Mixed);
    assert_eq!(checkbox.checked_state(), CheckState::False);
    assert_eq!(changes.get(), 0);

    checkbox.tri_state(true).mixed(true);
    assert_eq!(checkbox.checked_state(), CheckState::Mixed);
    assert_eq!(changes.get(), 1);

    checkbox.tri_state(false);
    assert_eq!(checkbox.checked_state(), CheckState::False);
    assert_eq!(changes.get(), 2);
}

#[derive(Clone)]
struct TestCheckboxIndicatorPresenter {
    root: FlexBox,
    applied_checked_state: Rc<Cell<SemanticCheckedState>>,
    applied_state: Rc<RefCell<Option<CheckboxIndicatorVisualState>>>,
    applied_colors: Rc<Cell<Option<LabeledControlColors>>>,
}

impl TestCheckboxIndicatorPresenter {
    fn new(
        applied_checked_state: Rc<Cell<SemanticCheckedState>>,
        applied_state: Rc<RefCell<Option<CheckboxIndicatorVisualState>>>,
        applied_colors: Rc<Cell<Option<LabeledControlColors>>>,
    ) -> Self {
        Self {
            root: flex_box(),
            applied_checked_state,
            applied_state,
            applied_colors,
        }
    }
}

impl PressableIndicatorPresenter for TestCheckboxIndicatorPresenter {
    fn root(&self) -> FlexBox {
        self.root.clone()
    }

    fn metrics(&self) -> PressableIndicatorMetrics {
        PressableIndicatorMetrics::new(20.0, 20.0)
    }
}

impl CheckboxIndicatorPresenter for TestCheckboxIndicatorPresenter {
    fn apply(
        &self,
        _theme: crate::theme::Theme,
        state: CheckboxIndicatorVisualState,
        colors: Option<LabeledControlColors>,
    ) {
        self.applied_checked_state.set(state.checked_state);
        *self.applied_state.borrow_mut() = Some(state);
        self.applied_colors.set(colors);
    }
}

struct TestCheckboxIndicatorTemplate {
    created: Rc<Cell<u32>>,
    applied_checked_state: Rc<Cell<SemanticCheckedState>>,
    applied_state: Rc<RefCell<Option<CheckboxIndicatorVisualState>>>,
    applied_colors: Rc<Cell<Option<LabeledControlColors>>>,
}

impl CheckboxIndicatorTemplate for TestCheckboxIndicatorTemplate {
    fn create(&self, _sizing: Option<LabeledControlSizing>) -> Rc<dyn CheckboxIndicatorPresenter> {
        self.created.set(self.created.get() + 1);
        Rc::new(TestCheckboxIndicatorPresenter::new(
            self.applied_checked_state.clone(),
            self.applied_state.clone(),
            self.applied_colors.clone(),
        ))
    }
}

#[derive(Clone)]
struct TestRadioIndicatorPresenter {
    root: FlexBox,
    checked: Rc<Cell<bool>>,
}

impl TestRadioIndicatorPresenter {
    fn new(checked: Rc<Cell<bool>>) -> Self {
        Self {
            root: flex_box(),
            checked,
        }
    }
}

impl PressableIndicatorPresenter for TestRadioIndicatorPresenter {
    fn root(&self) -> FlexBox {
        self.root.clone()
    }

    fn metrics(&self) -> PressableIndicatorMetrics {
        PressableIndicatorMetrics::new(20.0, 20.0)
    }
}

impl RadioIndicatorPresenter for TestRadioIndicatorPresenter {
    fn apply(
        &self,
        _theme: crate::theme::Theme,
        state: RadioIndicatorVisualState,
        _colors: Option<LabeledControlColors>,
    ) {
        self.checked.set(state.checked);
    }
}

struct TestRadioIndicatorTemplate {
    created: Rc<Cell<u32>>,
    checked: Rc<Cell<bool>>,
}

impl RadioIndicatorTemplate for TestRadioIndicatorTemplate {
    fn create(&self, _sizing: Option<LabeledControlSizing>) -> Rc<dyn RadioIndicatorPresenter> {
        self.created.set(self.created.get() + 1);
        Rc::new(TestRadioIndicatorPresenter::new(self.checked.clone()))
    }
}

#[derive(Clone)]
struct TestSwitchIndicatorPresenter {
    root: FlexBox,
    checked: Rc<Cell<bool>>,
}

impl TestSwitchIndicatorPresenter {
    fn new(checked: Rc<Cell<bool>>) -> Self {
        Self {
            root: flex_box(),
            checked,
        }
    }
}

impl PressableIndicatorPresenter for TestSwitchIndicatorPresenter {
    fn root(&self) -> FlexBox {
        self.root.clone()
    }

    fn metrics(&self) -> PressableIndicatorMetrics {
        PressableIndicatorMetrics::new(44.0, 26.0)
    }
}

impl SwitchIndicatorPresenter for TestSwitchIndicatorPresenter {
    fn apply(
        &self,
        _theme: crate::theme::Theme,
        state: SwitchIndicatorVisualState,
        _colors: Option<LabeledControlColors>,
    ) {
        self.checked.set(state.checked);
    }
}

struct TestSwitchIndicatorTemplate {
    created: Rc<Cell<u32>>,
    checked: Rc<Cell<bool>>,
}

impl SwitchIndicatorTemplate for TestSwitchIndicatorTemplate {
    fn create(&self, _sizing: Option<LabeledControlSizing>) -> Rc<dyn SwitchIndicatorPresenter> {
        self.created.set(self.created.get() + 1);
        Rc::new(TestSwitchIndicatorPresenter::new(self.checked.clone()))
    }
}

#[derive(Clone)]
struct TestSliderPresenter {
    root: FlexBox,
    metrics: SliderPresenterMetrics,
    last_layout_state: Rc<RefCell<Option<SliderVisualState>>>,
    last_layout_length: Rc<Cell<f32>>,
    last_apply_state: Rc<RefCell<Option<SliderVisualState>>>,
    last_apply_colors: Rc<Cell<Option<SliderColors>>>,
}

impl TestSliderPresenter {
    fn new(
        metrics: SliderPresenterMetrics,
        last_layout_state: Rc<RefCell<Option<SliderVisualState>>>,
        last_layout_length: Rc<Cell<f32>>,
        last_apply_state: Rc<RefCell<Option<SliderVisualState>>>,
        last_apply_colors: Rc<Cell<Option<SliderColors>>>,
    ) -> Self {
        Self {
            root: flex_box(),
            metrics,
            last_layout_state,
            last_layout_length,
            last_apply_state,
            last_apply_colors,
        }
    }
}

impl SliderPresenter for TestSliderPresenter {
    fn root(&self) -> FlexBox {
        self.root.clone()
    }

    fn metrics(&self) -> SliderPresenterMetrics {
        self.metrics
    }

    fn layout(&self, state: SliderVisualState, length: f32) {
        *self.last_layout_state.borrow_mut() = Some(state);
        self.last_layout_length.set(length);
    }

    fn apply(
        &self,
        _theme: crate::theme::Theme,
        state: SliderVisualState,
        colors: Option<SliderColors>,
    ) {
        *self.last_apply_state.borrow_mut() = Some(state);
        self.last_apply_colors.set(colors);
    }
}

struct TestSliderTemplate {
    created: Rc<Cell<u32>>,
    metrics: SliderPresenterMetrics,
    last_layout_state: Rc<RefCell<Option<SliderVisualState>>>,
    last_layout_length: Rc<Cell<f32>>,
    last_apply_state: Rc<RefCell<Option<SliderVisualState>>>,
    last_apply_colors: Rc<Cell<Option<SliderColors>>>,
}

impl SliderTemplate for TestSliderTemplate {
    fn create(&self, _sizing: Option<SliderSizing>) -> Rc<dyn SliderPresenter> {
        self.created.set(self.created.get() + 1);
        Rc::new(TestSliderPresenter::new(
            self.metrics,
            self.last_layout_state.clone(),
            self.last_layout_length.clone(),
            self.last_apply_state.clone(),
            self.last_apply_colors.clone(),
        ))
    }
}

#[test]
fn checkbox_template_and_sizing_follow_fui_as_surface() {
    ffi::test::reset();
    clear_control_templates();
    let created = Rc::new(Cell::new(0));
    let applied_checked_state = Rc::new(Cell::new(SemanticCheckedState::False));
    let applied_state = Rc::new(RefCell::new(None));
    let applied_colors = Rc::new(Cell::new(None));
    let templates = ControlTemplateSet {
        checkbox_indicator: Some(Rc::new(TestCheckboxIndicatorTemplate {
            created: created.clone(),
            applied_checked_state: applied_checked_state.clone(),
            applied_state: applied_state.clone(),
            applied_colors: applied_colors.clone(),
        })),
        ..Default::default()
    };
    use_control_templates(Some(templates));
    let checkbox = checkbox("Agree");

    checkbox.sizing(Some(LabeledControlSizing::new().label_font_size(21.0)));
    checkbox.check(true);

    assert_eq!(created.get(), 2);
    assert_eq!(applied_checked_state.get(), SemanticCheckedState::True);
    clear_control_templates();
}

#[test]
fn checkbox_local_template_replaces_indicator_before_mount() {
    ffi::test::reset();
    clear_control_templates();
    let created = Rc::new(Cell::new(0));
    let applied_checked_state = Rc::new(Cell::new(SemanticCheckedState::False));
    let applied_state = Rc::new(RefCell::new(None));
    let applied_colors = Rc::new(Cell::new(None));
    let checkbox = checkbox("Local template");
    checkbox
        .template(Some(Rc::new(TestCheckboxIndicatorTemplate {
            created: created.clone(),
            applied_checked_state: applied_checked_state.clone(),
            applied_state: applied_state.clone(),
            applied_colors: applied_colors.clone(),
        })))
        .check(true);

    Application::mount(checkbox.clone());

    assert_eq!(created.get(), 1);
    assert_eq!(applied_checked_state.get(), SemanticCheckedState::True);
    assert!(applied_state.borrow().is_some());
}

#[test]
fn checkbox_presenter_receives_fui_as_visual_state_and_colors() {
    ffi::test::reset();
    clear_control_templates();
    let created = Rc::new(Cell::new(0));
    let applied_checked_state = Rc::new(Cell::new(SemanticCheckedState::False));
    let applied_state = Rc::new(RefCell::new(None));
    let applied_colors = Rc::new(Cell::new(None));
    use_control_templates(Some(ControlTemplateSet {
        checkbox_indicator: Some(Rc::new(TestCheckboxIndicatorTemplate {
            created,
            applied_checked_state,
            applied_state: applied_state.clone(),
            applied_colors: applied_colors.clone(),
        })),
        ..Default::default()
    }));
    let colors = LabeledControlColors::new()
        .accent(0xAA00AAFF)
        .background(0x111111FF)
        .border(0x222222FF)
        .text_primary(0x333333FF)
        .text_muted(0x444444FF);
    let checkbox = checkbox("Agree");
    checkbox.tri_state(true).colors(Some(colors));
    Application::mount(checkbox.clone());
    let node_ref = checkbox.retained_node_ref();

    press_enter(&node_ref);
    press_down(&node_ref, 1);
    checkbox.mixed(true);
    let state = applied_state.borrow().expect("presenter state");
    assert_eq!(state.checked_state, SemanticCheckedState::Mixed);
    assert!(state.hovered);
    assert!(state.pressed);
    assert!(state.enabled);
    assert_eq!(applied_colors.get(), Some(colors));

    focus(&node_ref);
    blur(&node_ref);
    let state = applied_state.borrow().expect("presenter state after blur");
    assert!(!state.pressed);

    checkbox.enabled(false);
    let state = applied_state
        .borrow()
        .expect("presenter state after disabled");
    assert!(!state.hovered);
    assert!(!state.pressed);
    assert!(!state.enabled);

    Application::unmount();
    clear_control_templates();
}

#[test]
fn checkbox_default_presenter_matches_fui_as_mark_opacity() {
    ffi::test::reset();
    let checkbox = checkbox("Agree");
    Application::mount(checkbox.clone());
    let _ = ffi::test::take_calls();

    checkbox.check(true);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetLayerEffect { opacity, .. } if (*opacity - 1.0).abs() < f32::EPSILON
    )));

    checkbox.check(false);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetLayerEffect { opacity, .. } if (*opacity - 0.0).abs() < f32::EPSILON
    )));

    Application::unmount();
}

#[test]
fn labeled_control_sizing_and_label_colors_reach_host_calls() {
    ffi::test::reset();
    let checkbox = checkbox("Agree");
    checkbox
        .sizing(Some(LabeledControlSizing::new().label_font_size(23.0)))
        .colors(Some(
            LabeledControlColors::new()
                .text_primary(0x123456FF)
                .text_muted(0xABCDEF88),
        ));
    Application::mount(checkbox.clone());

    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetFont { size, .. } if (*size - 23.0).abs() < f32::EPSILON
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetTextColor { color, .. } if *color == 0x123456FF
    )));

    checkbox.enabled(false);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetTextColor { color, .. } if *color == 0xABCDEF88
    )));

    Application::unmount();
}

#[test]
fn radio_button_template_and_sizing_follow_fui_as_surface() {
    ffi::test::reset();
    clear_control_templates();
    let created = Rc::new(Cell::new(0));
    let checked = Rc::new(Cell::new(false));
    let templates = ControlTemplateSet {
        radio_indicator: Some(Rc::new(TestRadioIndicatorTemplate {
            created: created.clone(),
            checked: checked.clone(),
        })),
        ..Default::default()
    };
    use_control_templates(Some(templates));
    let radio = radio_button("alpha");

    radio
        .sizing(Some(LabeledControlSizing::new().label_font_size(19.0)))
        .check(true);

    assert_eq!(created.get(), 2);
    assert!(checked.get());
    clear_control_templates();
}

#[test]
fn switch_template_surface_follows_fui_as() {
    ffi::test::reset();
    clear_control_templates();
    let created = Rc::new(Cell::new(0));
    let checked = Rc::new(Cell::new(false));
    let templates = ControlTemplateSet {
        switch_indicator: Some(Rc::new(TestSwitchIndicatorTemplate {
            created: created.clone(),
            checked: checked.clone(),
        })),
        ..Default::default()
    };
    use_control_templates(Some(templates));
    let toggle = switch("Toggle");

    toggle.check(true);

    assert_eq!(created.get(), 1);
    assert!(checked.get());
    clear_control_templates();
}

#[test]
fn switch_sizing_updates_default_indicator_and_recreates_custom_template() {
    ffi::test::reset();
    clear_control_templates();
    let toggle = switch("Sized switch");
    toggle.sizing(Some(
        LabeledControlSizing::new()
            .indicator_size(32.0)
            .label_font_size(18.0),
    ));
    Application::mount(toggle.clone());
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetHeight { value, unit_enum, .. }
            if (*value - 32.0).abs() < f32::EPSILON && *unit_enum == Unit::Pixel as u32
    )));

    let created = Rc::new(Cell::new(0));
    let checked = Rc::new(Cell::new(false));
    let toggle = switch("Template switch");
    toggle
        .template(Some(Rc::new(TestSwitchIndicatorTemplate {
            created: created.clone(),
            checked: checked.clone(),
        })))
        .sizing(Some(LabeledControlSizing::new().indicator_size(30.0)));
    assert_eq!(created.get(), 2);
}

#[test]
fn slider_default_presenter_uses_sizing_for_geometry() {
    ffi::test::reset();
    let slider = slider();
    slider
        .sizing(Some(
            SliderSizing::new().thumb_size(24.0).track_thickness(8.0),
        ))
        .length(120.0);
    Application::mount(slider.clone());
    let calls = ffi::test::take_calls();
    let handle = slider.retained_node_ref().handle().raw();

    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetWidth { handle: call_handle, value, .. }
            if *call_handle == handle && (*value - 130.0).abs() < f32::EPSILON
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetHeight { handle: call_handle, value, .. }
            if *call_handle == handle && (*value - 36.0).abs() < f32::EPSILON
    )));

    slider.orientation(Orientation::Vertical);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetWidth { handle: call_handle, value, .. }
            if *call_handle == handle && (*value - 36.0).abs() < f32::EPSILON
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetHeight { handle: call_handle, value, .. }
            if *call_handle == handle && (*value - 130.0).abs() < f32::EPSILON
    )));
}

#[test]
fn slider_template_replacement_and_colors_follow_fui_as_surface() {
    ffi::test::reset();
    clear_control_templates();
    let created = Rc::new(Cell::new(0));
    let last_layout_state = Rc::new(RefCell::new(None));
    let last_layout_length = Rc::new(Cell::new(0.0));
    let last_apply_state = Rc::new(RefCell::new(None));
    let last_apply_colors = Rc::new(Cell::new(None));
    let slider = slider();
    Application::mount(slider.clone());
    let _ = ffi::test::take_calls();

    let template: Rc<dyn SliderTemplate> = Rc::new(TestSliderTemplate {
        created: created.clone(),
        metrics: SliderPresenterMetrics::new(26.0, 8.0).with_cross_axis_extra(4.0),
        last_layout_state: last_layout_state.clone(),
        last_layout_length: last_layout_length.clone(),
        last_apply_state: last_apply_state.clone(),
        last_apply_colors: last_apply_colors.clone(),
    });
    let colors = SliderColors::new()
        .track(0x112233FF)
        .fill(0x445566FF)
        .thumb(0x778899FF);

    slider
        .template(Some(template))
        .colors(Some(colors))
        .orientation(Orientation::Vertical)
        .length(140.0)
        .value(50.0);

    assert_eq!(created.get(), 1);
    assert_eq!(last_layout_length.get(), 140.0);
    let layout_state = last_layout_state.borrow().expect("slider layout state");
    assert_eq!(layout_state.orientation, Orientation::Vertical);
    let apply_state = last_apply_state.borrow().expect("slider apply state");
    assert_eq!(apply_state.value, 50.0);
    assert_eq!(last_apply_colors.get(), Some(colors));

    let calls = ffi::test::take_calls();
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::NodeRemoveChild { .. })));

    clear_control_templates();
}

#[test]
fn slider_control_template_set_and_local_template_precedence_follow_fui_as() {
    ffi::test::reset();
    clear_control_templates();
    let app_created = Rc::new(Cell::new(0));
    let local_created = Rc::new(Cell::new(0));
    let app_layout_state = Rc::new(RefCell::new(None));
    let app_layout_length = Rc::new(Cell::new(0.0));
    let app_apply_state = Rc::new(RefCell::new(None));
    let app_apply_colors = Rc::new(Cell::new(None));
    use_control_templates(Some(ControlTemplateSet {
        slider: Some(Rc::new(TestSliderTemplate {
            created: app_created.clone(),
            metrics: SliderPresenterMetrics::new(22.0, 7.0),
            last_layout_state: app_layout_state,
            last_layout_length: app_layout_length,
            last_apply_state: app_apply_state,
            last_apply_colors: app_apply_colors,
        })),
        ..Default::default()
    }));

    let slider = slider();
    slider.sizing(Some(SliderSizing::new().thumb_size(30.0)));
    assert_eq!(app_created.get(), 2);

    let local_layout_state = Rc::new(RefCell::new(None));
    let local_layout_length = Rc::new(Cell::new(0.0));
    let local_apply_state = Rc::new(RefCell::new(None));
    let local_apply_colors = Rc::new(Cell::new(None));
    slider.template(Some(Rc::new(TestSliderTemplate {
        created: local_created.clone(),
        metrics: SliderPresenterMetrics::new(28.0, 9.0),
        last_layout_state: local_layout_state.clone(),
        last_layout_length: local_layout_length,
        last_apply_state: local_apply_state,
        last_apply_colors: local_apply_colors,
    })));
    slider
        .sizing(Some(SliderSizing::new().thumb_size(36.0)))
        .length(160.0)
        .value(40.0);

    assert_eq!(app_created.get(), 2);
    assert_eq!(local_created.get(), 2);
    let state = local_layout_state
        .borrow()
        .expect("local slider template layout state");
    assert_eq!(state.value, 40.0);
    clear_control_templates();
}

#[test]
fn slider_presenter_receives_hover_drag_and_disabled_state() {
    ffi::test::reset();
    clear_control_templates();
    let last_layout_state = Rc::new(RefCell::new(None));
    let last_layout_length = Rc::new(Cell::new(0.0));
    let last_apply_state = Rc::new(RefCell::new(None));
    let last_apply_colors = Rc::new(Cell::new(None));
    let slider = slider();
    slider.template(Some(Rc::new(TestSliderTemplate {
        created: Rc::new(Cell::new(0)),
        metrics: SliderPresenterMetrics::new(18.0, 6.0),
        last_layout_state: last_layout_state.clone(),
        last_layout_length,
        last_apply_state: last_apply_state.clone(),
        last_apply_colors,
    })));
    Application::mount(slider.clone());
    let _ = ffi::test::take_calls();
    let node_ref = slider.retained_node_ref();

    press_enter(&node_ref);
    let state = last_apply_state.borrow().expect("hover state");
    assert!(state.hovered);
    assert!(!state.dragging);
    assert!(state.enabled);

    press_down(&node_ref, 1);
    let state = last_apply_state.borrow().expect("drag state");
    assert!(state.hovered);
    assert!(state.dragging);

    press_up(&node_ref);
    let state = last_apply_state.borrow().expect("released state");
    assert!(state.hovered);
    assert!(!state.dragging);

    slider.enabled(false);
    let state = last_apply_state.borrow().expect("disabled state");
    assert!(!state.enabled);
}

#[test]
fn checkbox_programmatic_state_changes_emit_changed_event() {
    ffi::test::reset();
    let changes = Rc::new(Cell::new(0));
    let changes_clone = changes.clone();
    let checkbox = checkbox("Agree");
    checkbox.on_changed(move |_event| changes_clone.set(changes_clone.get() + 1));

    checkbox.check(true);
    checkbox.check(false);
    checkbox.check_state(CheckState::True);
    checkbox.tri_state(true).mixed(true);
    assert_eq!(changes.get(), 4);
    let calls = ffi::test::take_calls();
    assert!(!calls
        .iter()
        .any(|call| matches!(call, Call::RequestSemanticAnnouncement { .. })));
}

#[test]
fn checkbox_user_activation_emits_changed_and_semantic_announcement() {
    ffi::test::reset();
    let changes = Rc::new(Cell::new(0));
    let last_state = Rc::new(Cell::new(CheckState::False));
    let changes_clone = changes.clone();
    let last_state_clone = last_state.clone();
    let checkbox = checkbox("Agree");
    checkbox.on_changed(move |event| {
        changes_clone.set(changes_clone.get() + 1);
        last_state_clone.set(event.state);
    });
    Application::mount(checkbox.clone());
    let _ = ffi::test::take_calls();
    let node_ref = checkbox.retained_node_ref();
    let handle = node_ref.handle().raw();

    press_down(&node_ref, 1);
    press_up(&node_ref);
    assert_eq!(changes.get(), 1);
    assert_eq!(last_state.get(), CheckState::True);

    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticChecked { handle: checked_handle, checked_state_enum }
            if *checked_handle == handle && *checked_state_enum == SemanticCheckedState::True as u32
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::RequestSemanticAnnouncement { handle: announced } if *announced == handle
    )));
    Application::unmount();
}

#[test]
fn radio_and_switch_programmatic_changes_emit_without_semantic_announcement() {
    ffi::test::reset();
    let radio_changes = Rc::new(Cell::new(0));
    let switch_changes = Rc::new(Cell::new(0));
    let radio_changes_clone = radio_changes.clone();
    let switch_changes_clone = switch_changes.clone();
    let radio = radio_button("Option");
    let switch = switch("Toggle");
    radio.on_changed(move |_event| radio_changes_clone.set(radio_changes_clone.get() + 1));
    switch.on_changed(move |_event| switch_changes_clone.set(switch_changes_clone.get() + 1));

    radio.checked(true);
    switch.checked(true);

    assert_eq!(radio_changes.get(), 1);
    assert_eq!(switch_changes.get(), 1);
    let calls = ffi::test::take_calls();
    assert!(!calls
        .iter()
        .any(|call| matches!(call, Call::RequestSemanticAnnouncement { .. })));
}

#[test]
fn checkbox_persisted_state_restores_and_emits_without_announcement() {
    ffi::test::reset();
    let source = checkbox("Agree");
    source.node_id("checkbox-persisted");
    source.tri_state(true).check_state(CheckState::Mixed);
    Application::mount(source.clone());
    Application::capture_persisted_ui_state();
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetPersistedState { node_id, kind, version, payload }
            if node_id == "checkbox-persisted"
                && kind == "checkbox-checked-state"
                && *version == 1
                && payload == "2"
    )));

    let changes = Rc::new(Cell::new(0));
    let changes_clone = changes.clone();
    let restored = checkbox("Agree");
    restored.node_id("checkbox-persisted");
    restored.tri_state(true);
    restored.on_changed(move |_event| changes_clone.set(changes_clone.get() + 1));
    Application::mount(restored.clone());
    let _ = ffi::test::take_calls();
    Application::restore_persisted_ui_state();

    assert_eq!(restored.checked_state(), CheckState::Mixed);
    assert_eq!(changes.get(), 1);
    let calls = ffi::test::take_calls();
    assert!(!calls
        .iter()
        .any(|call| matches!(call, Call::RequestSemanticAnnouncement { .. })));
    Application::unmount();
}

#[test]
fn switch_persisted_state_restores_and_emits_without_announcement() {
    ffi::test::reset();
    let source = switch("Toggle");
    source.node_id("switch-persisted");
    source.check(true);
    Application::mount(source.clone());
    Application::capture_persisted_ui_state();
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetPersistedState { node_id, kind, version, payload }
            if node_id == "switch-persisted"
                && kind == "switch-checked"
                && *version == 1
                && payload == "true"
    )));

    let changes = Rc::new(Cell::new(0));
    let changes_clone = changes.clone();
    let restored = switch("Toggle");
    restored.node_id("switch-persisted");
    restored.on_changed(move |_event| changes_clone.set(changes_clone.get() + 1));
    Application::mount(restored.clone());
    let _ = ffi::test::take_calls();
    Application::restore_persisted_ui_state();

    assert!(restored.is_checked());
    assert_eq!(changes.get(), 1);
    let calls = ffi::test::take_calls();
    assert!(!calls
        .iter()
        .any(|call| matches!(call, Call::RequestSemanticAnnouncement { .. })));
}

#[test]
fn persisted_restore_visits_children_before_parent() {
    ffi::test::reset();
    let root = flex_box();
    let child = switch("Child");
    root.node_id("restore-order-root").child(&child);
    child.node_id("restore-order-child").check(true);
    Application::mount(root.clone());
    Application::capture_persisted_ui_state();
    let _ = ffi::test::take_calls();

    let restore_order = Rc::new(RefCell::new(Vec::<&'static str>::new()));
    let restored_root = flex_box();
    let restored_child = switch("Child");
    restored_root
        .node_id("restore-order-root")
        .child(&restored_child);
    restored_child.node_id("restore-order-child");
    let child_order = restore_order.clone();
    restored_child.on_changed(move |_event| {
        child_order.borrow_mut().push("child");
    });
    let parent_order = restore_order.clone();
    restored_root.persist_state(crate::persisted::persisted_value_adapter(
        "parent-order",
        crate::persisted::PersistedBoolCodec,
        1,
        || Some(true),
        move |_| {
            parent_order.borrow_mut().push("parent");
        },
    ));
    crate::persisted::store_text_state("restore-order-root", "parent-order", 1, "true");
    Application::mount(restored_root);
    let _ = ffi::test::take_calls();
    Application::restore_persisted_ui_state();

    assert_eq!(restore_order.borrow().as_slice(), ["child", "parent"]);
}

#[test]
fn radio_group_persisted_state_restores_and_emits_without_announcement() {
    ffi::test::reset();
    let source = radio_group();
    source.node_id("radio-group-persisted");
    source.add_option("alpha", "Alpha");
    source.add_option("beta", "Beta");
    source.add_option("gamma", "Gamma");
    source.select_index(2);
    Application::mount(source.clone());
    Application::capture_persisted_ui_state();
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetPersistedState { node_id, kind, version, payload }
            if node_id == "radio-group-persisted"
                && kind == "radio-group-selected-index"
                && *version == 1
                && payload == "2"
    )));

    let last_value = Rc::new(RefCell::new(String::new()));
    let last_value_clone = last_value.clone();
    let restored = radio_group();
    restored.node_id("radio-group-persisted");
    restored.add_option("alpha", "Alpha");
    restored.add_option("beta", "Beta");
    restored.add_option("gamma", "Gamma");
    restored.on_changed(move |event| {
        *last_value_clone.borrow_mut() = event.value;
    });
    Application::mount(restored.clone());
    let _ = ffi::test::take_calls();
    Application::restore_persisted_ui_state();

    assert_eq!(restored.selected_index(), 2);
    assert_eq!(restored.selected_value(), "gamma");
    assert_eq!(last_value.borrow().as_str(), "gamma");
    let calls = ffi::test::take_calls();
    assert!(!calls
        .iter()
        .any(|call| matches!(call, Call::RequestSemanticAnnouncement { .. })));
}

#[test]
fn slider_programmatic_changes_emit_without_semantic_announcement() {
    ffi::test::reset();
    let changes = Rc::new(Cell::new(0));
    let last_value = Rc::new(Cell::new(25.0));
    let changes_clone = changes.clone();
    let last_value_clone = last_value.clone();
    let slider = slider();
    slider.value(25.0).on_changed(move |event| {
        changes_clone.set(changes_clone.get() + 1);
        last_value_clone.set(event.value);
    });

    slider.min(30.0);
    slider.max(80.0);
    slider.step(7.0);
    slider.value(63.0);

    assert_eq!(changes.get(), 2);
    assert_eq!(last_value.get(), 65.0);
    let calls = ffi::test::take_calls();
    assert!(!calls
        .iter()
        .any(|call| matches!(call, Call::RequestSemanticAnnouncement { .. })));
}

#[test]
fn slider_focus_adorner_tracks_keyboard_focus_visibility() {
    ffi::test::reset();
    focus_visibility::reset_keyboard_focus_visibility();
    let slider = slider();
    Application::mount(slider.clone());
    let _ = ffi::test::take_calls();
    let node_ref = slider.retained_node_ref();
    let handle = node_ref.handle().raw();

    focus_visibility::show_keyboard_focus_for_pointer_event(PointerEventType::Down);
    let _ = ffi::test::take_calls();
    event::__fui_on_focus_changed(handle, true);
    let calls = ffi::test::take_calls();
    assert!(!calls
        .iter()
        .any(|call| matches!(call, Call::NodeAddChild { .. })));

    key_event(KeyEventType::Down, "Tab", 0);
    let calls = ffi::test::take_calls();
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::NodeAddChild { .. })));
}

#[test]
fn slider_invalid_step_and_length_warn_and_clamp() {
    ffi::test::reset();
    let slider = slider();
    slider.step(0.0).length(10.0);
    Application::mount(slider.clone());
    let calls = ffi::test::take_calls();
    let handle = slider.retained_node_ref().handle().raw();

    assert!(calls.iter().any(|call| matches!(
        call,
        Call::Log { category, message }
            if category == "Warning/Layout"
                && message == "Slider.step() received 0; clamping to 1.0."
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::Log { category, message }
            if category == "Warning/Layout"
                && message == "Slider.length() received 10; clamping to a value above the thumb size."
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetWidth { handle: call_handle, value, .. }
            if *call_handle == handle && *value == 29.0
    )));
}

#[test]
fn slider_persisted_state_restores_and_emits_without_announcement() {
    ffi::test::reset();
    let source = slider();
    source.node_id("slider-persisted");
    source.value(63.0);
    Application::mount(source.clone());
    Application::capture_persisted_ui_state();
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetPersistedState { node_id, kind, version, payload }
            if node_id == "slider-persisted"
                && kind == "slider-value"
                && *version == 1
                && payload.starts_with("63")
    )));

    let changes = Rc::new(Cell::new(0));
    let last_value = Rc::new(Cell::new(0.0));
    let changes_clone = changes.clone();
    let last_value_clone = last_value.clone();
    let restored = slider();
    restored.node_id("slider-persisted");
    restored.on_changed(move |event| {
        changes_clone.set(changes_clone.get() + 1);
        last_value_clone.set(event.value);
    });
    Application::mount(restored.clone());
    let _ = ffi::test::take_calls();
    Application::restore_persisted_ui_state();

    assert_eq!(restored.current_value(), 63.0);
    assert_eq!(changes.get(), 1);
    assert_eq!(last_value.get(), 63.0);
    let calls = ffi::test::take_calls();
    assert!(!calls
        .iter()
        .any(|call| matches!(call, Call::RequestSemanticAnnouncement { .. })));
}

#[test]
fn slider_programmatic_noop_does_not_emit_duplicate_changed() {
    ffi::test::reset();
    let changes = Rc::new(Cell::new(0));
    let changes_clone = changes.clone();
    let slider = slider();
    slider.step(5.0).value(25.0).on_changed(move |_event| {
        changes_clone.set(changes_clone.get() + 1);
    });

    slider.value(26.0);
    slider.value(24.0);

    assert_eq!(changes.get(), 0);
}

#[test]
fn slider_default_semantic_label_does_not_overwrite_explicit_label() {
    ffi::test::reset();
    let slider = slider();
    slider
        .semantic_label("Gain")
        .orientation(Orientation::Vertical);
    Application::mount(slider.clone());
    let calls = ffi::test::take_calls();
    let handle = slider.retained_node_ref().handle().raw();

    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticLabel { handle: call_handle, label }
            if *call_handle == handle && label == "Gain"
    )));
    assert!(!calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticLabel { handle: call_handle, label }
            if *call_handle == handle && label == "Vertical slider"
    )));

    slider.orientation(Orientation::Horizontal);
    let calls = ffi::test::take_calls();
    assert!(!calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticLabel { handle: call_handle, label }
            if *call_handle == handle && label == "Slider"
    )));
}

#[test]
fn progress_bar_clamps_value_and_updates_semantics() {
    ffi::test::reset();
    let progress = progress_bar();
    progress.min(20.0).max(80.0).value(120.0);

    assert_eq!(progress.current_value(), 80.0);

    Application::mount(progress.clone());
    let calls = ffi::test::take_calls();
    let root_handle = progress.retained_node_ref().handle().raw();

    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticValueRange { handle, has_value_range, value_now, value_min, value_max }
            if *handle == root_handle
                && *has_value_range
                && *value_now == 80.0
                && *value_min == 20.0
                && *value_max == 80.0
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticLabel { handle, label }
            if *handle == root_handle
                && label == "Progress bar, value 80, range 20 to 80"
    )));
}

#[test]
fn progress_bar_default_semantic_label_does_not_overwrite_explicit_label() {
    ffi::test::reset();
    let progress = progress_bar();
    progress.semantic_label("Install progress");
    Application::mount(progress.clone());
    let calls = ffi::test::take_calls();
    let handle = progress.retained_node_ref().handle().raw();

    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticLabel { handle: call_handle, label }
            if *call_handle == handle && label == "Install progress"
    )));

    progress.value(50.0);
    let calls = ffi::test::take_calls();
    assert!(!calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticLabel { handle: call_handle, label }
            if *call_handle == handle && label.starts_with("Progress bar")
    )));
}

#[test]
fn progress_bar_invalid_length_and_thickness_warn_and_clamp() {
    ffi::test::reset();
    let progress = progress_bar();
    progress.length(0.0).thickness(-2.0);

    Application::mount(progress);
    let calls = ffi::test::take_calls();

    assert!(calls.iter().any(|call| matches!(
        call,
        Call::Log { category, message }
            if category == "Warning/Layout"
                && message == "ProgressBar.length() received 0; clamping to 1.0."
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::Log { category, message }
            if category == "Warning/Layout"
                && message == "ProgressBar.thickness() received -2; clamping to 1.0."
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetWidth { value, unit_enum, .. }
            if *value == 1.0 && *unit_enum == Unit::Pixel as u32
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetHeight { value, unit_enum, .. }
            if *value == 1.0 && *unit_enum == Unit::Pixel as u32
    )));
}

#[test]
fn progress_bar_theme_changes_update_default_colors() {
    ffi::test::reset();
    use_custom_theme(generate_theme(true, 0x112233FF));
    let progress = progress_bar();
    Application::mount(progress.clone());
    let _ = ffi::test::take_calls();

    use_custom_theme(generate_theme(false, 0x445566FF));
    let theme = current_theme();
    let calls = ffi::test::take_calls();

    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetBgColor { color, .. } if *color == theme.colors.accent
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetBgColor { color, .. } if *color == theme.colors.scrollbar_track
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetBoxStyle { border_color, .. } if *border_color == theme.colors.border
    )));

    use_system_theme();
}

#[test]
fn progress_bar_overrides_survive_theme_changes() {
    ffi::test::reset();
    let progress = progress_bar();
    progress
        .track_color(0x123456FF)
        .fill_color(0xABCDEF88)
        .corner_radius(9.0);
    Application::mount(progress.clone());
    let calls = ffi::test::take_calls();
    let root_handle = progress.retained_node_ref().handle().raw();
    let fill_handle = child_handles_for_parent(&calls, root_handle)
        .into_iter()
        .next()
        .expect("fill handle");

    use_custom_theme(generate_theme(true, 0x445566FF));
    let calls = ffi::test::take_calls();

    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetBgColor { handle, color }
            if *handle == root_handle && *color == 0x123456FF
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetBgColor { handle, color }
            if *handle == fill_handle && *color == 0xABCDEF88
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetBoxStyle { handle, radius_tl, radius_tr, radius_br, radius_bl, .. }
            if (*handle == root_handle || *handle == fill_handle)
                && *radius_tl == 9.0
                && *radius_tr == 9.0
                && *radius_br == 9.0
                && *radius_bl == 9.0
    )));

    use_system_theme();
}

#[test]
fn phase_4_8_public_forwarding_and_children_surface_compiles_and_builds() {
    ffi::test::reset();

    let title = text("Mixed child text");
    let action = button("Mixed child button");
    let container = column();
    container
        .children(vec![Child::from_node(&title), Child::from_node(&action)])
        .margin(1.0, 2.0, 3.0, 4.0)
        .padding(5.0, 6.0, 7.0, 8.0);

    let slide = slider();
    slide
        .orientation(Orientation::Vertical)
        .margin(9.0, 10.0, 11.0, 12.0)
        .fill_width_percent(50.0)
        .min_height(20.0, Unit::Pixel)
        .align_self(crate::ffi::AlignSelf::Center)
        .clip_to_bounds(true)
        .semantic_label("Forwarded slider");

    let progress = progress_bar();
    progress
        .margin(1.0, 0.0, 0.0, 0.0)
        .opacity(0.75)
        .position(2.0, 3.0)
        .semantic_label("Forwarded progress");

    let link = nav_link("/v2/fui-rs/demo/workbench/");
    link.fill_width()
        .min_width(12.0, Unit::Pixel)
        .align_self(crate::ffi::AlignSelf::Start)
        .clip_to_bounds(true);

    let selection = selection_area();
    selection
        .children(vec![Child::from_node(&container), Child::from_node(&slide)])
        .fill_size()
        .child(&progress)
        .child(&link);

    Application::mount(selection.clone());
    let calls = ffi::test::take_calls();

    assert!(calls.iter().any(
        |call| matches!(call, Call::SetSemanticLabel { label, .. } if label == "Forwarded slider")
    ));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetFillWidthPercent { percent, .. } if *percent == 50.0
    )));
    assert!(
        calls
            .iter()
            .filter(|call| matches!(call, Call::NodeAddChild { .. }))
            .count()
            >= 5
    );
}

#[test]
fn dropdown_select_index_clamps_and_keyboard_selection_emits() {
    ffi::test::reset();
    let changed = Rc::new(Cell::new(0));
    let last_index = Rc::new(Cell::new(-1));
    let dropdown = dropdown();
    dropdown
        .items(vec![
            DropdownItem::from_value("Calm"),
            DropdownItem::from_value("Focused"),
            DropdownItem::from_value("Energetic"),
        ])
        .on_changed({
            let changed = changed.clone();
            let last_index = last_index.clone();
            move |event| {
                changed.set(changed.get() + 1);
                last_index.set(event.selected_index);
            }
        })
        .select_index(99);

    Application::mount(dropdown.clone());
    let calls = ffi::test::take_calls();
    assert_eq!(dropdown.selected_index(), 2);
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::Log { category, message }
            if category == "Warning/Layout"
                && message == "Dropdown.selectIndex() received 99; clamping to 2."
    )));

    focus(&dropdown.retained_node_ref());
    key_event(KeyEventType::Down, "ArrowDown", 0);
    key_event(KeyEventType::Down, "Home", 0);
    key_event(KeyEventType::Down, "Enter", 0);

    assert_eq!(dropdown.selected_index(), 0);
    assert_eq!(changed.get(), 1);
    assert_eq!(last_index.get(), 0);
}

#[test]
fn dropdown_popup_builds_list_semantics_and_expanded_state() {
    ffi::test::reset();
    let dropdown = dropdown();
    dropdown
        .items(vec![
            DropdownItem::new("alpha", "Alpha"),
            DropdownItem::new("beta", "Beta"),
        ])
        .popup_width(280.0)
        .max_visible_items(4);

    Application::mount(dropdown.clone());
    let _ = ffi::test::take_calls();

    focus(&dropdown.retained_node_ref());
    key_event(KeyEventType::Down, "ArrowDown", 0);
    let calls = ffi::test::take_calls();

    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticRole { role_enum, .. } if *role_enum == SemanticRole::ComboBox as u32
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticRole { role_enum, .. } if *role_enum == SemanticRole::List as u32
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticExpanded { has_expanded, is_expanded, .. }
            if *has_expanded && *is_expanded
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticSelected { has_selected, selected, .. }
            if *has_selected && *selected
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetWidth { value, .. } if *value == 280.0
    )));
}

#[test]
fn dropdown_disabled_suppresses_open_and_activation() {
    ffi::test::reset();
    let changed = Rc::new(Cell::new(0));
    let dropdown = dropdown();
    dropdown
        .items(vec![
            DropdownItem::from_value("Alpha"),
            DropdownItem::from_value("Beta"),
        ])
        .on_changed({
            let changed = changed.clone();
            move |_event| changed.set(changed.get() + 1)
        })
        .enabled(false);

    Application::mount(dropdown.clone());
    let _ = ffi::test::take_calls();
    focus(&dropdown.retained_node_ref());
    key_event(KeyEventType::Down, "ArrowDown", 0);
    let calls = ffi::test::take_calls();

    assert_eq!(dropdown.selected_index(), 0);
    assert_eq!(changed.get(), 0);
    assert!(!calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticExpanded { has_expanded, is_expanded, .. }
            if *has_expanded && *is_expanded
    )));
}

#[test]
fn dropdown_pointer_selection_and_escape_close_match_fui_as() {
    ffi::test::reset();
    let changed = Rc::new(Cell::new(0));
    let last_index = Rc::new(Cell::new(-1));
    let dropdown = dropdown();
    dropdown
        .items(vec![
            DropdownItem::from_value("Alpha"),
            DropdownItem::from_value("Beta"),
            DropdownItem::from_value("Gamma"),
        ])
        .on_changed({
            let changed = changed.clone();
            let last_index = last_index.clone();
            move |event| {
                changed.set(changed.get() + 1);
                last_index.set(event.selected_index);
            }
        });

    Application::mount(dropdown.clone());
    let _ = ffi::test::take_calls();
    focus(&dropdown.retained_node_ref());
    key_event(KeyEventType::Down, "ArrowDown", 0);
    let calls = ffi::test::take_calls();
    let list_item_handle = calls
        .iter()
        .filter_map(|call| match call {
            Call::SetSemanticRole { handle, role_enum }
                if *role_enum == SemanticRole::ListItem as u32 =>
            {
                Some(*handle)
            }
            _ => None,
        })
        .nth(1)
        .expect("dropdown open should create list item semantics");

    event::__fui_on_pointer_event_with_metadata(
        PointerEventType::Enter as u32,
        list_item_handle,
        10.0,
        10.0,
        0,
        1,
        PointerType::Mouse as u32,
        0,
        0,
        0.0,
        0.0,
        0.0,
        0,
    );
    event::__fui_on_pointer_event_with_metadata(
        PointerEventType::Up as u32,
        list_item_handle,
        10.0,
        10.0,
        0,
        1,
        PointerType::Mouse as u32,
        0,
        0,
        0.0,
        0.0,
        0.0,
        0,
    );

    assert_eq!(changed.get(), 1);
    assert_eq!(last_index.get(), 1);

    key_event(KeyEventType::Down, "ArrowDown", 0);
    let _ = ffi::test::take_calls();
    key_event(KeyEventType::Down, "Escape", 0);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticExpanded { has_expanded, is_expanded, .. }
            if *has_expanded && !*is_expanded
    )));
}

#[test]
fn dropdown_scroll_hook_survives_event_router_reset() {
    ffi::test::reset();
    let first = dropdown();
    first.items(vec![DropdownItem::from_value("First")]);
    Application::mount(first);
    event::reset();

    let dropdown = dropdown();
    dropdown.items(vec![
        DropdownItem::from_value("Alpha"),
        DropdownItem::from_value("Beta"),
    ]);
    Application::mount(dropdown.clone());
    let _ = ffi::test::take_calls();
    focus(&dropdown.retained_node_ref());
    key_event(KeyEventType::Down, "ArrowDown", 0);
    let _ = ffi::test::take_calls();

    ffi::test::set_viewport(0.0, 0.0);
    event::dispatch_scroll(
        dropdown.retained_node_ref().handle(),
        0.0,
        12.0,
        100.0,
        200.0,
        100.0,
        50.0,
    );
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticExpanded { has_expanded, is_expanded, .. }
            if *has_expanded && !*is_expanded
    )));
}

#[test]
fn combobox_select_index_clamps_and_keyboard_selection_emits() {
    ffi::test::reset();
    let changed = Rc::new(Cell::new(0));
    let last_index = Rc::new(Cell::new(-1));
    let combo = combo_box();
    combo
        .items(vec!["Calm", "Focused", "Energetic"])
        .filter_mode(ComboBoxFilterMode::None)
        .on_changed({
            let changed = changed.clone();
            let last_index = last_index.clone();
            move |event| {
                changed.set(changed.get() + 1);
                last_index.set(event.selected_index);
            }
        })
        .select_index(99);

    Application::mount(combo.clone());
    let calls = ffi::test::take_calls();
    assert_eq!(combo.selected_index(), 2);
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::Log { category, message }
            if category == "Warning/Layout"
                && message == "ComboBox.selectIndex() received 99; clamping to 2."
    )));

    focus(&combo.retained_node_ref());
    key_event(KeyEventType::Down, "ArrowDown", 0);
    key_event(KeyEventType::Down, "Home", 0);
    key_event(KeyEventType::Down, "Enter", 0);

    assert_eq!(combo.selected_index(), 0);
    assert_eq!(changed.get(), 1);
    assert_eq!(last_index.get(), 0);
}

#[test]
fn combobox_popup_builds_list_semantics_and_expanded_state() {
    ffi::test::reset();
    let combo = combo_box();
    combo
        .items(vec!["Alpha", "Beta"])
        .select_index(0)
        .popup_width(280.0)
        .max_visible_items(4);

    Application::mount(combo.clone());
    let _ = ffi::test::take_calls();

    focus(&combo.retained_node_ref());
    key_event(KeyEventType::Down, "ArrowDown", 0);
    let calls = ffi::test::take_calls();

    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticRole { role_enum, .. } if *role_enum == SemanticRole::ComboBox as u32
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticRole { role_enum, .. } if *role_enum == SemanticRole::List as u32
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticExpanded { has_expanded, is_expanded, .. }
            if *has_expanded && *is_expanded
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticSelected { has_selected, selected, .. }
            if *has_selected && *selected
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetWidth { value, .. } if *value == 280.0
    )));
}

#[test]
fn combobox_filtering_and_exact_commit_match_fui_as() {
    ffi::test::reset();
    let changed = Rc::new(Cell::new(0));
    let combo = combo_box();
    combo
        .items(vec!["Alpha", "Beta", "Gamma"])
        .filter_mode(ComboBoxFilterMode::StartsWith)
        .commit_mode(ComboBoxCommitMode::SelectExactMatch)
        .on_changed({
            let changed = changed.clone();
            move |_event| changed.set(changed.get() + 1)
        });

    Application::mount(combo.clone());
    let _ = ffi::test::take_calls();
    combo.text("Ga");
    focus(&combo.retained_node_ref());
    key_event(KeyEventType::Down, "ArrowDown", 0);
    key_event(KeyEventType::Down, "Enter", 0);

    assert_eq!(combo.selected_index(), 2);
    assert_eq!(combo.value(), "Gamma");
    assert_eq!(changed.get(), 1);

    combo.text("th");
    key_event(KeyEventType::Down, "ArrowDown", 0);
    key_event(KeyEventType::Down, "Enter", 0);
    assert_eq!(combo.selected_index(), -1);
    assert_eq!(combo.value(), "th");
    assert_eq!(changed.get(), 1);
}

#[test]
fn combobox_editor_key_handling_consumes_text_navigation_like_fui_as() {
    ffi::test::reset();
    let combo = combo_box();
    combo
        .items(vec!["Alpha", "Apricot", "Banana"])
        .filter_mode(ComboBoxFilterMode::StartsWith);

    Application::mount(combo.clone());
    let calls = ffi::test::take_calls();
    let editor_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::SetSemanticRole {
                handle, role_enum, ..
            } if *role_enum == SemanticRole::Textbox as u32 => Some(*handle),
            _ => None,
        })
        .expect("expected combobox editor textbox to be mounted");
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetEditorCommandKeys { handle, enabled } if *handle == editor_handle && *enabled
    )));

    event::__fui_on_focus_changed(editor_handle, true);
    combo.text("Ap");

    assert!(key_event(KeyEventType::Down, "ArrowLeft", 0));
    assert!(key_event(
        KeyEventType::Down,
        "ArrowRight",
        ffi::KeyModifier::Shift as u32
    ));
    assert!(key_event(KeyEventType::Down, "Home", 0));
    assert!(key_event(KeyEventType::Down, "PageDown", 0));
    assert!(key_event(KeyEventType::Down, "ArrowDown", 0));
    assert!(combo.is_open());
    assert!(key_event(KeyEventType::Down, "Enter", 0));
    assert_eq!(combo.selected_index(), 1);
    assert_eq!(combo.value(), "Apricot");
    assert!(!key_event(
        KeyEventType::Down,
        "ArrowLeft",
        ffi::KeyModifier::Meta as u32
    ));
}

#[test]
fn combobox_revert_to_selection_commit_restores_committed_label_on_close() {
    ffi::test::reset();
    let combo = combo_box();
    combo
        .items(vec!["Alpha", "Beta", "Gamma"])
        .select_index(1)
        .commit_mode(ComboBoxCommitMode::RevertToSelection);

    Application::mount(combo.clone());
    let _ = ffi::test::take_calls();
    focus(&combo.retained_node_ref());
    key_event(KeyEventType::Down, "ArrowDown", 0);
    combo.text("Custom");
    key_event(KeyEventType::Down, "Escape", 0);

    assert_eq!(combo.selected_index(), 1);
    assert_eq!(combo.value(), "Beta");
}

#[test]
fn combobox_closes_popup_when_focus_leaves_for_another_input() {
    ffi::test::reset();
    let combo = combo_box();
    let input = text_input();
    let root = column();
    combo.items(vec!["Melbourne", "Sydney"]);
    root.child(&combo).child(&input);

    Application::mount(root.clone());
    let _ = ffi::test::take_calls();

    combo.text("Mel");
    focus(&combo.retained_node_ref());
    key_event(KeyEventType::Down, "ArrowDown", 0);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticExpanded {
            has_expanded,
            is_expanded,
            ..
        } if *has_expanded && *is_expanded
    )));

    blur(&combo.retained_node_ref());
    focus(&input.retained_node_ref());
    Application::flush_renders();
    let calls = ffi::test::take_calls();

    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticExpanded {
            has_expanded,
            is_expanded,
            ..
        } if *has_expanded && !*is_expanded
    )));
}

#[test]
fn combobox_pointer_option_selection_survives_blur_during_popup_click() {
    ffi::test::reset();
    let changed = Rc::new(Cell::new(0));
    let last_value = Rc::new(std::cell::RefCell::new(String::new()));
    let combo = combo_box();
    combo.items(vec!["Melbourne", "Sydney"]).on_changed({
        let changed = changed.clone();
        let last_value = last_value.clone();
        move |event| {
            changed.set(changed.get() + 1);
            *last_value.borrow_mut() = event.item.value;
        }
    });

    Application::mount(combo.clone());
    let _ = ffi::test::take_calls();

    focus(&combo.retained_node_ref());
    key_event(KeyEventType::Down, "ArrowDown", 0);
    let calls = ffi::test::take_calls();
    let option_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::SetSemanticLabel { handle, label } if label == "Sydney" => Some(*handle),
            _ => None,
        })
        .expect("expected Sydney option semantic label");

    event::__fui_on_pointer_event_with_metadata(
        PointerEventType::Down as u32,
        option_handle,
        30.0,
        72.0,
        0,
        1,
        PointerType::Touch as u32,
        0,
        1,
        0.0,
        0.0,
        0.0,
        1,
    );
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetPointerCapture { handle } if *handle == option_handle
    )));
    blur(&combo.retained_node_ref());
    event::__fui_on_pointer_event_with_metadata(
        PointerEventType::Up as u32,
        option_handle,
        30.0,
        72.0,
        0,
        1,
        PointerType::Touch as u32,
        0,
        0,
        0.0,
        0.0,
        0.0,
        1,
    );
    let calls = ffi::test::take_calls();
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::ReleasePointerCapture)));

    assert_eq!(combo.selected_index(), 1);
    assert_eq!(combo.value(), "Sydney");
    assert_eq!(changed.get(), 1);
    assert_eq!(&*last_value.borrow(), "Sydney");
}

#[test]
fn combobox_closes_after_popup_pointer_cancel_when_editor_blur_was_deferred() {
    ffi::test::reset();
    let combo = combo_box();
    combo.items(vec!["Melbourne", "Sydney"]);

    Application::mount(combo.clone());
    let _ = ffi::test::take_calls();

    focus(&combo.retained_node_ref());
    key_event(KeyEventType::Down, "ArrowDown", 0);
    let calls = ffi::test::take_calls();
    let option_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::SetSemanticLabel { handle, label } if label == "Sydney" => Some(*handle),
            _ => None,
        })
        .expect("expected Sydney option semantic label");

    event::__fui_on_pointer_event_with_metadata(
        PointerEventType::Down as u32,
        option_handle,
        30.0,
        72.0,
        0,
        1,
        PointerType::Touch as u32,
        0,
        1,
        0.0,
        0.0,
        0.0,
        1,
    );
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetPointerCapture { handle } if *handle == option_handle
    )));
    blur(&combo.retained_node_ref());
    event::__fui_on_pointer_event_with_metadata(
        PointerEventType::Cancel as u32,
        option_handle,
        30.0,
        72.0,
        0,
        1,
        PointerType::Touch as u32,
        0,
        0,
        0.0,
        0.0,
        0.0,
        1,
    );
    Application::flush_renders();
    let calls = ffi::test::take_calls();

    assert!(!combo.is_open());
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticExpanded {
            has_expanded,
            is_expanded,
            ..
        } if *has_expanded && !*is_expanded
    )));
}

#[test]
fn text_input_text_replaced_event_updates_value_and_emits_changed() {
    ffi::test::reset();
    let input = text_input();
    let last_text = Rc::new(std::cell::RefCell::new(String::new()));
    input
        .node_id("test-text-input")
        .placeholder("Username or email")
        .on_changed({
            let last_text = last_text.clone();
            move |event| {
                *last_text.borrow_mut() = event.text;
            }
        });

    Application::mount(input.clone());
    let calls = ffi::test::take_calls();
    let editor_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::SetNodeId { handle, node_id } if node_id == "test-text-input" => Some(*handle),
            _ => None,
        })
        .expect("expected text input editor node id to be applied");

    event::__fui_on_text_replaced(editor_handle, 0, 0, b"hello".as_ptr(), 5);

    assert_eq!(input.value(), "hello");
    assert_eq!(&*last_text.borrow(), "hello");
    Application::unmount();
}

#[test]
fn text_input_selection_range_uses_char_indices_and_sends_utf8_bytes() {
    ffi::test::reset();
    let input = text_input();
    input
        .node_id("emoji-text-input")
        .text("a😄b")
        .selection_range(1, 2);
    Application::mount(input.clone());
    let calls = ffi::test::take_calls();
    let editor_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::SetNodeId { handle, node_id } if node_id == "emoji-text-input" => Some(*handle),
            _ => None,
        })
        .expect("expected text input editor node id to be applied");
    assert_eq!(input.selection_start(), 1);
    assert_eq!(input.selection_end(), 2);
    assert_eq!(input.selection_start_byte_offset(), 1);
    assert_eq!(input.selection_end_byte_offset(), 5);
    input.selection_range(0, 3);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetTextSelectionRange { handle, start, end }
            if *handle == editor_handle && *start == 0 && *end == 6
    )));
    Application::unmount();
}

#[test]
fn text_input_focusable_forwards_to_editor_text_node() {
    ffi::test::reset();
    let input = text_input();
    input
        .node_id("focusable-text-input")
        .text("Focusable")
        .focusable(false, 0);
    Application::mount(input.clone());
    let calls = ffi::test::take_calls();
    let editor_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::SetNodeId { handle, node_id } if node_id == "focusable-text-input" => {
                Some(*handle)
            }
            _ => None,
        })
        .expect("expected text input editor node id to be applied");

    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetFocusable { handle, focusable, tab_index }
            if *handle == editor_handle && !*focusable && *tab_index == 0
    )));

    input.focusable(true, 7);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetFocusable { handle, focusable, tab_index }
            if *handle == editor_handle && *focusable && *tab_index == 7
    )));
    Application::unmount();
}

#[test]
fn text_area_focusable_forwards_to_editor_text_node() {
    ffi::test::reset();
    let input = text_area();
    input
        .node_id("focusable-text-area")
        .text("Focusable\narea")
        .focusable(false, 0);
    Application::mount(input.clone());
    let calls = ffi::test::take_calls();
    let editor_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::SetNodeId { handle, node_id } if node_id == "focusable-text-area" => {
                Some(*handle)
            }
            _ => None,
        })
        .expect("expected text area editor node id to be applied");

    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetFocusable { handle, focusable, tab_index }
            if *handle == editor_handle && !*focusable && *tab_index == 0
    )));
    Application::unmount();
}

#[test]
fn text_input_keyboard_focus_preserves_logical_end_selection_state() {
    ffi::test::reset();
    focus_visibility::reset_keyboard_focus_visibility();
    let input = text_input();
    input.node_id("tab-focus-text-input").text("a😄b");
    Application::mount(input.clone());
    let calls = ffi::test::take_calls();
    let editor_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::SetNodeId { handle, node_id } if node_id == "tab-focus-text-input" => {
                Some(*handle)
            }
            _ => None,
        })
        .expect("expected text input editor node id to be applied");

    event::__fui_on_focus_changed(editor_handle, true);
    let _ = ffi::test::take_calls();
    assert_eq!(input.selection_start(), 3);
    assert_eq!(input.selection_end(), 3);
    assert_eq!(input.selection_start_byte_offset(), 6);
    assert_eq!(input.selection_end_byte_offset(), 6);
    Application::unmount();
}

#[test]
fn text_input_reverse_tab_focus_restores_caret_to_logical_end_after_pointer_focus() {
    ffi::test::reset();
    focus_visibility::reset_keyboard_focus_visibility();
    focus_visibility::show_keyboard_focus_for_pointer_event(PointerEventType::Down);
    let input = text_input();
    input
        .node_id("reverse-tab-focus-text-input")
        .text("readonly")
        .read_only(true);
    Application::mount(input.clone());
    let calls = ffi::test::take_calls();
    let editor_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::SetNodeId { handle, node_id } if node_id == "reverse-tab-focus-text-input" => {
                Some(*handle)
            }
            _ => None,
        })
        .expect("expected text input editor node id to be applied");

    key_event(KeyEventType::Down, "Shift", 0);
    event::__fui_on_focus_changed(editor_handle, true);
    let calls = ffi::test::take_calls();
    assert!(!calls.iter().any(|call| matches!(
        call,
        Call::SetTextSelectionRange { handle, start, end }
            if *handle == editor_handle && *start == 8 && *end == 8
    )));
    assert_eq!(input.selection_start(), 8);
    assert_eq!(input.selection_end(), 8);
    Application::unmount();
}

#[test]
fn text_input_shell_pointer_down_preserves_existing_selection_in_mock_path() {
    ffi::test::reset();
    let input = text_input();
    input
        .node_id("shell-pointer-text-input")
        .text("Read-only sample")
        .read_only(true)
        .selection_range(0, 4);
    Application::mount(input.clone());
    let _ = ffi::test::take_calls();

    event::__fui_on_pointer_event_with_metadata(
        PointerEventType::Down as u32,
        input.retained_node_ref().handle().raw(),
        220.0,
        12.0,
        0,
        1,
        PointerType::Mouse as u32,
        0,
        1,
        0.0,
        0.0,
        0.0,
        1,
    );
    let _ = ffi::test::take_calls();
    assert_eq!(input.selection_start(), 0);
    assert_eq!(input.selection_end(), 4);
    Application::unmount();
}

#[test]
fn text_input_editor_pointer_down_does_not_bubble_into_shell_document_end_path() {
    ffi::test::reset();
    let input = text_input();
    input
        .node_id("editor-pointer-text-input")
        .text("Read-only sample")
        .read_only(true)
        .selection_range(0, 4);
    Application::mount(input.clone());
    let calls = ffi::test::take_calls();
    let editor_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::SetNodeId { handle, node_id } if node_id == "editor-pointer-text-input" => {
                Some(*handle)
            }
            _ => None,
        })
        .expect("expected text input editor node id to be applied");

    event::__fui_on_pointer_event_with_metadata(
        PointerEventType::Down as u32,
        editor_handle,
        12.0,
        12.0,
        0,
        1,
        PointerType::Mouse as u32,
        0,
        1,
        0.0,
        0.0,
        0.0,
        1,
    );
    let calls = ffi::test::take_calls();
    assert!(!calls.iter().any(|call| matches!(
        call,
        Call::SetTextSelectionRange { handle, start, end }
            if *handle == editor_handle && *start == 16 && *end == 16
    )));
    assert_eq!(input.selection_start(), 0);
    assert_eq!(input.selection_end(), 4);
    Application::unmount();
}

#[test]
fn text_input_text_replacement_clamps_selection_bytes_from_char_positions() {
    ffi::test::reset();
    let input = text_input();
    input
        .node_id("emoji-clamp-input")
        .text("a😄b")
        .selection_range(0, 3);
    Application::mount(input.clone());
    let calls = ffi::test::take_calls();
    let editor_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::SetNodeId { handle, node_id } if node_id == "emoji-clamp-input" => Some(*handle),
            _ => None,
        })
        .expect("expected text input editor node id to be applied");

    event::__fui_on_text_replaced(editor_handle, 1, 5, std::ptr::null(), 0);

    assert_eq!(input.value(), "ab");
    assert_eq!(input.selection_start(), 0);
    assert_eq!(input.selection_end(), 2);
    assert_eq!(input.selection_start_byte_offset(), 0);
    assert_eq!(input.selection_end_byte_offset(), 2);
    Application::unmount();
}

#[test]
fn text_input_accepts_tab_inserts_tab_and_consumes_plain_tab_only() {
    ffi::test::reset();
    let input = text_input();
    input
        .node_id("accepts-tab-input")
        .text("ab")
        .selection_range(1, 1);
    Application::mount(input.clone());
    let calls = ffi::test::take_calls();
    let editor_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::SetNodeId { handle, node_id } if node_id == "accepts-tab-input" => Some(*handle),
            _ => None,
        })
        .expect("expected text input editor node id to be applied");

    event::__fui_on_focus_changed(editor_handle, true);
    assert!(!key_event(KeyEventType::Down, "Tab", 0));
    assert_eq!(input.value(), "ab");
    assert!(key_event(KeyEventType::Down, "ArrowDown", 0));
    assert!(key_event(
        KeyEventType::Down,
        "ArrowLeft",
        ffi::KeyModifier::Shift as u32
    ));
    assert!(key_event(KeyEventType::Down, "Home", 0));
    assert!(key_event(KeyEventType::Down, "PageDown", 0));
    assert_eq!(input.value(), "ab");

    input.accepts_tab(true);
    focus_visibility::show_keyboard_focus_for_pointer_event(PointerEventType::Down);
    assert!(key_event(KeyEventType::Down, "Tab", 0));
    assert!(!focus_visibility::keyboard_focus_visible());
    assert_eq!(input.value(), "a\tb");
    assert_eq!(input.selection_start(), 2);
    assert_eq!(input.selection_end(), 2);
    assert_eq!(input.selection_start_byte_offset(), 2);
    assert_eq!(input.selection_end_byte_offset(), 2);

    assert!(!key_event(
        KeyEventType::Down,
        "Tab",
        ffi::KeyModifier::Shift as u32
    ));
    assert_eq!(input.value(), "a\tb");
    Application::unmount();
}

#[test]
fn text_area_accepts_tab_replaces_selection_and_respects_read_only() {
    ffi::test::reset();
    let input = text_area();
    input
        .node_id("accepts-tab-text-area")
        .text("a😄b\nc")
        .selection_range(1, 3)
        .accepts_tab(true);
    Application::mount(input.clone());
    let calls = ffi::test::take_calls();
    let editor_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::SetNodeId { handle, node_id } if node_id == "accepts-tab-text-area" => {
                Some(*handle)
            }
            _ => None,
        })
        .expect("expected text area editor node id to be applied");

    event::__fui_on_focus_changed(editor_handle, true);
    assert!(key_event(KeyEventType::Down, "Tab", 0));
    assert_eq!(input.value(), "a\t\nc");
    assert_eq!(input.selection_start(), 2);
    assert_eq!(input.selection_end(), 2);
    assert_eq!(input.selection_start_byte_offset(), 2);
    assert_eq!(input.selection_end_byte_offset(), 2);

    input.read_only(true);
    assert!(!key_event(KeyEventType::Down, "Tab", 0));
    assert_eq!(input.value(), "a\t\nc");
    Application::unmount();
}

#[test]
fn text_area_uses_multiline_profile_wrapping_and_scroll_chrome() {
    ffi::test::reset();
    let input = text_area();
    input
        .node_id("text-area-profile")
        .text("Wide multiline content")
        .wrapping(false)
        .vertical_scrollbar_visibility(ScrollBarVisibility::Always)
        .horizontal_scrollbar_visibility(ScrollBarVisibility::Always);
    Application::mount(input.clone());
    let calls = ffi::test::take_calls();
    let editor_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::SetNodeId { handle, node_id } if node_id == "text-area-profile" => Some(*handle),
            _ => None,
        })
        .expect("expected text area editor node id to be applied");

    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetTextLimits { handle, max_lines, .. }
            if *handle == editor_handle && *max_lines == 0
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetTextWrapping { handle, wrap }
            if *handle == editor_handle && !*wrap
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetTextVerticalAlign { handle, align_enum }
            if *handle == editor_handle && *align_enum == TextVerticalAlign::Top as u32
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetWidth { handle, unit_enum, .. }
            if *handle == editor_handle && *unit_enum == Unit::Auto as u32
    )));

    input.wrapping(true);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetTextWrapping { handle, wrap }
            if *handle == editor_handle && *wrap
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetWidth { handle, unit_enum, .. }
            if *handle == editor_handle && *unit_enum == Unit::Percent as u32
    )));
    Application::unmount();
}

#[test]
fn text_area_readonly_disabled_placeholder_and_line_height_follow_text_input_core() {
    ffi::test::reset();
    let input = text_area();
    input
        .node_id("text-area-readonly")
        .placeholder("Notes")
        .read_only(true)
        .line_height(26.0);
    Application::mount(input.clone());
    let calls = ffi::test::take_calls();
    let editor_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::SetNodeId { handle, node_id } if node_id == "text-area-readonly" => Some(*handle),
            _ => None,
        })
        .expect("expected text area editor node id to be applied");

    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetEditable { handle, editable }
            if *handle == editor_handle && !*editable
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSelectable { handle, selectable, .. }
            if *handle == editor_handle && *selectable
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetLineHeight { handle, line_height }
            if *handle == editor_handle && (*line_height - 26.0).abs() < f32::EPSILON
    )));

    event::__fui_on_text_replaced(editor_handle, 0, 0, b"Hello\nworld".as_ptr(), 11);
    assert_eq!(input.value(), "Hello\nworld");

    event::__fui_on_text_changed(editor_handle, b"alpha\nbeta".as_ptr(), 10);
    assert_eq!(input.value(), "alpha\nbeta");
    event::__fui_on_selection_changed(editor_handle, 6, 10);
    assert_eq!(input.selection_start(), 6);
    assert_eq!(input.selection_end(), 10);

    event::__fui_on_text_replaced(editor_handle, 0, 11, std::ptr::null(), 0);
    assert_eq!(input.value(), "");
    Application::unmount();
}

#[test]
fn text_area_reports_internal_scroll_offsets_and_uses_text_area_template_slot() {
    ffi::test::reset();
    clear_control_templates();
    let text_input_created = Rc::new(Cell::new(0));
    let text_area_created = Rc::new(Cell::new(0));
    let text_input_state = Rc::new(RefCell::new(None));
    let text_area_state = Rc::new(RefCell::new(None));
    use_control_templates(Some(ControlTemplateSet {
        text_input: Some(Rc::new(TestTextInputTemplate {
            created: text_input_created.clone(),
            last_state: text_input_state,
        })),
        text_area: Some(Rc::new(TestTextInputTemplate {
            created: text_area_created.clone(),
            last_state: text_area_state.clone(),
        })),
        ..Default::default()
    }));

    let input = text_area();
    input
        .node_id("text-area-template")
        .height(120.0, Unit::Pixel);
    Application::mount(input.clone());
    let calls = ffi::test::take_calls();
    let editor_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::SetNodeId { handle, node_id } if node_id == "text-area-template" => Some(*handle),
            _ => None,
        })
        .expect("expected text area editor node id to be applied");

    assert_eq!(text_input_created.get(), 0);
    assert_eq!(text_area_created.get(), 1);
    assert!(text_area_state
        .borrow()
        .as_ref()
        .is_some_and(|state| state.multiline));

    input.scroll_to(12.0, 34.0);
    let calls = ffi::test::take_calls();
    assert_eq!(input.scroll_offset_x(), 12.0);
    assert_eq!(input.scroll_offset_y(), 34.0);
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetScrollOffset { offset_x, offset_y, .. }
            if (*offset_x - 12.0).abs() < f32::EPSILON
                && (*offset_y - 34.0).abs() < f32::EPSILON
    )));
    assert!(editor_handle > 0);
    Application::unmount();
    clear_control_templates();
}

#[test]
fn text_area_shell_pointer_down_preserves_existing_selection_path() {
    ffi::test::reset();
    let input = text_area();
    input
        .node_id("text-area-shell-pointer")
        .placeholder("Notes")
        .text("First\nSecond")
        .selection_range(0, 5);
    Application::mount(input.clone());
    let calls = ffi::test::take_calls();
    let editor_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::SetNodeId { handle, node_id } if node_id == "text-area-shell-pointer" => {
                Some(*handle)
            }
            _ => None,
        })
        .expect("expected text area editor node id to be applied");

    event::__fui_on_pointer_event_with_metadata(
        PointerEventType::Down as u32,
        input.retained_node_ref().handle().raw(),
        8.0,
        8.0,
        0,
        1,
        PointerType::Mouse as u32,
        0,
        1,
        0.0,
        0.0,
        0.0,
        1,
    );
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetTextSelectionRange { handle, start, end }
            if *handle == editor_handle && *start == 0 && *end == 5
    )));
    assert_eq!(input.selection_start(), 0);
    assert_eq!(input.selection_end(), 5);
    Application::unmount();
}

#[test]
fn text_area_inherits_parent_disabled_state_to_editor() {
    ffi::test::reset();
    let parent = column();
    let input = text_area();
    input.node_id("text-area-disabled").text("Disabled notes");
    parent.child(&input);
    Application::mount(parent.clone());
    let calls = ffi::test::take_calls();
    let editor_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::SetNodeId { handle, node_id } if node_id == "text-area-disabled" => Some(*handle),
            _ => None,
        })
        .expect("expected text area editor node id to be applied");

    parent.enabled(false);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetEditable { handle, editable }
            if *handle == editor_handle && !*editable
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSelectable { handle, selectable, .. }
            if *handle == editor_handle && !*selectable
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticDisabled { handle, has_disabled, disabled }
            if *handle == editor_handle && *has_disabled && *disabled
    )));

    parent.enabled(true);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetEditable { handle, editable }
            if *handle == editor_handle && *editable
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSelectable { handle, selectable, .. }
            if *handle == editor_handle && *selectable
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticDisabled { handle, has_disabled, disabled }
            if *handle == editor_handle && *has_disabled && !*disabled
    )));
    Application::unmount();
}

#[test]
fn text_input_persisted_state_restores_and_emits_without_password_capture() {
    ffi::test::reset();
    let source = text_input();
    source.node_id("text-input-persisted").text("hello");
    Application::mount(source.clone());
    Application::capture_persisted_ui_state();
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetPersistedState { node_id, kind, version, payload }
            if node_id == "text-input-persisted"
                && kind == "text-input-value"
                && *version == 1
                && payload == "hello"
    )));

    let changes = Rc::new(Cell::new(0));
    let changes_clone = changes.clone();
    let restored = text_input();
    restored.node_id("text-input-persisted");
    restored.on_changed(move |_event| changes_clone.set(changes_clone.get() + 1));
    Application::mount(restored.clone());
    let _ = ffi::test::take_calls();
    Application::restore_persisted_ui_state();

    assert_eq!(restored.value(), "hello");
    assert_eq!(changes.get(), 1);

    let password = text_input();
    password
        .node_id("text-input-password-persisted")
        .password(true)
        .text("secret");
    Application::mount(password.clone());
    Application::capture_persisted_ui_state();
    let calls = ffi::test::take_calls();
    assert!(!calls.iter().any(|call| matches!(
        call,
        Call::SetPersistedState { node_id, kind, .. }
            if node_id == "text-input-password-persisted" && kind == "text-input-value"
    )));
    Application::unmount();
}

#[test]
fn text_input_focus_adorner_tracks_keyboard_focus_visibility() {
    ffi::test::reset();
    focus_visibility::reset_keyboard_focus_visibility();
    let input = text_input();
    input.node_id("focus-text-input");
    Application::mount(input.clone());
    let calls = ffi::test::take_calls();
    let editor_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::SetNodeId { handle, node_id } if node_id == "focus-text-input" => Some(*handle),
            _ => None,
        })
        .expect("expected text input editor node id to be applied");

    focus_visibility::show_keyboard_focus_for_pointer_event(PointerEventType::Down);
    let _ = ffi::test::take_calls();
    event::__fui_on_focus_changed(editor_handle, true);
    let calls = ffi::test::take_calls();
    assert!(!calls
        .iter()
        .any(|call| matches!(call, Call::NodeAddChild { .. })));

    key_event(KeyEventType::Down, "a", 0);
    Application::flush_renders();
    let calls = ffi::test::take_calls();
    assert!(focus_visibility::keyboard_focus_visible());
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::NodeAddChild { .. })));

    key_event(KeyEventType::Down, "Tab", 0);
    Application::flush_renders();
    assert!(focus_visibility::keyboard_focus_visible());
    Application::unmount();
}
