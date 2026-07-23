#![allow(clippy::too_many_arguments)]

use crate::context_menu_manager;
use crate::drag_drop;
use crate::external_drop;
use crate::ffi::{CursorStyle, KeyEventType, PointerEventType, SemanticRole};
use crate::focus_visibility;
use crate::keyboard_scroll::handle_keyboard_scroll_fallback;
use crate::keyboard_scroll_tracker::{
    register_keyboard_scroll_node, reset_keyboard_scroll_tracking,
    track_keyboard_scroll_pointer_up, unregister_keyboard_scroll_node,
};
use crate::mobile_text_selection_toolbar;
use crate::node::{NodeHandle, NodeRef, WeakNodeRef};
use crate::selection_handle_adorner;
use crate::tool_tip_manager;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

static mut KEY_BUFFER: [u8; 256] = [0; 256];
static mut TEXT_BUFFER: [u8; 16 * 1024] = [0; 16 * 1024];

pub const DEFAULT_LONG_PRESS_MINIMUM_DURATION_MS: i32 = 500;
pub const DEFAULT_LONG_PRESS_MOVEMENT_TOLERANCE: f32 = 10.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerType {
    Unknown = 0,
    Mouse = 1,
    Touch = 2,
    Pen = 3,
}

impl PointerType {
    pub fn from_raw(value: u32) -> Self {
        match value {
            1 => Self::Mouse,
            2 => Self::Touch,
            3 => Self::Pen,
            _ => Self::Unknown,
        }
    }
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerButton {
    None = -1,
    Primary = 0,
    Auxiliary = 1,
    Secondary = 2,
    Back = 3,
    Forward = 4,
}

impl PointerButton {
    pub fn from_raw(value: i32) -> Self {
        match value {
            0 => Self::Primary,
            1 => Self::Auxiliary,
            2 => Self::Secondary,
            3 => Self::Back,
            4 => Self::Forward,
            _ => Self::None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PointerButtons(u32);

impl PointerButtons {
    pub const NONE: Self = Self(0);
    pub const PRIMARY: Self = Self(1 << 0);
    pub const SECONDARY: Self = Self(1 << 1);
    pub const AUXILIARY: Self = Self(1 << 2);
    pub const BACK: Self = Self(1 << 3);
    pub const FORWARD: Self = Self(1 << 4);

    pub const fn from_raw(value: u32) -> Self {
        Self(value)
    }

    pub const fn bits(self) -> u32 {
        self.0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn contains(self, buttons: Self) -> bool {
        self.0 & buttons.0 == buttons.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GestureIntent {
    None = 0,
    Pan = 1,
    Pinch = 2,
    PanAndPinch = 3,
}

impl GestureIntent {
    pub fn from_callbacks(has_pan: bool, has_pinch: bool) -> Self {
        match (has_pan, has_pinch) {
            (false, false) => Self::None,
            (true, false) => Self::Pan,
            (false, true) => Self::Pinch,
            (true, true) => Self::PanAndPinch,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GestureEventPhase {
    Begin = 1,
    Update = 2,
    End = 3,
    Cancel = 4,
}

impl GestureEventPhase {
    pub fn from_raw(value: u32) -> Self {
        match value {
            1 => Self::Begin,
            2 => Self::Update,
            3 => Self::End,
            _ => Self::Cancel,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GestureEventKind {
    None = 0,
    Pan = 1,
    Pinch = 2,
}

impl GestureEventKind {
    pub fn from_raw(value: u32) -> Self {
        match value {
            1 => Self::Pan,
            2 => Self::Pinch,
            _ => Self::None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PointerEventArgs {
    target_handle: NodeHandle,
    pub event_type: PointerEventType,
    pub scene_x: f32,
    pub scene_y: f32,
    pub x: f32,
    pub y: f32,
    pub modifiers: u32,
    pub pointer_id: i32,
    pub pointer_type: PointerType,
    pub button: PointerButton,
    pub buttons: PointerButtons,
    pub pressure: f32,
    pub width: f32,
    pub height: f32,
    pub click_count: i32,
    pub handled: bool,
}

impl PointerEventArgs {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        target_handle: NodeHandle,
        event_type: PointerEventType,
        scene_x: f32,
        scene_y: f32,
        modifiers: u32,
        pointer_id: i32,
        pointer_type: PointerType,
        button: i32,
        buttons: u32,
        pressure: f32,
        width: f32,
        height: f32,
        click_count: i32,
    ) -> Self {
        Self {
            target_handle,
            event_type,
            scene_x,
            scene_y,
            x: scene_x,
            y: scene_y,
            modifiers,
            pointer_id,
            pointer_type,
            button: PointerButton::from_raw(button),
            buttons: PointerButtons::from_raw(buttons),
            pressure,
            width,
            height,
            click_count,
            handled: false,
        }
    }

    pub fn capture_pointer(&self) {
        crate::event::capture_pointer(self.target_handle);
        unsafe { crate::ffi::fui_set_pointer_capture(self.target_handle.raw()) };
    }

    pub fn release_pointer_capture(&self) {
        crate::event::release_pointer(self.target_handle);
        unsafe { crate::ffi::fui_release_pointer_capture() };
    }

    pub(crate) fn target_handle(&self) -> NodeHandle {
        self.target_handle
    }
}

#[derive(Clone, Debug)]
pub struct WheelEventArgs {
    pub scene_x: f32,
    pub scene_y: f32,
    pub x: f32,
    pub y: f32,
    pub delta_x: f32,
    pub delta_y: f32,
    pub delta_mode: u32,
    pub modifiers: u32,
    pub handled: bool,
}

impl WheelEventArgs {
    pub fn new(
        scene_x: f32,
        scene_y: f32,
        delta_x: f32,
        delta_y: f32,
        delta_mode: u32,
        modifiers: u32,
    ) -> Self {
        Self {
            scene_x,
            scene_y,
            x: scene_x,
            y: scene_y,
            delta_x,
            delta_y,
            delta_mode,
            modifiers,
            handled: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct GestureEventArgs {
    pub phase: GestureEventPhase,
    pub kind: GestureEventKind,
    pub scene_x: f32,
    pub scene_y: f32,
    pub delta_x: f32,
    pub delta_y: f32,
    pub scale: f32,
    pub pointer_count: i32,
    pub x: f32,
    pub y: f32,
    pub handled: bool,
}

impl GestureEventArgs {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        phase: GestureEventPhase,
        kind: GestureEventKind,
        scene_x: f32,
        scene_y: f32,
        delta_x: f32,
        delta_y: f32,
        scale: f32,
        pointer_count: i32,
    ) -> Self {
        Self {
            phase,
            kind,
            scene_x,
            scene_y,
            delta_x,
            delta_y,
            scale,
            pointer_count,
            x: scene_x,
            y: scene_y,
            handled: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct KeyEventArgs {
    pub event_type: KeyEventType,
    pub key: String,
    pub modifiers: u32,
    pub handled: bool,
}

impl KeyEventArgs {
    pub fn new(event_type: KeyEventType, key: String, modifiers: u32) -> Self {
        Self {
            event_type,
            key,
            modifiers,
            handled: false,
        }
    }
}

type GlobalKeyFilter = Rc<dyn Fn(KeyEventType, &str, u32) -> bool>;
type ScrollHook = Rc<dyn Fn()>;

struct GlobalKeyFilterEntry {
    token: u32,
    callback: GlobalKeyFilter,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FocusChangedEventArgs {
    pub focused: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TextChangedEventArgs {
    pub text: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SelectionChangedEventArgs {
    pub start: u32,
    pub end: u32,
}

#[derive(Clone, Debug)]
pub struct LongPressEventArgs {
    pub scene_x: f32,
    pub scene_y: f32,
    pub pointer_id: i32,
    pub pointer_type: PointerType,
    pub modifiers: u32,
    pub duration_ms: i32,
    pub x: f32,
    pub y: f32,
    pub handled: bool,
}

impl LongPressEventArgs {
    pub fn new(
        scene_x: f32,
        scene_y: f32,
        pointer_id: i32,
        pointer_type: PointerType,
        modifiers: u32,
        duration_ms: i32,
    ) -> Self {
        Self {
            scene_x,
            scene_y,
            pointer_id,
            pointer_type,
            modifiers,
            duration_ms,
            x: scene_x,
            y: scene_y,
            handled: false,
        }
    }
}

pub(crate) struct EventRouter {
    nodes: RefCell<HashMap<NodeHandle, WeakNodeRef>>,
    focused: RefCell<Option<NodeHandle>>,
    captured_pointer: RefCell<Option<NodeHandle>>,
    selection_cursor_owner: RefCell<Option<NodeHandle>>,
    hover_stack: RefCell<Vec<NodeHandle>>,
    current_cursor_style: Cell<CursorStyle>,
    key_filters: RefCell<Vec<GlobalKeyFilterEntry>>,
    scroll_hooks: RefCell<Vec<ScrollHook>>,
    next_key_filter_token: Cell<u32>,
}

impl EventRouter {
    pub(crate) fn new() -> Self {
        Self {
            nodes: RefCell::new(HashMap::new()),
            focused: RefCell::new(None),
            captured_pointer: RefCell::new(None),
            selection_cursor_owner: RefCell::new(None),
            hover_stack: RefCell::new(Vec::new()),
            current_cursor_style: Cell::new(CursorStyle::Default),
            key_filters: RefCell::new(Vec::new()),
            scroll_hooks: RefCell::new(Vec::new()),
            next_key_filter_token: Cell::new(1),
        }
    }

    fn register(&self, node: &NodeRef) {
        self.nodes
            .borrow_mut()
            .insert(node.handle(), node.downgrade());
        register_keyboard_scroll_node(node);
    }

    fn reset(&self) {
        self.nodes.borrow_mut().clear();
        self.focused.replace(None);
        self.captured_pointer.replace(None);
        self.selection_cursor_owner.replace(None);
        self.hover_stack.borrow_mut().clear();
        self.key_filters.borrow_mut().clear();
        // Scroll hooks are process-wide manager hooks in FUI-AS. Keep that
        // lifetime here as well; route/test resets should clear per-node state
        // but must not silently unregister static control-manager hooks.
        self.next_key_filter_token.set(1);
        drag_drop::reset();
        external_drop::reset();
        self.apply_cursor(CursorStyle::Default);
        reset_keyboard_scroll_tracking();
    }

    fn unregister(&self, handle: NodeHandle) {
        if let Some(node) = self.resolve_node(handle) {
            drag_drop::handle_node_destroyed(node.clone());
            external_drop::handle_node_destroyed(node);
        }
        self.nodes.borrow_mut().remove(&handle);
        self.pop_hover(handle);
        if self.focused.borrow().as_ref() == Some(&handle) {
            self.focused.replace(None);
        }
        if self.captured_pointer.borrow().as_ref() == Some(&handle) {
            self.captured_pointer.replace(None);
        }
        if self.selection_cursor_owner.borrow().as_ref() == Some(&handle) {
            self.selection_cursor_owner.replace(None);
        }
        self.apply_current_cursor();
        unregister_keyboard_scroll_node(handle);
    }

    fn push_key_filter(&self, callback: GlobalKeyFilter) -> u32 {
        let token = self.next_key_filter_token.get();
        self.next_key_filter_token.set(token + 1);
        self.key_filters
            .borrow_mut()
            .push(GlobalKeyFilterEntry { token, callback });
        token
    }

    fn remove_key_filter(&self, token: u32) {
        self.key_filters
            .borrow_mut()
            .retain(|entry| entry.token != token);
    }

    fn register_scroll_hook(&self, callback: ScrollHook) {
        self.scroll_hooks.borrow_mut().push(callback);
    }

    fn focused_node_is_enabled_button(&self) -> bool {
        let Some(handle) = *self.focused.borrow() else {
            return false;
        };
        let Some(node) = self.resolve_node(handle) else {
            return false;
        };
        node.semantic_role_for_routing() == Some(SemanticRole::Button)
            && node.is_enabled_for_routing()
    }

    pub(crate) fn dispatch_pointer_event(
        &self,
        handle: NodeHandle,
        event_type: PointerEventType,
        scene_x: f32,
        scene_y: f32,
        modifiers: u32,
        pointer_id: i32,
        pointer_type: PointerType,
        button: i32,
        buttons: u32,
        pressure: f32,
        width: f32,
        height: f32,
        click_count: i32,
    ) -> bool {
        context_menu_manager::track_pointer_event(event_type, handle.raw());
        selection_handle_adorner::record_pointer_event(event_type, pointer_type);
        let pointed_node = self.resolve_node(handle);
        if event_type == PointerEventType::Down
            && (button == PointerButton::Primary as i32
                || matches!(pointer_type, PointerType::Touch | PointerType::Pen))
            && crate::controls::ContextMenu::dismiss_for_outside_pointer_down(scene_x, scene_y)
        {
            drag_drop::handle_pointer_event(None, event_type, scene_x, scene_y, modifiers);
            self.apply_current_cursor();
            return true;
        }
        if event_type == PointerEventType::Down
            && !preserves_selection_on_pointer_down_for_routing(pointed_node.as_ref())
            && mobile_text_selection_toolbar::dismiss_for_outside_pointer_down(scene_x, scene_y)
        {
            drag_drop::handle_pointer_event(None, event_type, scene_x, scene_y, modifiers);
            self.apply_current_cursor();
            return true;
        }
        if event_type == PointerEventType::Down
            && button == PointerButton::Primary as i32
            && pointer_type != PointerType::Touch
        {
            self.selection_cursor_owner.replace(
                pointed_node
                    .as_ref()
                    .and_then(selection_cursor_target)
                    .map(|node| node.handle()),
            );
        } else if matches!(event_type, PointerEventType::Up | PointerEventType::Cancel) {
            self.selection_cursor_owner.replace(None);
        }
        if event_type == PointerEventType::Up {
            track_keyboard_scroll_pointer_up(pointed_node.clone(), scene_x, scene_y);
        }
        if matches!(
            event_type,
            PointerEventType::Move | PointerEventType::Up | PointerEventType::Cancel
        ) {
            let captured_handle = *self.captured_pointer.borrow();
            if let Some(captured_handle) = captured_handle {
                if let Some(captured_node) = self.resolve_node(captured_handle) {
                    let captured_at_dispatch = captured_handle;
                    let handled = self.dispatch_pointer_to_node(
                        captured_node,
                        captured_handle,
                        event_type,
                        scene_x,
                        scene_y,
                        modifiers,
                        pointer_id,
                        pointer_type,
                        button,
                        buttons,
                        pressure,
                        width,
                        height,
                        click_count,
                    );
                    let captured_after_dispatch = *self.captured_pointer.borrow();
                    if matches!(event_type, PointerEventType::Up | PointerEventType::Cancel)
                        && captured_after_dispatch == Some(captured_at_dispatch)
                    {
                        self.captured_pointer.replace(None);
                    }
                    drag_drop::handle_pointer_event(
                        pointed_node,
                        event_type,
                        scene_x,
                        scene_y,
                        modifiers,
                    );
                    self.apply_current_cursor();
                    return handled;
                }
                self.captured_pointer.replace(None);
                self.apply_current_cursor();
            }
        }
        if matches!(
            event_type,
            PointerEventType::Move | PointerEventType::Up | PointerEventType::Cancel
        ) {
            let mut selection_event = PointerEventArgs::new(
                handle,
                event_type,
                scene_x,
                scene_y,
                modifiers,
                pointer_id,
                pointer_type,
                button,
                buttons,
                pressure,
                width,
                height,
                click_count,
            );
            if selection_handle_adorner::route_active_handle_drag_event(&mut selection_event) {
                drag_drop::handle_pointer_event(
                    pointed_node,
                    event_type,
                    scene_x,
                    scene_y,
                    modifiers,
                );
                self.apply_current_cursor();
                return selection_event.handled;
            }
        }

        if handle == NodeHandle::INVALID {
            if event_type == PointerEventType::Leave {
                self.clear_hover_stack();
            }
            drag_drop::handle_pointer_event(None, event_type, scene_x, scene_y, modifiers);
            self.apply_current_cursor();
            return false;
        }

        let Some(node) = pointed_node.clone() else {
            if event_type == PointerEventType::Leave {
                self.clear_hover_stack();
            }
            drag_drop::handle_pointer_event(None, event_type, scene_x, scene_y, modifiers);
            self.apply_current_cursor();
            return false;
        };
        if event_type == PointerEventType::Enter {
            self.push_hover(handle);
        } else if event_type == PointerEventType::Leave {
            self.pop_hover(handle);
        }
        let handled = self.dispatch_pointer_to_node(
            node,
            handle,
            event_type,
            scene_x,
            scene_y,
            modifiers,
            pointer_id,
            pointer_type,
            button,
            buttons,
            pressure,
            width,
            height,
            click_count,
        );
        drag_drop::handle_pointer_event(pointed_node, event_type, scene_x, scene_y, modifiers);
        self.apply_current_cursor();
        handled
    }

    #[allow(clippy::too_many_arguments)]
    fn dispatch_pointer_to_node(
        &self,
        node: NodeRef,
        target_handle: NodeHandle,
        event_type: PointerEventType,
        scene_x: f32,
        scene_y: f32,
        modifiers: u32,
        pointer_id: i32,
        pointer_type: PointerType,
        button: i32,
        buttons: u32,
        pressure: f32,
        width: f32,
        height: f32,
        click_count: i32,
    ) -> bool {
        let mut event = PointerEventArgs::new(
            target_handle,
            event_type,
            scene_x,
            scene_y,
            modifiers,
            pointer_id,
            pointer_type,
            button,
            buttons,
            pressure,
            width,
            height,
            click_count,
        );
        node.handle_pointer_event(&mut event);
        let mut parent = node.parent();
        while let Some(current) = parent {
            if event.handled {
                break;
            }
            current.handle_bubbled_pointer_event(&mut event);
            parent = current.parent();
        }
        event.handled
    }

    fn capture_pointer(&self, handle: NodeHandle) {
        if self.resolve_node(handle).is_some() {
            self.captured_pointer.replace(Some(handle));
            self.apply_current_cursor();
        }
    }

    fn release_pointer(&self, handle: NodeHandle) {
        let captured = *self.captured_pointer.borrow();
        if captured == Some(handle) {
            self.captured_pointer.replace(None);
            self.apply_current_cursor();
        }
    }

    pub(crate) fn dispatch_wheel_event(
        &self,
        handle: NodeHandle,
        scene_x: f32,
        scene_y: f32,
        delta_x: f32,
        delta_y: f32,
        delta_mode: u32,
        modifiers: u32,
    ) -> bool {
        let Some(mut node) = self.resolve_node(handle) else {
            return false;
        };
        let mut event =
            WheelEventArgs::new(scene_x, scene_y, delta_x, delta_y, delta_mode, modifiers);
        loop {
            node.handle_wheel_event(&mut event);
            if event.handled {
                return true;
            }
            let Some(parent) = node.parent() else {
                return false;
            };
            node = parent;
        }
    }

    pub(crate) fn dispatch_key_event(
        &self,
        event_type: KeyEventType,
        key: String,
        modifiers: u32,
    ) -> bool {
        let key_filters = self
            .key_filters
            .borrow()
            .iter()
            .map(|entry| entry.callback.clone())
            .collect::<Vec<_>>();
        for filter in key_filters.iter().rev() {
            if filter(event_type, key.as_str(), modifiers) {
                return true;
            }
        }
        let Some(handle) = *self.focused.borrow() else {
            return event_type == KeyEventType::Down
                && handle_keyboard_scroll_fallback(key.as_str(), modifiers);
        };
        let Some(mut node) = self.resolve_node(handle) else {
            return event_type == KeyEventType::Down
                && handle_keyboard_scroll_fallback(key.as_str(), modifiers);
        };
        let fallback_key = key.clone();
        let mut event = KeyEventArgs::new(event_type, key, modifiers);
        loop {
            node.handle_key_event(&mut event);
            if event.handled {
                return true;
            }
            let Some(parent) = node.parent() else {
                return event_type == KeyEventType::Down
                    && handle_keyboard_scroll_fallback(fallback_key.as_str(), modifiers);
            };
            node = parent;
        }
    }

    pub(crate) fn dispatch_focus_changed(&self, handle: NodeHandle, focused: bool) {
        if focused {
            self.focused.replace(Some(handle));
        } else if self.focused.borrow().as_ref() == Some(&handle) {
            self.focused.replace(None);
        }

        if let Some(node) = self.resolve_node(handle) {
            node.handle_focus_changed(FocusChangedEventArgs { focused });
        }
    }

    pub(crate) fn dispatch_scroll(
        &self,
        handle: NodeHandle,
        offset_x: f32,
        offset_y: f32,
        content_width: f32,
        content_height: f32,
        viewport_width: f32,
        viewport_height: f32,
    ) {
        tool_tip_manager::ToolTipManager::handle_scroll();
        selection_handle_adorner::refresh_active_geometry();
        mobile_text_selection_toolbar::refresh_active_geometry(
            selection_handle_adorner::is_visible(),
        );
        let hooks = self.scroll_hooks.borrow().clone();
        for hook in hooks {
            hook();
        }
        if let Some(node) = self.resolve_node(handle) {
            node.handle_scroll_changed(
                offset_x,
                offset_y,
                content_width,
                content_height,
                viewport_width,
                viewport_height,
            );
        }
    }

    pub(crate) fn dispatch_selection_changed(&self, handle: NodeHandle, start: u32, end: u32) {
        let Some(node) = self.resolve_node(handle) else {
            selection_handle_adorner::clear();
            mobile_text_selection_toolbar::clear();
            return;
        };
        let (chrome_handle, chrome_node) = selection_chrome_target(handle, &node);
        selection_handle_adorner::handle_selection_changed(chrome_handle, start, end);
        mobile_text_selection_toolbar::handle_selection_changed(
            chrome_handle,
            &chrome_node,
            start,
            end,
            selection_handle_adorner::is_visible(),
        );
        node.handle_selection_changed(start, end);
    }

    pub(crate) fn dispatch_text_changed(&self, handle: NodeHandle, text: String) {
        if let Some(node) = self.resolve_node(handle) {
            node.handle_text_changed(text);
        }
    }

    pub(crate) fn dispatch_text_replaced(
        &self,
        handle: NodeHandle,
        start: u32,
        end: u32,
        text: String,
    ) {
        if let Some(node) = self.resolve_node(handle) {
            node.handle_text_replaced(start, end, text);
        }
    }

    pub(crate) fn dispatch_cross_selection_changed(&self, handle: NodeHandle, text: String) {
        crate::context_menu_manager::handle_selection_changed(&text);
        let Some(node) = self.resolve_node(handle) else {
            selection_handle_adorner::clear();
            mobile_text_selection_toolbar::clear();
            return;
        };
        selection_handle_adorner::handle_cross_selection_changed(handle, &text);
        mobile_text_selection_toolbar::handle_cross_selection_changed(
            handle,
            &node,
            &text,
            selection_handle_adorner::is_visible(),
        );
        node.handle_cross_selection_changed(text);
    }

    pub(crate) fn resolve_gesture_owner(&self, handle: NodeHandle) -> NodeHandle {
        let mut node = self.resolve_node(handle);
        while let Some(current) = node {
            if current.is_effectively_enabled_for_routing()
                && current.is_effectively_visible_for_routing()
                && current.gesture_intent_for_routing() != GestureIntent::None
            {
                return current.handle();
            }
            node = current.parent();
        }
        NodeHandle::INVALID
    }

    pub(crate) fn get_gesture_intent(&self, handle: NodeHandle) -> GestureIntent {
        self.resolve_node(handle)
            .map(|node| node.gesture_intent_for_routing())
            .unwrap_or(GestureIntent::None)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn dispatch_gesture_event(
        &self,
        handle: NodeHandle,
        phase: GestureEventPhase,
        kind: GestureEventKind,
        scene_x: f32,
        scene_y: f32,
        delta_x: f32,
        delta_y: f32,
        scale: f32,
        pointer_count: i32,
    ) -> bool {
        let Some(node) = self.resolve_node(handle) else {
            return false;
        };
        let mut event = GestureEventArgs::new(
            phase,
            kind,
            scene_x,
            scene_y,
            delta_x,
            delta_y,
            scale,
            pointer_count,
        );
        node.handle_gesture_event(&mut event);
        let mut parent = node.parent();
        while let Some(current) = parent {
            if event.handled {
                break;
            }
            current.handle_bubbled_gesture_event(&mut event);
            parent = current.parent();
        }
        event.handled
    }

    pub(crate) fn resolve_long_press_owner(&self, handle: NodeHandle) -> NodeHandle {
        let mut node = self.resolve_node(handle);
        while let Some(current) = node {
            if current.is_effectively_enabled_for_routing()
                && current.is_effectively_visible_for_routing()
                && current.has_long_press_for_routing()
            {
                return current.handle();
            }
            node = current.parent();
        }
        let mut node = self.resolve_node(handle);
        while let Some(current) = node {
            if current.is_effectively_enabled_for_routing()
                && current.is_effectively_visible_for_routing()
                && (current.link_url_for_routing().is_some()
                    || current.is_image_or_svg_for_routing()
                    || self.has_image_context_menu_target_descendant(&current))
            {
                return current.handle();
            }
            if current.is_effectively_enabled_for_routing()
                && current.is_effectively_visible_for_routing()
                && (current.is_selectable_text_for_routing()
                    || current.is_editable_text_for_routing())
            {
                return current.handle();
            }
            node = current.parent();
        }
        NodeHandle::INVALID
    }

    fn has_image_context_menu_target_descendant(&self, node: &NodeRef) -> bool {
        for child in node.children() {
            if child.is_image_or_svg_for_routing()
                || self.has_image_context_menu_target_descendant(&child)
            {
                return true;
            }
        }
        false
    }

    pub(crate) fn get_long_press_minimum_duration_ms(&self, handle: NodeHandle) -> i32 {
        self.resolve_node(handle)
            .map(|node| node.long_press_minimum_duration_ms_for_routing())
            .unwrap_or(DEFAULT_LONG_PRESS_MINIMUM_DURATION_MS)
    }

    pub(crate) fn get_long_press_movement_tolerance(&self, handle: NodeHandle) -> f32 {
        self.resolve_node(handle)
            .map(|node| node.long_press_movement_tolerance_for_routing())
            .unwrap_or(DEFAULT_LONG_PRESS_MOVEMENT_TOLERANCE)
    }

    pub(crate) fn dispatch_long_press_event(
        &self,
        handle: NodeHandle,
        scene_x: f32,
        scene_y: f32,
        pointer_id: i32,
        pointer_type: PointerType,
        modifiers: u32,
        duration_ms: i32,
    ) -> bool {
        let Some(node) = self.resolve_node(handle) else {
            return false;
        };
        let mut event = LongPressEventArgs::new(
            scene_x,
            scene_y,
            pointer_id,
            pointer_type,
            modifiers,
            duration_ms,
        );
        node.handle_long_press_event(&mut event);
        let mut parent = node.parent();
        while let Some(current) = parent {
            if event.handled {
                break;
            }
            current.handle_bubbled_long_press_event(&mut event);
            parent = current.parent();
        }
        if !event.handled
            && matches!(pointer_type, PointerType::Touch | PointerType::Pen)
            && (node.is_selectable_text_for_routing() || node.is_editable_text_for_routing())
        {
            mobile_text_selection_toolbar::set_pending_cross_selection_text_handle(handle);
            event.handled = crate::bindings::ui::select_word_at(handle.raw(), scene_x, scene_y);
            if !event.handled {
                mobile_text_selection_toolbar::set_pending_cross_selection_text_handle(
                    NodeHandle::INVALID,
                );
            }
        }
        if !event.handled {
            event.handled =
                context_menu_manager::show_for_long_press(handle.raw(), scene_x, scene_y);
        }
        event.handled
    }

    pub(crate) fn dispatch_external_drop_event(
        &self,
        handle: NodeHandle,
        event_type: external_drop::ExternalDragEventType,
        x: f32,
        y: f32,
        modifiers: u32,
        items: Vec<external_drop::ExternalDropItemInfo>,
    ) -> crate::drag_drop::DragDropEffects {
        let pointed_node = if handle == NodeHandle::INVALID {
            None
        } else {
            self.resolve_node(handle)
        };
        external_drop::handle_event(pointed_node, event_type, x, y, modifiers, items)
    }

    pub(crate) fn resolve_node(&self, handle: NodeHandle) -> Option<NodeRef> {
        self.nodes
            .borrow()
            .get(&handle)
            .and_then(WeakNodeRef::upgrade)
    }

    fn handle_cursor_style_changed(&self, handle: NodeHandle) {
        let captured = *self.captured_pointer.borrow();
        if captured == Some(handle) {
            self.apply_current_cursor();
            return;
        }
        if captured.is_some() {
            return;
        }
        if self.hover_stack.borrow().last().copied() == Some(handle) {
            self.apply_current_cursor();
        }
    }

    fn push_hover(&self, handle: NodeHandle) {
        let mut hover_stack = self.hover_stack.borrow_mut();
        if let Some(index) = hover_stack.iter().position(|hovered| *hovered == handle) {
            hover_stack.remove(index);
        }
        hover_stack.push(handle);
    }

    fn pop_hover(&self, handle: NodeHandle) {
        let mut hover_stack = self.hover_stack.borrow_mut();
        if let Some(index) = hover_stack.iter().position(|hovered| *hovered == handle) {
            hover_stack.remove(index);
        }
    }

    fn clear_hover_stack(&self) {
        self.hover_stack.borrow_mut().clear();
    }

    fn apply_current_cursor(&self) {
        let drag_cursor = drag_drop::cursor_override_style();
        let style = if drag_cursor != CursorStyle::Default {
            drag_cursor
        } else if let Some(handle) = *self.captured_pointer.borrow() {
            self.resolve_node(handle)
                .map(|node| node.cursor_style_for_routing())
                .unwrap_or(CursorStyle::Default)
        } else if let Some(handle) = *self.selection_cursor_owner.borrow() {
            self.resolve_node(handle)
                .map(|node| node.cursor_style_for_routing())
                .unwrap_or(CursorStyle::Default)
        } else if let Some(handle) = self.hover_stack.borrow().last().copied() {
            self.resolve_node(handle)
                .map(|node| node.cursor_style_for_routing())
                .unwrap_or(CursorStyle::Default)
        } else {
            CursorStyle::Default
        };
        self.apply_cursor(style);
    }

    fn apply_cursor(&self, style: CursorStyle) {
        if self.current_cursor_style.get() == style {
            return;
        }
        self.current_cursor_style.set(style);
        unsafe { crate::ffi::fui_set_cursor(style as u32) };
    }
}

fn selection_cursor_target(node: &NodeRef) -> Option<NodeRef> {
    if !node.is_effectively_enabled_for_routing() || !node.is_effectively_visible_for_routing() {
        return None;
    }
    if node.is_selectable_text_for_routing() || node.is_editable_text_for_routing() {
        return Some(node.clone());
    }
    find_editable_text_descendant(node)
}

fn selection_chrome_target(handle: NodeHandle, node: &NodeRef) -> (NodeHandle, NodeRef) {
    if node.is_selectable_text_for_routing() || node.is_editable_text_for_routing() {
        return (handle, node.clone());
    }
    if let Some(editor) = find_editable_text_descendant(node) {
        return (editor.handle(), editor);
    }
    (handle, node.clone())
}

fn find_editable_text_descendant(node: &NodeRef) -> Option<NodeRef> {
    for child in node.children() {
        if child.is_editable_text_for_routing() {
            return Some(child);
        }
        if let Some(descendant) = find_editable_text_descendant(&child) {
            return Some(descendant);
        }
    }
    None
}

fn preserves_selection_on_pointer_down_for_routing(node: Option<&NodeRef>) -> bool {
    let mut current = node.cloned();
    while let Some(value) = current {
        if value.preserves_selection_on_pointer_down_for_routing() {
            return true;
        }
        current = value.parent();
    }
    false
}

thread_local! {
    static EVENT_ROUTER: EventRouter = EventRouter::new();
}

fn with_event_router<T>(callback: impl FnOnce(&EventRouter) -> T) -> T {
    EVENT_ROUTER.with(callback)
}

pub(crate) fn register_node(node: &NodeRef) {
    with_event_router(|router| router.register(node));
}

pub(crate) fn unregister_node(handle: NodeHandle) {
    with_event_router(|router| router.unregister(handle));
}

pub(crate) fn reset() {
    with_event_router(|router| router.reset());
}

pub(crate) fn resolve_node(handle: NodeHandle) -> Option<NodeRef> {
    with_event_router(|router| router.resolve_node(handle))
}

pub(crate) fn focused_node() -> Option<NodeRef> {
    with_event_router(|router| {
        let handle = *router.focused.borrow();
        handle.and_then(|handle| router.resolve_node(handle))
    })
}

pub(crate) fn capture_pointer(handle: NodeHandle) {
    with_event_router(|router| router.capture_pointer(handle));
}

pub(crate) fn release_pointer(handle: NodeHandle) {
    with_event_router(|router| router.release_pointer(handle));
}

pub(crate) fn handle_cursor_style_changed(handle: NodeHandle) {
    with_event_router(|router| router.handle_cursor_style_changed(handle));
}

pub(crate) fn begin_drag_session(source: NodeRef) -> bool {
    drag_drop::begin_session(source)
}

pub(crate) fn cancel_drag_session(session: crate::drag_drop::DragSession) {
    drag_drop::cancel_session(session);
}

pub(crate) fn cancel_drag_session_for_source(source: &NodeRef) {
    drag_drop::cancel_session_for_source(source);
}

pub(crate) fn dispatch_pointer_event(
    handle: NodeHandle,
    event_type: PointerEventType,
    scene_x: f32,
    scene_y: f32,
    modifiers: u32,
    pointer_id: i32,
    pointer_type: PointerType,
    button: i32,
    buttons: u32,
    pressure: f32,
    width: f32,
    height: f32,
    click_count: i32,
) -> bool {
    focus_visibility::show_keyboard_focus_for_pointer_event(event_type);
    with_event_router(|router| {
        router.dispatch_pointer_event(
            handle,
            event_type,
            scene_x,
            scene_y,
            modifiers,
            pointer_id,
            pointer_type,
            button,
            buttons,
            pressure,
            width,
            height,
            click_count,
        )
    })
}

pub(crate) fn dispatch_wheel_event(
    handle: NodeHandle,
    scene_x: f32,
    scene_y: f32,
    delta_x: f32,
    delta_y: f32,
    delta_mode: u32,
    modifiers: u32,
) -> bool {
    with_event_router(|router| {
        router.dispatch_wheel_event(
            handle, scene_x, scene_y, delta_x, delta_y, delta_mode, modifiers,
        )
    })
}

pub(crate) fn dispatch_key_event(event_type: KeyEventType, key: String, modifiers: u32) -> bool {
    let handled =
        with_event_router(|router| router.dispatch_key_event(event_type, key.clone(), modifiers));
    if !handled || key.as_str() != "Tab" || modifiers != 0 {
        focus_visibility::show_keyboard_focus_for_key_event(event_type, key.as_str(), modifiers);
    }
    handled
}

pub(crate) fn dispatch_focus_changed(handle: NodeHandle, focused: bool) {
    with_event_router(|router| router.dispatch_focus_changed(handle, focused));
}

pub(crate) fn push_key_filter(callback: impl Fn(KeyEventType, &str, u32) -> bool + 'static) -> u32 {
    with_event_router(|router| router.push_key_filter(Rc::new(callback)))
}

pub(crate) fn remove_key_filter(token: u32) {
    with_event_router(|router| router.remove_key_filter(token));
}

pub(crate) fn register_scroll_hook(callback: impl Fn() + 'static) {
    with_event_router(|router| router.register_scroll_hook(Rc::new(callback)));
}

pub(crate) fn focused_node_is_enabled_button() -> bool {
    with_event_router(|router| router.focused_node_is_enabled_button())
}

pub(crate) fn dispatch_scroll(
    handle: NodeHandle,
    offset_x: f32,
    offset_y: f32,
    content_width: f32,
    content_height: f32,
    viewport_width: f32,
    viewport_height: f32,
) {
    with_event_router(|router| {
        router.dispatch_scroll(
            handle,
            offset_x,
            offset_y,
            content_width,
            content_height,
            viewport_width,
            viewport_height,
        )
    });
}

pub(crate) fn dispatch_selection_changed(handle: NodeHandle, start: u32, end: u32) {
    with_event_router(|router| router.dispatch_selection_changed(handle, start, end));
}

pub(crate) fn dispatch_text_changed(handle: NodeHandle, text: String) {
    with_event_router(|router| router.dispatch_text_changed(handle, text));
}

pub(crate) fn dispatch_text_replaced(handle: NodeHandle, start: u32, end: u32, text: String) {
    with_event_router(|router| router.dispatch_text_replaced(handle, start, end, text));
}

pub(crate) fn dispatch_cross_selection_changed(handle: NodeHandle, text: String) {
    with_event_router(|router| router.dispatch_cross_selection_changed(handle, text));
}

pub(crate) fn resolve_gesture_owner(handle: NodeHandle) -> NodeHandle {
    with_event_router(|router| router.resolve_gesture_owner(handle))
}

pub(crate) fn get_gesture_intent(handle: NodeHandle) -> GestureIntent {
    with_event_router(|router| router.get_gesture_intent(handle))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn dispatch_gesture_event(
    handle: NodeHandle,
    phase: GestureEventPhase,
    kind: GestureEventKind,
    scene_x: f32,
    scene_y: f32,
    delta_x: f32,
    delta_y: f32,
    scale: f32,
    pointer_count: i32,
) -> bool {
    with_event_router(|router| {
        router.dispatch_gesture_event(
            handle,
            phase,
            kind,
            scene_x,
            scene_y,
            delta_x,
            delta_y,
            scale,
            pointer_count,
        )
    })
}

pub(crate) fn resolve_long_press_owner(handle: NodeHandle) -> NodeHandle {
    with_event_router(|router| router.resolve_long_press_owner(handle))
}

pub(crate) fn registered_node(handle: u64) -> Option<NodeRef> {
    with_event_router(|router| router.resolve_node(NodeHandle::from_raw(handle)))
}

pub(crate) fn get_long_press_minimum_duration_ms(handle: NodeHandle) -> i32 {
    with_event_router(|router| router.get_long_press_minimum_duration_ms(handle))
}

pub(crate) fn get_long_press_movement_tolerance(handle: NodeHandle) -> f32 {
    with_event_router(|router| router.get_long_press_movement_tolerance(handle))
}

pub(crate) fn dispatch_long_press_event(
    handle: NodeHandle,
    scene_x: f32,
    scene_y: f32,
    pointer_id: i32,
    pointer_type: PointerType,
    modifiers: u32,
    duration_ms: i32,
) -> bool {
    with_event_router(|router| {
        router.dispatch_long_press_event(
            handle,
            scene_x,
            scene_y,
            pointer_id,
            pointer_type,
            modifiers,
            duration_ms,
        )
    })
}

pub(crate) fn dispatch_external_drop_event(
    handle: NodeHandle,
    event_type: external_drop::ExternalDragEventType,
    x: f32,
    y: f32,
    modifiers: u32,
    items: Vec<external_drop::ExternalDropItemInfo>,
) -> crate::drag_drop::DragDropEffects {
    with_event_router(|router| {
        router.dispatch_external_drop_event(handle, event_type, x, y, modifiers, items)
    })
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
pub extern "C" fn __fui_key_buffer() -> *const u8 {
    std::ptr::addr_of_mut!(KEY_BUFFER).cast::<u8>()
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
pub extern "C" fn __fui_key_buffer_size() -> u32 {
    256
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
pub extern "C" fn __fui_text_buffer() -> *const u8 {
    std::ptr::addr_of_mut!(TEXT_BUFFER).cast::<u8>()
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
pub extern "C" fn __fui_text_buffer_size() -> u32 {
    (16 * 1024) as u32
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
pub extern "C" fn __fui_on_pointer_event_with_metadata(
    event_type: u32,
    handle: u64,
    x: f32,
    y: f32,
    modifiers: u32,
    pointer_id: i32,
    pointer_type: u32,
    button: i32,
    buttons: u32,
    pressure: f32,
    width: f32,
    height: f32,
    click_count: i32,
) -> bool {
    let routed_event_type = match event_type {
        1 => PointerEventType::Down,
        2 => PointerEventType::Up,
        3 => PointerEventType::Move,
        4 => PointerEventType::Enter,
        5 => PointerEventType::Leave,
        _ => PointerEventType::Cancel,
    };
    crate::context_menu_manager::handle_pointer_selection_event(
        routed_event_type == PointerEventType::Down,
        handle,
    );
    dispatch_pointer_event(
        NodeHandle::from_raw(handle),
        routed_event_type,
        x,
        y,
        modifiers,
        pointer_id,
        PointerType::from_raw(pointer_type),
        button,
        buttons,
        pressure,
        width,
        height,
        click_count,
    )
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
pub extern "C" fn __fui_on_wheel_event(
    handle: u64,
    x: f32,
    y: f32,
    delta_x: f32,
    delta_y: f32,
    delta_mode: u32,
    modifiers: u32,
) -> bool {
    dispatch_wheel_event(
        NodeHandle::from_raw(handle),
        x,
        y,
        delta_x,
        delta_y,
        delta_mode,
        modifiers,
    )
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
/// # Safety
/// `key_ptr` must be null for an empty key or point to `key_len` readable bytes.
pub unsafe extern "C" fn __fui_on_key_event(
    event_type: u32,
    key_ptr: *const u8,
    key_len: u32,
    modifiers: u32,
) -> bool {
    let key = if key_ptr.is_null() || key_len == 0 {
        String::new()
    } else {
        let bytes = unsafe { std::slice::from_raw_parts(key_ptr, key_len as usize) };
        String::from_utf8_lossy(bytes).into_owned()
    };
    dispatch_key_event(
        match event_type {
            2 => KeyEventType::Up,
            _ => KeyEventType::Down,
        },
        key,
        modifiers,
    )
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
pub extern "C" fn __fui_on_focus_changed(handle: u64, focused: bool) {
    dispatch_focus_changed(NodeHandle::from_raw(handle), focused);
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
/// # Safety
/// `text_ptr` must be null for empty text or point to `text_len` readable bytes.
pub unsafe extern "C" fn __fui_on_text_changed(handle: u64, text_ptr: *const u8, text_len: u32) {
    let text = if text_ptr.is_null() || text_len == 0 {
        String::new()
    } else {
        let bytes = unsafe { std::slice::from_raw_parts(text_ptr, text_len as usize) };
        String::from_utf8_lossy(bytes).into_owned()
    };
    dispatch_text_changed(NodeHandle::from_raw(handle), text);
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
/// # Safety
/// `text_ptr` must be null for empty replacement text or point to `text_len` readable bytes.
pub unsafe extern "C" fn __fui_on_text_replaced(
    handle: u64,
    start: u32,
    end: u32,
    text_ptr: *const u8,
    text_len: u32,
) {
    let text = if text_ptr.is_null() || text_len == 0 {
        String::new()
    } else {
        let bytes = unsafe { std::slice::from_raw_parts(text_ptr, text_len as usize) };
        String::from_utf8_lossy(bytes).into_owned()
    };
    dispatch_text_replaced(NodeHandle::from_raw(handle), start, end, text);
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
pub extern "C" fn __fui_on_selection_changed(handle: u64, start: u32, end: u32) {
    dispatch_selection_changed(NodeHandle::from_raw(handle), start, end);
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
/// # Safety
/// `text_ptr` must be null for empty text or point to `text_len` readable bytes.
pub unsafe extern "C" fn __fui_on_cross_selection_changed(
    handle: u64,
    text_ptr: *const u8,
    text_len: u32,
) {
    let text = if text_ptr.is_null() || text_len == 0 {
        String::new()
    } else {
        let bytes = unsafe { std::slice::from_raw_parts(text_ptr, text_len as usize) };
        String::from_utf8_lossy(bytes).into_owned()
    };
    dispatch_cross_selection_changed(NodeHandle::from_raw(handle), text);
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
pub extern "C" fn __fui_resolve_gesture_owner(handle: u64) -> u64 {
    resolve_gesture_owner(NodeHandle::from_raw(handle)).raw()
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
pub extern "C" fn __fui_get_gesture_intent(handle: u64) -> u32 {
    get_gesture_intent(NodeHandle::from_raw(handle)) as u32
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
pub extern "C" fn __fui_on_gesture_event(
    handle: u64,
    phase: u32,
    kind: u32,
    scene_x: f32,
    scene_y: f32,
    delta_x: f32,
    delta_y: f32,
    scale: f32,
    pointer_count: i32,
) -> bool {
    dispatch_gesture_event(
        NodeHandle::from_raw(handle),
        GestureEventPhase::from_raw(phase),
        GestureEventKind::from_raw(kind),
        scene_x,
        scene_y,
        delta_x,
        delta_y,
        scale,
        pointer_count,
    )
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
pub extern "C" fn __fui_resolve_long_press_owner(handle: u64) -> u64 {
    resolve_long_press_owner(NodeHandle::from_raw(handle)).raw()
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
pub extern "C" fn __fui_get_long_press_minimum_duration_ms(handle: u64) -> i32 {
    get_long_press_minimum_duration_ms(NodeHandle::from_raw(handle))
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
pub extern "C" fn __fui_get_long_press_movement_tolerance(handle: u64) -> f32 {
    get_long_press_movement_tolerance(NodeHandle::from_raw(handle))
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
pub extern "C" fn __fui_long_press_continues_pointer_events(handle: u64) -> bool {
    registered_node(handle).is_some_and(|node| node.has_drag_source())
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
pub extern "C" fn __fui_on_long_press_event(
    handle: u64,
    scene_x: f32,
    scene_y: f32,
    pointer_id: i32,
    pointer_type: u32,
    modifiers: u32,
    duration_ms: i32,
) -> bool {
    dispatch_long_press_event(
        NodeHandle::from_raw(handle),
        scene_x,
        scene_y,
        pointer_id,
        PointerType::from_raw(pointer_type),
        modifiers,
        duration_ms,
    )
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
/// # Safety
/// `payload_ptr` must be null for an empty payload or point to `payload_len` readable bytes.
pub unsafe extern "C" fn __fui_on_external_drag_event(
    event_type: u32,
    handle: u64,
    x: f32,
    y: f32,
    modifiers: u32,
    payload_ptr: *const u8,
    payload_len: u32,
) -> u32 {
    let items = external_drop::decode_payload(payload_ptr, payload_len);
    let effect = dispatch_external_drop_event(
        NodeHandle::from_raw(handle),
        external_drop::ExternalDragEventType::from_raw(event_type),
        x,
        y,
        modifiers,
        items.clone(),
    );
    if !payload_ptr.is_null() && payload_len > 0 && items.is_empty() {
        crate::logger::error(
            "ExternalDrop",
            &format!(
                "Dropped malformed external payload for handle {}.",
                NodeHandle::from_raw(handle).raw()
            ),
        );
    }
    effect as u32
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
pub extern "C" fn fui_dispatch_custom_draw(handle: u64, canvas_ptr: usize) {
    if let Some(node) = resolve_node(NodeHandle::from_raw(handle)) {
        node.handle_custom_draw(canvas_ptr);
    }
}
