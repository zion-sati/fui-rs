use crate::bindings::ui;
use crate::controls::{run_context_menu_action, ContextMenuAction, MenuItem};
use crate::ffi::{
    AlignItems, FlexDirection, JustifyContent, PositionType, SemanticRole, TextOverflow, Unit,
    Visibility,
};
use crate::node::FlexBoxSurface;
use crate::node::{
    flex_box, portal, text, FlexBox, Node, NodeHandle, NodeRef, ScrollBox, TextNode,
};
use crate::theme::current_theme;
use std::cell::RefCell;

const TOOLBAR_MARGIN: f32 = 8.0;
const EDGE_MARGIN: f32 = 8.0;
const READONLY_BUTTON_WIDTH: f32 = 96.0;
const EDITABLE_BUTTON_WIDTH: f32 = 72.0;
const OVERFLOW_BUTTON_WIDTH: f32 = 44.0;
const VERTICAL_MENU_WIDTH: f32 = 168.0;
const VERTICAL_MENU_MAX_HEIGHT: f32 = 184.0;
const TOOLBAR_HORIZONTAL_PADDING: f32 = 4.0;
const MAX_HORIZONTAL_ACTIONS: usize = 3;
const OVERFLOW_SLOT: i32 = -1;
const BACK_SLOT: i32 = -2;
const OVERFLOW_LABEL: &str = "More";

#[derive(Clone)]
struct ToolbarButton {
    root: FlexBox,
    label_node: Option<TextNode>,
    dot_nodes: Vec<FlexBox>,
}

struct State {
    host_root: Option<FlexBox>,
    panel: Option<FlexBox>,
    overflow_panel: Option<FlexBox>,
    overflow_scroll_box: Option<ScrollBox>,
    overflow_content: Option<FlexBox>,
    buttons: Vec<ToolbarButton>,
    overflow_buttons: Vec<ToolbarButton>,
    separators: Vec<FlexBox>,
    overflow_separators: Vec<FlexBox>,
    active_items: Vec<MenuItem>,
    active_handle: NodeHandle,
    active_start: u32,
    active_end: u32,
    active_cross_selection_text: String,
    active_cross_selection_select_all_target: NodeHandle,
    pending_cross_selection_text_handle: NodeHandle,
    hidden_for_handle_drag: bool,
    overflow_visible: bool,
    horizontal_item_count: usize,
    last_panel_x: f32,
    last_panel_y: f32,
    last_panel_width: f32,
}

impl State {
    fn new() -> Self {
        Self {
            host_root: None,
            panel: None,
            overflow_panel: None,
            overflow_scroll_box: None,
            overflow_content: None,
            buttons: Vec::new(),
            overflow_buttons: Vec::new(),
            separators: Vec::new(),
            overflow_separators: Vec::new(),
            active_items: Vec::new(),
            active_handle: NodeHandle::INVALID,
            active_start: 0,
            active_end: 0,
            active_cross_selection_text: String::new(),
            active_cross_selection_select_all_target: NodeHandle::INVALID,
            pending_cross_selection_text_handle: NodeHandle::INVALID,
            hidden_for_handle_drag: false,
            overflow_visible: false,
            horizontal_item_count: 0,
            last_panel_x: EDGE_MARGIN,
            last_panel_y: EDGE_MARGIN,
            last_panel_width: 0.0,
        }
    }
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::new());
}

impl ToolbarButton {
    fn new(label: &str, width: f32, slot: i32) -> Self {
        let child = if slot == OVERFLOW_SLOT {
            ToolbarButtonChild::Overflow(create_overflow_icon())
        } else {
            ToolbarButtonChild::Label(create_label(label, slot))
        };
        let (child_node, label_node, dot_nodes) = match child {
            ToolbarButtonChild::Label(node) => (node.node_ref(), Some(node), Vec::new()),
            ToolbarButtonChild::Overflow((icon, dots)) => (icon.node_ref(), None, dots),
        };
        let theme = current_theme();
        let root = flex_box();
        root.width(width, Unit::Pixel)
            .height(theme.context_menu.item.height, Unit::Pixel)
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .padding(
                theme.context_menu.item.padding_left,
                theme.context_menu.item.padding_top,
                theme.context_menu.item.padding_right,
                theme.context_menu.item.padding_bottom,
            )
            .corner_radius(theme.context_menu.item.corner_radius)
            .semantic_role(SemanticRole::Button)
            .semantic_label(label)
            .interactive(true)
            .preserve_selection_on_pointer_down(true)
            .on_pointer_up(move |event| {
                activate_toolbar_slot(slot);
                event.handled = true;
            })
            .on_click(move |_| activate_toolbar_slot(slot));
        append_child_ref(&root, &child_node);
        Self {
            root,
            label_node,
            dot_nodes,
        }
    }

    fn set_label(&self, label: &str) {
        if let Some(label_node) = self.label_node.as_ref() {
            label_node.text(label);
        }
        self.root.semantic_label(label.to_owned());
    }

    fn apply_style(&self) {
        let theme = current_theme();
        self.root
            .height(theme.context_menu.item.height, Unit::Pixel)
            .padding(
                theme.context_menu.item.padding_left,
                theme.context_menu.item.padding_top,
                theme.context_menu.item.padding_right,
                theme.context_menu.item.padding_bottom,
            )
            .corner_radius(theme.context_menu.item.corner_radius)
            .bg_color(theme.context_menu.item.background);
        if let Some(label_node) = self.label_node.as_ref() {
            label_node
                .font_family(theme.context_menu.item.font_family.clone())
                .font_size(theme.context_menu.item.font_size)
                .text_color(theme.context_menu.item.text_color);
        }
        for dot in &self.dot_nodes {
            dot.bg_color(theme.context_menu.item.text_color);
        }
    }
}

enum ToolbarButtonChild {
    Label(TextNode),
    Overflow((FlexBox, Vec<FlexBox>)),
}

fn create_label(label: &str, slot: i32) -> TextNode {
    let theme = current_theme();
    let label_node = text(label);
    label_node
        .font_family(theme.context_menu.item.font_family.clone())
        .font_size(theme.context_menu.item.font_size)
        .text_color(theme.context_menu.item.text_color)
        .text_overflow(TextOverflow::Ellipsis)
        .selectable(false)
        .interactive(true)
        .preserve_selection_on_pointer_down(true)
        .on_pointer_up(move |event| {
            activate_toolbar_slot(slot);
            event.handled = true;
        });
    label_node
}

fn create_overflow_icon() -> (FlexBox, Vec<FlexBox>) {
    let theme = current_theme();
    let mut dots = Vec::new();
    let icon = flex_box();
    icon.width(16.0, Unit::Pixel)
        .height(theme.context_menu.item.height, Unit::Pixel)
        .flex_direction(FlexDirection::Column)
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::Center)
        .preserve_selection_on_pointer_down(true)
        .on_pointer_up(|event| {
            activate_toolbar_slot(OVERFLOW_SLOT);
            event.handled = true;
        });
    for _ in 0..3 {
        let dot = flex_box();
        dot.width(3.0, Unit::Pixel)
            .height(3.0, Unit::Pixel)
            .margin(0.0, 1.25, 0.0, 1.25)
            .corner_radius(2.0)
            .bg_color(theme.context_menu.item.text_color);
        icon.child(&dot);
        dots.push(dot);
    }
    (icon, dots)
}

fn make_separator(vertical: bool) -> FlexBox {
    let theme = current_theme();
    let separator = flex_box();
    if vertical {
        separator
            .width(100.0, Unit::Percent)
            .height(1.0, Unit::Pixel)
            .bg_color(theme.context_menu.separator_color);
    } else {
        separator
            .width(1.0, Unit::Pixel)
            .height(theme.context_menu.item.height - 10.0, Unit::Pixel)
            .bg_color(theme.context_menu.separator_color);
    }
    separator
}

pub(crate) fn create_default_host() -> FlexBox {
    STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        if let Some(host_root) = state.host_root.as_ref() {
            return host_root.clone();
        }
        let theme = current_theme();
        let panel = flex_box();
        panel
            .position_type(PositionType::Absolute)
            .height(theme.context_menu.item.height + 8.0, Unit::Pixel)
            .flex_direction(FlexDirection::Row)
            .align_items(AlignItems::Center)
            .padding(
                TOOLBAR_HORIZONTAL_PADDING,
                4.0,
                TOOLBAR_HORIZONTAL_PADDING,
                4.0,
            )
            .corner_radius(theme.context_menu.panel_corner_radius)
            .border(1.0, theme.context_menu.panel_border_color)
            .bg_color(theme.context_menu.panel_background)
            .background_blur(10.0)
            .drop_shadow(
                theme.context_menu.panel_shadow_color,
                0.0,
                theme.context_menu.shadow_offset_y,
                theme.context_menu.shadow_blur,
                theme.context_menu.shadow_spread,
            )
            .preserve_selection_on_pointer_down(true)
            .visibility(Visibility::Collapsed);

        let overflow_content = flex_box();
        overflow_content
            .flex_direction(FlexDirection::Column)
            .width(100.0, Unit::Percent)
            .bg_color(0x00000000)
            .preserve_selection_on_pointer_down(true);
        let overflow_scroll_box = ScrollBox::new();
        overflow_scroll_box
            .scroll_enabled_x(false)
            .scroll_enabled_y(true)
            .vertical_scrollbar_visibility(crate::node::ScrollBarVisibility::Auto)
            .horizontal_scrollbar_visibility(crate::node::ScrollBarVisibility::Never)
            .scrollbar_gutter(2.0)
            .width(100.0, Unit::Percent)
            .height(VERTICAL_MENU_MAX_HEIGHT, Unit::Pixel)
            .child(&overflow_content)
            .preserve_selection_on_pointer_down(true);
        let overflow_panel = flex_box();
        overflow_panel
            .position_type(PositionType::Absolute)
            .width(VERTICAL_MENU_WIDTH, Unit::Pixel)
            .height(VERTICAL_MENU_MAX_HEIGHT, Unit::Pixel)
            .corner_radius(theme.context_menu.panel_corner_radius)
            .border(1.0, theme.context_menu.panel_border_color)
            .bg_color(theme.context_menu.panel_background)
            .background_blur(10.0)
            .drop_shadow(
                theme.context_menu.panel_shadow_color,
                0.0,
                theme.context_menu.shadow_offset_y,
                theme.context_menu.shadow_blur,
                theme.context_menu.shadow_spread,
            )
            .preserve_selection_on_pointer_down(true)
            .child(&overflow_scroll_box)
            .visibility(Visibility::Collapsed);

        state.buttons = vec![
            ToolbarButton::new("Copy", READONLY_BUTTON_WIDTH, 0),
            ToolbarButton::new("Select all", READONLY_BUTTON_WIDTH, 1),
            ToolbarButton::new("Paste", EDITABLE_BUTTON_WIDTH, 2),
            ToolbarButton::new(OVERFLOW_LABEL, OVERFLOW_BUTTON_WIDTH, OVERFLOW_SLOT),
        ];
        state.overflow_buttons = vec![
            ToolbarButton::new("Select all", VERTICAL_MENU_WIDTH, 3),
            ToolbarButton::new("Extra", VERTICAL_MENU_WIDTH, 4),
            ToolbarButton::new("Extra", VERTICAL_MENU_WIDTH, 5),
            ToolbarButton::new("<", VERTICAL_MENU_WIDTH, BACK_SLOT),
        ];
        state.separators = vec![
            make_separator(false),
            make_separator(false),
            make_separator(false),
        ];
        state.overflow_separators = vec![
            make_separator(true),
            make_separator(true),
            make_separator(true),
            make_separator(true),
        ];

        let host_root = portal();
        host_root
            .position_type(PositionType::Absolute)
            .position(0.0, 0.0)
            .width(100.0, Unit::Percent)
            .height(100.0, Unit::Percent)
            .child(&panel)
            .child(&overflow_panel);
        state.host_root = Some(host_root.clone());
        state.panel = Some(panel);
        state.overflow_panel = Some(overflow_panel);
        state.overflow_scroll_box = Some(overflow_scroll_box);
        state.overflow_content = Some(overflow_content);
        host_root
    })
}

pub(crate) fn reset() {
    STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        if let Some(host_root) = state.host_root.take() {
            host_root.dispose();
        }
        *state = State::new();
    });
}

pub(crate) fn clear() {
    STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        state.active_handle = NodeHandle::INVALID;
        state.active_start = 0;
        state.active_end = 0;
        state.active_cross_selection_text.clear();
        state.active_cross_selection_select_all_target = NodeHandle::INVALID;
        state.active_items.clear();
        state.pending_cross_selection_text_handle = NodeHandle::INVALID;
        state.hidden_for_handle_drag = false;
        state.overflow_visible = false;
        state.horizontal_item_count = 0;
    });
    hide();
}

pub(crate) fn set_pending_cross_selection_text_handle(handle: NodeHandle) {
    STATE.with(|slot| slot.borrow_mut().pending_cross_selection_text_handle = handle);
}

pub(crate) fn handle_selection_changed(
    handle: NodeHandle,
    target: &NodeRef,
    start: u32,
    end: u32,
    selection_chrome_visible: bool,
) {
    STATE.with(|slot| slot.borrow_mut().active_cross_selection_text.clear());
    let hidden_for_drag = STATE.with(|slot| slot.borrow().hidden_for_handle_drag);
    let active_handle = STATE.with(|slot| slot.borrow().active_handle);
    if start == end && hidden_for_drag && active_handle == handle {
        STATE.with(|slot| {
            let mut state = slot.borrow_mut();
            state.active_start = start;
            state.active_end = end;
        });
        hide();
        return;
    }
    if !selection_chrome_visible
        || start == end
        || !(target.is_selectable_text_for_routing() || target.is_editable_text_for_routing())
    {
        clear();
        return;
    }
    let text_content = target.text_content_for_routing().unwrap_or_default();
    let selected_text = resolve_selected_text(&text_content, start, end);
    let active_handle = handle;
    STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        state.active_handle = active_handle;
        state.active_start = start;
        state.active_end = end;
        state.active_cross_selection_select_all_target = active_handle;
    });
    create_default_host();
    build_items_for_text(
        active_handle,
        target.is_editable_text_for_routing(),
        &text_content,
        start,
        end,
        &selected_text,
    );
    position_for_text_range(active_handle, start, end);
}

pub(crate) fn handle_cross_selection_changed(
    handle: NodeHandle,
    _area: &NodeRef,
    text: &str,
    selection_chrome_visible: bool,
) {
    let hidden_for_drag = STATE.with(|slot| slot.borrow().hidden_for_handle_drag);
    let active_handle = STATE.with(|slot| slot.borrow().active_handle);
    if !selection_chrome_visible || text.is_empty() {
        if text.is_empty() && hidden_for_drag && active_handle == handle {
            STATE.with(|slot| {
                let mut state = slot.borrow_mut();
                state.active_start = 0;
                state.active_end = 0;
                state.active_cross_selection_text.clear();
            });
            hide();
            return;
        }
        clear();
        return;
    }
    let (previous_handle, previous_text, previous_target, pending_handle) = STATE.with(|slot| {
        let state = slot.borrow();
        (
            state.active_handle,
            state.active_cross_selection_text.clone(),
            state.active_cross_selection_select_all_target,
            state.pending_cross_selection_text_handle,
        )
    });
    let mut select_all_target = pending_handle;
    if select_all_target == NodeHandle::INVALID
        && previous_handle == handle
        && previous_text == text
    {
        select_all_target = previous_target;
    }
    if select_all_target == NodeHandle::INVALID {
        select_all_target = handle;
    }
    STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        state.active_handle = handle;
        state.active_start = 0;
        state.active_end = text.chars().count() as u32;
        state.active_cross_selection_text = text.to_owned();
        state.active_cross_selection_select_all_target = select_all_target;
        state.pending_cross_selection_text_handle = NodeHandle::INVALID;
        state.active_items.clear();
        state.active_items.push(
            MenuItem::new("Copy", ContextMenuAction::CopyCurrentSelection).payload(text.to_owned()),
        );
        state.active_items.push(
            MenuItem::new("Select all", ContextMenuAction::SelectAllText)
                .target_handle(select_all_target.raw()),
        );
    });
    apply_items(false);
    position_for_cross_selection(handle);
}

pub(crate) fn refresh_active_geometry(selection_chrome_visible: bool) {
    let (active_handle, hidden_for_handle_drag, has_cross_text, active_start, active_end) = STATE
        .with(|slot| {
            let state = slot.borrow();
            (
                state.active_handle,
                state.hidden_for_handle_drag,
                !state.active_cross_selection_text.is_empty(),
                state.active_start,
                state.active_end,
            )
        });
    if !selection_chrome_visible || active_handle == NodeHandle::INVALID || hidden_for_handle_drag {
        hide();
        return;
    }
    if has_cross_text {
        position_for_cross_selection(active_handle);
    } else {
        position_for_text_range(active_handle, active_start, active_end);
    }
}

pub(crate) fn hide_for_handle_drag() {
    STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        state.hidden_for_handle_drag = true;
        state.overflow_visible = false;
    });
    hide();
}

pub(crate) fn show_after_handle_drag(selection_chrome_visible: bool) {
    STATE.with(|slot| slot.borrow_mut().hidden_for_handle_drag = false);
    refresh_active_geometry(selection_chrome_visible);
}

pub(crate) fn dismiss_for_outside_pointer_down(scene_x: f32, scene_y: f32) -> bool {
    if STATE.with(|slot| slot.borrow().active_items.is_empty()) {
        return false;
    }
    let (panel, overflow_panel) = STATE.with(|slot| {
        let state = slot.borrow();
        (state.panel.clone(), state.overflow_panel.clone())
    });
    if panel
        .as_ref()
        .is_some_and(|panel| point_hits_node(panel, scene_x, scene_y))
        || overflow_panel
            .as_ref()
            .is_some_and(|panel| point_hits_node(panel, scene_x, scene_y))
        || ui::is_point_in_selection(scene_x, scene_y)
    {
        return false;
    }
    hide();
    true
}

fn build_items_for_text(
    handle: NodeHandle,
    editable: bool,
    content: &str,
    start: u32,
    end: u32,
    selected_text: &str,
) {
    let selected_payload = (!selected_text.is_empty()).then(|| selected_text.to_owned());
    let has_text = !content.is_empty();
    STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        state.active_items.clear();
        if editable {
            let mut cut = MenuItem::new("Cut", ContextMenuAction::CutTextSelection)
                .target_handle(handle.raw())
                .focus_target_after_action(true)
                .with_selection_range(start, end);
            if let Some(payload) = selected_payload.clone() {
                cut = cut.payload(payload);
            }
            state.active_items.push(cut);

            let mut copy = MenuItem::new("Copy", ContextMenuAction::CopyCurrentSelection)
                .target_handle(handle.raw())
                .focus_target_after_action(true);
            if let Some(payload) = selected_payload.clone() {
                copy = copy.payload(payload);
            }
            state.active_items.push(copy);

            state.active_items.push(
                MenuItem::new("Paste", ContextMenuAction::PasteText)
                    .target_handle(handle.raw())
                    .focus_target_after_action(true),
            );
            state.active_items.push(
                MenuItem::new("Select all", ContextMenuAction::SelectAllText)
                    .disabled(!has_text)
                    .target_handle(handle.raw())
                    .focus_target_after_action(true),
            );
        } else {
            let mut copy = MenuItem::new("Copy", ContextMenuAction::CopyCurrentSelection)
                .target_handle(handle.raw());
            if let Some(payload) = selected_payload {
                copy = copy.payload(payload);
            }
            state.active_items.push(copy);
            state.active_items.push(
                MenuItem::new("Select all", ContextMenuAction::SelectAllText)
                    .disabled(!has_text)
                    .target_handle(handle.raw()),
            );
        }
    });
    apply_items(editable);
}

fn activate_toolbar_slot(slot: i32) {
    if slot == OVERFLOW_SLOT {
        show_overflow_menu();
        return;
    }
    if slot == BACK_SLOT {
        show_horizontal_menu();
        return;
    }
    let item = STATE.with(|slot_state| {
        let state = slot_state.borrow();
        if slot < 0 || slot as usize >= state.active_items.len() {
            None
        } else {
            Some(state.active_items[slot as usize].clone())
        }
    });
    let Some(item) = item else {
        return;
    };
    run_context_menu_action(&item);
    if item.action == ContextMenuAction::CopyCurrentSelection {
        ui::clear_current_selection();
        clear();
        return;
    }
    if item.action == ContextMenuAction::SelectAllText {
        return;
    }
    hide();
}

fn apply_items(editable: bool) {
    create_default_host();
    let theme = current_theme();
    let button_width = if editable {
        EDITABLE_BUTTON_WIDTH
    } else {
        READONLY_BUTTON_WIDTH
    };
    let (item_count, has_overflow) = STATE.with(|slot| {
        let state = slot.borrow();
        (
            state.active_items.len(),
            state.active_items.len() > MAX_HORIZONTAL_ACTIONS,
        )
    });
    let horizontal_item_count = if has_overflow {
        MAX_HORIZONTAL_ACTIONS + 1
    } else {
        item_count
    };
    STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        state.horizontal_item_count = horizontal_item_count;
        if let Some(panel) = state.panel.as_ref() {
            panel
                .height(theme.context_menu.item.height + 8.0, Unit::Pixel)
                .width(
                    active_width_for_button_width(button_width, item_count, has_overflow),
                    Unit::Pixel,
                )
                .bg_color(theme.context_menu.panel_background)
                .background_blur(10.0)
                .corner_radius(theme.context_menu.panel_corner_radius)
                .border(1.0, theme.context_menu.panel_border_color)
                .drop_shadow(
                    theme.context_menu.panel_shadow_color,
                    0.0,
                    theme.context_menu.shadow_offset_y,
                    theme.context_menu.shadow_blur,
                    theme.context_menu.shadow_spread,
                );
            clear_children(panel);
            let visible_actions = if has_overflow {
                MAX_HORIZONTAL_ACTIONS
            } else {
                item_count
            };
            for index in 0..visible_actions {
                if index > 0 {
                    let separator = &state.separators[index - 1];
                    separator
                        .height(theme.context_menu.item.height - 10.0, Unit::Pixel)
                        .bg_color(theme.context_menu.separator_color);
                    panel.child(separator);
                }
                let label = state.active_items[index].label.clone();
                let button = &state.buttons[index];
                button.root.width(button_width, Unit::Pixel);
                button.apply_style();
                button.set_label(&label);
                panel.child(&button.root);
            }
            if has_overflow {
                let separator = &state.separators[MAX_HORIZONTAL_ACTIONS - 1];
                separator
                    .height(theme.context_menu.item.height - 10.0, Unit::Pixel)
                    .bg_color(theme.context_menu.separator_color);
                panel.child(separator);
                let overflow_button = &state.buttons[MAX_HORIZONTAL_ACTIONS];
                overflow_button
                    .root
                    .width(OVERFLOW_BUTTON_WIDTH, Unit::Pixel);
                overflow_button.apply_style();
                overflow_button.set_label(OVERFLOW_LABEL);
                panel.child(&overflow_button.root);
            }
        }
    });
    apply_overflow_items(button_width);
    show_horizontal_menu();
}

fn apply_overflow_items(_button_width: f32) {
    let theme = current_theme();
    STATE.with(|slot| {
        let state = slot.borrow();
        let overflow_count = state
            .active_items
            .len()
            .saturating_sub(MAX_HORIZONTAL_ACTIONS);
        let total_rows = overflow_count + 1;
        let content_height = (theme.context_menu.item.height * total_rows as f32)
            + (total_rows.saturating_sub(1) as f32);
        let max_panel_height = theme
            .context_menu
            .item
            .height
            .max(VERTICAL_MENU_MAX_HEIGHT.min(ui::get_viewport_height() - (EDGE_MARGIN * 2.0)));
        let panel_height = content_height.min(max_panel_height);
        if let Some(overflow_panel) = state.overflow_panel.as_ref() {
            overflow_panel
                .width(VERTICAL_MENU_WIDTH, Unit::Pixel)
                .height(panel_height, Unit::Pixel)
                .bg_color(theme.context_menu.panel_background)
                .background_blur(10.0)
                .corner_radius(theme.context_menu.panel_corner_radius)
                .border(1.0, theme.context_menu.panel_border_color)
                .drop_shadow(
                    theme.context_menu.panel_shadow_color,
                    0.0,
                    theme.context_menu.shadow_offset_y,
                    theme.context_menu.shadow_blur,
                    theme.context_menu.shadow_spread,
                );
        }
        if let Some(scroll_box) = state.overflow_scroll_box.as_ref() {
            scroll_box
                .width(100.0, Unit::Percent)
                .height(panel_height, Unit::Pixel)
                .scroll_content_size(-1.0, content_height);
        }
        if let Some(content) = state.overflow_content.as_ref() {
            clear_children(content);
            for index in 0..overflow_count {
                if index > 0 {
                    let separator = &state.overflow_separators[index - 1];
                    separator
                        .width(100.0, Unit::Percent)
                        .height(1.0, Unit::Pixel)
                        .bg_color(theme.context_menu.separator_color);
                    content.child(separator);
                }
                let item_index = MAX_HORIZONTAL_ACTIONS + index;
                let button = &state.overflow_buttons[index];
                button.root.width(VERTICAL_MENU_WIDTH, Unit::Pixel);
                button.apply_style();
                button.set_label(&state.active_items[item_index].label);
                content.child(&button.root);
            }
            if overflow_count > 0 {
                let separator = &state.overflow_separators[overflow_count - 1];
                separator
                    .width(100.0, Unit::Percent)
                    .height(1.0, Unit::Pixel)
                    .bg_color(theme.context_menu.separator_color);
                content.child(separator);
            }
            let back_button = state.overflow_buttons.last().expect("back button");
            back_button.root.width(VERTICAL_MENU_WIDTH, Unit::Pixel);
            back_button.apply_style();
            back_button.set_label("<");
            content.child(&back_button.root);
            content.width(100.0, Unit::Percent);
        }
    });
}

fn show_overflow_menu() {
    let (last_x, last_y, last_width, can_show) = STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        let can_show = state.panel.is_some()
            && state.overflow_panel.is_some()
            && state.active_items.len() > MAX_HORIZONTAL_ACTIONS;
        if can_show {
            state.overflow_visible = true;
        }
        (
            state.last_panel_x,
            state.last_panel_y,
            state.last_panel_width,
            can_show,
        )
    });
    if !can_show {
        return;
    }
    STATE.with(|slot| {
        let state = slot.borrow();
        if let Some(panel) = state.panel.as_ref() {
            panel.visibility(Visibility::Collapsed);
        }
    });
    position_overflow_panel(
        last_x,
        last_y,
        last_width,
        current_theme().context_menu.item.height + 8.0,
    );
    STATE.with(|slot| {
        if let Some(overflow_panel) = slot.borrow().overflow_panel.as_ref() {
            overflow_panel.visibility(Visibility::Normal);
        }
    });
}

fn show_horizontal_menu() {
    STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        state.overflow_visible = false;
        if let Some(overflow_panel) = state.overflow_panel.as_ref() {
            overflow_panel.visibility(Visibility::Collapsed);
        }
        if let Some(panel) = state.panel.as_ref() {
            if !state.active_items.is_empty() && !state.hidden_for_handle_drag {
                panel.visibility(Visibility::Normal);
            }
        }
    });
}

fn normalize_start(start: u32, end: u32) -> u32 {
    start.min(end)
}

fn normalize_end(start: u32, end: u32) -> u32 {
    start.max(end)
}

fn resolve_selected_text(content: &str, start: u32, end: u32) -> String {
    if start == end {
        return String::new();
    }
    content
        .chars()
        .skip(normalize_start(start, end) as usize)
        .take((normalize_end(start, end) - normalize_start(start, end)) as usize)
        .collect()
}

fn position_for_text_range(handle: NodeHandle, start: u32, end: u32) {
    if start == end {
        hide();
        return;
    }
    let rects = ui::get_text_range_rects(
        handle.raw(),
        normalize_start(start, end),
        normalize_end(start, end),
    );
    if rects.is_empty() {
        hide();
        return;
    }
    let first = rects.first().copied().unwrap();
    let last = rects.last().copied().unwrap();
    position_at_selection_bounds(
        first.x,
        first.y,
        first.height,
        last.x + last.width,
        last.y + last.height,
    );
}

fn position_for_cross_selection(handle: NodeHandle) {
    let Some(rects) = ui::get_cross_selection_endpoint_rects(handle.raw()) else {
        hide();
        return;
    };
    position_at_selection_bounds(
        rects.start.x,
        rects.start.y,
        rects.start.height,
        rects.end.x + rects.end.width,
        rects.end.y + rects.end.height,
    );
}

fn position_at_selection_bounds(
    start_x: f32,
    top_y: f32,
    start_height: f32,
    _end_x: f32,
    bottom_y: f32,
) {
    if STATE.with(|slot| slot.borrow().hidden_for_handle_drag) {
        return;
    }
    let (active_len, overflow_visible) = STATE.with(|slot| {
        let state = slot.borrow();
        (state.active_items.len(), state.overflow_visible)
    });
    let has_overflow = active_len > MAX_HORIZONTAL_ACTIONS;
    let button_width = if active_len > 2 {
        EDITABLE_BUTTON_WIDTH
    } else {
        READONLY_BUTTON_WIDTH
    };
    let width = active_width_for_button_width(button_width, active_len, has_overflow);
    let height = current_theme().context_menu.item.height + 8.0;
    let viewport_width = ui::get_viewport_width();
    let viewport_height = ui::get_viewport_height();
    let max_x = (viewport_width - width - EDGE_MARGIN).max(EDGE_MARGIN);
    let x = (start_x - (width * 0.5)).clamp(EDGE_MARGIN, max_x);
    let top_candidate = top_y - height - TOOLBAR_MARGIN;
    let bottom_candidate = bottom_y + TOOLBAR_MARGIN + 12.0;
    let mut y = top_candidate;
    if top_candidate < EDGE_MARGIN {
        y = bottom_candidate;
        if bottom_candidate + height > viewport_height - EDGE_MARGIN {
            y = top_y + (start_height * 0.5) - (height * 0.5);
        }
    }
    let max_y = (viewport_height - height - EDGE_MARGIN).max(EDGE_MARGIN);
    y = y.clamp(EDGE_MARGIN, max_y);
    STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        state.last_panel_x = x;
        state.last_panel_y = y;
        state.last_panel_width = width;
        if let Some(panel) = state.panel.as_ref() {
            panel.position(x, y);
            if !overflow_visible {
                panel.visibility(Visibility::Normal);
            }
        }
    });
    if overflow_visible {
        position_overflow_panel(x, y, width, height);
    }
}

fn position_overflow_panel(x: f32, y: f32, horizontal_width: f32, horizontal_height: f32) {
    let viewport_width = ui::get_viewport_width();
    let viewport_height = ui::get_viewport_height();
    let panel_height = VERTICAL_MENU_MAX_HEIGHT.min(viewport_height - (EDGE_MARGIN * 2.0));
    let max_x = (viewport_width - VERTICAL_MENU_WIDTH - EDGE_MARGIN).max(EDGE_MARGIN);
    let overflow_x = (x + horizontal_width - VERTICAL_MENU_WIDTH).clamp(EDGE_MARGIN, max_x);
    let mut overflow_y = y + horizontal_height + 4.0;
    if overflow_y + panel_height > viewport_height - EDGE_MARGIN {
        overflow_y = y - panel_height - 4.0;
    }
    let max_y = (viewport_height - panel_height - EDGE_MARGIN).max(EDGE_MARGIN);
    overflow_y = overflow_y.clamp(EDGE_MARGIN, max_y);
    STATE.with(|slot| {
        if let Some(overflow_panel) = slot.borrow().overflow_panel.as_ref() {
            overflow_panel.position(overflow_x, overflow_y);
        }
    });
}

fn point_hits_node(node: &FlexBox, scene_x: f32, scene_y: f32) -> bool {
    if node.handle() == NodeHandle::INVALID || !is_visible_node(node) {
        return false;
    }
    let Some(bounds) = ui::get_bounds(node.handle().raw()) else {
        return false;
    };
    scene_x >= bounds[0]
        && scene_x <= (bounds[0] + bounds[2])
        && scene_y >= bounds[1]
        && scene_y <= (bounds[1] + bounds[3])
}

fn active_width_for_button_width(button_width: f32, item_count: usize, has_overflow: bool) -> f32 {
    let visible_actions = if has_overflow {
        MAX_HORIZONTAL_ACTIONS
    } else {
        item_count
    };
    let visible_count = if has_overflow {
        visible_actions + 1
    } else {
        visible_actions
    };
    let action_width = button_width * visible_actions as f32;
    let overflow_width = if has_overflow {
        OVERFLOW_BUTTON_WIDTH
    } else {
        0.0
    };
    action_width
        + overflow_width
        + (visible_count.saturating_sub(1) as f32)
        + (TOOLBAR_HORIZONTAL_PADDING * 2.0)
}

fn append_child_ref(parent: &FlexBox, child: &NodeRef) {
    parent.retained_node_ref().append_child_ref(child);
}

fn clear_children(parent: &FlexBox) {
    for child in parent.retained_node_ref().children() {
        child.detach_from_parent();
    }
}

fn is_visible_node(node: &FlexBox) -> bool {
    node.retained_node_ref().is_visible_for_routing()
}

fn hide() {
    STATE.with(|slot| {
        let state = slot.borrow();
        if let Some(panel) = state.panel.as_ref() {
            panel.visibility(Visibility::Collapsed);
        }
        if let Some(overflow_panel) = state.overflow_panel.as_ref() {
            overflow_panel.visibility(Visibility::Collapsed);
        }
    });
}
