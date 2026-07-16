use super::control_template_set::get_control_templates;
use super::internal::dropdown_chevron_presenter::{
    create_default_dropdown_chevron_presenter, DropdownChevronPresenter, DropdownChevronTemplate,
    DropdownChevronVisualState,
};
use super::internal::dropdown_option_row_presenter::DropdownOptionRowTemplate;
use super::internal::selectable_popup_list::{
    SelectablePopupList, SelectablePopupListOwner, SELECTABLE_POPUP_LIST_PANEL_PADDING,
};
use super::internal::text_input_presenter::{
    TextInputPresenter, TextInputTemplate, TextInputVisualState,
};
use super::{DropdownColors, DropdownSizing, TextInput, TextInputColors};
use crate::bindings::ui;
use crate::event::{self, TextChangedEventArgs};
use crate::ffi::{
    AlignItems, CursorStyle, FlexDirection, KeyEventType, KeyModifier, NodeType, SemanticRole, Unit,
};
use crate::focus_adorner;
use crate::focus_visibility;
use crate::logger;
use crate::node::{
    row, FlexBox, FlexBoxSurface, HasFlexBoxRoot, Node, NodeRef, TextCore, WeakFlexBox,
};
use crate::signal::SubscriptionGuard;
use crate::theme::{current_theme, subscribe, Theme};
use crate::{app, frame_scheduler};
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

const DEFAULT_PANEL_BACKGROUND_BLUR_SIGMA: f32 = 10.0;

fn strings_equal_ignore_case(left: &str, right: &str) -> bool {
    left.to_lowercase() == right.to_lowercase()
}

fn string_contains_ignore_case(value: &str, query: &str) -> bool {
    value.to_lowercase().contains(&query.to_lowercase())
}

fn string_starts_with_ignore_case(value: &str, query: &str) -> bool {
    value.to_lowercase().starts_with(&query.to_lowercase())
}

fn create_chevron_presenter(
    template: Option<Rc<dyn DropdownChevronTemplate>>,
    sizing: Option<DropdownSizing>,
) -> Rc<dyn DropdownChevronPresenter> {
    if let Some(template) = template {
        return template.create(sizing);
    }
    if let Some(template) = get_control_templates().and_then(|set| set.dropdown_chevron) {
        return template.create(sizing);
    }
    create_default_dropdown_chevron_presenter(sizing)
}

fn resolve_text_input_colors(
    colors: Option<DropdownColors>,
    theme: &Theme,
) -> Option<TextInputColors> {
    let colors = colors?;
    let input_colors = TextInputColors::new();
    if colors.has_background() {
        input_colors.background(colors.background_color());
    }
    if colors.has_text_primary() {
        input_colors.text_primary(colors.text_primary_color());
    }
    if colors.has_placeholder() {
        input_colors.placeholder(colors.placeholder_color());
    }
    if colors.has_border() {
        input_colors.border(colors.border_color());
    }
    if colors.has_accent() {
        input_colors
            .accent(colors.accent_color())
            .caret(colors.accent_color());
    } else {
        input_colors.caret(theme.colors.accent);
    }
    Some(input_colors)
}

#[derive(Clone, Default)]
struct ComboBoxEditorPresenter {
    editor_host: RefCell<Option<TextCore>>,
    placeholder_host: RefCell<Option<FlexBox>>,
}

impl TextInputPresenter for ComboBoxEditorPresenter {
    fn bind(&self, editor_host: TextCore, placeholder_host: FlexBox) {
        *self.editor_host.borrow_mut() = Some(editor_host);
        *self.placeholder_host.borrow_mut() = Some(placeholder_host);
    }

    fn present(
        &self,
        _theme: Theme,
        state: &TextInputVisualState,
        _colors: Option<TextInputColors>,
    ) -> crate::PresenterHostStyle {
        let Some(editor_host) = self.editor_host.borrow().clone() else {
            return crate::PresenterHostStyle::new();
        };
        let Some(placeholder_host) = self.placeholder_host.borrow().clone() else {
            return crate::PresenterHostStyle::new();
        };
        let editable_cursor = if state.enabled {
            CursorStyle::Text
        } else {
            CursorStyle::Default
        };
        editor_host.cursor(editable_cursor);
        placeholder_host
            .position(0.0, 0.0)
            .width(100.0, Unit::Percent)
            .cursor(editable_cursor);
        crate::PresenterHostStyle::new()
            .background(0x00000000)
            .corners(crate::Corners::all(0.0))
            .border(crate::Border::solid(0.0, 0x00000000))
            .padding(crate::EdgeInsets::all(0.0))
            .align_items(AlignItems::Center)
            .cursor(editable_cursor)
            .opacity(if state.enabled { 1.0 } else { 0.6 })
    }
}

#[derive(Clone)]
struct ComboBoxEditorTemplate;

impl TextInputTemplate for ComboBoxEditorTemplate {
    fn create(&self) -> Rc<dyn TextInputPresenter> {
        Rc::new(ComboBoxEditorPresenter::default())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComboBoxFilterMode {
    None,
    StartsWith,
    Contains,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComboBoxCommitMode {
    KeepText,
    RevertToSelection,
    SelectExactMatch,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComboBoxItem {
    pub value: String,
}

impl ComboBoxItem {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    pub fn from_value(value: impl Into<String>) -> Self {
        Self::new(value)
    }
}

#[derive(Clone)]
pub struct ComboBox {
    root: FlexBox,
    shared: Rc<ComboBoxShared>,
}

struct ComboBoxShared {
    self_weak: RefCell<Weak<ComboBoxShared>>,
    root: WeakFlexBox,
    editor: TextInput,
    chevron_host: FlexBox,
    chevron_template_value: RefCell<Option<Rc<dyn DropdownChevronTemplate>>>,
    sizing_value: Cell<Option<DropdownSizing>>,
    colors_value: Cell<Option<DropdownColors>>,
    chevron_presenter: RefCell<Rc<dyn DropdownChevronPresenter>>,
    popup_list: SelectablePopupList,
    items_value: RefCell<Vec<ComboBoxItem>>,
    filtered_indices: RefCell<Vec<i32>>,
    open_state: Cell<bool>,
    popup_pointer_pressed_state: Cell<bool>,
    pointer_pressed_state: Cell<bool>,
    hovered_state: Cell<bool>,
    focused_state: Cell<bool>,
    wrapper_focused_state: Cell<bool>,
    editor_focused_state: Cell<bool>,
    deferred_blur_close_pending_state: Cell<bool>,
    allow_custom_value: Cell<bool>,
    auto_complete_value: Cell<bool>,
    open_on_focus_value: Cell<bool>,
    stays_open_on_edit_value: Cell<bool>,
    filter_mode_value: Cell<ComboBoxFilterMode>,
    commit_mode_value: Cell<ComboBoxCommitMode>,
    key_filter_token: Cell<u32>,
    selected_index_value: Cell<i32>,
    committed_selected_index_value: Cell<i32>,
    highlighted_index_value: Cell<i32>,
    text_value: RefCell<String>,
    popup_panel_color_value: Cell<u32>,
    popup_panel_background_blur_sigma_value: Cell<f32>,
    popup_panel_color_overridden: Cell<bool>,
    popup_panel_background_blur_overridden: Cell<bool>,
    suppress_editor_changed: Cell<bool>,
    last_auto_complete_text_value: RefCell<String>,
    changed_callback:
        RefCell<Option<Rc<dyn Fn(crate::controls::ComboBoxChangedEventArgs<ComboBoxItem>)>>>,
    text_changed_callback: RefCell<Option<Rc<dyn Fn(TextChangedEventArgs)>>>,
    theme_guard: RefCell<Option<SubscriptionGuard>>,
    focus_visibility_guard: RefCell<Option<SubscriptionGuard>>,
}

thread_local! {
    static ACTIVE_COMBOBOX: RefCell<Option<Weak<ComboBoxShared>>> = const { RefCell::new(None) };
    static COMBOBOX_SCROLL_HOOK_REGISTERED: Cell<bool> = const { Cell::new(false) };
}

impl Default for ComboBox {
    fn default() -> Self {
        Self::new()
    }
}

impl ComboBox {
    pub fn new() -> Self {
        Self::with_text("")
    }

    pub fn with_text(text: impl Into<String>) -> Self {
        Self::with_initial_text(text.into())
    }

    fn with_initial_text(text: String) -> Self {
        Self::ensure_scroll_hook();
        let root = row();
        root.semantic_role(SemanticRole::ComboBox)
            .focusable(true, 0)
            .interactive(true)
            .cursor(CursorStyle::Text)
            .flex_direction(FlexDirection::Row)
            .align_items(AlignItems::Center)
            .reflect_semantic_disabled_from_enabled()
            .default_semantic_label("Combo box");

        let editor = TextInput::new();
        editor
            .text(text.clone())
            .template(Rc::new(ComboBoxEditorTemplate));
        editor.fill_width();

        let chevron_presenter = create_chevron_presenter(None, None);
        let chevron_host = row();
        chevron_host
            .width(32.0, Unit::Pixel)
            .height(100.0, Unit::Percent)
            .align_items(AlignItems::Center)
            .justify_content(crate::ffi::JustifyContent::Center)
            .child(&chevron_presenter.root());

        let weak_root = root.downgrade();
        let shared_slot: Rc<RefCell<Option<Weak<ComboBoxShared>>>> = Rc::new(RefCell::new(None));
        let owner = SelectablePopupListOwner {
            item_count: {
                let shared_slot = shared_slot.clone();
                Rc::new(move || {
                    shared_slot
                        .borrow()
                        .as_ref()
                        .and_then(Weak::upgrade)
                        .map(|shared| shared.filtered_indices.borrow().len() as i32)
                        .unwrap_or(0)
                })
            },
            item_label: {
                let shared_slot = shared_slot.clone();
                Rc::new(move |index| {
                    let Some(shared) = shared_slot.borrow().as_ref().and_then(Weak::upgrade) else {
                        return String::new();
                    };
                    let filtered = shared.filtered_indices.borrow();
                    let Some(source_index) = filtered.get(index as usize).copied() else {
                        return String::new();
                    };
                    let value = shared
                        .items_value
                        .borrow()
                        .get(source_index as usize)
                        .map(|item| item.value.clone())
                        .unwrap_or_default();
                    value
                })
            },
            item_selected: {
                let shared_slot = shared_slot.clone();
                Rc::new(move |index| {
                    let Some(shared) = shared_slot.borrow().as_ref().and_then(Weak::upgrade) else {
                        return false;
                    };
                    let filtered = shared.filtered_indices.borrow();
                    filtered.get(index as usize).is_some_and(|source_index| {
                        *source_index == shared.selected_index_value.get()
                    })
                })
            },
            enabled: {
                let weak_root = weak_root.clone();
                Rc::new(move || {
                    weak_root
                        .upgrade()
                        .is_some_and(|root| root.retained_node_ref().is_enabled_for_routing())
                })
            },
            highlight_index: {
                let shared_slot = shared_slot.clone();
                Rc::new(move |index| {
                    if let Some(shared) = shared_slot.borrow().as_ref().and_then(Weak::upgrade) {
                        shared.highlight_index(index);
                    }
                })
            },
            activate_index: {
                let shared_slot = shared_slot.clone();
                Rc::new(move |index| {
                    if let Some(shared) = shared_slot.borrow().as_ref().and_then(Weak::upgrade) {
                        shared.popup_list_activate_index(index);
                    }
                })
            },
            pointer_down: {
                let shared_slot = shared_slot.clone();
                Rc::new(move |_index| {
                    if let Some(shared) = shared_slot.borrow().as_ref().and_then(Weak::upgrade) {
                        shared.popup_pointer_pressed_state.set(true);
                    }
                })
            },
            pointer_up: {
                let shared_slot = shared_slot.clone();
                Rc::new(move |_index| {
                    if let Some(shared) = shared_slot.borrow().as_ref().and_then(Weak::upgrade) {
                        shared.popup_list_pointer_up();
                    }
                })
            },
        };
        let popup_list = SelectablePopupList::new(owner);
        let shared = Rc::new(ComboBoxShared {
            self_weak: RefCell::new(Weak::new()),
            root: weak_root,
            editor: editor.clone(),
            chevron_host: chevron_host.clone(),
            chevron_template_value: RefCell::new(None),
            sizing_value: Cell::new(None),
            colors_value: Cell::new(None),
            chevron_presenter: RefCell::new(chevron_presenter.clone()),
            popup_list: popup_list.clone(),
            items_value: RefCell::new(Vec::new()),
            filtered_indices: RefCell::new(Vec::new()),
            open_state: Cell::new(false),
            popup_pointer_pressed_state: Cell::new(false),
            pointer_pressed_state: Cell::new(false),
            hovered_state: Cell::new(false),
            focused_state: Cell::new(false),
            wrapper_focused_state: Cell::new(false),
            editor_focused_state: Cell::new(false),
            deferred_blur_close_pending_state: Cell::new(false),
            allow_custom_value: Cell::new(true),
            auto_complete_value: Cell::new(false),
            open_on_focus_value: Cell::new(false),
            stays_open_on_edit_value: Cell::new(true),
            filter_mode_value: Cell::new(ComboBoxFilterMode::Contains),
            commit_mode_value: Cell::new(ComboBoxCommitMode::KeepText),
            key_filter_token: Cell::new(0),
            selected_index_value: Cell::new(-1),
            committed_selected_index_value: Cell::new(-1),
            highlighted_index_value: Cell::new(-1),
            text_value: RefCell::new(text),
            popup_panel_color_value: Cell::new(0x00000000),
            popup_panel_background_blur_sigma_value: Cell::new(DEFAULT_PANEL_BACKGROUND_BLUR_SIGMA),
            popup_panel_color_overridden: Cell::new(false),
            popup_panel_background_blur_overridden: Cell::new(false),
            suppress_editor_changed: Cell::new(false),
            last_auto_complete_text_value: RefCell::new(String::new()),
            changed_callback: RefCell::new(None),
            text_changed_callback: RefCell::new(None),
            theme_guard: RefCell::new(None),
            focus_visibility_guard: RefCell::new(None),
        });
        *shared.self_weak.borrow_mut() = Rc::downgrade(&shared);
        *shared_slot.borrow_mut() = Some(Rc::downgrade(&shared));
        root.retained_node_ref().retain_attachment(shared.clone());
        shared.rebuild_filtered_indices();

        popup_list.popup_presenter.overlay_node().on_click({
            let weak_shared = Rc::downgrade(&shared);
            move |_event| {
                if let Some(shared) = weak_shared.upgrade() {
                    shared.close();
                }
            }
        });

        root.child(&editor)
            .child(&chevron_host)
            .child(&popup_list.root);

        let control = Self { root, shared };
        control.install_visual_subscriptions();
        control.install_effective_enabled_subscription();
        control.bind_events();
        control.shared.sync_semantic_label();
        control.shared.handle_theme_changed();
        control
    }

    fn ensure_scroll_hook() {
        COMBOBOX_SCROLL_HOOK_REGISTERED.with(|registered| {
            if registered.get() {
                return;
            }
            registered.set(true);
            event::register_scroll_hook(|| {
                ACTIVE_COMBOBOX.with(|slot| {
                    let Some(shared) = slot.borrow().as_ref().and_then(Weak::upgrade) else {
                        return;
                    };
                    if !shared.is_trigger_visible_in_viewport() {
                        shared.close();
                    }
                });
            });
        });
    }

    fn install_visual_subscriptions(&self) {
        *self.shared.theme_guard.borrow_mut() = Some(subscribe({
            let weak_shared = Rc::downgrade(&self.shared);
            move |_theme| {
                if let Some(shared) = weak_shared.upgrade() {
                    shared.handle_theme_changed();
                }
            }
        }));
        *self.shared.focus_visibility_guard.borrow_mut() = Some(focus_visibility::subscribe({
            let weak_shared = Rc::downgrade(&self.shared);
            move |_visible| {
                if let Some(shared) = weak_shared.upgrade() {
                    shared.sync_focus_chrome();
                }
            }
        }));
    }

    fn install_effective_enabled_subscription(&self) {
        let weak_shared = Rc::downgrade(&self.shared);
        self.root
            .retained_node_ref()
            .on_effective_enabled_changed(Rc::new(move |_enabled| {
                let Some(shared) = weak_shared.upgrade() else {
                    return;
                };
                shared.editor.enabled(shared.is_enabled());
                if !shared.is_enabled() {
                    shared.pointer_pressed_state.set(false);
                    shared.hovered_state.set(false);
                    shared.close();
                }
                shared.handle_theme_changed();
            }));
    }

    fn bind_events(&self) {
        let shared = self.shared.clone();
        self.root.on_pointer_enter(move |_event| {
            if !shared.is_enabled() {
                return;
            }
            shared.hovered_state.set(true);
            shared.handle_theme_changed();
        });

        let shared = self.shared.clone();
        self.root.on_pointer_leave(move |_event| {
            shared.pointer_pressed_state.set(false);
            shared.hovered_state.set(false);
            shared.handle_theme_changed();
        });

        let shared = self.shared.clone();
        self.root.on_pointer_down(move |_event| {
            if !shared.is_enabled() {
                return;
            }
            shared.pointer_pressed_state.set(true);
            shared.editor.focus_now();
            shared.handle_theme_changed();
        });

        let shared = self.shared.clone();
        self.root.on_pointer_up(move |event| {
            if !shared.is_enabled() || !shared.pointer_pressed_state.get() {
                return;
            }
            shared.pointer_pressed_state.set(false);
            shared.toggle_open();
            shared.handle_theme_changed();
            event.handled = true;
        });

        let shared = self.shared.clone();
        self.root.on_key_down(move |event| {
            if shared.handle_global_key_event(
                KeyEventType::Down,
                event.key.as_str(),
                event.modifiers,
            ) {
                event.handled = true;
                return;
            }
            if !shared.is_enabled() || event.modifiers != 0 {
                return;
            }
            if !shared.open_state.get()
                && (event.key == "Enter" || event.key == " " || event.key == "ArrowDown")
            {
                shared.open();
                event.handled = true;
                return;
            }
            if !shared.open_state.get() && event.key == "ArrowUp" {
                shared.open();
                shared.move_highlight(-1);
                event.handled = true;
            }
        });

        let shared = self.shared.clone();
        self.root.on_key_up(move |event| {
            if shared.handle_global_key_event(KeyEventType::Up, event.key.as_str(), event.modifiers)
            {
                event.handled = true;
            }
        });

        let shared = self.shared.clone();
        self.root.on_focus_changed(move |event| {
            shared.wrapper_focused_state.set(event.focused);
            if !event.focused && !shared.open_state.get() {
                shared.pointer_pressed_state.set(false);
            }
            shared.sync_focused_state();
        });

        let weak_shared = Rc::downgrade(&self.shared);
        self.shared.editor.on_changed(move |event| {
            let Some(shared) = weak_shared.upgrade() else {
                return;
            };
            shared.handle_editor_text_changed(event.text);
        });

        let weak_shared = Rc::downgrade(&self.shared);
        self.shared.editor.on_focus_changed(move |event| {
            let Some(shared) = weak_shared.upgrade() else {
                return;
            };
            shared.handle_editor_focus_changed(event.focused);
        });

        let weak_shared = Rc::downgrade(&self.shared);
        self.shared.editor.editor_node().on_key_down(move |event| {
            let Some(shared) = weak_shared.upgrade() else {
                return;
            };
            if shared.handle_editor_key_down(event.key.as_str(), event.modifiers) {
                event.handled = true;
            }
        });
        self.shared.editor.editor_node().editor_command_keys(true);

        let weak_shared = Rc::downgrade(&self.shared);
        self.shared.chevron_host.on_click(move |event| {
            let Some(shared) = weak_shared.upgrade() else {
                return;
            };
            if !shared.is_enabled() {
                return;
            }
            shared.toggle_from_chevron();
            event.handled = true;
        });

        let weak_shared = Rc::downgrade(&self.shared);
        self.shared.chevron_host.on_pointer_enter(move |_event| {
            let Some(shared) = weak_shared.upgrade() else {
                return;
            };
            shared.hovered_state.set(true);
            shared.handle_theme_changed();
        });

        let weak_shared = Rc::downgrade(&self.shared);
        self.shared.chevron_host.on_pointer_leave(move |_event| {
            let Some(shared) = weak_shared.upgrade() else {
                return;
            };
            shared.pointer_pressed_state.set(false);
            shared.hovered_state.set(false);
            shared.handle_theme_changed();
        });

        let weak_shared = Rc::downgrade(&self.shared);
        self.shared.chevron_host.on_pointer_down(move |_event| {
            let Some(shared) = weak_shared.upgrade() else {
                return;
            };
            shared.pointer_pressed_state.set(true);
            shared.focus_editor_from_chevron();
            shared.handle_theme_changed();
        });

        let weak_shared = Rc::downgrade(&self.shared);
        self.shared.chevron_host.on_pointer_up(move |_event| {
            let Some(shared) = weak_shared.upgrade() else {
                return;
            };
            shared.pointer_pressed_state.set(false);
            shared.handle_theme_changed();
        });
    }

    pub fn selected_index(&self) -> i32 {
        self.shared.selected_index_value.get()
    }

    pub fn value(&self) -> String {
        self.shared.text_value.borrow().clone()
    }

    pub fn filtered_count(&self) -> usize {
        self.shared.filtered_indices.borrow().len()
    }

    pub fn highlighted_index(&self) -> i32 {
        self.shared.highlighted_index_value.get()
    }

    pub fn is_open(&self) -> bool {
        self.shared.open_state.get()
    }

    pub fn items<I, S>(&self, items: I) -> &Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.shared.close();
        let mut slot = self.shared.items_value.borrow_mut();
        slot.clear();
        slot.extend(items.into_iter().map(ComboBoxItem::new));
        drop(slot);
        self.shared.sync_selection_from_text();
        self.shared.rebuild_filtered_indices();
        self.shared.popup_list.refresh_panel_layout();
        self.shared.sync_option_visuals();
        self.shared.sync_semantic_label();
        self
    }

    pub fn text(&self, value: impl Into<String>) -> &Self {
        self.shared.set_text(value.into(), false);
        self
    }

    pub fn placeholder(&self, value: impl Into<String>) -> &Self {
        self.shared.editor.placeholder(value);
        self.shared.sync_semantic_label();
        self
    }

    pub fn allow_custom(&self, flag: bool) -> &Self {
        self.shared.allow_custom_value.set(flag);
        self.shared.sync_selection_from_text();
        self
    }

    pub fn auto_complete(&self, flag: bool) -> &Self {
        self.shared.auto_complete_value.set(flag);
        self
    }

    pub fn filter_mode(&self, mode: ComboBoxFilterMode) -> &Self {
        self.shared.filter_mode_value.set(mode);
        self.shared.rebuild_filtered_indices();
        self.shared.refresh_popup_after_filter();
        self.shared.sync_option_visuals();
        self
    }

    pub fn commit_mode(&self, mode: ComboBoxCommitMode) -> &Self {
        self.shared.commit_mode_value.set(mode);
        self
    }

    pub fn open_on_focus(&self, flag: bool) -> &Self {
        self.shared.open_on_focus_value.set(flag);
        self
    }

    pub fn stays_open_on_edit(&self, flag: bool) -> &Self {
        self.shared.stays_open_on_edit_value.set(flag);
        self
    }

    pub fn max_visible_items(&self, count: i32) -> &Self {
        self.shared.popup_list.max_visible_items(count);
        self
    }

    pub fn popup_width(&self, value: f32) -> &Self {
        self.shared.popup_list.popup_width(value);
        self
    }

    pub fn popup_panel_color(&self, color: u32) -> &Self {
        self.shared.popup_panel_color_overridden.set(true);
        self.shared.popup_panel_color_value.set(color);
        self.shared.popup_list.panel_node.bg_color(color);
        self
    }

    pub fn popup_panel_background_blur(&self, sigma: f32) -> &Self {
        self.shared.popup_panel_background_blur_overridden.set(true);
        if sigma < 0.0 {
            logger::warn(
                "Layout",
                &format!("ComboBox.popupPanelBackgroundBlur() received {sigma}; clamping to 0.0."),
            );
        }
        self.shared
            .popup_panel_background_blur_sigma_value
            .set(sigma.max(0.0));
        self.shared
            .popup_list
            .panel_node
            .background_blur(self.shared.popup_panel_background_blur_sigma_value.get());
        self
    }

    pub fn sizing(&self, sizing: DropdownSizing) -> &Self {
        self.set_sizing(Some(sizing))
    }

    pub fn clear_sizing(&self) -> &Self {
        self.set_sizing(None)
    }

    fn set_sizing(&self, sizing: Option<DropdownSizing>) -> &Self {
        self.shared.sizing_value.set(sizing);
        let previous_presenter = self.shared.chevron_presenter.borrow().clone();
        let next_presenter =
            create_chevron_presenter(self.shared.chevron_template_value.borrow().clone(), sizing);
        *self.shared.chevron_presenter.borrow_mut() = next_presenter.clone();
        self.shared
            .chevron_host
            .remove_child(&previous_presenter.root());
        self.shared.chevron_host.child(&next_presenter.root());
        previous_presenter.root().dispose();
        self.shared.popup_list.sizing(sizing);
        self.shared.handle_theme_changed();
        self
    }

    pub fn colors(&self, colors: DropdownColors) -> &Self {
        self.set_colors(Some(colors))
    }

    pub fn clear_colors(&self) -> &Self {
        self.set_colors(None)
    }

    fn set_colors(&self, colors: Option<DropdownColors>) -> &Self {
        self.shared.colors_value.set(colors);
        self.shared.popup_list.colors(colors);
        self.shared.handle_theme_changed();
        self
    }

    pub fn chevron_template(&self, template: Rc<dyn DropdownChevronTemplate>) -> &Self {
        self.set_chevron_template(Some(template))
    }

    pub fn clear_chevron_template(&self) -> &Self {
        self.set_chevron_template(None)
    }

    fn set_chevron_template(&self, template: Option<Rc<dyn DropdownChevronTemplate>>) -> &Self {
        *self.shared.chevron_template_value.borrow_mut() = template.clone();
        let previous_presenter = self.shared.chevron_presenter.borrow().clone();
        let next_presenter = create_chevron_presenter(template, self.shared.sizing_value.get());
        *self.shared.chevron_presenter.borrow_mut() = next_presenter.clone();
        self.shared
            .chevron_host
            .remove_child(&previous_presenter.root());
        self.shared.chevron_host.child(&next_presenter.root());
        previous_presenter.root().dispose();
        self.shared.handle_theme_changed();
        self
    }

    pub fn option_row_template(&self, template: Rc<dyn DropdownOptionRowTemplate>) -> &Self {
        self.set_option_row_template(Some(template))
    }

    pub fn clear_option_row_template(&self) -> &Self {
        self.set_option_row_template(None)
    }

    fn set_option_row_template(
        &self,
        template: Option<Rc<dyn DropdownOptionRowTemplate>>,
    ) -> &Self {
        self.shared.close();
        self.shared.popup_list.option_row_template(template);
        self
    }

    pub fn select_index(&self, index: i32) -> &Self {
        self.shared.set_selected_index(index, false);
        self
    }

    pub fn on_changed(
        &self,
        callback: impl Fn(crate::controls::ComboBoxChangedEventArgs<ComboBoxItem>) + 'static,
    ) -> &Self {
        *self.shared.changed_callback.borrow_mut() = Some(Rc::new(callback));
        self
    }

    pub fn on_text_changed(&self, callback: impl Fn(TextChangedEventArgs) + 'static) -> &Self {
        *self.shared.text_changed_callback.borrow_mut() = Some(Rc::new(callback));
        self
    }

    pub fn focus_now(&self) -> &Self {
        if self.root.handle() != crate::node::NodeHandle::INVALID {
            ui::request_focus(self.root.handle().raw());
        }
        self
    }

    pub fn enabled(&self, enabled: bool) -> &Self {
        self.root.enabled(enabled);
        if !enabled {
            self.shared.pointer_pressed_state.set(false);
            self.shared.hovered_state.set(false);
            self.shared.close();
        }
        self.shared.handle_theme_changed();
        self
    }
}

impl HasFlexBoxRoot for ComboBox {
    fn flex_box_root(&self) -> &FlexBox {
        &self.root
    }
}

impl Node for ComboBox {
    fn retained_node_ref(&self) -> NodeRef {
        self.root.retained_node_ref()
    }

    fn build_self(&self) {
        self.root.build_self();
    }

    fn dispose(&self) {
        self.shared.close();
        focus_adorner::hide_owner(&self.root);
        *self.shared.theme_guard.borrow_mut() = None;
        *self.shared.focus_visibility_guard.borrow_mut() = None;
        self.shared.popup_list.dispose();
        self.root.dispose();
    }
}

impl ComboBoxShared {
    fn is_enabled(&self) -> bool {
        self.root
            .upgrade()
            .is_some_and(|root| root.retained_node_ref().is_enabled_for_routing())
    }

    fn handle_global_key_event(&self, event_type: KeyEventType, key: &str, modifiers: u32) -> bool {
        if !self.open_state.get() || modifiers != 0 || event_type != KeyEventType::Down {
            return false;
        }
        match key {
            "Escape" => {
                self.close();
                true
            }
            "Enter" => {
                self.select_highlighted();
                true
            }
            "Home" => {
                self.highlight_index(0);
                true
            }
            "End" => {
                self.highlight_index(self.filtered_indices.borrow().len() as i32 - 1);
                true
            }
            "ArrowDown" => {
                self.move_highlight(1);
                true
            }
            "ArrowUp" => {
                self.move_highlight(-1);
                true
            }
            _ => false,
        }
    }

    fn toggle_from_chevron(&self) {
        if !self.is_enabled() {
            return;
        }
        self.focus_editor_from_chevron();
        self.toggle_open();
        self.handle_theme_changed();
    }

    fn focus_editor_from_chevron(&self) {
        self.editor.caret_to_end();
        self.editor.focus_now();
        self.editor.caret_to_end();
    }

    fn toggle_open(&self) {
        if self.open_state.get() {
            self.close();
        } else {
            self.open();
        }
    }

    fn handle_editor_key_down(&self, key: &str, modifiers: u32) -> bool {
        if !self.is_enabled() {
            return false;
        }
        if modifiers != 0 {
            return Self::is_text_navigation_key(key, modifiers);
        }
        match key {
            "ArrowDown" => {
                if !self.open_state.get() {
                    self.open();
                } else {
                    self.move_highlight(1);
                }
                true
            }
            "ArrowUp" => {
                if !self.open_state.get() {
                    self.open();
                }
                self.move_highlight(-1);
                true
            }
            "Enter" if self.open_state.get() => {
                self.select_highlighted();
                true
            }
            "Escape" if self.open_state.get() => {
                self.close();
                true
            }
            _ => Self::is_text_navigation_key(key, modifiers),
        }
    }

    fn is_text_navigation_key(key: &str, modifiers: u32) -> bool {
        let non_shift_modifiers = modifiers
            & ((KeyModifier::Ctrl as u32) | (KeyModifier::Alt as u32) | (KeyModifier::Meta as u32));
        non_shift_modifiers == 0
            && matches!(
                key,
                "ArrowLeft"
                    | "ArrowRight"
                    | "ArrowUp"
                    | "ArrowDown"
                    | "Home"
                    | "End"
                    | "PageUp"
                    | "PageDown"
            )
    }

    fn handle_editor_text_changed(&self, value: String) {
        if self.suppress_editor_changed.get() {
            return;
        }
        let mut next_value = value.clone();
        let mut completion_selection: Option<(u32, u32)> = None;
        let deleting_text = value.len() < self.text_value.borrow().len();
        let should_auto_complete = self.auto_complete_value.get()
            && !value.is_empty()
            && !deleting_text
            && value != *self.last_auto_complete_text_value.borrow();
        self.last_auto_complete_text_value.borrow_mut().clear();
        if should_auto_complete {
            let auto_complete_index = self.find_auto_complete_match(&value);
            if auto_complete_index >= 0 {
                let completed_value = self.items_value.borrow()[auto_complete_index as usize]
                    .value
                    .clone();
                if completed_value.len() > value.len() {
                    let selection_start = value.chars().count() as u32;
                    let selection_end = completed_value.chars().count() as u32;
                    next_value = completed_value;
                    completion_selection = Some((selection_start, selection_end));
                    *self.last_auto_complete_text_value.borrow_mut() = value.clone();
                }
            }
        }
        if let Some((selection_start, selection_end)) = completion_selection {
            self.suppress_editor_changed.set(true);
            self.editor.text(next_value.clone());
            self.editor.selection_range(selection_start, selection_end);
            self.suppress_editor_changed.set(false);
        }
        *self.text_value.borrow_mut() = next_value.clone();
        self.sync_selection_from_text();
        self.rebuild_filtered_indices();
        if self.filtered_indices.borrow().is_empty() {
            self.close();
        } else if self.stays_open_on_edit_value.get() {
            self.highlighted_index_value.set(0);
            if self.open_state.get() {
                self.refresh_open_popup();
            } else {
                self.open();
            }
        }
        self.refresh_popup_after_filter();
        self.sync_option_visuals();
        self.sync_semantic_label();
        self.emit_text_changed(next_value);
    }

    fn handle_editor_focus_changed(&self, focused: bool) {
        self.editor_focused_state.set(focused);
        if focused && self.open_on_focus_value.get() {
            self.open();
        }
        if !focused && !self.open_state.get() {
            self.commit_current_text();
            self.pointer_pressed_state.set(false);
        }
        self.sync_focused_state();
    }

    fn sync_focused_state(&self) {
        let next_focused = self.wrapper_focused_state.get() || self.editor_focused_state.get();
        if !next_focused && !self.popup_pointer_pressed_state.get() {
            self.schedule_deferred_blur_close();
        }
        if self.focused_state.get() == next_focused {
            return;
        }
        self.focused_state.set(next_focused);
        self.handle_theme_changed();
    }

    fn schedule_deferred_blur_close(&self) {
        if self.deferred_blur_close_pending_state.get() {
            return;
        }
        self.deferred_blur_close_pending_state.set(true);
        let weak = self.self_weak.borrow().clone();
        app::after_next_commit(move || {
            if let Some(shared) = weak.upgrade() {
                shared.fire_deferred_blur_close();
            }
        });
        frame_scheduler::mark_needs_commit();
    }

    fn fire_deferred_blur_close(&self) {
        self.deferred_blur_close_pending_state.set(false);
        let next_focused = self.wrapper_focused_state.get() || self.editor_focused_state.get();
        if !next_focused && !self.popup_pointer_pressed_state.get() {
            self.pointer_pressed_state.set(false);
            self.close();
        }
    }

    fn set_text(&self, value: String, emit: bool) {
        if *self.text_value.borrow() == value {
            return;
        }
        *self.text_value.borrow_mut() = value.clone();
        self.suppress_editor_changed.set(true);
        self.editor.text(value.clone());
        self.suppress_editor_changed.set(false);
        self.sync_selection_from_text();
        self.rebuild_filtered_indices();
        self.refresh_popup_after_filter();
        self.sync_option_visuals();
        self.sync_semantic_label();
        if emit {
            self.emit_text_changed(value);
        }
    }

    fn set_selected_index(&self, index: i32, emit: bool) {
        if index == -1 {
            self.selected_index_value.set(-1);
            self.committed_selected_index_value.set(-1);
            self.highlighted_index_value.set(-1);
            self.popup_list.set_highlighted_index(-1);
            self.sync_semantic_label();
            return;
        }
        let count = self.items_value.borrow().len() as i32;
        if count == 0 {
            if index != -1 {
                logger::warn(
                    "Layout",
                    &format!(
                        "ComboBox.selectIndex() received {index} before any items were assigned."
                    ),
                );
            }
            return;
        }
        let clamped_index = index.clamp(0, count - 1);
        if clamped_index != index {
            logger::warn(
                "Layout",
                &format!("ComboBox.selectIndex() received {index}; clamping to {clamped_index}."),
            );
        }
        let changed = self.selected_index_value.get() != clamped_index;
        self.selected_index_value.set(clamped_index);
        self.committed_selected_index_value.set(clamped_index);
        let item = self.items_value.borrow()[clamped_index as usize].clone();
        self.set_text(item.value, false);
        self.editor.caret_to_end();
        self.rebuild_filtered_indices();
        let visible_index = self.find_visible_index_for_source_index(clamped_index);
        self.highlighted_index_value.set(visible_index);
        self.popup_list.set_highlighted_index(visible_index);
        self.sync_semantic_label();
        if emit && changed {
            if let Some(root) = self.root.upgrade() {
                root.request_semantic_announcement();
            }
            self.emit_selection_changed();
        }
    }

    fn sync_selection_from_text(&self) {
        let exact_index = self.find_exact_text_match(&self.text_value.borrow());
        if exact_index >= 0 {
            self.selected_index_value.set(exact_index);
            return;
        }
        if self.allow_custom_value.get() {
            self.selected_index_value.set(-1);
        }
    }

    fn commit_current_text(&self) {
        match self.commit_mode_value.get() {
            ComboBoxCommitMode::KeepText => {}
            ComboBoxCommitMode::SelectExactMatch => {
                let exact_index = self.find_exact_text_match(&self.text_value.borrow());
                if exact_index >= 0 {
                    self.set_selected_index(exact_index, true);
                }
            }
            ComboBoxCommitMode::RevertToSelection => {
                let committed = self.committed_selected_index_value.get();
                if committed >= 0 && committed < self.items_value.borrow().len() as i32 {
                    let value = self.items_value.borrow()[committed as usize].value.clone();
                    self.set_text(value, true);
                    self.editor.caret_to_end();
                    self.selected_index_value.set(committed);
                }
            }
        }
    }

    fn emit_selection_changed(&self) {
        let selected_index = self.selected_index_value.get();
        if selected_index < 0 {
            return;
        }
        let Some(item) = self
            .items_value
            .borrow()
            .get(selected_index as usize)
            .cloned()
        else {
            return;
        };
        if let Some(callback) = self.changed_callback.borrow().clone() {
            callback(crate::controls::ComboBoxChangedEventArgs {
                item,
                selected_index,
            });
        }
    }

    fn emit_text_changed(&self, value: String) {
        if let Some(callback) = self.text_changed_callback.borrow().clone() {
            callback(TextChangedEventArgs { text: value });
        }
    }

    fn open(&self) {
        let Some(root) = self.root.upgrade() else {
            return;
        };
        if self.open_state.get()
            || self.filtered_indices.borrow().is_empty()
            || root.handle() == crate::node::NodeHandle::INVALID
        {
            return;
        }
        let initial_highlight = if self.selected_index_value.get() >= 0 {
            self.find_visible_index_for_source_index(self.selected_index_value.get())
        } else {
            0
        };
        self.popup_list.set_highlighted_index(initial_highlight);
        self.highlighted_index_value
            .set(self.popup_list.highlighted_index());
        let Some(bounds) = ui::get_bounds(root.handle().raw()) else {
            return;
        };
        if !self.popup_list.open(
            bounds[0],
            bounds[1],
            bounds[2],
            bounds[3],
            initial_highlight,
        ) {
            return;
        }
        self.highlighted_index_value
            .set(self.popup_list.highlighted_index());
        self.open_state.set(true);
        ACTIVE_COMBOBOX.with(|slot| {
            *slot.borrow_mut() = Some(self.self_weak.borrow().clone());
        });
        ui::set_semantic_expanded(root.handle().raw(), true, true);
        root.request_semantic_announcement();
        if self.key_filter_token.get() == 0 {
            let weak_self = self.self_weak.borrow().clone();
            let token = event::push_key_filter(move |event_type, key, modifiers| {
                weak_self.upgrade().is_some_and(|shared| {
                    shared.handle_global_key_event(event_type, key, modifiers)
                })
            });
            self.key_filter_token.set(token);
        }
        self.handle_theme_changed();
    }

    fn close(&self) {
        if !self.open_state.get() && !self.popup_list.is_open() {
            return;
        }
        self.deferred_blur_close_pending_state.set(false);
        self.popup_pointer_pressed_state.set(false);
        self.popup_list.close();
        self.open_state.set(false);
        ACTIVE_COMBOBOX.with(|slot| {
            let should_clear = slot
                .borrow()
                .as_ref()
                .and_then(Weak::upgrade)
                .and_then(|active| active.root.upgrade())
                .and_then(|active_root| {
                    self.root
                        .upgrade()
                        .map(|root| active_root.handle() == root.handle())
                })
                .unwrap_or(false);
            if should_clear {
                slot.borrow_mut().take();
            }
        });
        if let Some(root) = self.root.upgrade() {
            ui::set_semantic_expanded(root.handle().raw(), true, false);
            root.request_semantic_announcement();
        }
        let token = self.key_filter_token.replace(0);
        if token != 0 {
            event::remove_key_filter(token);
        }
        self.commit_current_text();
        self.handle_theme_changed();
    }

    fn is_trigger_visible_in_viewport(&self) -> bool {
        let Some(root) = self.root.upgrade() else {
            return true;
        };
        let Some(bounds) = ui::get_bounds(root.handle().raw()) else {
            return true;
        };
        let x = bounds[0];
        let y = bounds[1];
        let width = bounds[2];
        let height = bounds[3];
        if width <= 0.0 || height <= 0.0 {
            return true;
        }
        let right = x + width;
        let bottom = y + height;
        let mut current = root.retained_node_ref().parent();
        while let Some(node) = current {
            if node.node_type() == NodeType::ScrollView {
                if let Some(scroll_bounds) = ui::get_bounds(node.handle().raw()) {
                    let sv_x = scroll_bounds[0];
                    let sv_y = scroll_bounds[1];
                    let sv_right = sv_x + scroll_bounds[2];
                    let sv_bottom = sv_y + scroll_bounds[3];
                    return right > sv_x && bottom > sv_y && x < sv_right && y < sv_bottom;
                }
                break;
            }
            current = node.parent();
        }
        let viewport_width = ui::get_viewport_width();
        let viewport_height = ui::get_viewport_height();
        right > 0.0 && bottom > 0.0 && x < viewport_width && y < viewport_height
    }

    fn highlight_index(&self, index: i32) {
        self.popup_list.highlight_index(index);
        self.highlighted_index_value
            .set(self.popup_list.highlighted_index());
    }

    fn move_highlight(&self, delta: i32) {
        self.popup_list.move_highlight(delta);
        self.highlighted_index_value
            .set(self.popup_list.highlighted_index());
    }

    fn refresh_popup_after_filter(&self) {
        if self.open_state.get() {
            self.refresh_open_popup();
        } else {
            self.popup_list.refresh_panel_layout();
        }
    }

    fn refresh_open_popup(&self) {
        let Some(root) = self.root.upgrade() else {
            self.popup_list.refresh_panel_layout();
            return;
        };
        let Some(bounds) = ui::get_bounds(root.handle().raw()) else {
            self.popup_list.refresh_panel_layout();
            return;
        };
        self.popup_list.refresh_open(
            bounds[0],
            bounds[1],
            bounds[2],
            bounds[3],
            self.highlighted_index_value.get(),
        );
        self.highlighted_index_value
            .set(self.popup_list.highlighted_index());
    }

    fn select_highlighted(&self) {
        let highlighted = self.highlighted_index_value.get();
        let source_index = {
            let filtered = self.filtered_indices.borrow();
            if highlighted < 0 || highlighted >= filtered.len() as i32 {
                return;
            }
            filtered[highlighted as usize]
        };
        if highlighted < 0 {
            return;
        }
        self.set_selected_index(source_index, true);
        self.close();
    }

    fn rebuild_filtered_indices(&self) {
        let mut filtered = self.filtered_indices.borrow_mut();
        filtered.clear();
        for (index, item) in self.items_value.borrow().iter().enumerate() {
            if self.should_include_item(item) {
                filtered.push(index as i32);
            }
        }
        if self.highlighted_index_value.get() >= filtered.len() as i32 {
            self.highlighted_index_value.set(if filtered.is_empty() {
                -1
            } else {
                filtered.len() as i32 - 1
            });
        }
    }

    fn should_include_item(&self, item: &ComboBoxItem) -> bool {
        let text = self.text_value.borrow();
        if self.filter_mode_value.get() == ComboBoxFilterMode::None || text.is_empty() {
            return true;
        }
        if self.filter_mode_value.get() == ComboBoxFilterMode::StartsWith {
            return string_starts_with_ignore_case(&item.value, &text);
        }
        string_contains_ignore_case(&item.value, &text)
    }

    fn find_auto_complete_match(&self, text: &str) -> i32 {
        for (index, item) in self.items_value.borrow().iter().enumerate() {
            if string_starts_with_ignore_case(&item.value, text) {
                return index as i32;
            }
        }
        -1
    }

    fn find_exact_text_match(&self, text: &str) -> i32 {
        for (index, item) in self.items_value.borrow().iter().enumerate() {
            if strings_equal_ignore_case(&item.value, text) {
                return index as i32;
            }
        }
        -1
    }

    fn find_visible_index_for_source_index(&self, source_index: i32) -> i32 {
        for (index, visible) in self.filtered_indices.borrow().iter().enumerate() {
            if *visible == source_index {
                return index as i32;
            }
        }
        if self.filtered_indices.borrow().is_empty() {
            -1
        } else {
            0
        }
    }

    fn handle_theme_changed(&self) {
        let Some(root) = self.root.upgrade() else {
            return;
        };
        let theme = current_theme();
        if !self.popup_panel_color_overridden.get() {
            self.popup_panel_color_value
                .set(theme.context_menu.panel_background);
        }
        if !self.popup_panel_background_blur_overridden.get() {
            self.popup_panel_background_blur_sigma_value
                .set(DEFAULT_PANEL_BACKGROUND_BLUR_SIGMA);
        }
        let sizing = self.sizing_value.get();
        let field_height = sizing
            .filter(|value| value.has_field_height())
            .map(|value| value.field_height_px())
            .unwrap_or(32.0);
        let chevron_box_size = sizing
            .filter(|value| value.has_chevron_box_size())
            .map(|value| value.chevron_box_size_px())
            .unwrap_or(32.0);
        let field_font_size = sizing
            .filter(|value| value.has_field_font_size())
            .map(|value| value.field_font_size_px())
            .unwrap_or(theme.fonts.size_body);
        let field_border_width = 2.0;
        let field_content_height = (field_height - (field_border_width * 2.0)).max(0.0);
        root.cursor(if self.is_enabled() {
            CursorStyle::Text
        } else {
            CursorStyle::Default
        })
        .corner_radius(0.0)
        .border(0.0, 0x00000000)
        .padding(0.0, 0.0, 0.0, 0.0)
        .bg_color(0x00000000)
        .opacity(if self.is_enabled() { 1.0 } else { 0.6 });
        let colors = self.colors_value.get();
        let field_background = colors
            .filter(|value| value.has_background())
            .map(|value| value.background_color())
            .unwrap_or(theme.colors.surface);
        let field_border_color = colors
            .filter(|value| value.has_border())
            .map(|value| value.border_color())
            .unwrap_or(theme.colors.border);
        root.height(field_height, Unit::Pixel)
            .corner_radius(theme.spacing.sm)
            .border(field_border_width, field_border_color)
            .padding(16.0, 0.0, 8.0, 0.0)
            .bg_color(field_background);
        self.editor
            .height(field_content_height, Unit::Pixel)
            .font_size(field_font_size)
            .line_height(field_content_height);
        if let Some(colors) = resolve_text_input_colors(colors, &theme) {
            self.editor.colors(colors);
        } else {
            self.editor.clear_colors();
        }
        self.chevron_host
            .width(chevron_box_size, Unit::Pixel)
            .height(field_content_height, Unit::Pixel)
            .align_items(AlignItems::Center)
            .justify_content(crate::ffi::JustifyContent::Center)
            .cursor(if self.is_enabled() {
                CursorStyle::Pointer
            } else {
                CursorStyle::Default
            });
        self.chevron_presenter.borrow().apply(
            theme.clone(),
            DropdownChevronVisualState::new(
                self.open_state.get(),
                self.hovered_state.get(),
                self.is_enabled(),
            ),
        );
        self.popup_list
            .panel_node
            .padding(
                SELECTABLE_POPUP_LIST_PANEL_PADDING,
                SELECTABLE_POPUP_LIST_PANEL_PADDING,
                SELECTABLE_POPUP_LIST_PANEL_PADDING,
                SELECTABLE_POPUP_LIST_PANEL_PADDING,
            )
            .corner_radius(theme.spacing.sm)
            .bg_color(self.popup_panel_color_value.get())
            .border(1.0, theme.context_menu.panel_border_color)
            .background_blur(self.popup_panel_background_blur_sigma_value.get())
            .drop_shadow(
                theme.context_menu.panel_shadow_color,
                0.0,
                theme.context_menu.shadow_offset_y,
                theme.context_menu.shadow_blur,
                theme.context_menu.shadow_spread,
            );
        self.popup_list.popup_scroll_box.bg_color(0x00000000);
        self.sync_option_visuals();
        self.sync_focus_chrome();
    }

    fn sync_option_visuals(&self) {
        self.popup_list.sync_option_visuals();
    }

    fn sync_semantic_label(&self) {
        if let Some(root) = self.root.upgrade() {
            if !self.text_value.borrow().is_empty() {
                root.default_semantic_label(self.text_value.borrow().clone());
            } else {
                root.default_semantic_label("Combo box");
            }
        }
    }

    fn sync_focus_chrome(&self) {
        let Some(root) = self.root.upgrade() else {
            return;
        };
        if self.focused_state.get()
            && self.is_enabled()
            && focus_visibility::keyboard_focus_visible()
        {
            focus_adorner::show_standard(&root, current_theme().spacing.sm);
        } else {
            focus_adorner::hide_owner(&root);
        }
    }

    fn popup_list_activate_index(&self, index: i32) {
        self.highlight_index(index);
        self.select_highlighted();
    }

    fn popup_list_pointer_up(&self) {
        self.popup_pointer_pressed_state.set(false);
        self.sync_focused_state();
    }
}
