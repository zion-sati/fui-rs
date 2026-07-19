use super::control_template_set::get_control_templates;
use super::internal::dropdown_chevron_presenter::{
    create_default_dropdown_chevron_presenter, DropdownChevronPresenter, DropdownChevronTemplate,
    DropdownChevronVisualState,
};
use super::internal::dropdown_field_presenter::{
    create_default_dropdown_field_presenter, DropdownFieldPresenter, DropdownFieldTemplate,
    DropdownFieldVisualState,
};
use super::internal::dropdown_option_row_presenter::DropdownOptionRowTemplate;
use super::internal::selectable_popup_list::{
    SelectablePopupList, SelectablePopupListOwner, SELECTABLE_POPUP_LIST_PANEL_PADDING,
};
use super::{DropdownChangedEventArgs, DropdownColors, DropdownSizing};
use crate::bindings::ui;
use crate::event::{self, KeyEventArgs};
use crate::ffi::{AlignItems, CursorStyle, FlexDirection, KeyEventType, NodeType, SemanticRole};
use crate::focus_adorner;
use crate::focus_visibility;
use crate::logger;
use crate::node::{row, BoxStyleSurface, FlexBox, HasFlexBoxRoot, Node, NodeRef, WeakFlexBox};
use crate::persisted::{persisted_value_adapter, PersistedInt32Codec};
use crate::signal::SubscriptionGuard;
use crate::theme::{current_theme, subscribe};
use crate::ThemeBindable;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

type DropdownChangedCallback = Rc<dyn Fn(DropdownChangedEventArgs<DropdownItem>)>;

const DEFAULT_PANEL_BACKGROUND_BLUR_SIGMA: f32 = 10.0;

fn is_activation_key(event: &KeyEventArgs) -> bool {
    event.key == "Enter" || event.key == " " || event.key == "ArrowDown"
}

fn create_field_presenter(
    template: Option<Rc<dyn DropdownFieldTemplate>>,
    sizing: Option<DropdownSizing>,
) -> Rc<dyn DropdownFieldPresenter> {
    if let Some(template) = template {
        return template.create(sizing);
    }
    if let Some(template) = get_control_templates().and_then(|set| set.dropdown_field) {
        return template.create(sizing);
    }
    create_default_dropdown_field_presenter(sizing)
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DropdownItem {
    pub value: String,
    pub label: String,
}

impl DropdownItem {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }

    pub fn from_value(value: impl Into<String>) -> Self {
        let value = value.into();
        Self {
            label: value.clone(),
            value,
        }
    }
}

#[derive(Clone)]
pub struct Dropdown {
    root: FlexBox,
    shared: Rc<DropdownShared>,
}

struct DropdownShared {
    self_weak: RefCell<Weak<DropdownShared>>,
    root: WeakFlexBox,
    field_template_value: RefCell<Option<Rc<dyn DropdownFieldTemplate>>>,
    chevron_template_value: RefCell<Option<Rc<dyn DropdownChevronTemplate>>>,
    option_row_template_value: RefCell<Option<Rc<dyn DropdownOptionRowTemplate>>>,
    sizing_value: Cell<Option<DropdownSizing>>,
    colors_value: Cell<Option<DropdownColors>>,
    field_presenter: RefCell<Rc<dyn DropdownFieldPresenter>>,
    chevron_presenter: RefCell<Rc<dyn DropdownChevronPresenter>>,
    popup_list: SelectablePopupList,
    items_value: RefCell<Vec<DropdownItem>>,
    open_state: Cell<bool>,
    pointer_pressed_state: Cell<bool>,
    hovered_state: Cell<bool>,
    focused_state: Cell<bool>,
    key_filter_token: Cell<u32>,
    selected_index_value: Cell<i32>,
    highlighted_index_value: Cell<i32>,
    popup_panel_color_value: Cell<u32>,
    popup_panel_background_blur_sigma_value: Cell<f32>,
    popup_panel_color_overridden: Cell<bool>,
    popup_panel_background_blur_overridden: Cell<bool>,
    changed_callback: RefCell<Option<DropdownChangedCallback>>,
    theme_guard: RefCell<Option<SubscriptionGuard>>,
    focus_visibility_guard: RefCell<Option<SubscriptionGuard>>,
}

thread_local! {
    static ACTIVE_DROPDOWN: RefCell<Option<Weak<DropdownShared>>> = const { RefCell::new(None) };
    static DROPDOWN_SCROLL_HOOK_REGISTERED: Cell<bool> = const { Cell::new(false) };
}

impl Default for Dropdown {
    fn default() -> Self {
        Self::new()
    }
}

impl Dropdown {
    pub fn new() -> Self {
        Self::ensure_scroll_hook();
        let root = row();
        root.semantic_role(SemanticRole::ComboBox)
            .focusable(true, 0)
            .interactive(true)
            .cursor(CursorStyle::Pointer)
            .flex_direction(FlexDirection::Row)
            .align_items(AlignItems::Center)
            .reflect_semantic_disabled_from_enabled()
            .default_semantic_label("Dropdown");

        let weak_root = root.downgrade();
        let field_presenter = create_field_presenter(None, None);
        field_presenter.root().fill_width();
        let chevron_presenter = create_chevron_presenter(None, None);
        field_presenter
            .chevron_host()
            .child(&chevron_presenter.root());

        let shared_slot: Rc<RefCell<Option<Weak<DropdownShared>>>> = Rc::new(RefCell::new(None));
        let owner = SelectablePopupListOwner {
            item_count: {
                let shared_slot = shared_slot.clone();
                Rc::new(move || {
                    shared_slot
                        .borrow()
                        .as_ref()
                        .and_then(Weak::upgrade)
                        .map(|shared| shared.items_value.borrow().len() as i32)
                        .unwrap_or(0)
                })
            },
            item_label: {
                let shared_slot = shared_slot.clone();
                Rc::new(move |index| {
                    shared_slot
                        .borrow()
                        .as_ref()
                        .and_then(Weak::upgrade)
                        .and_then(|shared| shared.items_value.borrow().get(index as usize).cloned())
                        .map(|item| item.label)
                        .unwrap_or_default()
                })
            },
            item_selected: {
                let shared_slot = shared_slot.clone();
                Rc::new(move |index| {
                    shared_slot
                        .borrow()
                        .as_ref()
                        .and_then(Weak::upgrade)
                        .is_some_and(|shared| shared.selected_index_value.get() == index)
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
            pointer_down: Rc::new(move |_index| {}),
            pointer_up: Rc::new(move |_index| {}),
        };
        let popup_list = SelectablePopupList::new(owner);
        let shared = Rc::new(DropdownShared {
            self_weak: RefCell::new(Weak::new()),
            root: weak_root,
            field_template_value: RefCell::new(None),
            chevron_template_value: RefCell::new(None),
            option_row_template_value: RefCell::new(None),
            sizing_value: Cell::new(None),
            colors_value: Cell::new(None),
            field_presenter: RefCell::new(field_presenter.clone()),
            chevron_presenter: RefCell::new(chevron_presenter.clone()),
            popup_list: popup_list.clone(),
            items_value: RefCell::new(Vec::new()),
            open_state: Cell::new(false),
            pointer_pressed_state: Cell::new(false),
            hovered_state: Cell::new(false),
            focused_state: Cell::new(false),
            key_filter_token: Cell::new(0),
            selected_index_value: Cell::new(-1),
            highlighted_index_value: Cell::new(-1),
            popup_panel_color_value: Cell::new(0x00000000),
            popup_panel_background_blur_sigma_value: Cell::new(DEFAULT_PANEL_BACKGROUND_BLUR_SIGMA),
            popup_panel_color_overridden: Cell::new(false),
            popup_panel_background_blur_overridden: Cell::new(false),
            changed_callback: RefCell::new(None),
            theme_guard: RefCell::new(None),
            focus_visibility_guard: RefCell::new(None),
        });
        *shared.self_weak.borrow_mut() = Rc::downgrade(&shared);
        *shared_slot.borrow_mut() = Some(Rc::downgrade(&shared));
        root.retained_node_ref().retain_attachment(shared.clone());

        popup_list.popup_presenter.overlay_node().on_pointer_click({
            let weak_shared = Rc::downgrade(&shared);
            move |_event| {
                if let Some(shared) = weak_shared.upgrade() {
                    shared.close();
                }
            }
        });

        root.child(&field_presenter.root()).child(&popup_list.root);

        let control = Self { root, shared };
        control.install_visual_subscriptions();
        control.bind_events();
        control.shared.sync_value_label();
        control.shared.handle_theme_changed();
        control.root.persist_state(persisted_value_adapter(
            "dropdown-selected-index",
            PersistedInt32Codec,
            1,
            {
                let shared = control.shared.clone();
                move || {
                    let value = shared.selected_index_value.get();
                    if value >= 0 {
                        Some(value)
                    } else {
                        None
                    }
                }
            },
            {
                let shared = control.shared.clone();
                move |value| {
                    shared.set_selected_index(value, true);
                }
            },
        ));
        control
    }

    fn ensure_scroll_hook() {
        DROPDOWN_SCROLL_HOOK_REGISTERED.with(|registered| {
            if registered.get() {
                return;
            }
            registered.set(true);
            event::register_scroll_hook(|| {
                ACTIVE_DROPDOWN.with(|slot| {
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
            shared.handle_theme_changed();
        });

        let shared = self.shared.clone();
        self.root.on_pointer_up(move |event| {
            if !shared.is_enabled() || !shared.pointer_pressed_state.get() {
                return;
            }
            shared.pointer_pressed_state.set(false);
            if shared.open_state.get() {
                shared.close();
            } else {
                shared.open();
            }
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
            if !shared.open_state.get() && is_activation_key(event) {
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
            shared.focused_state.set(event.focused);
            if !event.focused && !shared.open_state.get() {
                shared.pointer_pressed_state.set(false);
            }
            shared.handle_theme_changed();
        });
    }

    pub fn selected_index(&self) -> i32 {
        self.shared.selected_index_value.get()
    }

    pub fn items<I>(&self, items: I) -> &Self
    where
        I: IntoIterator<Item = DropdownItem>,
    {
        self.shared.clear_items();
        self.shared.items_value.borrow_mut().extend(items);
        let count = self.shared.items_value.borrow().len() as i32;
        let selected = self.shared.selected_index_value.get();
        if selected >= count {
            self.shared
                .selected_index_value
                .set(if count > 0 { 0 } else { -1 });
        } else if selected < 0 && count > 0 {
            self.shared.selected_index_value.set(0);
        }
        self.shared.popup_list.refresh_panel_layout();
        self.shared.sync_value_label();
        self.shared.handle_theme_changed();
        self
    }

    pub fn on_changed(
        &self,
        callback: impl Fn(DropdownChangedEventArgs<DropdownItem>) + 'static,
    ) -> &Self {
        *self.shared.changed_callback.borrow_mut() = Some(Rc::new(callback));
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
                &format!("Dropdown.popupPanelBackgroundBlur() received {sigma}; clamping to 0.0."),
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
        self.shared.close();
        self.shared.sizing_value.set(sizing);
        if self.shared.uses_default_field_presenter() {
            self.shared.replace_field_presenter(
                create_field_presenter(self.shared.field_template_value.borrow().clone(), sizing),
                create_chevron_presenter(
                    self.shared.chevron_template_value.borrow().clone(),
                    sizing,
                ),
            );
        } else if self.shared.uses_default_chevron_presenter() {
            let previous_presenter = self.shared.chevron_presenter.borrow().clone();
            let next_presenter = create_chevron_presenter(
                self.shared.chevron_template_value.borrow().clone(),
                sizing,
            );
            *self.shared.chevron_presenter.borrow_mut() = next_presenter.clone();
            self.shared
                .field_presenter
                .borrow()
                .chevron_host()
                .remove_child(&previous_presenter.root());
            self.shared
                .field_presenter
                .borrow()
                .chevron_host()
                .child(&next_presenter.root());
            previous_presenter.root().dispose();
        }
        self.shared.popup_list.sizing(sizing);
        self.shared.sync_value_label();
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

    pub fn field_template(&self, template: Rc<dyn DropdownFieldTemplate>) -> &Self {
        self.set_field_template(Some(template))
    }

    pub fn clear_field_template(&self) -> &Self {
        self.set_field_template(None)
    }

    fn set_field_template(&self, template: Option<Rc<dyn DropdownFieldTemplate>>) -> &Self {
        self.shared.close();
        *self.shared.field_template_value.borrow_mut() = template.clone();
        let next_field_presenter = create_field_presenter(template, self.shared.sizing_value.get());
        let next_chevron_presenter = create_chevron_presenter(
            self.shared.chevron_template_value.borrow().clone(),
            self.shared.sizing_value.get(),
        );
        self.shared
            .replace_field_presenter(next_field_presenter, next_chevron_presenter);
        self.shared.sync_value_label();
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
        self.shared.close();
        *self.shared.chevron_template_value.borrow_mut() = template.clone();
        let previous_presenter = self.shared.chevron_presenter.borrow().clone();
        let next_presenter = create_chevron_presenter(template, self.shared.sizing_value.get());
        *self.shared.chevron_presenter.borrow_mut() = next_presenter.clone();
        self.shared
            .field_presenter
            .borrow()
            .chevron_host()
            .remove_child(&previous_presenter.root());
        self.shared
            .field_presenter
            .borrow()
            .chevron_host()
            .child(&next_presenter.root());
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
        *self.shared.option_row_template_value.borrow_mut() = template.clone();
        self.shared.popup_list.option_row_template(template);
        self
    }

    pub fn select_index(&self, index: i32) -> &Self {
        self.shared.set_selected_index(index, false);
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

impl Node for Dropdown {
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

impl HasFlexBoxRoot for Dropdown {
    fn flex_box_root(&self) -> &FlexBox {
        &self.root
    }
}

impl ThemeBindable for Dropdown {
    fn theme_binding_node(&self) -> NodeRef {
        self.root.retained_node_ref()
    }

    fn weak_theme_target(&self) -> Box<dyn Fn() -> Option<Self>> {
        let weak_root = self.root.downgrade();
        let weak_shared = Rc::downgrade(&self.shared);
        Box::new(move || {
            Some(Dropdown {
                root: weak_root.upgrade()?,
                shared: weak_shared.upgrade()?,
            })
        })
    }
}

impl DropdownShared {
    fn is_enabled(&self) -> bool {
        self.root
            .upgrade()
            .is_some_and(|root| root.retained_node_ref().is_enabled_for_routing())
    }

    fn popup_list_activate_index(&self, index: i32) {
        if index < 0 || index >= self.items_value.borrow().len() as i32 {
            return;
        }
        self.set_selected_index(index, true);
        self.close();
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
                self.highlight_index(self.items_value.borrow().len() as i32 - 1);
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

    fn set_selected_index(&self, index: i32, emit: bool) {
        if index == -1 {
            self.selected_index_value.set(-1);
            self.highlighted_index_value.set(-1);
            self.popup_list.set_highlighted_index(-1);
            self.sync_value_label();
            self.handle_theme_changed();
            return;
        }
        let count = self.items_value.borrow().len() as i32;
        if count == 0 {
            if index != -1 {
                logger::warn(
                    "Layout",
                    &format!(
                        "Dropdown.selectIndex() received {index} before any items were assigned."
                    ),
                );
            }
            return;
        }
        let clamped_index = index.clamp(0, count - 1);
        if clamped_index != index {
            logger::warn(
                "Layout",
                &format!("Dropdown.selectIndex() received {index}; clamping to {clamped_index}."),
            );
        }
        let changed = self.selected_index_value.get() != clamped_index;
        self.selected_index_value.set(clamped_index);
        self.highlighted_index_value.set(clamped_index);
        self.popup_list.set_highlighted_index(clamped_index);
        self.sync_value_label();
        self.handle_theme_changed();
        if emit && changed {
            if let Some(root) = self.root.upgrade() {
                root.request_semantic_announcement();
            }
            self.emit_selection_changed();
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
            callback(DropdownChangedEventArgs {
                item,
                selected_index,
            });
        }
    }

    fn open(&self) {
        if self.open_state.get() || self.items_value.borrow().is_empty() {
            return;
        }
        let Some(root) = self.root.upgrade() else {
            return;
        };
        if root.handle() == crate::node::NodeHandle::INVALID {
            return;
        }
        let initial_highlight = if self.selected_index_value.get() >= 0 {
            self.selected_index_value.get()
        } else if !self.items_value.borrow().is_empty() {
            0
        } else {
            -1
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
        ACTIVE_DROPDOWN.with(|slot| {
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
        self.popup_list.close();
        self.open_state.set(false);
        ACTIVE_DROPDOWN.with(|slot| {
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
        self.handle_theme_changed();
    }

    fn sync_value_label(&self) {
        let selected_index = self.selected_index_value.get();
        if let Some(item) = self.items_value.borrow().get(selected_index as usize) {
            self.field_presenter
                .borrow()
                .value_node()
                .text(item.label.clone());
            if let Some(root) = self.root.upgrade() {
                root.default_semantic_label(item.label.clone());
            }
            return;
        }
        self.field_presenter.borrow().value_node().text("");
        if let Some(root) = self.root.upgrade() {
            root.default_semantic_label("Dropdown");
        }
    }

    fn highlight_index(&self, index: i32) {
        self.popup_list.highlight_index(index);
        self.highlighted_index_value
            .set(self.popup_list.highlighted_index());
    }

    fn move_highlight(&self, delta: i32) {
        if self.popup_list.highlighted_index() < 0 && self.selected_index_value.get() >= 0 {
            self.popup_list
                .set_highlighted_index(self.selected_index_value.get());
        }
        self.popup_list.move_highlight(delta);
        self.highlighted_index_value
            .set(self.popup_list.highlighted_index());
    }

    fn select_highlighted(&self) {
        let highlighted = self.highlighted_index_value.get();
        if highlighted < 0 || highlighted >= self.items_value.borrow().len() as i32 {
            return;
        }
        self.set_selected_index(highlighted, true);
        self.close();
    }

    fn clear_items(&self) {
        self.close();
        self.items_value.borrow_mut().clear();
        self.selected_index_value.set(-1);
        self.highlighted_index_value.set(-1);
        self.popup_list.clear();
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
        root.cursor(if self.is_enabled() {
            CursorStyle::Pointer
        } else {
            CursorStyle::Default
        });
        root.corner_radius(0.0)
            .border(0.0, 0x00000000)
            .padding(0.0, 0.0, 0.0, 0.0)
            .bg_color(0x00000000)
            .opacity(if self.is_enabled() { 1.0 } else { 0.6 });

        self.field_presenter.borrow().root().fill_width();
        let selected_label = self
            .items_value
            .borrow()
            .get(self.selected_index_value.get() as usize)
            .map(|item| item.label.clone())
            .unwrap_or_default();
        self.field_presenter.borrow().apply(
            theme.clone(),
            &DropdownFieldVisualState::new(
                self.open_state.get(),
                self.focused_state.get(),
                self.is_enabled(),
                self.pointer_pressed_state.get(),
                selected_label,
            ),
            self.colors_value.get(),
        );
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
        self.popup_list.sync_option_visuals();
        self.sync_focus_chrome();
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

    fn replace_field_presenter(
        &self,
        next_field_presenter: Rc<dyn DropdownFieldPresenter>,
        next_chevron_presenter: Rc<dyn DropdownChevronPresenter>,
    ) {
        let Some(root) = self.root.upgrade() else {
            return;
        };
        let previous_field_root = self.field_presenter.borrow().root();
        *self.field_presenter.borrow_mut() = next_field_presenter.clone();
        *self.chevron_presenter.borrow_mut() = next_chevron_presenter.clone();
        next_field_presenter.root().fill_width();
        next_field_presenter
            .chevron_host()
            .child(&next_chevron_presenter.root());
        root.remove_child(&previous_field_root);
        root.child(&next_field_presenter.root());
        root.child(&self.popup_list.root);
        previous_field_root.dispose();
    }

    fn uses_default_field_presenter(&self) -> bool {
        if self.field_template_value.borrow().is_some() {
            return false;
        }
        let template_set = get_control_templates();
        template_set.is_none() || template_set.is_some_and(|set| set.dropdown_field.is_none())
    }

    fn uses_default_chevron_presenter(&self) -> bool {
        if self.chevron_template_value.borrow().is_some() {
            return false;
        }
        let template_set = get_control_templates();
        template_set.is_none() || template_set.is_some_and(|set| set.dropdown_chevron.is_none())
    }
}
