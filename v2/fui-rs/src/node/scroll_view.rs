use super::core::*;
use super::*;
use crate::animation::{animate_float, Animation, AnimationTiming};
use crate::transitions::NodeTransitions;

#[derive(Default)]
struct ScrollViewAnimationState {
    scroll_offset: Option<Animation>,
}

#[derive(Clone)]
pub struct ScrollView {
    core: Rc<RefCell<NodeCore>>,
    props: Rc<RefCell<ScrollViewProps>>,
    bound_scroll_state: Rc<RefCell<Option<ScrollState>>>,
    active_animations: Rc<RefCell<ScrollViewAnimationState>>,
}

impl ScrollView {
    pub fn new() -> Self {
        let core = Rc::new(RefCell::new(NodeCore::new(NodeKind::ScrollView)));
        core.borrow_mut().handlers.pointer_down = Some(Rc::new(|_event: &mut PointerEventArgs| {}));
        core.borrow_mut().scroll_routing = Some(ScrollRoutingState {
            enabled_x: true,
            enabled_y: true,
            ..ScrollRoutingState::default()
        });
        Self {
            core,
            props: Rc::new(RefCell::new(ScrollViewProps {
                width: None,
                height: None,
                bg_color: None,
                padding: None,
                enable_scroll_x: true,
                enable_scroll_y: true,
                friction: None,
                smooth_scrolling: true,
                scroll_offset: None,
                content_size: None,
                persist_scroll: true,
                transitions: None,
            })),
            bound_scroll_state: Rc::new(RefCell::new(None)),
            active_animations: Rc::new(RefCell::new(ScrollViewAnimationState::default())),
        }
    }

    pub fn bind_scroll_state(&self, scroll_state: ScrollState) -> &Self {
        *self.bound_scroll_state.borrow_mut() = Some(scroll_state.clone());
        self.sync_bound_scroll_state_from_props();
        let bound_scroll_state = self.bound_scroll_state.clone();
        self.core.borrow_mut().handlers.scroll_changed = Some(Rc::new(
            move |offset_x,
                  offset_y,
                  content_width,
                  content_height,
                  viewport_width,
                  viewport_height| {
                if let Some(state) = bound_scroll_state.borrow().clone() {
                    state.set_offset_x(offset_x);
                    state.set_offset_y(offset_y);
                    state.set_content_width(content_width);
                    state.set_content_height(content_height);
                    state.set_viewport_width(viewport_width);
                    state.set_viewport_height(viewport_height);
                }
            },
        ));
        self
    }

    pub fn width(&self, width: f32, unit: Unit) -> &Self {
        self.props.borrow_mut().width = Some((width, unit));
        self.set_viewport_width_if_pixel(width, unit);
        let mut core = self.core.borrow_mut();
        core.behavior.fill_width = false;
        core.behavior.fill_width_percent = None;
        self
    }

    pub fn width_len(&self, length: Length) -> &Self {
        let (width, unit) = length;
        self.width(width, unit)
    }

    pub fn height(&self, height: f32, unit: Unit) -> &Self {
        self.props.borrow_mut().height = Some((height, unit));
        self.set_viewport_height_if_pixel(height, unit);
        let mut core = self.core.borrow_mut();
        core.behavior.fill_height = false;
        core.behavior.fill_height_percent = None;
        self
    }

    pub fn height_len(&self, length: Length) -> &Self {
        let (height, unit) = length;
        self.height(height, unit)
    }

    pub fn fill_width(&self) -> &Self {
        self.props.borrow_mut().width = None;
        let mut core = self.core.borrow_mut();
        core.behavior.fill_width = true;
        core.behavior.fill_width_percent = None;
        self
    }

    pub fn fill_height(&self) -> &Self {
        self.props.borrow_mut().height = None;
        let mut core = self.core.borrow_mut();
        core.behavior.fill_height = true;
        core.behavior.fill_height_percent = None;
        self
    }

    pub fn fill_size(&self) -> &Self {
        self.fill_width();
        self.fill_height();
        self
    }

    pub fn bg_color(&self, color: u32) -> &Self {
        self.props.borrow_mut().bg_color = Some(color);
        self
    }

    pub fn padding(&self, left: f32, top: f32, right: f32, bottom: f32) -> &Self {
        self.props.borrow_mut().padding = Some((left, top, right, bottom));
        self
    }

    pub fn scroll_enabled(&self, enabled_x: bool, enabled_y: bool) -> &Self {
        let mut props = self.props.borrow_mut();
        props.enable_scroll_x = enabled_x;
        props.enable_scroll_y = enabled_y;
        self.retained_node_ref()
            .set_scroll_routing_enabled(enabled_x, enabled_y);
        self
    }

    pub fn scroll_enabled_x(&self, enabled: bool) -> &Self {
        self.props.borrow_mut().enable_scroll_x = enabled;
        let enabled_y = self
            .retained_node_ref()
            .scroll_routing_state()
            .map(|state| state.enabled_y)
            .unwrap_or(true);
        self.retained_node_ref()
            .set_scroll_routing_enabled(enabled, enabled_y);
        self
    }

    pub fn scroll_enabled_y(&self, enabled: bool) -> &Self {
        self.props.borrow_mut().enable_scroll_y = enabled;
        let enabled_x = self
            .retained_node_ref()
            .scroll_routing_state()
            .map(|state| state.enabled_x)
            .unwrap_or(true);
        self.retained_node_ref()
            .set_scroll_routing_enabled(enabled_x, enabled);
        self
    }

    pub fn scroll_friction(&self, friction: f32) -> &Self {
        self.props.borrow_mut().friction = Some(friction);
        self
    }

    pub fn smooth_scrolling(&self, smooth_scrolling: bool) -> &Self {
        self.props.borrow_mut().smooth_scrolling = smooth_scrolling;
        if self.has_built_handle() {
            ui::set_smooth_scrolling(self.handle().raw(), smooth_scrolling);
        }
        self
    }

    pub fn scroll_offset(&self, offset_x: f32, offset_y: f32) -> &Self {
        self.cancel_scroll_offset_transition();
        if self.should_animate_scroll_offset(offset_x, offset_y) {
            if let Some(timing) = self
                .props
                .borrow()
                .transitions
                .as_ref()
                .and_then(NodeTransitions::scroll_offset_timing)
            {
                self.take_programmatic_scroll_ownership();
                self.start_scroll_offset_animation(offset_x, offset_y, timing);
                return self;
            }
        }
        if self.has_built_handle() && self.current_scroll_offset() != (offset_x, offset_y) {
            self.take_programmatic_scroll_ownership();
        }
        self.apply_animated_scroll_offset(offset_x, offset_y);
        self
    }

    pub fn content_size(&self, width: f32, height: f32) -> &Self {
        self.props.borrow_mut().content_size = Some((width, height));
        self.set_content_size_metrics(width, height);
        self
    }

    pub fn scroll_content_size(&self, width: f32, height: f32) -> &Self {
        self.content_size(width, height)
    }

    pub fn persist_scroll(&self, persist: bool) -> &Self {
        self.props.borrow_mut().persist_scroll = persist;
        self
    }

    pub fn set_runtime_scroll_offset(&self, offset_x: f32, offset_y: f32) {
        self.cancel_scroll_offset_transition();
        self.apply_animated_scroll_offset(offset_x, offset_y);
    }

    pub fn scroll_to(&self, offset_x: f32, offset_y: f32) -> &Self {
        self.cancel_scroll_offset_transition();
        if self.has_built_handle() && self.current_scroll_offset() != (offset_x, offset_y) {
            self.take_programmatic_scroll_ownership();
        }
        self.apply_animated_scroll_offset(offset_x, offset_y);
        self
    }

    pub fn scroll_to_animated(
        &self,
        offset_x: f32,
        offset_y: f32,
        timing: AnimationTiming,
    ) -> &Self {
        self.cancel_scroll_offset_transition();
        if !self.has_built_handle() || self.current_scroll_offset() == (offset_x, offset_y) {
            self.apply_animated_scroll_offset(offset_x, offset_y);
            return self;
        }
        self.take_programmatic_scroll_ownership();
        self.start_scroll_offset_animation(offset_x, offset_y, timing);
        self
    }

    pub fn transitions(&self, transitions: Option<NodeTransitions>) -> &Self {
        self.props.borrow_mut().transitions = transitions;
        self
    }

    pub fn focusable(&self, enabled: bool, tab_index: i32) -> &Self {
        if enabled {
            self.retained_node_ref().require_interactive();
        }
        let mut core = self.core.borrow_mut();
        core.behavior.focusable = Some((enabled, tab_index));
        let interactive = core.behavior.enabled && core.behavior.inherited_enabled;
        let handle = core.handle;
        drop(core);
        if handle != NodeHandle::INVALID {
            ui::set_focusable(handle.raw(), interactive && enabled, tab_index);
            self.notify_retained_mutation();
        }
        self
    }

    pub fn on_wheel(&self, handler: impl Fn(&mut WheelEventArgs) + 'static) -> &Self {
        self.core.borrow_mut().handlers.wheel = Some(Rc::new(handler));
        self.retained_node_ref().require_interactive();
        self
    }

    pub fn on_pan_gesture(&self, handler: impl Fn(&mut GestureEventArgs) + 'static) -> &Self {
        self.core.borrow_mut().handlers.pan_gesture = Some(Rc::new(handler));
        self
    }

    pub fn on_pinch_gesture(&self, handler: impl Fn(&mut GestureEventArgs) + 'static) -> &Self {
        self.core.borrow_mut().handlers.pinch_gesture = Some(Rc::new(handler));
        self
    }

    pub fn long_press_options(&self, minimum_duration_ms: i32, movement_tolerance: f32) -> &Self {
        let mut core = self.core.borrow_mut();
        core.handlers.long_press_minimum_duration_ms = minimum_duration_ms.max(0);
        core.handlers.long_press_movement_tolerance = movement_tolerance.max(0.0);
        self
    }

    pub fn on_long_press(&self, handler: impl Fn(&mut LongPressEventArgs) + 'static) -> &Self {
        self.core.borrow_mut().handlers.long_press = Some(Rc::new(handler));
        self.retained_node_ref().require_interactive();
        self
    }

    pub fn child<T: Node>(&self, child: &T) -> &Self {
        self.append_child(child);
        self
    }

    pub fn children<I, C>(&self, children: I) -> &Self
    where
        I: IntoIterator<Item = C>,
        C: Into<Child>,
    {
        for child in children {
            self.retained_node_ref()
                .append_child_ref(&child.into().node_ref);
        }
        self
    }

    fn sync_bound_scroll_state_from_props(&self) {
        let props = self.props.borrow();
        let width = props.width;
        let height = props.height;
        let scroll_offset = props.scroll_offset;
        let content_size = props.content_size;
        drop(props);

        if let Some((width, unit)) = width {
            self.set_viewport_width_if_pixel(width, unit);
        }
        if let Some((height, unit)) = height {
            self.set_viewport_height_if_pixel(height, unit);
        }
        if let Some((offset_x, offset_y)) = scroll_offset {
            self.set_scroll_offset_metrics(offset_x, offset_y);
        }
        if let Some((width, height)) = content_size {
            self.set_content_size_metrics(width, height);
        }
    }

    fn set_viewport_width_if_pixel(&self, width: f32, unit: Unit) {
        if unit != Unit::Pixel {
            return;
        }
        if let Some(state) = self.bound_scroll_state.borrow().clone() {
            state.set_viewport_width(width);
        }
        self.update_scroll_routing_metrics(|state| state.viewport_width = width);
    }

    fn set_viewport_height_if_pixel(&self, height: f32, unit: Unit) {
        if unit != Unit::Pixel {
            return;
        }
        if let Some(state) = self.bound_scroll_state.borrow().clone() {
            state.set_viewport_height(height);
        }
        self.update_scroll_routing_metrics(|state| state.viewport_height = height);
    }

    fn set_content_size_metrics(&self, width: f32, height: f32) {
        if let Some(state) = self.bound_scroll_state.borrow().clone() {
            if width >= 0.0 {
                state.set_content_width(width);
            }
            if height >= 0.0 {
                state.set_content_height(height);
            }
        }
        self.update_scroll_routing_metrics(|state| {
            if width >= 0.0 {
                state.content_width = width;
            }
            if height >= 0.0 {
                state.content_height = height;
            }
        });
    }

    fn set_scroll_offset_metrics(&self, offset_x: f32, offset_y: f32) {
        if let Some(state) = self.bound_scroll_state.borrow().clone() {
            state.set_offset_x(offset_x);
            state.set_offset_y(offset_y);
        }
        self.retained_node_ref()
            .set_scroll_routing_offsets(offset_x, offset_y);
    }

    fn update_scroll_routing_metrics(&self, update: impl FnOnce(&mut ScrollRoutingState)) {
        let node = self.retained_node_ref();
        let Some(mut state) = node.scroll_routing_state() else {
            return;
        };
        update(&mut state);
        node.set_scroll_routing_metrics(
            state.offset_x,
            state.offset_y,
            state.content_width,
            state.content_height,
            state.viewport_width,
            state.viewport_height,
        );
    }

    fn current_scroll_offset(&self) -> (f32, f32) {
        self.props.borrow().scroll_offset.unwrap_or((0.0, 0.0))
    }

    fn apply_animated_scroll_offset(&self, offset_x: f32, offset_y: f32) {
        self.props.borrow_mut().scroll_offset = Some((offset_x, offset_y));
        self.set_scroll_offset_metrics(offset_x, offset_y);
        if self.has_built_handle() {
            self.prepare_programmatic_scroll(offset_x, offset_y);
            ui::set_scroll_offset(self.handle().raw(), offset_x, offset_y);
            self.notify_retained_mutation();
        }
    }

    fn should_animate_scroll_offset(&self, offset_x: f32, offset_y: f32) -> bool {
        self.has_built_handle()
            && self
                .props
                .borrow()
                .transitions
                .as_ref()
                .and_then(NodeTransitions::scroll_offset_timing)
                .is_some()
            && self.current_scroll_offset() != (offset_x, offset_y)
    }

    fn cancel_scroll_offset_transition(&self) {
        if let Some(animation) = self.active_animations.borrow_mut().scroll_offset.take() {
            animation.cancel();
        }
    }

    fn start_scroll_offset_animation(&self, offset_x: f32, offset_y: f32, timing: AnimationTiming) {
        let (from_x, from_y) = self.current_scroll_offset();
        let weak_core = Rc::downgrade(&self.core);
        let weak_props = Rc::downgrade(&self.props);
        let weak_state = Rc::downgrade(&self.bound_scroll_state);
        let animation = animate_float(0.0, 1.0, timing, move |progress| {
            let Some(core) = weak_core.upgrade() else {
                return;
            };
            let Some(props) = weak_props.upgrade() else {
                return;
            };
            let next_x = from_x + ((offset_x - from_x) * progress);
            let next_y = from_y + ((offset_y - from_y) * progress);
            props.borrow_mut().scroll_offset = Some((next_x, next_y));
            if let Some(bound) = weak_state.upgrade().and_then(|slot| slot.borrow().clone()) {
                bound.set_offset_x(next_x);
                bound.set_offset_y(next_y);
            }
            let node = NodeRef::from_core(core);
            node.set_scroll_routing_offsets(next_x, next_y);
            if node.handle() != NodeHandle::INVALID {
                // FUI-AS marks every programmatic set as a pending native-scroll ack while
                // ownership is taken once before starting the animation.
                ui::set_scroll_offset(node.handle().raw(), next_x, next_y);
                crate::frame_scheduler::mark_needs_commit();
            }
        });
        self.active_animations.borrow_mut().scroll_offset = Some(animation);
    }

    fn take_programmatic_scroll_ownership(&self) {
        ui::clear_momentum_scroll();
    }

    fn prepare_programmatic_scroll(&self, _offset_x: f32, _offset_y: f32) {
        // FUI-AS keeps a pending programmatic ack so its scroll callback can ignore the
        // matching native echo. FUI-RS updates scroll metrics in the central event
        // dispatcher and does not cancel transitions from that echo yet, so there is no
        // local ack state to update here. Keep the call site split faithful so the ack
        // state can be added without changing public behavior when scroll dispatch is
        // ported to the exact FUI-AS ownership model.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::{self, Call};

    #[test]
    fn scroll_to_takes_programmatic_scroll_ownership_before_setting_offset() {
        ffi::test::reset();
        let view = ScrollView::new();
        view.build();
        ffi::test::take_calls();

        view.scroll_to(12.0, 34.0);

        let calls = ffi::test::take_calls();
        let clear_index = calls
            .iter()
            .position(|call| matches!(call, Call::ClearMomentumScroll))
            .expect("programmatic scroll should clear momentum first");
        let set_index = calls
            .iter()
            .position(|call| matches!(call, Call::SetScrollOffset { offset_x, offset_y, .. } if *offset_x == 12.0 && *offset_y == 34.0))
            .expect("programmatic scroll should set native offset");
        assert!(clear_index < set_index);
    }

    #[test]
    fn animated_scroll_takes_programmatic_scroll_ownership_before_first_tick() {
        ffi::test::reset();
        crate::animation::reset_animations();
        let view = ScrollView::new();
        view.build();
        ffi::test::take_calls();

        view.scroll_to_animated(0.0, 80.0, AnimationTiming::new(100.0));

        let calls = ffi::test::take_calls();
        assert!(
            calls
                .iter()
                .any(|call| matches!(call, Call::ClearMomentumScroll)),
            "animated programmatic scroll should clear native momentum before ticking"
        );
    }

    #[test]
    fn smooth_wheel_scrolling_defaults_on_and_supports_retained_opt_out() {
        ffi::test::reset();
        let view = ScrollView::new();
        view.build();

        let calls = ffi::test::take_calls();
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::SetSmoothScrolling {
                smooth_scrolling: true,
                ..
            }
        )));

        view.smooth_scrolling(false);
        let calls = ffi::test::take_calls();
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::SetSmoothScrolling {
                smooth_scrolling: false,
                ..
            }
        )));
    }
}

impl Node for ScrollView {
    fn retained_node_ref(&self) -> NodeRef {
        NodeRef::from_node(self.core.clone(), self.clone())
    }

    fn build_self(&self) {
        apply_scroll_view_props(
            self.handle(),
            &self.props.borrow(),
            self.core.borrow().behavior.clone(),
        );
    }
}
