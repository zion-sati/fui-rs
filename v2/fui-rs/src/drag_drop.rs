use crate::ffi::CursorStyle;
use crate::ffi::PointerEventType;
use crate::node::NodeRef;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

const DRAG_DROP_TEXT_FORMAT: &str = "text/plain";

#[repr(u32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DragDropEffects {
    #[default]
    None = 0,
    Copy = 1,
    Move = 2,
    Link = 4,
}

fn normalize_effect(candidate: DragDropEffects, allowed: DragDropEffects) -> DragDropEffects {
    let masked = (candidate as u32) & (allowed as u32);
    if masked == DragDropEffects::None as u32 {
        return DragDropEffects::None;
    }
    if (masked & DragDropEffects::Move as u32) != 0 {
        return DragDropEffects::Move;
    }
    if (masked & DragDropEffects::Copy as u32) != 0 {
        return DragDropEffects::Copy;
    }
    if (masked & DragDropEffects::Link as u32) != 0 {
        return DragDropEffects::Link;
    }
    DragDropEffects::None
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DragDataObject {
    formats: HashMap<String, String>,
}

impl DragDataObject {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_text(mut self, value: impl Into<String>) -> Self {
        self.formats
            .insert(String::from(DRAG_DROP_TEXT_FORMAT), value.into());
        self
    }

    pub fn set_format(mut self, format: impl Into<String>, value: impl Into<String>) -> Self {
        self.formats.insert(format.into(), value.into());
        self
    }

    pub fn has_format(&self, format: &str) -> bool {
        self.formats.contains_key(format)
    }

    pub fn get_text(&self) -> Option<String> {
        self.get_format(DRAG_DROP_TEXT_FORMAT)
    }

    pub fn get_format(&self, format: &str) -> Option<String> {
        self.formats.get(format).cloned()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DragCompletedEventArgs {
    pub effect: DragDropEffects,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DropProposal {
    pub effect: DragDropEffects,
    pub show_insertion_marker: bool,
}

impl DropProposal {
    pub fn new(effect: DragDropEffects, show_insertion_marker: bool) -> Self {
        Self {
            effect,
            show_insertion_marker,
        }
    }

    pub fn none() -> Self {
        Self::default()
    }
}

type DragCompletedCallback = Rc<dyn Fn(DragCompletedEventArgs)>;

struct DragSessionState {
    source: NodeRef,
    current_effect: DragDropEffects,
    active: bool,
    completed_callback: Option<DragCompletedCallback>,
}

#[derive(Clone)]
pub struct DragSession {
    pub data: DragDataObject,
    pub allowed_effects: DragDropEffects,
    inner: Rc<RefCell<DragSessionState>>,
}

impl DragSession {
    pub(crate) fn new(
        source: NodeRef,
        data: DragDataObject,
        allowed_effects: DragDropEffects,
    ) -> Self {
        Self {
            data,
            allowed_effects,
            inner: Rc::new(RefCell::new(DragSessionState {
                source,
                current_effect: DragDropEffects::None,
                active: true,
                completed_callback: None,
            })),
        }
    }

    pub fn current_effect(&self) -> DragDropEffects {
        self.inner.borrow().current_effect
    }

    pub fn is_active(&self) -> bool {
        self.inner.borrow().active
    }

    pub fn on_completed(&self, callback: impl Fn(DragCompletedEventArgs) + 'static) -> &Self {
        self.inner.borrow_mut().completed_callback = Some(Rc::new(callback));
        self
    }

    pub fn cancel(&self) {
        if !self.is_active() {
            return;
        }
        crate::event::cancel_drag_session(self.clone());
    }

    pub(crate) fn source(&self) -> NodeRef {
        self.inner.borrow().source.clone()
    }

    pub(crate) fn set_current_effect(&self, effect: DragDropEffects) {
        self.inner.borrow_mut().current_effect = effect;
    }

    pub(crate) fn complete(&self, effect: DragDropEffects) {
        let callback = {
            let mut state = self.inner.borrow_mut();
            if !state.active {
                return;
            }
            state.active = false;
            state.current_effect = effect;
            state.completed_callback.clone()
        };
        if let Some(callback) = callback {
            callback(DragCompletedEventArgs { effect });
        }
    }
}

#[derive(Clone)]
pub struct DragEventArgs {
    pub session: DragSession,
    pub x: f32,
    pub y: f32,
    pub modifiers: u32,
}

impl DragEventArgs {
    pub fn new(session: DragSession, x: f32, y: f32, modifiers: u32) -> Self {
        Self {
            session,
            x,
            y,
            modifiers,
        }
    }
}

#[derive(Default)]
struct DragDropState {
    active_session: Option<DragSession>,
    active_target: Option<NodeRef>,
}

thread_local! {
    static STATE: RefCell<DragDropState> = RefCell::new(DragDropState::default());
}

fn is_default_proposal(proposal: DropProposal) -> bool {
    proposal.effect == DragDropEffects::None && !proposal.show_insertion_marker
}

fn with_state<T>(callback: impl FnOnce(&mut DragDropState) -> T) -> T {
    STATE.with(|slot| callback(&mut slot.borrow_mut()))
}

pub(crate) fn cursor_override_style() -> CursorStyle {
    STATE.with(|slot| {
        let state = slot.borrow();
        let Some(session) = state.active_session.as_ref() else {
            return CursorStyle::Default;
        };
        if !session.is_active() {
            return CursorStyle::Default;
        }
        if session.current_effect() == DragDropEffects::None {
            CursorStyle::Grabbing
        } else {
            CursorStyle::Move
        }
    })
}

pub(crate) fn begin_session(source: NodeRef) -> bool {
    let existing = STATE.with(|slot| slot.borrow().active_session.clone());
    if let Some(existing) = existing {
        finish_session(existing, DragDropEffects::None, 0.0, 0.0, 0, true);
    }
    if !source.has_drag_source() {
        return false;
    }
    let Some(data) = source.create_drag_data_object() else {
        return false;
    };
    let allowed = source.get_drag_allowed_effects();
    if allowed == DragDropEffects::None {
        return false;
    }
    with_state(|state| {
        state.active_target = None;
        state.active_session = Some(DragSession::new(source, data, allowed));
    });
    true
}

pub(crate) fn cancel_session(session: DragSession) {
    let is_active = STATE.with(|slot| {
        slot.borrow()
            .active_session
            .as_ref()
            .map(|active| Rc::ptr_eq(&active.inner, &session.inner))
            .unwrap_or(false)
    });
    if !is_active {
        return;
    }
    finish_session(session, DragDropEffects::None, 0.0, 0.0, 0, true);
}

pub(crate) fn cancel_session_for_source(source: &NodeRef) {
    let active = STATE.with(|slot| slot.borrow().active_session.clone());
    let Some(session) = active else {
        return;
    };
    if !session.source().ptr_eq(source) {
        return;
    }
    finish_session(session, DragDropEffects::None, 0.0, 0.0, 0, true);
}

pub(crate) fn handle_node_destroyed(node: NodeRef) {
    let active = STATE.with(|slot| {
        let state = slot.borrow();
        (state.active_session.clone(), state.active_target.clone())
    });
    if let Some(session) = active.0 {
        if session.source().ptr_eq(&node) {
            finish_session(session, DragDropEffects::None, 0.0, 0.0, 0, true);
            return;
        }
    }
    if let Some(target) = active.1 {
        if target.ptr_eq(&node) {
            with_state(|state| {
                state.active_target = None;
                if let Some(session) = state.active_session.as_ref() {
                    session.set_current_effect(DragDropEffects::None);
                }
            });
        }
    }
}

pub(crate) fn handle_pointer_event(
    pointed_node: Option<NodeRef>,
    event_type: PointerEventType,
    x: f32,
    y: f32,
    modifiers: u32,
) {
    let active = STATE.with(|slot| slot.borrow().active_session.clone());
    let Some(session) = active else {
        return;
    };
    if !session.is_active() {
        return;
    }
    match event_type {
        PointerEventType::Down
        | PointerEventType::Enter
        | PointerEventType::Move
        | PointerEventType::Leave => {
            update_target(pointed_node, session, x, y, modifiers);
        }
        PointerEventType::Up => {
            let effect = update_target(pointed_node, session.clone(), x, y, modifiers);
            let target = STATE.with(|slot| slot.borrow().active_target.clone());
            if let Some(target) = target {
                if effect != DragDropEffects::None {
                    target.handle_drop_event(DragEventArgs::new(session.clone(), x, y, modifiers));
                }
            }
            finish_session(session, effect, x, y, modifiers, true);
        }
        PointerEventType::Cancel => {}
    }
}

pub(crate) fn reset() {
    with_state(|state| {
        state.active_session = None;
        state.active_target = None;
    });
}

fn update_target(
    pointed_node: Option<NodeRef>,
    session: DragSession,
    x: f32,
    y: f32,
    modifiers: u32,
) -> DragDropEffects {
    let target = resolve_drop_target(pointed_node);
    let args = DragEventArgs::new(session.clone(), x, y, modifiers);
    let mut proposal = DropProposal::none();
    let previous_target = STATE.with(|slot| slot.borrow().active_target.clone());
    let target_changed = match (&target, &previous_target) {
        (Some(left), Some(right)) => !left.ptr_eq(right),
        (None, None) => false,
        _ => true,
    };
    if target_changed {
        if let Some(previous_target) = previous_target {
            previous_target.handle_drag_leave(args.clone());
        }
        with_state(|state| {
            state.active_target = target.clone();
        });
        if let Some(target) = target.as_ref() {
            if target.has_drag_enter_handler() {
                proposal = target.handle_drag_enter(args.clone());
            }
        }
    }
    let Some(target) = target else {
        session.set_current_effect(DragDropEffects::None);
        return DragDropEffects::None;
    };
    if target.has_drag_over_handler() {
        proposal = target.handle_drag_over(args);
    } else if !target_changed && is_default_proposal(proposal) {
        return session.current_effect();
    }
    let effect = normalize_effect(proposal.effect, session.allowed_effects);
    session.set_current_effect(effect);
    effect
}

fn resolve_drop_target(pointed_node: Option<NodeRef>) -> Option<NodeRef> {
    let mut current = pointed_node;
    while let Some(node) = current {
        if node.allows_drop() {
            return Some(node);
        }
        current = node.parent();
    }
    None
}

fn finish_session(
    session: DragSession,
    effect: DragDropEffects,
    x: f32,
    y: f32,
    modifiers: u32,
    notify_target_leave: bool,
) {
    let target = with_state(|state| {
        let Some(active) = state.active_session.as_ref() else {
            return None;
        };
        if !Rc::ptr_eq(&active.inner, &session.inner) {
            return None;
        }
        let target = state.active_target.clone();
        state.active_session = None;
        state.active_target = None;
        Some(target)
    });
    let Some(target) = target else {
        return;
    };
    session.set_current_effect(effect);
    if notify_target_leave {
        if let Some(target) = target {
            target.handle_drag_leave(DragEventArgs::new(session.clone(), x, y, modifiers));
        }
    }
    session.complete(effect);
    session.source().notify_drag_completed(effect);
}
