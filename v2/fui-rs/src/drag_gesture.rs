use crate::event;
use crate::node::{NodeHandle, NodeRef, WeakNodeRef};

const DEFAULT_DRAG_THRESHOLD: f32 = 4.0;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct DragStartedEvent {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) modifiers: u32,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct DragCompletedEvent {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) total_delta_x: f32,
    pub(crate) total_delta_y: f32,
    pub(crate) modifiers: u32,
    pub(crate) cancelled: bool,
}

type DragStartedCallback = Box<dyn Fn(DragStartedEvent)>;
type DragCompletedCallback = Box<dyn Fn(DragCompletedEvent)>;

pub(crate) struct DragGesture {
    host: WeakNodeRef,
    threshold_value: f32,
    pointer_down_value: bool,
    drag_started_value: bool,
    start_x: f32,
    start_y: f32,
    last_pointer_x: f32,
    last_pointer_y: f32,
    last_dispatched_x: f32,
    last_dispatched_y: f32,
    last_modifiers: u32,
    started_callback: Option<DragStartedCallback>,
    completed_callback: Option<DragCompletedCallback>,
}

impl DragGesture {
    pub(crate) fn new(host: &NodeRef) -> Self {
        Self {
            host: host.downgrade(),
            threshold_value: DEFAULT_DRAG_THRESHOLD,
            pointer_down_value: false,
            drag_started_value: false,
            start_x: 0.0,
            start_y: 0.0,
            last_pointer_x: 0.0,
            last_pointer_y: 0.0,
            last_dispatched_x: 0.0,
            last_dispatched_y: 0.0,
            last_modifiers: 0,
            started_callback: None,
            completed_callback: None,
        }
    }

    pub(crate) fn threshold(&mut self, value: f32) -> &mut Self {
        self.threshold_value = if value > 0.0 { value } else { 0.0 };
        self
    }

    pub(crate) fn set_started(&mut self, callback: impl Fn(DragStartedEvent) + 'static) {
        self.started_callback = Some(Box::new(callback));
    }

    pub(crate) fn set_completed(&mut self, callback: impl Fn(DragCompletedEvent) + 'static) {
        self.completed_callback = Some(Box::new(callback));
    }

    pub(crate) fn is_dragging(&self) -> bool {
        self.drag_started_value
    }

    pub(crate) fn handle_pointer_down(&mut self, x: f32, y: f32, modifiers: u32) {
        if self.pointer_down_value {
            self.cancel();
        }
        self.pointer_down_value = true;
        self.drag_started_value = false;
        self.start_x = x;
        self.start_y = y;
        self.last_pointer_x = x;
        self.last_pointer_y = y;
        self.last_dispatched_x = x;
        self.last_dispatched_y = y;
        self.last_modifiers = modifiers;
        self.capture_drag_pointer();
        if self.threshold_value <= 0.0 {
            self.begin_drag(x, y, modifiers);
        }
    }

    pub(crate) fn handle_pointer_move(&mut self, x: f32, y: f32, modifiers: u32) {
        if !self.pointer_down_value {
            return;
        }
        self.last_pointer_x = x;
        self.last_pointer_y = y;
        self.last_modifiers = modifiers;
        if !self.drag_started_value
            && !self.has_exceeded_threshold(x - self.start_x, y - self.start_y)
        {
            return;
        }
        if !self.drag_started_value {
            self.begin_drag(x, y, modifiers);
        }
        self.emit_delta(x, y);
    }

    pub(crate) fn handle_pointer_up(&mut self, x: f32, y: f32, modifiers: u32) {
        if !self.pointer_down_value {
            return;
        }
        self.last_pointer_x = x;
        self.last_pointer_y = y;
        self.last_modifiers = modifiers;
        if self.drag_started_value {
            self.emit_delta(x, y);
            if let Some(callback) = self.completed_callback.as_ref() {
                callback(DragCompletedEvent {
                    x,
                    y,
                    total_delta_x: x - self.start_x,
                    total_delta_y: y - self.start_y,
                    modifiers,
                    cancelled: false,
                });
            }
        }
        self.pointer_down_value = false;
        self.drag_started_value = false;
        self.release_drag_pointer();
    }

    pub(crate) fn cancel(&mut self) {
        if !self.pointer_down_value {
            return;
        }
        if self.drag_started_value {
            if let Some(callback) = self.completed_callback.as_ref() {
                callback(DragCompletedEvent {
                    x: self.last_pointer_x,
                    y: self.last_pointer_y,
                    total_delta_x: self.last_pointer_x - self.start_x,
                    total_delta_y: self.last_pointer_y - self.start_y,
                    modifiers: self.last_modifiers,
                    cancelled: true,
                });
            }
        }
        self.pointer_down_value = false;
        self.drag_started_value = false;
        self.release_drag_pointer();
    }

    fn begin_drag(&mut self, x: f32, y: f32, modifiers: u32) {
        if self.drag_started_value {
            return;
        }
        self.drag_started_value = true;
        self.last_dispatched_x = self.start_x;
        self.last_dispatched_y = self.start_y;
        if let Some(callback) = self.started_callback.as_ref() {
            callback(DragStartedEvent { x, y, modifiers });
        }
    }

    fn emit_delta(&mut self, x: f32, y: f32) {
        if x == self.last_dispatched_x && y == self.last_dispatched_y {
            return;
        }
        self.last_dispatched_x = x;
        self.last_dispatched_y = y;
    }

    fn has_exceeded_threshold(&self, total_delta_x: f32, total_delta_y: f32) -> bool {
        if self.threshold_value <= 0.0 {
            return true;
        }
        ((total_delta_x * total_delta_x) + (total_delta_y * total_delta_y))
            >= (self.threshold_value * self.threshold_value)
    }

    fn capture_drag_pointer(&self) {
        let Some(host) = self.host.upgrade() else {
            return;
        };
        let handle = host.handle();
        if handle == NodeHandle::INVALID {
            return;
        }
        event::capture_pointer(handle);
        unsafe { crate::ffi::fui_set_pointer_capture(handle.raw()) };
    }

    fn release_drag_pointer(&self) {
        let Some(host) = self.host.upgrade() else {
            return;
        };
        let handle = host.handle();
        if handle == NodeHandle::INVALID {
            return;
        }
        event::release_pointer(handle);
        unsafe { crate::ffi::fui_release_pointer_capture() };
    }
}
