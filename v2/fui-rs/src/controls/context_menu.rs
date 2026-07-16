use super::ContextMenuAppearance;
use crate::bindings::ui;
use crate::event::{self, PointerEventArgs, PointerType};
use crate::ffi::{
    BorderStyle, CursorStyle, HandleValue, KeyEventType, SemanticRole, TextAlign, TextOverflow,
    Unit,
};
use crate::logger::warn;
use crate::navigation;
use crate::node::{
    column, flex_box, grid, portal, Border, FlexBox, Grid, GridTrack, Node, NodeRef, TextNode,
    WeakFlexBox,
};
use crate::popup_presenter::{PopupPresenter, PopupPresenterEventTarget};
use crate::theme::{current_theme, subscribe, Theme};
use crate::{FontFamily, FontStyle, FontWeight};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

const MENU_WIDTH: f32 = 220.0;
const MENU_SEPARATOR_HEIGHT: f32 = 9.0;
const MENU_EDGE_PADDING: f32 = 8.0;
const MAX_ITEMS: usize = 25;
const DEFAULT_PANEL_BACKGROUND_BLUR_SIGMA: f32 = 10.0;
const SHORTCUT_SHARED_SIZE_GROUP: &str = "ContextMenuShortcutColumn";

thread_local! {
    static ACTIVE_CONTEXT_MENU: RefCell<Option<ContextMenuEventTarget>> = const { RefCell::new(None) };
}

fn is_primary_activation_pointer(event: &PointerEventArgs) -> bool {
    event.button == 0
        || event.pointer_type == PointerType::Touch
        || event.pointer_type == PointerType::Pen
}

fn write_payload_to_clipboard(text: &str) {
    let bytes = text.as_bytes();
    unsafe {
        crate::ffi::fui_copy_text(
            if bytes.is_empty() {
                0
            } else {
                bytes.as_ptr() as usize
            },
            bytes.len() as u32,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MenuItemKind {
    Action,
    Separator,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContextMenuAction {
    CopyCurrentSelection = 0,
    ReloadPage = 1,
    OpenLink = 2,
    OpenLinkInNewTab = 3,
    NavigateBack = 4,
    NavigateForward = 5,
    UndoTextEdit = 6,
    RedoTextEdit = 7,
    CutTextSelection = 8,
    PasteText = 9,
    SelectAllText = 10,
    OpenImage = 11,
    OpenImageInNewTab = 12,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ContextMenuVisibilityChangedEventArgs {
    pub visible: bool,
}

type MenuInvokeCallback = Rc<dyn Fn()>;
type VisibilityChangedCallback = Rc<dyn Fn(ContextMenuVisibilityChangedEventArgs)>;

#[derive(Clone)]
pub struct MenuItem {
    pub label: String,
    pub action: ContextMenuAction,
    pub payload: Option<String>,
    pub shortcut_label: Option<String>,
    pub disabled: bool,
    target_handle: u64,
    selection_start: u32,
    selection_end: u32,
    focus_target_after_action: bool,
    on_invoke: Option<MenuInvokeCallback>,
    kind: MenuItemKind,
}

impl MenuItem {
    pub fn new(label: impl Into<String>, action: ContextMenuAction) -> Self {
        Self {
            label: label.into(),
            action,
            payload: None,
            shortcut_label: None,
            disabled: false,
            target_handle: HandleValue::Invalid as u64,
            selection_start: 0,
            selection_end: 0,
            focus_target_after_action: false,
            on_invoke: None,
            kind: MenuItemKind::Action,
        }
    }

    pub fn separator() -> Self {
        Self {
            label: String::new(),
            action: ContextMenuAction::ReloadPage,
            payload: None,
            shortcut_label: None,
            disabled: false,
            target_handle: HandleValue::Invalid as u64,
            selection_start: 0,
            selection_end: 0,
            focus_target_after_action: false,
            on_invoke: None,
            kind: MenuItemKind::Separator,
        }
    }

    pub fn payload(&self, value: impl Into<String>) -> Self {
        let mut next = self.clone();
        next.payload = Some(value.into());
        next
    }

    pub fn shortcut_label(&self, value: impl Into<String>) -> Self {
        let mut next = self.clone();
        next.shortcut_label = Some(value.into());
        next
    }

    pub fn disabled(&self, flag: bool) -> Self {
        let mut next = self.clone();
        next.disabled = flag;
        next
    }

    pub fn target<T: Node>(&self, target: &T) -> Self {
        let mut next = self.clone();
        next.target_handle = target.handle().raw();
        next
    }

    pub(crate) fn target_handle(&self, target_handle: u64) -> Self {
        let mut next = self.clone();
        next.target_handle = target_handle;
        next
    }

    pub fn with_selection_range(&self, start: u32, end: u32) -> Self {
        let mut next = self.clone();
        next.selection_start = start;
        next.selection_end = end;
        next
    }

    pub fn focus_target_after_action(&self, flag: bool) -> Self {
        let mut next = self.clone();
        next.focus_target_after_action = flag;
        next
    }

    pub fn on_invoke(&self, callback: impl Fn() + 'static) -> Self {
        let mut next = self.clone();
        next.on_invoke = Some(Rc::new(callback));
        next
    }

    pub fn is_separator(&self) -> bool {
        self.kind == MenuItemKind::Separator
    }
}

fn commit_focused_text_action_if_needed(item: &MenuItem) {
    if item.focus_target_after_action && item.target_handle != HandleValue::Invalid as u64 {
        unsafe { crate::ffi::fui_commit_text_action_focus(item.target_handle) };
    }
}

pub fn run_context_menu_action(item: &MenuItem) {
    if item.disabled {
        let action_needs_live_selection = item.target_handle != HandleValue::Invalid as u64
            && matches!(
                item.action,
                ContextMenuAction::CopyCurrentSelection | ContextMenuAction::CutTextSelection
            )
            && (unsafe { crate::ffi::fui_has_text_selection_snapshot(item.target_handle) }
                || unsafe { crate::ffi::ui_has_text_selection(item.target_handle) });
        if !action_needs_live_selection {
            return;
        }
    }

    if let Some(callback) = item.on_invoke.as_ref() {
        callback();
    }

    match item.action {
        ContextMenuAction::CopyCurrentSelection => {
            if let Some(payload) = item.payload.as_ref() {
                write_payload_to_clipboard(payload);
                commit_focused_text_action_if_needed(item);
                return;
            }
            if item.target_handle != HandleValue::Invalid as u64
                && unsafe { crate::ffi::fui_copy_text_selection_snapshot(item.target_handle) }
            {
                commit_focused_text_action_if_needed(item);
                return;
            }
            if item.target_handle != HandleValue::Invalid as u64 {
                unsafe { crate::ffi::ui_copy_text_selection(item.target_handle) };
                commit_focused_text_action_if_needed(item);
                return;
            }
            unsafe { crate::ffi::ui_copy_current_selection() };
        }
        ContextMenuAction::UndoTextEdit if item.target_handle != HandleValue::Invalid as u64 => {
            unsafe { crate::ffi::ui_undo_text_edit(item.target_handle) };
            commit_focused_text_action_if_needed(item);
        }
        ContextMenuAction::RedoTextEdit if item.target_handle != HandleValue::Invalid as u64 => {
            unsafe { crate::ffi::ui_redo_text_edit(item.target_handle) };
            commit_focused_text_action_if_needed(item);
        }
        ContextMenuAction::CutTextSelection
            if item.target_handle != HandleValue::Invalid as u64 =>
        {
            if let Some(payload) = item.payload.as_ref() {
                write_payload_to_clipboard(payload);
            }
            if item.selection_start != item.selection_end
                && unsafe { crate::ffi::fui_cut_text_selection_snapshot(item.target_handle) }
            {
                commit_focused_text_action_if_needed(item);
                return;
            }
            if unsafe { crate::ffi::fui_cut_text_selection_snapshot(item.target_handle) } {
                commit_focused_text_action_if_needed(item);
                return;
            }
            if item.selection_start != item.selection_end
                && unsafe {
                    crate::ffi::fui_delete_focused_text_range(
                        item.selection_start,
                        item.selection_end,
                    )
                }
            {
                commit_focused_text_action_if_needed(item);
                return;
            }
            if unsafe { crate::ffi::fui_cut_focused_text_selection() } {
                commit_focused_text_action_if_needed(item);
                return;
            }
            if item.payload.is_none() {
                unsafe { crate::ffi::fui_copy_text_selection_snapshot(item.target_handle) };
            }
            unsafe { crate::ffi::ui_cut_text_selection(item.target_handle) };
            commit_focused_text_action_if_needed(item);
        }
        ContextMenuAction::PasteText if item.target_handle != HandleValue::Invalid as u64 => {
            unsafe { crate::ffi::ui_paste_text(item.target_handle) };
            commit_focused_text_action_if_needed(item);
        }
        ContextMenuAction::SelectAllText if item.target_handle != HandleValue::Invalid as u64 => {
            unsafe { crate::ffi::ui_select_all_text(item.target_handle) };
            commit_focused_text_action_if_needed(item);
        }
        ContextMenuAction::ReloadPage => unsafe { crate::ffi::fui_reload_page() },
        ContextMenuAction::OpenLink | ContextMenuAction::OpenImage => {
            if let Some(payload) = item.payload.as_ref() {
                navigation::navigate_to(payload, false);
            }
        }
        ContextMenuAction::OpenLinkInNewTab | ContextMenuAction::OpenImageInNewTab => {
            if let Some(payload) = item.payload.as_ref() {
                navigation::navigate_to(payload, true);
            }
        }
        ContextMenuAction::NavigateBack => navigation::navigate_back(),
        ContextMenuAction::NavigateForward => navigation::navigate_forward(),
        _ => {}
    }
}

#[derive(Clone)]
struct ContextMenuEntryStyle {
    item_height: f32,
    padding_left: f32,
    padding_top: f32,
    padding_right: f32,
    padding_bottom: f32,
    corner_top_left: f32,
    corner_top_right: f32,
    corner_bottom_right: f32,
    corner_bottom_left: f32,
    text_color: u32,
    background_color: u32,
    hover_background_color: u32,
    font_family: FontFamily,
    font_weight: FontWeight,
    font_style: FontStyle,
    font_size: f32,
}

impl ContextMenuEntryStyle {
    fn from_theme(theme: &Theme) -> Self {
        Self {
            item_height: theme.context_menu.item.height,
            padding_left: theme.context_menu.item.padding_left,
            padding_top: theme.context_menu.item.padding_top,
            padding_right: theme.context_menu.item.padding_right,
            padding_bottom: theme.context_menu.item.padding_bottom,
            corner_top_left: theme.context_menu.item.corner_radius,
            corner_top_right: theme.context_menu.item.corner_radius,
            corner_bottom_right: theme.context_menu.item.corner_radius,
            corner_bottom_left: theme.context_menu.item.corner_radius,
            text_color: theme.context_menu.item.text_color,
            background_color: theme.context_menu.item.background,
            hover_background_color: theme.context_menu.item.hover_background,
            font_family: theme.context_menu.item.font_family.clone(),
            font_weight: FontWeight::Regular,
            font_style: FontStyle::Normal,
            font_size: theme.context_menu.item.font_size,
        }
    }
}

fn context_menu_entry_line_height(style: &ContextMenuEntryStyle) -> f32 {
    let content_height = style.item_height - style.padding_top - style.padding_bottom;
    if content_height > 1.0 {
        content_height
    } else {
        1.0
    }
}

#[derive(Clone)]
struct ContextMenuEntry {
    root: Grid,
    label_node: TextNode,
    shortcut_node: TextNode,
    slot: usize,
    hovered: Rc<Cell<bool>>,
    pressed: Rc<Cell<bool>>,
    disabled: Rc<Cell<bool>>,
    style: Rc<RefCell<ContextMenuEntryStyle>>,
}

impl ContextMenuEntry {
    fn new(slot: usize) -> Self {
        let theme = current_theme();
        let root = grid();
        let label_node = TextNode::new("");
        label_node
            .font_family(theme.context_menu.item.font_family.clone())
            .font_size(theme.context_menu.item.font_size)
            .line_height(
                theme.context_menu.item.height
                    - theme.context_menu.item.padding_top
                    - theme.context_menu.item.padding_bottom,
            )
            .text_color(theme.context_menu.item.text_color)
            .text_overflow(TextOverflow::Ellipsis)
            .selectable(false);
        let shortcut_node = TextNode::new("");
        shortcut_node
            .font_family(theme.context_menu.item.font_family.clone())
            .font_size(theme.context_menu.item.font_size)
            .line_height(
                theme.context_menu.item.height
                    - theme.context_menu.item.padding_top
                    - theme.context_menu.item.padding_bottom,
            )
            .text_color(theme.colors.text_muted)
            .text_align(TextAlign::Left)
            .selectable(false);

        root.width(100.0, Unit::Percent)
            .height(theme.context_menu.item.height, Unit::Pixel)
            .padding(
                theme.context_menu.item.padding_left,
                theme.context_menu.item.padding_top,
                theme.context_menu.item.padding_right,
                theme.context_menu.item.padding_bottom,
            )
            .interactive(true)
            .semantic_role(SemanticRole::Button)
            .cursor(CursorStyle::Pointer)
            .columns(vec![GridTrack::star(1.0), GridTrack::auto()])
            .rows(vec![GridTrack::star(1.0)])
            .column_shared_size_group(1, SHORTCUT_SHARED_SIZE_GROUP)
            .place_child(&label_node, 0, 0, 1, 1)
            .place_child(&shortcut_node, 0, 1, 1, 1);

        let entry = Self {
            root,
            label_node,
            shortcut_node,
            slot,
            hovered: Rc::new(Cell::new(false)),
            pressed: Rc::new(Cell::new(false)),
            disabled: Rc::new(Cell::new(false)),
            style: Rc::new(RefCell::new(ContextMenuEntryStyle::from_theme(&theme))),
        };
        entry.bind_events();
        entry.apply_theme();
        entry
    }

    fn bind_events(&self) {
        let event_target = self.event_target();
        self.root.on_pointer_enter(move |_event| {
            if event_target.disabled.get() {
                return;
            }
            event_target.hovered.set(true);
            event_target.apply_theme();
        });

        let event_target = self.event_target();
        self.root.on_pointer_leave(move |_event| {
            event_target.hovered.set(false);
            event_target.pressed.set(false);
            event_target.apply_theme();
        });

        let event_target = self.event_target();
        self.root.on_pointer_down(move |_event| {
            event_target.pressed.set(!event_target.disabled.get());
        });

        let event_target = self.event_target();
        self.root.on_pointer_up(move |event| {
            let should_invoke = event_target.pressed.get()
                && event_target.hovered.get()
                && !event_target.disabled.get();
            event_target.pressed.set(false);
            event.handled = true;
            if should_invoke {
                ContextMenu::invoke_active_slot(event_target.slot as i32);
            }
        });

        let event_target = self.event_target();
        self.root.on_pointer_cancel(move |_event| {
            event_target.pressed.set(false);
        });
    }

    fn item(&self, item: &MenuItem) {
        self.hovered.set(false);
        self.pressed.set(false);
        self.disabled.set(item.disabled);
        self.root
            .semantic_label(item.label.clone())
            .semantic_disabled(item.disabled)
            .cursor(if item.disabled {
                CursorStyle::Default
            } else {
                CursorStyle::Pointer
            });
        self.label_node.text(item.label.clone());
        self.shortcut_node
            .text(item.shortcut_label.clone().unwrap_or_default());
        self.apply_theme();
    }

    fn configure_style(&self, style: &ContextMenuEntryStyle) {
        *self.style.borrow_mut() = style.clone();
        self.apply_theme();
    }

    fn apply_theme(&self) {
        let style = self.style.borrow().clone();
        let theme = current_theme();
        self.root
            .height(style.item_height, Unit::Pixel)
            .padding(
                style.padding_left,
                style.padding_top,
                style.padding_right,
                style.padding_bottom,
            )
            .corners(
                style.corner_top_left,
                style.corner_top_right,
                style.corner_bottom_right,
                style.corner_bottom_left,
            )
            .bg_color(if self.hovered.get() && !self.disabled.get() {
                style.hover_background_color
            } else {
                style.background_color
            });
        self.label_node
            .font_family(style.font_family.clone())
            .font_weight(style.font_weight)
            .font_style(style.font_style)
            .font_size(style.font_size)
            .line_height(context_menu_entry_line_height(&style))
            .text_color(if self.disabled.get() {
                theme.colors.text_muted
            } else {
                style.text_color
            });
        self.shortcut_node
            .font_family(style.font_family.clone())
            .font_weight(style.font_weight)
            .font_style(style.font_style)
            .font_size(style.font_size)
            .line_height(context_menu_entry_line_height(&style))
            .text_color(theme.colors.text_muted);
    }

    fn event_target(&self) -> ContextMenuEntryEventTarget {
        ContextMenuEntryEventTarget {
            slot: self.slot,
            hovered: self.hovered.clone(),
            pressed: self.pressed.clone(),
            disabled: self.disabled.clone(),
            root: self.root.downgrade(),
            label_node: self.label_node.clone(),
            shortcut_node: self.shortcut_node.clone(),
            style: self.style.clone(),
        }
    }
}

#[derive(Clone)]
struct ContextMenuEntryEventTarget {
    slot: usize,
    hovered: Rc<Cell<bool>>,
    pressed: Rc<Cell<bool>>,
    disabled: Rc<Cell<bool>>,
    root: WeakFlexBox,
    label_node: TextNode,
    shortcut_node: TextNode,
    style: Rc<RefCell<ContextMenuEntryStyle>>,
}

impl ContextMenuEntryEventTarget {
    fn apply_theme(&self) {
        let style = self.style.borrow().clone();
        let theme = current_theme();
        if let Some(root) = self.root.upgrade() {
            root.bg_color(if self.hovered.get() && !self.disabled.get() {
                style.hover_background_color
            } else {
                style.background_color
            });
        }
        self.label_node
            .font_family(style.font_family.clone())
            .font_size(style.font_size)
            .line_height(context_menu_entry_line_height(&style))
            .text_color(if self.disabled.get() {
                theme.colors.text_muted
            } else {
                style.text_color
            });
        self.shortcut_node
            .font_family(style.font_family.clone())
            .font_size(style.font_size)
            .line_height(context_menu_entry_line_height(&style))
            .text_color(theme.colors.text_muted);
    }
}

#[derive(Clone)]
struct ContextMenuSeparator {
    root: FlexBox,
    line: FlexBox,
    color: Rc<Cell<u32>>,
}

impl ContextMenuSeparator {
    fn new() -> Self {
        let theme = current_theme();
        let root = column();
        let line = flex_box();
        line.width(100.0, Unit::Percent).height(1.0, Unit::Pixel);
        root.width(100.0, Unit::Percent)
            .height(MENU_SEPARATOR_HEIGHT, Unit::Pixel)
            .padding(4.0, 0.0, 4.0, 0.0)
            .child(&line);
        let separator = Self {
            root,
            line,
            color: Rc::new(Cell::new(theme.context_menu.separator_color)),
        };
        separator.apply_theme();
        separator
    }

    fn configure_style(&self, color: u32) {
        self.color.set(color);
        self.apply_theme();
    }

    fn apply_theme(&self) {
        self.line.bg_color(self.color.get());
    }
}

struct ContextMenuState {
    appearance: Option<ContextMenuAppearance>,
    visible: bool,
    suppress_next_pointer_up_activation: bool,
    key_filter_token: u32,
    menu_width: f32,
    item_style: ContextMenuEntryStyle,
    panel_background_color: u32,
    panel_border_width: f32,
    panel_border_color: u32,
    panel_border_style: BorderStyle,
    panel_corner_top_left: f32,
    panel_corner_top_right: f32,
    panel_corner_bottom_right: f32,
    panel_corner_bottom_left: f32,
    separator_color: u32,
    panel_shadow_color: u32,
    panel_shadow_offset_x: f32,
    panel_shadow_offset_y: f32,
    panel_shadow_blur: f32,
    panel_shadow_spread: f32,
    panel_background_blur_sigma: f32,
    panel_border_dash_on: f32,
    panel_border_dash_off: f32,
    visibility_changed_callback: Option<VisibilityChangedCallback>,
}

impl ContextMenuState {
    fn from_theme(theme: Theme) -> Self {
        Self {
            appearance: None,
            visible: false,
            suppress_next_pointer_up_activation: false,
            key_filter_token: 0,
            menu_width: MENU_WIDTH,
            item_style: ContextMenuEntryStyle::from_theme(&theme),
            panel_background_color: theme.context_menu.panel_background,
            panel_border_width: 1.0,
            panel_border_color: theme.context_menu.panel_border_color,
            panel_border_style: BorderStyle::Solid,
            panel_corner_top_left: theme.context_menu.panel_corner_radius,
            panel_corner_top_right: theme.context_menu.panel_corner_radius,
            panel_corner_bottom_right: theme.context_menu.panel_corner_radius,
            panel_corner_bottom_left: theme.context_menu.panel_corner_radius,
            separator_color: theme.context_menu.separator_color,
            panel_shadow_color: theme.context_menu.panel_shadow_color,
            panel_shadow_offset_x: 0.0,
            panel_shadow_offset_y: theme.context_menu.shadow_offset_y,
            panel_shadow_blur: theme.context_menu.shadow_blur,
            panel_shadow_spread: theme.context_menu.shadow_spread,
            panel_background_blur_sigma: DEFAULT_PANEL_BACKGROUND_BLUR_SIGMA,
            panel_border_dash_on: 0.0,
            panel_border_dash_off: 0.0,
            visibility_changed_callback: None,
        }
    }
}

#[derive(Clone)]
struct ContextMenuEventTarget {
    panel: WeakFlexBox,
    presenter: PopupPresenterEventTarget,
    entries: Vec<ContextMenuEntry>,
    separators: Vec<ContextMenuSeparator>,
    current_items: Rc<RefCell<Vec<MenuItem>>>,
    current_item_tops: Rc<RefCell<Vec<f32>>>,
    current_item_heights: Rc<RefCell<Vec<f32>>>,
    state: Rc<RefCell<ContextMenuState>>,
}

impl ContextMenuEventTarget {
    fn clear_panel(&self) {
        let Some(panel) = self.panel.upgrade() else {
            return;
        };
        for entry in &self.entries {
            panel.remove_child(&entry.root);
        }
        for separator in &self.separators {
            panel.remove_child(&separator.root);
        }
    }

    fn hide(&self) {
        let was_visible = self.state.borrow().visible || self.presenter.is_open();
        if !was_visible {
            return;
        }
        self.clear_panel();
        self.current_items.borrow_mut().clear();
        self.current_item_tops.borrow_mut().clear();
        self.current_item_heights.borrow_mut().clear();
        self.presenter.hide();
        let callback = {
            let mut state = self.state.borrow_mut();
            state.visible = false;
            state.suppress_next_pointer_up_activation = false;
            let token = state.key_filter_token;
            state.key_filter_token = 0;
            if token != 0 {
                event::remove_key_filter(token);
            }
            state.visibility_changed_callback.clone()
        };
        ACTIVE_CONTEXT_MENU.with(|slot| {
            let should_clear = slot
                .borrow()
                .as_ref()
                .is_some_and(|active| Rc::ptr_eq(&active.state, &self.state));
            if should_clear {
                slot.borrow_mut().take();
            }
        });
        if let Some(callback) = callback {
            callback(ContextMenuVisibilityChangedEventArgs { visible: false });
        }
    }

    fn invoke_slot(&self, slot: usize) {
        let item = self.current_items.borrow().get(slot).cloned();
        if let Some(item) = item {
            run_context_menu_action(&item);
            self.hide();
        }
    }

    fn handle_overlay_pointer_up(&self, event: &mut PointerEventArgs) {
        if !self.state.borrow().visible {
            return;
        }
        if !is_primary_activation_pointer(event) {
            if ContextMenu::consume_opening_pointer_up_suppression() {
                event.handled = true;
                return;
            }
            event.handled = true;
            return;
        }
        let item_count = self.current_items.borrow().len();
        for slot in 0..item_count {
            let entry = &self.entries[slot];
            let Some(bounds) = ui::get_bounds(entry.root.handle().raw()) else {
                continue;
            };
            let left = bounds[0];
            let top = bounds[1];
            let right = left + bounds[2];
            let bottom = top + bounds[3];
            if event.scene_x >= left
                && event.scene_x <= right
                && event.scene_y >= top
                && event.scene_y <= bottom
            {
                event.handled = true;
                self.invoke_slot(slot);
                return;
            }
        }
        let local_x = event.scene_x - self.presenter.surface_x();
        let local_y = event.scene_y - self.presenter.surface_y();
        if local_x < 0.0 || local_x > self.state.borrow().menu_width || local_y < 0.0 {
            event.handled = true;
            self.hide();
            return;
        }
        let slot_to_invoke = {
            let tops = self.current_item_tops.borrow();
            let heights = self.current_item_heights.borrow();
            let mut slot_to_invoke = None;
            for slot in 0..tops.len() {
                let top = tops[slot];
                let height = heights[slot];
                if local_y >= top && local_y <= top + height {
                    slot_to_invoke = Some(slot);
                    break;
                }
            }
            slot_to_invoke
        };
        if let Some(slot) = slot_to_invoke {
            event.handled = true;
            self.invoke_slot(slot);
            return;
        }
        event.handled = true;
        self.hide();
    }

    fn handle_global_key_event(&self, event_type: KeyEventType, key: &str) -> bool {
        if event_type == KeyEventType::Down && key == "Escape" {
            self.hide();
            return true;
        }
        false
    }

    fn apply_theme(&self) {
        let Some(panel) = self.panel.upgrade() else {
            return;
        };
        let state = self.state.borrow();
        panel
            .shared_size_scope(true)
            .width(state.menu_width, Unit::Pixel)
            .bg_color(state.panel_background_color)
            .background_blur(state.panel_background_blur_sigma)
            .corners(
                state.panel_corner_top_left,
                state.panel_corner_top_right,
                state.panel_corner_bottom_right,
                state.panel_corner_bottom_left,
            )
            .border_config(Border {
                width: state.panel_border_width,
                color: state.panel_border_color,
                style: state.panel_border_style,
                dash_on: state.panel_border_dash_on,
                dash_off: state.panel_border_dash_off,
            })
            .drop_shadow(
                state.panel_shadow_color,
                state.panel_shadow_offset_x,
                state.panel_shadow_offset_y,
                state.panel_shadow_blur,
                state.panel_shadow_spread,
            );
        let item_style = state.item_style.clone();
        let separator_color = state.separator_color;
        drop(state);
        for entry in &self.entries {
            entry.configure_style(&item_style);
        }
        for separator in &self.separators {
            separator.configure_style(separator_color);
        }
    }

    fn handle_theme_changed(&self) {
        let theme = current_theme();
        let mut state = self.state.borrow_mut();
        let appearance = state.appearance.clone().unwrap_or_default();
        let panel = appearance.panel.unwrap_or_default();
        let backdrop = appearance.backdrop.unwrap_or_default();
        let item = appearance.item.unwrap_or_default();

        state.menu_width = appearance.width.unwrap_or(MENU_WIDTH);
        state.panel_background_color = panel
            .background
            .unwrap_or(theme.context_menu.panel_background);
        let border = panel
            .border
            .unwrap_or_else(|| Border::solid(1.0, theme.context_menu.panel_border_color));
        state.panel_border_width = border.width;
        state.panel_border_color = border.color;
        state.panel_border_style = border.style;
        state.panel_border_dash_on = border.dash_on;
        state.panel_border_dash_off = border.dash_off;
        let corners = panel
            .corners
            .unwrap_or_else(|| crate::Corners::all(theme.context_menu.panel_corner_radius));
        state.panel_corner_top_left = corners.top_left;
        state.panel_corner_top_right = corners.top_right;
        state.panel_corner_bottom_right = corners.bottom_right;
        state.panel_corner_bottom_left = corners.bottom_left;
        let shadow = panel.shadow.unwrap_or_else(|| {
            crate::Shadow::new(
                theme.context_menu.panel_shadow_color,
                0.0,
                theme.context_menu.shadow_offset_y,
                theme.context_menu.shadow_blur,
                theme.context_menu.shadow_spread,
            )
        });
        state.panel_shadow_color = shadow.color;
        state.panel_shadow_offset_x = shadow.offset_x;
        state.panel_shadow_offset_y = shadow.offset_y;
        state.panel_shadow_blur = shadow.blur_sigma;
        state.panel_shadow_spread = shadow.spread;
        state.panel_background_blur_sigma = panel
            .background_blur
            .unwrap_or(DEFAULT_PANEL_BACKGROUND_BLUR_SIGMA);
        state.separator_color = appearance
            .separator_color
            .unwrap_or(theme.context_menu.separator_color);

        state.item_style.item_height = item.height.unwrap_or(theme.context_menu.item.height);
        let padding = item.padding.unwrap_or_else(|| {
            crate::EdgeInsets::new(
                theme.context_menu.item.padding_left,
                theme.context_menu.item.padding_top,
                theme.context_menu.item.padding_right,
                theme.context_menu.item.padding_bottom,
            )
        });
        state.item_style.padding_left = padding.left;
        state.item_style.padding_top = padding.top;
        state.item_style.padding_right = padding.right;
        state.item_style.padding_bottom = padding.bottom;
        state.item_style.text_color = item
            .text_color
            .unwrap_or(theme.context_menu.item.text_color);
        state.item_style.background_color = item
            .background
            .unwrap_or(theme.context_menu.item.background);
        state.item_style.hover_background_color = item
            .hover_background
            .unwrap_or(theme.context_menu.item.hover_background);
        let item_corners = item
            .corners
            .unwrap_or_else(|| crate::Corners::all(theme.context_menu.item.corner_radius));
        state.item_style.corner_top_left = item_corners.top_left;
        state.item_style.corner_top_right = item_corners.top_right;
        state.item_style.corner_bottom_right = item_corners.bottom_right;
        state.item_style.corner_bottom_left = item_corners.bottom_left;
        state.item_style.font_family = item
            .font_family
            .unwrap_or_else(|| theme.context_menu.item.font_family.clone());
        state.item_style.font_weight = item.font_weight.unwrap_or(FontWeight::Regular);
        state.item_style.font_style = item.font_style.unwrap_or(FontStyle::Normal);
        state.item_style.font_size = item.font_size.unwrap_or(theme.context_menu.item.font_size);
        drop(state);
        self.presenter
            .backdrop_color(backdrop.color.unwrap_or(0x00000000));
        self.presenter.background_blur(backdrop.blur.unwrap_or(0.0));
        self.apply_theme();
    }
}

#[derive(Clone)]
pub struct ContextMenu {
    root: FlexBox,
    panel: FlexBox,
    popup_presenter: PopupPresenter,
    items: Rc<RefCell<Vec<MenuItem>>>,
    entries: Vec<ContextMenuEntry>,
    separators: Vec<ContextMenuSeparator>,
    current_items: Rc<RefCell<Vec<MenuItem>>>,
    current_item_tops: Rc<RefCell<Vec<f32>>>,
    current_item_heights: Rc<RefCell<Vec<f32>>>,
    state: Rc<RefCell<ContextMenuState>>,
}

impl Default for ContextMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextMenu {
    pub fn new() -> Self {
        let theme = current_theme();
        let root = portal();
        let panel = column();
        panel
            .position_type(crate::ffi::PositionType::Absolute)
            .shared_size_scope(true)
            .width(MENU_WIDTH, Unit::Pixel)
            .padding(4.0, 4.0, 4.0, 4.0)
            .border(1.0, theme.context_menu.panel_border_color);
        let popup_presenter = PopupPresenter::new(root.clone(), panel.clone());
        root.position_type(crate::ffi::PositionType::Absolute)
            .position(0.0, 0.0)
            .width(100.0, Unit::Percent)
            .height(100.0, Unit::Percent);

        let menu = Self {
            root,
            panel,
            popup_presenter,
            items: Rc::new(RefCell::new(Vec::new())),
            entries: (0..MAX_ITEMS).map(ContextMenuEntry::new).collect(),
            separators: (0..MAX_ITEMS)
                .map(|_| ContextMenuSeparator::new())
                .collect(),
            current_items: Rc::new(RefCell::new(Vec::new())),
            current_item_tops: Rc::new(RefCell::new(Vec::new())),
            current_item_heights: Rc::new(RefCell::new(Vec::new())),
            state: Rc::new(RefCell::new(ContextMenuState::from_theme(theme))),
        };
        menu.bind_events();
        menu.install_theme_subscription();
        menu.apply_theme();
        menu
    }

    pub fn hide_active_menu() {
        let menu = ACTIVE_CONTEXT_MENU.with(|slot| slot.borrow().as_ref().cloned());
        if let Some(menu) = menu {
            menu.hide();
        }
    }

    pub(crate) fn invoke_active_slot(slot: i32) {
        if slot < 0 {
            return;
        }
        let menu = ACTIVE_CONTEXT_MENU.with(|slot_state| slot_state.borrow().as_ref().cloned());
        if let Some(menu) = menu {
            menu.invoke_slot(slot as usize);
        }
    }

    pub(crate) fn consume_opening_pointer_up_suppression() -> bool {
        ACTIVE_CONTEXT_MENU.with(|slot| {
            let Some(menu) = slot.borrow().as_ref().cloned() else {
                return false;
            };
            let mut state = menu.state.borrow_mut();
            if !state.suppress_next_pointer_up_activation {
                return false;
            }
            state.suppress_next_pointer_up_activation = false;
            true
        })
    }

    pub fn items<I>(&self, items: I) -> &Self
    where
        I: IntoIterator<Item = MenuItem>,
    {
        self.items.borrow_mut().clear();
        self.items.borrow_mut().extend(items);
        self
    }

    pub fn clear_items(&self) -> &Self {
        self.items.borrow_mut().clear();
        self
    }

    pub fn is_open(&self) -> bool {
        self.state.borrow().visible
    }

    pub fn on_visibility_changed(
        &self,
        handler: impl Fn(ContextMenuVisibilityChangedEventArgs) + 'static,
    ) -> &Self {
        self.state.borrow_mut().visibility_changed_callback = Some(Rc::new(handler));
        self
    }

    pub fn clear_visibility_changed(&self) -> &Self {
        self.state.borrow_mut().visibility_changed_callback = None;
        self
    }

    pub fn appearance(&self, appearance: ContextMenuAppearance) -> &Self {
        self.state.borrow_mut().appearance = Some(appearance);
        self.event_target().handle_theme_changed();
        self
    }

    pub fn clear_appearance(&self) -> &Self {
        self.state.borrow_mut().appearance = None;
        self.event_target().handle_theme_changed();
        self
    }

    pub fn show(&self, x: f32, y: f32) {
        self.show_impl(x, y, false);
    }

    pub fn show_relative_to<T: Node>(&self, target: &T, x: f32, y: f32) {
        if let Some(bounds) = ui::get_bounds(target.handle().raw()) {
            self.show(bounds[0] + x, bounds[1] + y);
        } else {
            self.show(x, y);
        }
    }

    pub fn show_from_context_pointer(&self, x: f32, y: f32) {
        self.show_impl(x, y, true);
    }

    pub fn show_from_context_pointer_relative_to<T: Node>(&self, target: &T, x: f32, y: f32) {
        if let Some(bounds) = ui::get_bounds(target.handle().raw()) {
            self.show_from_context_pointer(bounds[0] + x, bounds[1] + y);
        } else {
            self.show_from_context_pointer(x, y);
        }
    }

    fn show_impl(&self, x: f32, y: f32, suppress_opening_pointer_up: bool) {
        self.clear_panel();
        self.apply_theme();
        self.current_items.borrow_mut().clear();
        self.current_item_tops.borrow_mut().clear();
        self.current_item_heights.borrow_mut().clear();

        let items = self.items.borrow().clone();
        let item_height = self.state.borrow().item_style.item_height;
        let menu_width = self.state.borrow().menu_width;
        let mut action_count = 0usize;
        let mut separator_count = 0usize;
        let mut estimated_height = 8.0;
        let mut content_y = 0.0;
        let mut last_was_separator = true;
        let count = items.len().min(MAX_ITEMS);

        if items.len() > MAX_ITEMS {
            warn(
                "Layout",
                &format!(
                    "ContextMenu.show() received {} items; truncating to {MAX_ITEMS}.",
                    items.len()
                ),
            );
        }

        for (index, item) in items.into_iter().take(count).enumerate() {
            if item.is_separator() {
                if last_was_separator || index == count - 1 {
                    continue;
                }
                let separator = &self.separators[separator_count];
                separator.apply_theme();
                self.panel.child(&separator.root);
                separator_count += 1;
                estimated_height += MENU_SEPARATOR_HEIGHT;
                content_y += MENU_SEPARATOR_HEIGHT;
                last_was_separator = true;
                continue;
            }

            let entry = &self.entries[action_count];
            entry.item(&item);
            entry.apply_theme();
            self.current_items.borrow_mut().push(item);
            self.current_item_tops.borrow_mut().push(content_y);
            self.current_item_heights.borrow_mut().push(item_height);
            self.panel.child(&entry.root);
            action_count += 1;
            estimated_height += item_height;
            content_y += item_height;
            last_was_separator = false;
        }

        if action_count == 0 {
            self.clear_panel();
            return;
        }

        let max_x = (ui::get_viewport_width() - menu_width - MENU_EDGE_PADDING).max(0.0);
        let max_y = (ui::get_viewport_height() - estimated_height - MENU_EDGE_PADDING).max(0.0);
        let clamped_x = x.clamp(MENU_EDGE_PADDING, max_x);
        let clamped_y = y.clamp(MENU_EDGE_PADDING, max_y);
        self.popup_presenter
            .show_at_point(clamped_x, clamped_y, menu_width, estimated_height);
        {
            let mut state = self.state.borrow_mut();
            state.visible = true;
            state.suppress_next_pointer_up_activation = suppress_opening_pointer_up;
        }
        ACTIVE_CONTEXT_MENU.with(|slot| {
            *slot.borrow_mut() = Some(self.event_target());
        });
        if let Some(callback) = self.state.borrow().visibility_changed_callback.clone() {
            callback(ContextMenuVisibilityChangedEventArgs { visible: true });
        }
        if self.state.borrow().key_filter_token == 0 {
            let event_target = self.event_target();
            let token = event::push_key_filter(move |event_type, key, _modifiers| {
                event_target.handle_global_key_event(event_type, key)
            });
            self.state.borrow_mut().key_filter_token = token;
        }
    }

    pub fn hide(&self) {
        self.event_target().hide();
    }

    fn bind_events(&self) {
        let event_target = self.event_target();
        self.popup_presenter
            .overlay_node()
            .interactive(true)
            .on_pointer_up(move |event| {
                event_target.handle_overlay_pointer_up(event);
            });
    }

    fn install_theme_subscription(&self) {
        let event_target = self.event_target();
        let guard = subscribe(move |_theme| {
            event_target.handle_theme_changed();
        });
        self.root
            .retained_node_ref()
            .retain_attachment(Rc::new(guard));
    }

    fn clear_panel(&self) {
        self.event_target().clear_panel();
    }

    fn apply_theme(&self) {
        self.event_target().apply_theme();
    }

    fn event_target(&self) -> ContextMenuEventTarget {
        ContextMenuEventTarget {
            panel: self.panel.downgrade(),
            presenter: self.popup_presenter.event_target(),
            entries: self.entries.clone(),
            separators: self.separators.clone(),
            current_items: self.current_items.clone(),
            current_item_tops: self.current_item_tops.clone(),
            current_item_heights: self.current_item_heights.clone(),
            state: self.state.clone(),
        }
    }
}

impl Node for ContextMenu {
    fn retained_node_ref(&self) -> NodeRef {
        self.root.retained_node_ref()
    }

    fn retained_owner_attachment(&self) -> Option<Rc<dyn std::any::Any>> {
        Some(Rc::new(self.clone()))
    }

    fn build_self(&self) {
        self.root.build_self();
    }

    fn dispose(&self) {
        self.hide();
        self.popup_presenter.dispose();
        for entry in &self.entries {
            if entry.root.handle() != crate::node::NodeHandle::INVALID {
                entry.root.dispose();
            }
        }
        for separator in &self.separators {
            if separator.root.handle() != crate::node::NodeHandle::INVALID {
                separator.root.dispose();
            }
        }
        self.root.dispose();
    }
}

impl crate::node::HasFlexBoxRoot for ContextMenu {
    fn flex_box_root(&self) -> &FlexBox {
        &self.root
    }
}
