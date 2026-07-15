use crate::drag_drop::{DragDropEffects, DropProposal};
use crate::file::{register_browser_file, BrowserFile};
use crate::node::NodeRef;
use std::cell::RefCell;
#[cfg(feature = "native-runtime")]
use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExternalDragEventType {
    Enter = 1,
    Over = 2,
    Leave = 3,
    Drop = 4,
    Unknown,
}

impl ExternalDragEventType {
    pub(crate) fn from_raw(value: u32) -> Self {
        match value {
            1 => Self::Enter,
            2 => Self::Over,
            3 => Self::Leave,
            4 => Self::Drop,
            _ => Self::Unknown,
        }
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExternalDropItemKind {
    File = 1,
    Text = 2,
    Uri = 3,
    Unknown(u32),
}

impl ExternalDropItemKind {
    pub(crate) fn from_raw(value: u32) -> Self {
        match value {
            1 => Self::File,
            2 => Self::Text,
            3 => Self::Uri,
            _ => Self::Unknown(value),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ExternalDropItemInfo {
    pub id: String,
    pub kind: ExternalDropItemKind,
    pub name: String,
    pub mime_type: Option<String>,
    pub size_bytes: f64,
    pub file: Option<BrowserFile>,
}

impl ExternalDropItemInfo {
    pub fn new(
        id: impl Into<String>,
        kind: ExternalDropItemKind,
        name: impl Into<String>,
        mime_type: Option<String>,
        size_bytes: f64,
        file: Option<BrowserFile>,
    ) -> Self {
        Self {
            id: id.into(),
            kind,
            name: name.into(),
            mime_type,
            size_bytes,
            file,
        }
    }

    #[cfg(feature = "native-runtime")]
    pub fn native_path(&self) -> Option<&Path> {
        (self.kind == ExternalDropItemKind::File).then(|| Path::new(&self.id))
    }
}

#[derive(Clone, Debug)]
pub struct ExternalDropEventArgs {
    pub x: f32,
    pub y: f32,
    pub modifiers: u32,
    pub items: Vec<ExternalDropItemInfo>,
}

impl ExternalDropEventArgs {
    pub fn new(x: f32, y: f32, modifiers: u32, items: Vec<ExternalDropItemInfo>) -> Self {
        Self {
            x,
            y,
            modifiers,
            items,
        }
    }
}

#[derive(Default)]
struct ExternalDropState {
    active_target: Option<NodeRef>,
    active_effect: DragDropEffects,
}

thread_local! {
    static STATE: RefCell<ExternalDropState> = RefCell::new(ExternalDropState::default());
}

fn is_default_proposal(proposal: DropProposal) -> bool {
    proposal.effect == DragDropEffects::None && !proposal.show_insertion_marker
}

fn normalize_effect(candidate: DragDropEffects) -> DragDropEffects {
    let masked = (candidate as u32)
        & ((DragDropEffects::Copy as u32)
            | (DragDropEffects::Move as u32)
            | (DragDropEffects::Link as u32));
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

fn resolve_drop_target(pointed_node: Option<NodeRef>) -> Option<NodeRef> {
    let mut current = pointed_node;
    while let Some(node) = current {
        if node.allows_external_drop() {
            return Some(node);
        }
        current = node.parent();
    }
    None
}

fn finish(
    state: &mut ExternalDropState,
    x: f32,
    y: f32,
    modifiers: u32,
    items: &[ExternalDropItemInfo],
    notify_target_leave: bool,
) {
    let target = state.active_target.take();
    state.active_effect = DragDropEffects::None;
    if notify_target_leave {
        if let Some(target) = target {
            target.handle_external_drag_leave(ExternalDropEventArgs::new(
                x,
                y,
                modifiers,
                items.to_vec(),
            ));
        }
    }
}

pub(crate) fn handle_event(
    pointed_node: Option<NodeRef>,
    event_type: ExternalDragEventType,
    x: f32,
    y: f32,
    modifiers: u32,
    items: Vec<ExternalDropItemInfo>,
) -> DragDropEffects {
    STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        if event_type == ExternalDragEventType::Leave {
            finish(&mut state, x, y, modifiers, &items, true);
            return DragDropEffects::None;
        }

        let target = resolve_drop_target(pointed_node);
        let args = ExternalDropEventArgs::new(x, y, modifiers, items.clone());
        let mut proposal = DropProposal::none();
        let target_changed = match (&target, &state.active_target) {
            (Some(target), Some(active)) => target.handle() != active.handle(),
            (None, None) => false,
            _ => true,
        };
        if target_changed {
            if let Some(previous_target) = state.active_target.take() {
                previous_target.handle_external_drag_leave(args.clone());
            }
            state.active_target = target.clone();
            state.active_effect = DragDropEffects::None;
            if let Some(target) = target.as_ref() {
                if target.has_external_drag_enter_handler() {
                    proposal = target.handle_external_drag_enter(args.clone());
                }
            }
        }

        let Some(target) = target else {
            state.active_effect = DragDropEffects::None;
            return DragDropEffects::None;
        };

        if target.has_external_drag_over_handler() {
            proposal = target.handle_external_drag_over(args.clone());
        } else if is_default_proposal(proposal) {
            proposal = DropProposal::new(state.active_effect, false);
        }

        let effect = normalize_effect(proposal.effect);
        state.active_effect = effect;
        if event_type == ExternalDragEventType::Drop {
            if effect != DragDropEffects::None {
                target.handle_external_drop_event(args);
            }
            finish(&mut state, x, y, modifiers, &items, true);
        }
        effect
    })
}

pub(crate) fn handle_node_destroyed(node: NodeRef) {
    STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        let Some(active_target) = state.active_target.as_ref() else {
            return;
        };
        if active_target.handle() == node.handle() {
            state.active_target = None;
            state.active_effect = DragDropEffects::None;
        }
    });
}

pub(crate) fn reset() {
    STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        state.active_target = None;
        state.active_effect = DragDropEffects::None;
    });
}

pub(crate) fn decode_payload(
    payload_ptr: *const u8,
    payload_len: u32,
) -> Vec<ExternalDropItemInfo> {
    let mut items = Vec::new();
    if payload_ptr.is_null() || payload_len == 0 {
        return items;
    }
    let bytes = unsafe { std::slice::from_raw_parts(payload_ptr, payload_len as usize) };
    if bytes.len() < 4 {
        crate::logger::warn("ExternalDrop", "Malformed external drop payload header.");
        return items;
    }
    let mut cursor = 0usize;
    let item_count = u32::from_le_bytes(bytes[cursor..cursor + 4].try_into().unwrap_or([0; 4]));
    cursor += 4;
    for index in 0..item_count {
        if cursor + 12 > bytes.len() {
            crate::logger::warn(
                "ExternalDrop",
                &format!("Truncated external drop item header at index {}.", index),
            );
            return items;
        }
        let kind = ExternalDropItemKind::from_raw(u32::from_le_bytes(
            bytes[cursor..cursor + 4].try_into().unwrap_or([0; 4]),
        ));
        cursor += 4;
        let size_bytes = f64::from_le_bytes(bytes[cursor..cursor + 8].try_into().unwrap_or([0; 8]));
        cursor += 8;

        let Some(id) = decode_string(bytes, &mut cursor, "id", index) else {
            return items;
        };
        let Some(name) = decode_string(bytes, &mut cursor, "name", index) else {
            return items;
        };
        let Some(mime_type) = decode_string(bytes, &mut cursor, "mime", index) else {
            return items;
        };
        let mime_type = if mime_type.is_empty() {
            None
        } else {
            Some(mime_type)
        };
        let file = if kind == ExternalDropItemKind::File && !id.is_empty() {
            Some(register_browser_file(
                id.clone(),
                name.clone(),
                mime_type.clone(),
                size_bytes as u64,
                0,
            ))
        } else {
            None
        };
        items.push(ExternalDropItemInfo::new(
            id, kind, name, mime_type, size_bytes, file,
        ));
    }
    items
}

fn decode_string(bytes: &[u8], cursor: &mut usize, label: &str, index: u32) -> Option<String> {
    if *cursor + 4 > bytes.len() {
        crate::logger::warn(
            "ExternalDrop",
            &format!(
                "Truncated external drop item {} length at index {}.",
                label, index
            ),
        );
        return None;
    }
    let len = u32::from_le_bytes(bytes[*cursor..*cursor + 4].try_into().unwrap_or([0; 4])) as usize;
    *cursor += 4;
    if *cursor + len > bytes.len() {
        crate::logger::warn(
            "ExternalDrop",
            &format!("Truncated external drop item {} at index {}.", label, index),
        );
        return None;
    }
    let value = String::from_utf8_lossy(&bytes[*cursor..*cursor + len]).into_owned();
    *cursor += len;
    Some(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drag_drop::DropProposal;
    use crate::event::reset as reset_events;
    use crate::node::{flex_box, Node};
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    #[test]
    fn routes_external_drop_enter_over_leave_and_drop() {
        reset();
        reset_events();

        let root = flex_box();
        let target = flex_box();
        root.child(&target);

        let enter_count = Rc::new(Cell::new(0));
        let over_count = Rc::new(Cell::new(0));
        let leave_count = Rc::new(Cell::new(0));
        let drop_count = Rc::new(Cell::new(0));
        let last_name = Rc::new(RefCell::new(String::new()));

        target
            .allow_external_drop(true)
            .on_external_drag_enter({
                let enter_count = enter_count.clone();
                let last_name = last_name.clone();
                move |args| {
                    enter_count.set(enter_count.get() + 1);
                    if let Some(item) = args.items.first() {
                        last_name.replace(item.name.clone());
                    }
                    DropProposal::new(DragDropEffects::Copy, false)
                }
            })
            .on_external_drag_over({
                let over_count = over_count.clone();
                move |_args| {
                    over_count.set(over_count.get() + 1);
                    DropProposal::new(DragDropEffects::Copy, false)
                }
            })
            .on_external_drag_leave({
                let leave_count = leave_count.clone();
                move |_args| {
                    leave_count.set(leave_count.get() + 1);
                }
            })
            .on_external_drop({
                let drop_count = drop_count.clone();
                move |_args| {
                    drop_count.set(drop_count.get() + 1);
                }
            });

        let items = vec![ExternalDropItemInfo::new(
            "external-drop-1",
            ExternalDropItemKind::File,
            "todo.txt",
            Some("text/plain".to_string()),
            10.0,
            None,
        )];

        assert_eq!(
            handle_event(
                Some(target.node_ref()),
                ExternalDragEventType::Enter,
                12.0,
                18.0,
                0,
                items.clone(),
            ),
            DragDropEffects::Copy
        );
        assert_eq!(
            handle_event(
                Some(target.node_ref()),
                ExternalDragEventType::Over,
                14.0,
                19.0,
                0,
                items.clone(),
            ),
            DragDropEffects::Copy
        );
        assert_eq!(
            handle_event(
                Some(target.node_ref()),
                ExternalDragEventType::Drop,
                16.0,
                20.0,
                0,
                items,
            ),
            DragDropEffects::Copy
        );

        assert_eq!(enter_count.get(), 1);
        assert_eq!(over_count.get(), 3);
        assert_eq!(leave_count.get(), 1);
        assert_eq!(drop_count.get(), 1);
        assert_eq!(last_name.borrow().as_str(), "todo.txt");
    }

    #[test]
    fn decodes_unknown_external_item_kind_without_registering_file() {
        reset();
        reset_events();

        let mut payload = Vec::new();
        payload.extend_from_slice(&1u32.to_le_bytes());
        payload.extend_from_slice(&99u32.to_le_bytes());
        payload.extend_from_slice(&123.0f64.to_le_bytes());
        for value in ["external-drop-unknown", "note.txt", "text/plain"] {
            payload.extend_from_slice(&(value.len() as u32).to_le_bytes());
            payload.extend_from_slice(value.as_bytes());
        }

        let items = decode_payload(payload.as_ptr(), payload.len() as u32);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].kind, ExternalDropItemKind::Unknown(99));
        assert_eq!(items[0].id, "external-drop-unknown");
        assert_eq!(items[0].name, "note.txt");
        assert_eq!(items[0].mime_type.as_deref(), Some("text/plain"));
        assert!(items[0].file.is_none());
    }
}
