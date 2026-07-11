use super::*;
use crate::drag_drop::{
    DragCompletedEventArgs, DragDataObject, DragDropEffects, DragEventArgs, DropProposal,
};
use crate::drag_gesture::{DragCompletedEvent, DragGesture, DragStartedEvent};
use crate::event::{SelectionChangedEventArgs, TextChangedEventArgs};
use crate::external_drop::ExternalDropEventArgs;
use crate::persisted::PersistedStateAdapter;
use crate::tool_tip::ToolTip;
use crate::transitions::NodeTransitions;

pub(crate) type PointerCallback = Rc<dyn Fn(&mut PointerEventArgs)>;
pub(crate) type WheelCallback = Rc<dyn Fn(&mut WheelEventArgs)>;
pub(crate) type GestureCallback = Rc<dyn Fn(&mut GestureEventArgs)>;
pub(crate) type KeyCallback = Rc<dyn Fn(&mut KeyEventArgs)>;
pub(crate) type LongPressCallback = Rc<dyn Fn(&mut LongPressEventArgs)>;
pub(crate) type FocusChangedCallback = Rc<dyn Fn(FocusChangedEventArgs)>;
pub(crate) type TextChangedCallback = Rc<dyn Fn(TextChangedEventArgs)>;
pub(crate) type TextReplacedCallback = Rc<dyn Fn(u32, u32, String)>;
pub(crate) type ScrollChangedCallback = Rc<dyn Fn(f32, f32, f32, f32, f32, f32)>;
pub(crate) type SelectionChangedCallback = Rc<dyn Fn(SelectionChangedEventArgs)>;
pub(crate) type CrossSelectionChangedCallback = Rc<dyn Fn(String)>;
pub(crate) type DrawCallback = Rc<dyn Fn(&mut DrawContext)>;
pub(crate) type ContextMenuCallback = Rc<dyn Fn(ContextMenuEventArgs)>;
pub(crate) type DragDataCallback = Rc<dyn Fn() -> Option<DragDataObject>>;
pub(crate) type DragCompletedCallback = Rc<dyn Fn(DragCompletedEventArgs)>;
pub(crate) type DragProposalCallback = Rc<dyn Fn(DragEventArgs) -> DropProposal>;
pub(crate) type DragEventCallback = Rc<dyn Fn(DragEventArgs)>;
pub(crate) type ExternalDragProposalCallback = Rc<dyn Fn(ExternalDropEventArgs) -> DropProposal>;
pub(crate) type ExternalDragEventCallback = Rc<dyn Fn(ExternalDropEventArgs)>;
pub(crate) type EffectiveEnabledChangedCallback = Rc<dyn Fn(bool)>;

pub(crate) fn is_primary_activation_pointer(event: &PointerEventArgs) -> bool {
    event.button == 0
        || event.pointer_type == PointerType::Touch
        || event.pointer_type == PointerType::Pen
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeHandle(u64);

impl NodeHandle {
    pub const INVALID: Self = Self(HandleValue::Invalid as u64);

    pub(crate) fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    pub fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContextMenuEventArgs {
    pub target: NodeHandle,
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NodeKind {
    FlexBox,
    Text,
    Grid,
    Image,
    Svg,
    ScrollView,
}

impl NodeKind {
    pub(crate) fn node_type(self) -> NodeType {
        match self {
            Self::FlexBox => NodeType::FlexBox,
            Self::Text => NodeType::Text,
            Self::Grid => NodeType::Grid,
            Self::Image => NodeType::Image,
            Self::Svg => NodeType::Svg,
            Self::ScrollView => NodeType::ScrollView,
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct EventHandlers {
    pub(crate) pointer_click: Option<PointerCallback>,
    pub(crate) pointer_down: Option<PointerCallback>,
    pub(crate) pointer_move: Option<PointerCallback>,
    pub(crate) pointer_up: Option<PointerCallback>,
    pub(crate) pointer_enter: Option<PointerCallback>,
    pub(crate) pointer_leave: Option<PointerCallback>,
    pub(crate) pointer_cancel: Option<PointerCallback>,
    pub(crate) wheel: Option<WheelCallback>,
    pub(crate) pan_gesture: Option<GestureCallback>,
    pub(crate) pinch_gesture: Option<GestureCallback>,
    pub(crate) long_press: Option<LongPressCallback>,
    pub(crate) long_press_minimum_duration_ms: i32,
    pub(crate) long_press_movement_tolerance: f32,
    pub(crate) key_down: Option<KeyCallback>,
    pub(crate) key_up: Option<KeyCallback>,
    pub(crate) focus_changed: Option<FocusChangedCallback>,
    pub(crate) text_changed: Option<TextChangedCallback>,
    pub(crate) text_replaced: Option<TextReplacedCallback>,
    pub(crate) scroll_changed: Option<ScrollChangedCallback>,
    pub(crate) selection_changed: Option<SelectionChangedCallback>,
    pub(crate) cross_selection_changed: Option<CrossSelectionChangedCallback>,
}

impl EventHandlers {
    pub(crate) fn new() -> Self {
        Self {
            long_press_minimum_duration_ms: DEFAULT_LONG_PRESS_MINIMUM_DURATION_MS,
            long_press_movement_tolerance: DEFAULT_LONG_PRESS_MOVEMENT_TOLERANCE,
            ..Self::default()
        }
    }
}
#[derive(Clone)]
pub(crate) struct NodeBehavior {
    pub(crate) enabled: bool,
    pub(crate) inherited_enabled: bool,
    pub(crate) last_effective_enabled: bool,
    pub(crate) interactive: bool,
    pub(crate) focusable: Option<(bool, i32)>,
    pub(crate) cursor: Option<CursorStyle>,
    pub(crate) node_id: Option<String>,
    pub(crate) semantic_role: Option<SemanticRole>,
    pub(crate) semantic_label: Option<String>,
    pub(crate) default_semantic_label: Option<String>,
    pub(crate) semantic_disabled: Option<bool>,
    pub(crate) semantic_checked: Option<SemanticCheckedState>,
    pub(crate) semantic_selected: Option<bool>,
    pub(crate) semantic_value_range: Option<(f32, f32, f32)>,
    pub(crate) semantic_orientation: Option<Orientation>,
    pub(crate) selectable_text: bool,
    pub(crate) editable_text: bool,
    pub(crate) text_content: Option<String>,
    pub(crate) link_url: Option<String>,
    pub(crate) image_url: Option<String>,
    pub(crate) link_preview_pin: Option<Rc<dyn Fn()>>,
    pub(crate) link_preview_release: Option<Rc<dyn Fn()>>,
    pub(crate) context_menu_disabled: bool,
    pub(crate) context_menu_handler: Option<ContextMenuCallback>,
    pub(crate) tool_tip: Option<ToolTip>,
    pub(crate) track_semantic_disabled_from_enabled: bool,
    pub(crate) request_semantic_announcement: bool,
    pub(crate) visibility: Option<Visibility>,
    pub(crate) is_portal: bool,
    pub(crate) fill_width: bool,
    pub(crate) fill_height: bool,
    pub(crate) fill_width_percent: Option<f32>,
    pub(crate) fill_height_percent: Option<f32>,
    pub(crate) min_width: Option<(f32, Unit)>,
    pub(crate) max_width: Option<(f32, Unit)>,
    pub(crate) min_height: Option<(f32, Unit)>,
    pub(crate) max_height: Option<(f32, Unit)>,
    pub(crate) flex_basis: Option<f32>,
    pub(crate) justify_content: Option<JustifyContent>,
    pub(crate) align_items: Option<AlignItems>,
    pub(crate) align_self: Option<AlignSelf>,
    pub(crate) margin: Option<(f32, f32, f32, f32)>,
    pub(crate) position_type: Option<PositionType>,
    pub(crate) position: Option<(f32, f32, f32, f32)>,
    pub(crate) is_shared_size_scope: bool,
    pub(crate) custom_drawable: bool,
    pub(crate) flex_wrap: Option<FlexWrap>,
    pub(crate) clip_to_bounds: Option<bool>,
    pub(crate) selection_area: bool,
    pub(crate) selection_area_barrier: bool,
    pub(crate) preserve_selection_on_pointer_down: bool,
    pub(crate) scroll_proxy_target: Option<u64>,
    pub(crate) external_drop_allowed: bool,
}

impl Default for NodeBehavior {
    fn default() -> Self {
        Self {
            enabled: true,
            inherited_enabled: true,
            last_effective_enabled: true,
            interactive: false,
            focusable: None,
            cursor: None,
            node_id: None,
            semantic_role: None,
            semantic_label: None,
            default_semantic_label: None,
            semantic_disabled: None,
            semantic_checked: None,
            semantic_selected: None,
            semantic_value_range: None,
            semantic_orientation: None,
            selectable_text: false,
            editable_text: false,
            text_content: None,
            link_url: None,
            image_url: None,
            link_preview_pin: None,
            link_preview_release: None,
            context_menu_disabled: false,
            context_menu_handler: None,
            tool_tip: None,
            track_semantic_disabled_from_enabled: false,
            request_semantic_announcement: false,
            visibility: None,
            is_portal: false,
            fill_width: false,
            fill_height: false,
            fill_width_percent: None,
            fill_height_percent: None,
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            flex_basis: None,
            justify_content: None,
            align_items: None,
            align_self: None,
            margin: None,
            position_type: None,
            position: None,
            is_shared_size_scope: false,
            custom_drawable: false,
            flex_wrap: None,
            clip_to_bounds: None,
            selection_area: false,
            selection_area_barrier: false,
            preserve_selection_on_pointer_down: false,
            scroll_proxy_target: None,
            external_drop_allowed: false,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct BoxStyle {
    pub(crate) radius_tl: f32,
    pub(crate) radius_tr: f32,
    pub(crate) radius_br: f32,
    pub(crate) radius_bl: f32,
    pub(crate) border_width: f32,
    pub(crate) border_color: u32,
    pub(crate) border_style: BorderStyle,
    pub(crate) border_dash_on: f32,
    pub(crate) border_dash_off: f32,
}

#[derive(Clone, Copy)]
pub(crate) struct DropShadow {
    pub(crate) color: u32,
    pub(crate) offset_x: f32,
    pub(crate) offset_y: f32,
    pub(crate) blur_sigma: f32,
    pub(crate) spread: f32,
}
#[derive(Clone)]
pub(crate) struct LinearGradient {
    pub(crate) sx: f32,
    pub(crate) sy: f32,
    pub(crate) ex: f32,
    pub(crate) ey: f32,
    pub(crate) offsets: Vec<f32>,
    pub(crate) colors: Vec<u32>,
}
#[derive(Clone)]
pub(crate) struct FlexBoxProps {
    pub(crate) width: Option<(f32, Unit)>,
    pub(crate) height: Option<(f32, Unit)>,
    pub(crate) bg_color: Option<u32>,
    pub(crate) padding: Option<(f32, f32, f32, f32)>,
    pub(crate) flex_direction: Option<FlexDirection>,
    pub(crate) box_style: Option<BoxStyle>,
    pub(crate) opacity: Option<f32>,
    pub(crate) blur_sigma: Option<f32>,
    pub(crate) drop_shadow: Option<DropShadow>,
    pub(crate) background_blur_sigma: Option<f32>,
    pub(crate) linear_gradient: Option<LinearGradient>,
    pub(crate) transitions: Option<NodeTransitions>,
}

impl Default for FlexBoxProps {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            bg_color: None,
            padding: None,
            flex_direction: None,
            box_style: None,
            opacity: None,
            blur_sigma: None,
            drop_shadow: None,
            background_blur_sigma: None,
            linear_gradient: None,
            transitions: None,
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct TextProps {
    pub(crate) content: String,
    pub(crate) width: Option<(f32, Unit)>,
    pub(crate) height: Option<(f32, Unit)>,
    pub(crate) font_id: u32,
    pub(crate) font_size: f32,
    pub(crate) has_font: bool,
    pub(crate) font_family: Option<FontFamily>,
    pub(crate) font_weight: FontWeight,
    pub(crate) font_style: FontStyle,
    pub(crate) uses_direct_font_id: bool,
    pub(crate) text_color: Option<u32>,
    pub(crate) style_runs: Vec<u32>,
    pub(crate) has_style_runs: bool,
    pub(crate) line_height: Option<f32>,
    pub(crate) text_align: Option<TextAlign>,
    pub(crate) text_vertical_align: Option<TextVerticalAlign>,
    pub(crate) text_limits: Option<(i32, i32)>,
    pub(crate) wrapping: Option<bool>,
    pub(crate) overflow: Option<TextOverflow>,
    pub(crate) overflow_fade: Option<(bool, bool)>,
    pub(crate) selectable: Option<(bool, u32)>,
    pub(crate) uses_theme_selection_color: bool,
    pub(crate) editable: Option<bool>,
    pub(crate) editor_command_keys: Option<bool>,
    pub(crate) editor_accepts_tab: Option<bool>,
    pub(crate) obscured: Option<bool>,
    pub(crate) caret_color: Option<u32>,
}

#[derive(Clone, Default)]
pub(crate) struct GridProps {
    pub(crate) width: Option<(f32, Unit)>,
    pub(crate) height: Option<(f32, Unit)>,
    pub(crate) bg_color: Option<u32>,
    pub(crate) padding: Option<(f32, f32, f32, f32)>,
    pub(crate) columns: Vec<f32>,
    pub(crate) column_types: Vec<GridUnit>,
    pub(crate) rows: Vec<f32>,
    pub(crate) row_types: Vec<GridUnit>,
    pub(crate) column_shared_size_groups: Vec<(u32, String)>,
    pub(crate) row_shared_size_groups: Vec<(u32, String)>,
}
#[derive(Clone)]
pub(crate) struct ImageProps {
    pub(crate) width: Option<(f32, Unit)>,
    pub(crate) height: Option<(f32, Unit)>,
    pub(crate) texture_id: u32,
    pub(crate) source_url: Option<String>,
    pub(crate) object_fit: ObjectFit,
    pub(crate) sampling_kind: ImageSamplingKind,
    pub(crate) max_aniso: u32,
    pub(crate) image_nine: Option<(f32, f32, f32, f32)>,
}

#[derive(Clone)]
pub(crate) struct SvgProps {
    pub(crate) width: Option<(f32, Unit)>,
    pub(crate) height: Option<(f32, Unit)>,
    pub(crate) svg_id: u32,
    pub(crate) source_url: Option<String>,
    pub(crate) tint_color: u32,
    pub(crate) sampling_kind: ImageSamplingKind,
    pub(crate) max_aniso: u32,
    pub(crate) opacity: Option<f32>,
}

#[derive(Clone)]
pub(crate) struct ScrollViewProps {
    pub(crate) width: Option<(f32, Unit)>,
    pub(crate) height: Option<(f32, Unit)>,
    pub(crate) bg_color: Option<u32>,
    pub(crate) padding: Option<(f32, f32, f32, f32)>,
    pub(crate) enable_scroll_x: bool,
    pub(crate) enable_scroll_y: bool,
    pub(crate) show_scrollbars: bool,
    pub(crate) friction: Option<f32>,
    pub(crate) smooth_scrolling: bool,
    pub(crate) scroll_offset: Option<(f32, f32)>,
    pub(crate) content_size: Option<(f32, f32)>,
    pub(crate) persist_scroll: bool,
    pub(crate) transitions: Option<NodeTransitions>,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ScrollRoutingState {
    pub(crate) enabled_x: bool,
    pub(crate) enabled_y: bool,
    pub(crate) offset_x: f32,
    pub(crate) offset_y: f32,
    pub(crate) content_width: f32,
    pub(crate) content_height: f32,
    pub(crate) viewport_width: f32,
    pub(crate) viewport_height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GridPlacement {
    pub(crate) row: u32,
    pub(crate) col: u32,
    pub(crate) row_span: u32,
    pub(crate) col_span: u32,
}

pub(crate) struct NodeCore {
    pub(crate) handle: NodeHandle,
    pub(crate) kind: NodeKind,
    pub(crate) parent: Weak<RefCell<NodeCore>>,
    pub(crate) children: Vec<NodeRef>,
    pub(crate) mounted: bool,
    pub(crate) children_built: bool,
    pub(crate) behavior: NodeBehavior,
    pub(crate) handlers: EventHandlers,
    pub(crate) draw_callback: Option<DrawCallback>,
    pub(crate) retained_attachments: Vec<Rc<dyn Any>>,
    pub(crate) persisted_state_adapters: Vec<Rc<dyn PersistedStateAdapter>>,
    pub(crate) drag_data_callback: Option<DragDataCallback>,
    pub(crate) drag_allowed_effects: DragDropEffects,
    pub(crate) drag_completed_callback: Option<DragCompletedCallback>,
    pub(crate) drop_allowed: bool,
    pub(crate) drag_enter_callback: Option<DragProposalCallback>,
    pub(crate) drag_over_callback: Option<DragProposalCallback>,
    pub(crate) drag_leave_callback: Option<DragEventCallback>,
    pub(crate) drop_callback: Option<DragEventCallback>,
    pub(crate) external_drag_enter_callback: Option<ExternalDragProposalCallback>,
    pub(crate) external_drag_over_callback: Option<ExternalDragProposalCallback>,
    pub(crate) external_drag_leave_callback: Option<ExternalDragEventCallback>,
    pub(crate) external_drop_callback: Option<ExternalDragEventCallback>,
    pub(crate) drag_gesture: Option<Rc<RefCell<DragGesture>>>,
    pub(crate) drag_click_pending: bool,
    pub(crate) drag_click_pending_count: i32,
    pub(crate) click_pending: bool,
    pub(crate) click_pending_count: i32,
    pub(crate) scroll_routing: Option<ScrollRoutingState>,
    pub(crate) effective_enabled_changed_callbacks: Vec<EffectiveEnabledChangedCallback>,
}

impl NodeCore {
    pub(crate) fn new(kind: NodeKind) -> Self {
        Self {
            handle: NodeHandle::INVALID,
            kind,
            parent: Weak::new(),
            children: Vec::new(),
            mounted: false,
            children_built: false,
            behavior: NodeBehavior::default(),
            handlers: EventHandlers::new(),
            draw_callback: None,
            retained_attachments: Vec::new(),
            persisted_state_adapters: Vec::new(),
            drag_data_callback: None,
            drag_allowed_effects: DragDropEffects::Copy,
            drag_completed_callback: None,
            drop_allowed: false,
            drag_enter_callback: None,
            drag_over_callback: None,
            drag_leave_callback: None,
            drop_callback: None,
            external_drag_enter_callback: None,
            external_drag_over_callback: None,
            external_drag_leave_callback: None,
            external_drop_callback: None,
            drag_gesture: None,
            drag_click_pending: false,
            drag_click_pending_count: 0,
            click_pending: false,
            click_pending_count: 0,
            scroll_routing: None,
            effective_enabled_changed_callbacks: Vec::new(),
        }
    }
}

#[derive(Clone)]
#[doc(hidden)]
pub struct NodeRef {
    inner: Rc<RefCell<NodeCore>>,
    build_callback: Option<Rc<dyn Fn()>>,
}

#[derive(Clone)]
pub(crate) struct WeakNodeRef {
    inner: Weak<RefCell<NodeCore>>,
}

impl WeakNodeRef {
    pub(crate) fn upgrade(&self) -> Option<NodeRef> {
        self.inner.upgrade().map(NodeRef::from_core)
    }
}

#[derive(Clone)]
pub(crate) struct WeakFlexBox {
    pub(crate) core: Weak<RefCell<NodeCore>>,
    pub(crate) props: Weak<RefCell<FlexBoxProps>>,
    pub(crate) active_animations: Weak<RefCell<crate::node::flex_box::FlexBoxAnimations>>,
}

impl WeakFlexBox {
    pub(crate) fn upgrade(&self) -> Option<FlexBox> {
        Some(FlexBox {
            core: self.core.upgrade()?,
            props: self.props.upgrade()?,
            active_animations: self.active_animations.upgrade()?,
        })
    }
}

impl NodeRef {
    pub(crate) fn from_core(inner: Rc<RefCell<NodeCore>>) -> Self {
        Self {
            inner,
            build_callback: None,
        }
    }

    pub(crate) fn from_node<T: Node>(inner: Rc<RefCell<NodeCore>>, node: T) -> Self {
        Self {
            inner,
            build_callback: Some(Rc::new(move || node.build())),
        }
    }

    pub(crate) fn with_build_callback(mut self, callback: impl Fn() + 'static) -> Self {
        self.build_callback = Some(Rc::new(callback));
        self
    }

    pub(crate) fn handle(&self) -> NodeHandle {
        self.inner.borrow().handle
    }

    pub(crate) fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }

    pub(crate) fn parent(&self) -> Option<Self> {
        self.inner.borrow().parent.upgrade().map(Self::from_core)
    }

    pub(crate) fn node_type(&self) -> NodeType {
        self.inner.borrow().kind.node_type()
    }

    pub(crate) fn children(&self) -> Vec<Self> {
        self.inner.borrow().children.clone()
    }

    pub(crate) fn append_child_ref(&self, child: &NodeRef) {
        if Rc::ptr_eq(&self.inner, &child.inner) {
            return;
        }
        if self
            .inner
            .borrow()
            .children
            .iter()
            .any(|candidate| Rc::ptr_eq(&candidate.inner, &child.inner))
        {
            return;
        }
        if let Some(previous_parent) = child.inner.borrow().parent.upgrade() {
            if Rc::ptr_eq(&previous_parent, &self.inner) {
                return;
            }
            let child_handle = child.handle();
            {
                let mut previous_parent_core = previous_parent.borrow_mut();
                previous_parent_core
                    .children
                    .retain(|candidate| !Rc::ptr_eq(&candidate.inner, &child.inner));
            }
            if previous_parent.borrow().handle != NodeHandle::INVALID
                && child_handle != NodeHandle::INVALID
            {
                ui::remove_child(previous_parent.borrow().handle.raw(), child_handle.raw());
            }
        }

        child.inner.borrow_mut().parent = Rc::downgrade(&self.inner);
        self.inner.borrow_mut().children.push(child.clone());
        self.inner.borrow_mut().children_built = false;
        child.set_inherited_enabled(self.is_enabled_for_routing());
        if self.handle() != NodeHandle::INVALID {
            child.build();
            ui::add_child(self.handle().raw(), child.handle().raw());
            self.inner.borrow_mut().children_built = true;
            crate::frame_scheduler::mark_needs_commit();
        }
    }

    pub(crate) fn dispose(&self) {
        let children = self.inner.borrow().children.clone();
        for child in children {
            if child.handle() != NodeHandle::INVALID && self.handle() != NodeHandle::INVALID {
                ui::remove_child(self.handle().raw(), child.handle().raw());
            }
            child.dispose();
        }
        let handle = self.handle();
        if handle != NodeHandle::INVALID {
            crate::focus_adorner::handle_owner_destroyed(handle);
            crate::tool_tip_manager::ToolTipManager::handle_owner_destroyed(handle);
            crate::event::unregister_node(handle);
            ui::delete_node(handle.raw());
        }
        {
            let mut core = self.inner.borrow_mut();
            core.handle = NodeHandle::INVALID;
            core.mounted = false;
            core.children_built = false;
        }
    }

    pub(crate) fn detach_from_parent(&self) {
        let Some(parent) = self.parent() else {
            return;
        };
        let removed = {
            let mut core = parent.inner.borrow_mut();
            let Some(index) = core
                .children
                .iter()
                .position(|candidate| Rc::ptr_eq(&candidate.inner, &self.inner))
            else {
                return;
            };
            core.children.remove(index)
        };
        removed.inner.borrow_mut().parent = Weak::new();
        removed.set_inherited_enabled(true);
        parent.inner.borrow_mut().children_built = false;
        if parent.handle() != NodeHandle::INVALID && self.handle() != NodeHandle::INVALID {
            ui::remove_child(parent.handle().raw(), self.handle().raw());
            parent.inner.borrow_mut().children_built = true;
            crate::frame_scheduler::mark_needs_commit();
        }
    }

    pub(crate) fn is_visible_for_routing(&self) -> bool {
        !matches!(
            self.inner.borrow().behavior.visibility,
            Some(Visibility::Hidden | Visibility::Collapsed)
        )
    }

    pub(crate) fn is_effectively_visible_for_routing(&self) -> bool {
        let mut node = Some(self.clone());
        while let Some(current) = node {
            if !current.is_visible_for_routing() {
                return false;
            }
            node = current.parent();
        }
        true
    }

    pub(crate) fn is_enabled_for_routing(&self) -> bool {
        let behavior = &self.inner.borrow().behavior;
        behavior.enabled && behavior.inherited_enabled
    }

    pub(crate) fn is_effectively_enabled_for_routing(&self) -> bool {
        let mut node = Some(self.clone());
        while let Some(current) = node {
            if !current.is_enabled_for_routing() {
                return false;
            }
            node = current.parent();
        }
        true
    }

    pub(crate) fn set_own_enabled(&self, enabled: bool) {
        {
            let mut core = self.inner.borrow_mut();
            if core.behavior.enabled == enabled {
                return;
            }
            core.behavior.enabled = enabled;
        }
        self.apply_enabled_changed();
    }

    pub(crate) fn set_inherited_enabled(&self, enabled: bool) {
        {
            let mut core = self.inner.borrow_mut();
            if core.behavior.inherited_enabled == enabled {
                return;
            }
            core.behavior.inherited_enabled = enabled;
        }
        self.apply_enabled_changed();
    }

    pub(crate) fn on_effective_enabled_changed(&self, handler: EffectiveEnabledChangedCallback) {
        self.inner
            .borrow_mut()
            .effective_enabled_changed_callbacks
            .push(handler);
    }

    fn apply_enabled_changed(&self) {
        let (
            effective,
            handle,
            interactive,
            focusable,
            visible,
            track_semantic_disabled,
            callbacks,
            children,
        ) = {
            let mut core = self.inner.borrow_mut();
            let effective = core.behavior.enabled && core.behavior.inherited_enabled;
            if core.behavior.last_effective_enabled == effective {
                return;
            }
            core.behavior.last_effective_enabled = effective;
            (
                effective,
                core.handle,
                core.behavior.interactive,
                core.behavior.focusable,
                !matches!(
                    core.behavior.visibility,
                    Some(Visibility::Hidden | Visibility::Collapsed)
                ),
                core.behavior.track_semantic_disabled_from_enabled,
                core.effective_enabled_changed_callbacks.clone(),
                core.children.clone(),
            )
        };

        if handle != NodeHandle::INVALID {
            ui::set_interactive(handle.raw(), effective && visible && interactive);
            if let Some((focusable, tab_index)) = focusable {
                ui::set_focusable(handle.raw(), effective && visible && focusable, tab_index);
            }
            if track_semantic_disabled {
                ui::set_semantic_disabled(handle.raw(), true, !effective);
            }
            crate::frame_scheduler::mark_needs_commit();
        }

        for callback in callbacks {
            callback(effective);
        }
        if !effective {
            self.cancel_drag_state();
        }
        for child in children {
            child.set_inherited_enabled(effective);
        }
    }

    pub(crate) fn semantic_role_for_routing(&self) -> Option<SemanticRole> {
        self.inner.borrow().behavior.semantic_role
    }

    pub(crate) fn cursor_style_for_routing(&self) -> CursorStyle {
        self.inner
            .borrow()
            .behavior
            .cursor
            .unwrap_or(CursorStyle::Default)
    }

    pub(crate) fn require_interactive(&self) {
        let (handle, enabled, changed) = {
            let mut inner = self.inner.borrow_mut();
            let changed = !inner.behavior.interactive;
            inner.behavior.interactive = true;
            (inner.handle, inner.behavior.enabled, changed)
        };
        if handle != NodeHandle::INVALID {
            ui::set_interactive(handle.raw(), enabled);
            if changed {
                crate::frame_scheduler::mark_needs_commit();
            }
        }
    }

    pub(crate) fn is_selectable_text_for_routing(&self) -> bool {
        self.inner.borrow().behavior.selectable_text
    }

    pub(crate) fn is_editable_text_for_routing(&self) -> bool {
        self.inner.borrow().behavior.editable_text
    }

    pub(crate) fn preserves_selection_on_pointer_down_for_routing(&self) -> bool {
        self.inner
            .borrow()
            .behavior
            .preserve_selection_on_pointer_down
    }

    pub(crate) fn link_url_for_routing(&self) -> Option<String> {
        self.inner.borrow().behavior.link_url.clone()
    }

    pub(crate) fn image_url_for_routing(&self) -> Option<String> {
        self.inner.borrow().behavior.image_url.clone()
    }

    pub(crate) fn is_image_or_svg_for_routing(&self) -> bool {
        matches!(self.inner.borrow().kind, NodeKind::Image | NodeKind::Svg)
    }

    pub(crate) fn text_content_for_routing(&self) -> Option<String> {
        self.inner.borrow().behavior.text_content.clone()
    }

    pub(crate) fn set_link_url_for_routing(&self, value: Option<String>) {
        self.inner.borrow_mut().behavior.link_url = value;
    }

    pub(crate) fn set_text_content_for_routing(&self, value: Option<String>) {
        self.inner.borrow_mut().behavior.text_content = value;
    }

    pub(crate) fn set_semantic_label_for_routing(&self, value: Option<String>) {
        self.inner.borrow_mut().behavior.semantic_label = value;
    }

    pub(crate) fn set_image_url_for_routing(&self, value: Option<String>) {
        self.inner.borrow_mut().behavior.image_url = value;
    }

    pub(crate) fn set_link_preview_handlers_for_routing(
        &self,
        pin: Option<Rc<dyn Fn()>>,
        release: Option<Rc<dyn Fn()>>,
    ) {
        let mut inner = self.inner.borrow_mut();
        inner.behavior.link_preview_pin = pin;
        inner.behavior.link_preview_release = release;
    }

    pub(crate) fn pin_link_preview_for_routing(&self) {
        let callback = self.inner.borrow().behavior.link_preview_pin.clone();
        if let Some(callback) = callback {
            callback();
        }
    }

    pub(crate) fn release_link_preview_for_routing(&self) {
        let callback = self.inner.borrow().behavior.link_preview_release.clone();
        if let Some(callback) = callback {
            callback();
        }
    }

    pub(crate) fn is_context_menu_disabled_for_routing(&self) -> bool {
        self.inner.borrow().behavior.context_menu_disabled
    }

    pub(crate) fn context_menu_handler_for_routing(&self) -> Option<ContextMenuCallback> {
        self.inner.borrow().behavior.context_menu_handler.clone()
    }

    pub(crate) fn tool_tip_for_routing(&self) -> Option<ToolTip> {
        self.inner.borrow().behavior.tool_tip.clone()
    }

    pub(crate) fn node_id(&self) -> Option<String> {
        self.inner.borrow().behavior.node_id.clone()
    }

    pub(crate) fn gesture_intent_for_routing(&self) -> crate::event::GestureIntent {
        let handlers = &self.inner.borrow().handlers;
        crate::event::GestureIntent::from_callbacks(
            handlers.pan_gesture.is_some(),
            handlers.pinch_gesture.is_some(),
        )
    }

    pub(crate) fn is_scroll_view_for_routing(&self) -> bool {
        self.inner.borrow().kind == NodeKind::ScrollView
    }

    pub(crate) fn has_long_press_for_routing(&self) -> bool {
        self.inner.borrow().handlers.long_press.is_some() || self.has_drag_source()
    }

    pub(crate) fn scroll_routing_state(&self) -> Option<ScrollRoutingState> {
        self.inner.borrow().scroll_routing
    }

    pub(crate) fn set_scroll_routing_enabled(&self, enabled_x: bool, enabled_y: bool) {
        let mut inner = self.inner.borrow_mut();
        let state = inner.scroll_routing.get_or_insert(ScrollRoutingState {
            enabled_x,
            enabled_y,
            ..ScrollRoutingState::default()
        });
        state.enabled_x = enabled_x;
        state.enabled_y = enabled_y;
    }

    pub(crate) fn set_scroll_routing_offsets(&self, offset_x: f32, offset_y: f32) {
        if let Some(state) = self.inner.borrow_mut().scroll_routing.as_mut() {
            state.offset_x = offset_x;
            state.offset_y = offset_y;
        }
    }

    pub(crate) fn set_scroll_routing_metrics(
        &self,
        offset_x: f32,
        offset_y: f32,
        content_width: f32,
        content_height: f32,
        viewport_width: f32,
        viewport_height: f32,
    ) {
        if let Some(state) = self.inner.borrow_mut().scroll_routing.as_mut() {
            state.offset_x = offset_x;
            state.offset_y = offset_y;
            state.content_width = content_width;
            state.content_height = content_height;
            state.viewport_width = viewport_width;
            state.viewport_height = viewport_height;
        }
    }

    pub(crate) fn absolute_to_local_position(&self, absolute_x: f32, absolute_y: f32) -> [f32; 2] {
        let bounds = if self.handle() == NodeHandle::INVALID {
            [0.0; 4]
        } else {
            ui::get_bounds(self.handle().raw()).unwrap_or([0.0; 4])
        };
        [absolute_x - bounds[0], absolute_y - bounds[1]]
    }

    pub(crate) fn retain_attachment<T: Any>(&self, attachment: Rc<T>) {
        let retained: Rc<dyn Any> = attachment;
        self.inner.borrow_mut().retained_attachments.push(retained);
    }

    pub(crate) fn set_node_id(&self, node_id: impl Into<String>) {
        let node_id = node_id.into();
        self.inner.borrow_mut().behavior.node_id = Some(node_id.clone());
        if self.handle() != NodeHandle::INVALID {
            ui::set_node_id(self.handle().raw(), &node_id);
        }
    }

    pub(crate) fn bind_scroll_proxy_target_handle(&self, scroll_handle: u64) {
        self.inner.borrow_mut().behavior.scroll_proxy_target = Some(scroll_handle);
        if self.handle() != NodeHandle::INVALID {
            ui::set_scroll_proxy_target(self.handle().raw(), scroll_handle);
        }
    }

    pub(crate) fn register_persisted_state_adapter(&self, adapter: Rc<dyn PersistedStateAdapter>) {
        let mut core = self.inner.borrow_mut();
        if let Some(index) = core
            .persisted_state_adapters
            .iter()
            .position(|existing| existing.kind() == adapter.kind())
        {
            core.persisted_state_adapters[index] = adapter;
            return;
        }
        core.persisted_state_adapters.push(adapter);
    }

    pub(crate) fn capture_persisted_state_tree(&self) {
        self.capture_persisted_state();
        let children = self.inner.borrow().children.clone();
        for child in children {
            child.capture_persisted_state_tree();
        }
    }

    pub(crate) fn restore_persisted_state_tree(&self) {
        let children = self.inner.borrow().children.clone();
        for child in children {
            child.restore_persisted_state_tree();
        }
        self.restore_persisted_state();
    }

    fn capture_persisted_state(&self) {
        let (node_id, adapters) = {
            let core = self.inner.borrow();
            (
                core.behavior.node_id.clone(),
                core.persisted_state_adapters.clone(),
            )
        };
        let Some(node_id) = node_id else {
            return;
        };
        if node_id.is_empty() {
            return;
        }
        for adapter in adapters {
            let Some(payload) = adapter.capture() else {
                continue;
            };
            crate::persisted::store_text_state(
                &node_id,
                adapter.kind(),
                adapter.version(),
                &payload,
            );
        }
    }

    fn restore_persisted_state(&self) {
        let (node_id, adapters) = {
            let core = self.inner.borrow();
            (
                core.behavior.node_id.clone(),
                core.persisted_state_adapters.clone(),
            )
        };
        let Some(node_id) = node_id else {
            return;
        };
        if node_id.is_empty() {
            return;
        }
        for adapter in adapters {
            let Some(persisted) = crate::persisted::try_load_text_state(&node_id, adapter.kind())
            else {
                continue;
            };
            adapter.restore(&persisted.payload, persisted.version);
        }
    }

    pub(crate) fn long_press_minimum_duration_ms_for_routing(&self) -> i32 {
        self.inner.borrow().handlers.long_press_minimum_duration_ms
    }

    pub(crate) fn long_press_movement_tolerance_for_routing(&self) -> f32 {
        self.inner.borrow().handlers.long_press_movement_tolerance
    }

    pub(crate) fn handle_pointer_event(&self, event: &mut PointerEventArgs) {
        if !self.is_enabled_for_routing() {
            self.cancel_drag_state();
            return;
        }
        [event.x, event.y] = self.absolute_to_local_position(event.scene_x, event.scene_y);
        let handlers = {
            let core = self.inner.borrow();
            core.handlers.clone()
        };
        match event.event_type {
            crate::ffi::PointerEventType::Down => {
                crate::tool_tip_manager::ToolTipManager::handle_pointer_down(self);
                if let Some(handler) = handlers.pointer_down {
                    handler(event);
                }
                if event.handled {
                    return;
                }
                let drag_gesture = if self.has_drag_source() {
                    Some(self.ensure_drag_gesture())
                } else {
                    None
                };
                let is_primary_button = is_primary_activation_pointer(event);
                if let Some(drag_gesture) = drag_gesture {
                    if is_primary_button {
                        drag_gesture.borrow_mut().handle_pointer_down(
                            event.x,
                            event.y,
                            event.modifiers,
                            matches!(event.pointer_type, PointerType::Touch | PointerType::Pen),
                        );
                        let mut core = self.inner.borrow_mut();
                        core.drag_click_pending = handlers.pointer_click.is_some();
                        core.drag_click_pending_count = event.click_count;
                        core.click_pending = false;
                        core.click_pending_count = 0;
                        return;
                    }
                }
                let mut core = self.inner.borrow_mut();
                core.drag_click_pending = false;
                core.drag_click_pending_count = 0;
                core.click_pending =
                    handlers.pointer_click.is_some() && is_primary_activation_pointer(event);
                core.click_pending_count = event.click_count;
            }
            crate::ffi::PointerEventType::Move => {
                crate::tool_tip_manager::ToolTipManager::handle_pointer_move(
                    self,
                    event.scene_x,
                    event.scene_y,
                );
                if let Some(handler) = handlers.pointer_move {
                    handler(event);
                }
                if event.handled {
                    return;
                }
                let drag_gesture = self.inner.borrow().drag_gesture.clone();
                if let Some(drag_gesture) = drag_gesture {
                    let has_drag_source = self.has_drag_source();
                    let is_dragging = drag_gesture
                        .try_borrow()
                        .map(|gesture| gesture.is_dragging())
                        .unwrap_or(true);
                    if has_drag_source || is_dragging {
                        if let Ok(mut gesture) = drag_gesture.try_borrow_mut() {
                            gesture.handle_pointer_move(event.x, event.y, event.modifiers);
                        }
                    }
                }
            }
            crate::ffi::PointerEventType::Up => {
                if let Some(handler) = handlers.pointer_up {
                    handler(event);
                }
                if event.handled {
                    self.cancel_drag_state();
                    return;
                }
                let drag_gesture = self.inner.borrow().drag_gesture.clone();
                let has_drag_source = self.has_drag_source();
                let drag_gesture_is_dragging = drag_gesture
                    .as_ref()
                    .map(|gesture| {
                        gesture
                            .try_borrow()
                            .map(|gesture| gesture.is_dragging())
                            .unwrap_or(true)
                    })
                    .unwrap_or(false);
                let drag_gesture_active = drag_gesture
                    .as_ref()
                    .map(|_| has_drag_source || drag_gesture_is_dragging)
                    .unwrap_or(false);
                let (can_fire_pending_click, pending_click_count, can_fire_click, click_count) = {
                    let core = self.inner.borrow();
                    let can_fire_pending_click = core.drag_click_pending
                        && (!drag_gesture_active || !drag_gesture_is_dragging);
                    (
                        can_fire_pending_click,
                        core.drag_click_pending_count,
                        core.click_pending,
                        core.click_pending_count,
                    )
                };
                if let Some(drag_gesture) = drag_gesture.as_ref() {
                    if drag_gesture_active {
                        if let Ok(mut gesture) = drag_gesture.try_borrow_mut() {
                            gesture.handle_pointer_up(event.x, event.y, event.modifiers);
                        }
                    }
                }
                if can_fire_click {
                    if let Some(handler) = handlers.pointer_click.clone() {
                        event.click_count = if click_count > 0 { click_count } else { 1 };
                        handler(event);
                    }
                }
                self.clear_click_pending_state();
                if can_fire_pending_click {
                    if let Some(handler) = handlers.pointer_click {
                        event.click_count = if pending_click_count > 0 {
                            pending_click_count
                        } else {
                            1
                        };
                        handler(event);
                    }
                }
                self.clear_drag_click_pending_state();
            }
            crate::ffi::PointerEventType::Enter => {
                if let Some(handler) = handlers.pointer_enter {
                    handler(event);
                }
                if !event.handled {
                    crate::tool_tip_manager::ToolTipManager::handle_pointer_enter(
                        self,
                        self.tool_tip_for_routing(),
                        event.scene_x,
                        event.scene_y,
                    );
                }
            }
            crate::ffi::PointerEventType::Leave => {
                self.clear_pending_pointer_state();
                if let Some(handler) = handlers.pointer_leave {
                    handler(event);
                }
                crate::tool_tip_manager::ToolTipManager::handle_pointer_leave(self);
            }
            crate::ffi::PointerEventType::Cancel => {
                self.cancel_drag_state();
                if let Some(handler) = handlers.pointer_cancel {
                    handler(event);
                }
                crate::tool_tip_manager::ToolTipManager::handle_pointer_leave(self);
            }
        }
    }

    fn cancel_drag_state(&self) {
        let gesture = {
            let mut core = self.inner.borrow_mut();
            core.drag_click_pending = false;
            core.drag_click_pending_count = 0;
            core.click_pending = false;
            core.click_pending_count = 0;
            core.drag_gesture.clone()
        };
        if let Some(gesture) = gesture {
            gesture.borrow_mut().cancel();
        }
    }

    fn clear_pending_pointer_state(&self) {
        let Ok(mut core) = self.inner.try_borrow_mut() else {
            return;
        };
        core.drag_click_pending = false;
        core.drag_click_pending_count = 0;
        core.click_pending = false;
        core.click_pending_count = 0;
    }

    fn clear_click_pending_state(&self) {
        let Ok(mut core) = self.inner.try_borrow_mut() else {
            return;
        };
        core.click_pending = false;
        core.click_pending_count = 0;
    }

    fn clear_drag_click_pending_state(&self) {
        let Ok(mut core) = self.inner.try_borrow_mut() else {
            return;
        };
        core.drag_click_pending = false;
        core.drag_click_pending_count = 0;
    }

    pub(crate) fn handle_bubbled_pointer_event(&self, event: &mut PointerEventArgs) {
        self.handle_pointer_event(event);
    }

    pub(crate) fn handle_wheel_event(&self, event: &mut WheelEventArgs) {
        if !self.is_enabled_for_routing() || !self.is_visible_for_routing() {
            return;
        }
        [event.x, event.y] = self.absolute_to_local_position(event.scene_x, event.scene_y);
        if let Some(handler) = self.inner.borrow().handlers.wheel.clone() {
            handler(event);
        }
    }

    pub(crate) fn handle_key_event(&self, event: &mut KeyEventArgs) {
        if !self.is_enabled_for_routing() {
            return;
        }
        let handlers = self.inner.borrow().handlers.clone();
        match event.event_type {
            crate::ffi::KeyEventType::Down => {
                if let Some(handler) = handlers.key_down {
                    handler(event);
                }
            }
            crate::ffi::KeyEventType::Up => {
                if let Some(handler) = handlers.key_up {
                    handler(event);
                }
            }
        }
    }

    pub(crate) fn handle_focus_changed(&self, event: FocusChangedEventArgs) {
        let handler = self.inner.borrow().handlers.focus_changed.clone();
        if let Some(handler) = handler {
            handler(event);
        }
        crate::tool_tip_manager::ToolTipManager::handle_focus_changed(
            self,
            self.tool_tip_for_routing(),
            event.focused,
        );
    }

    pub(crate) fn handle_scroll_changed(
        &self,
        offset_x: f32,
        offset_y: f32,
        content_width: f32,
        content_height: f32,
        viewport_width: f32,
        viewport_height: f32,
    ) {
        self.set_scroll_routing_metrics(
            offset_x,
            offset_y,
            content_width,
            content_height,
            viewport_width,
            viewport_height,
        );
        let handler = self.inner.borrow().handlers.scroll_changed.clone();
        if let Some(handler) = handler {
            handler(
                offset_x,
                offset_y,
                content_width,
                content_height,
                viewport_width,
                viewport_height,
            );
        }
    }

    pub(crate) fn handle_selection_changed(&self, start: u32, end: u32) {
        let handler = self.inner.borrow().handlers.selection_changed.clone();
        if let Some(handler) = handler {
            handler(SelectionChangedEventArgs { start, end });
        }
    }

    pub(crate) fn handle_text_changed(&self, text: String) {
        let handler = self.inner.borrow().handlers.text_changed.clone();
        if let Some(handler) = handler {
            handler(TextChangedEventArgs { text });
        }
    }

    pub(crate) fn handle_text_replaced(&self, start: u32, end: u32, text: String) {
        let handler = self.inner.borrow().handlers.text_replaced.clone();
        if let Some(handler) = handler {
            handler(start, end, text);
        }
    }

    pub(crate) fn handle_cross_selection_changed(&self, text: String) {
        let handler = self.inner.borrow().handlers.cross_selection_changed.clone();
        if let Some(handler) = handler {
            handler(text);
        }
    }

    pub(crate) fn handle_gesture_event(&self, event: &mut GestureEventArgs) {
        if !self.is_enabled_for_routing() || !self.is_visible_for_routing() {
            return;
        }
        [event.x, event.y] = self.absolute_to_local_position(event.scene_x, event.scene_y);
        let handlers = self.inner.borrow().handlers.clone();
        match event.kind {
            crate::event::GestureEventKind::Pan => {
                if let Some(handler) = handlers.pan_gesture {
                    handler(event);
                }
            }
            crate::event::GestureEventKind::Pinch => {
                if let Some(handler) = handlers.pinch_gesture {
                    handler(event);
                }
            }
            crate::event::GestureEventKind::None => {}
        }
    }

    pub(crate) fn handle_bubbled_gesture_event(&self, event: &mut GestureEventArgs) {
        self.handle_gesture_event(event);
    }

    pub(crate) fn handle_long_press_event(&self, event: &mut LongPressEventArgs) {
        if !self.is_enabled_for_routing() || !self.is_visible_for_routing() {
            return;
        }
        [event.x, event.y] = self.absolute_to_local_position(event.scene_x, event.scene_y);
        if let Some(handler) = self.inner.borrow().handlers.long_press.clone() {
            handler(event);
        }
        if !event.handled
            && matches!(event.pointer_type, PointerType::Touch | PointerType::Pen)
            && self.has_drag_source()
        {
            event.handled = self.ensure_drag_gesture().borrow_mut().handle_long_press(
                event.x,
                event.y,
                event.modifiers,
            );
        }
    }

    pub(crate) fn handle_bubbled_long_press_event(&self, event: &mut LongPressEventArgs) {
        self.handle_long_press_event(event);
    }

    pub(crate) fn handle_custom_draw(&self, canvas_ptr: usize) {
        if let Some(callback) = self.inner.borrow().draw_callback.clone() {
            let mut context = DrawContext::new(canvas_ptr);
            callback(&mut context);
        }
    }

    pub(crate) fn build(&self) {
        if let Some(callback) = self.build_callback.as_ref() {
            callback();
        }
    }

    pub(crate) fn downgrade(&self) -> WeakNodeRef {
        WeakNodeRef {
            inner: Rc::downgrade(&self.inner),
        }
    }

    pub(crate) fn ensure_drag_gesture(&self) -> Rc<RefCell<DragGesture>> {
        if let Some(gesture) = self.inner.borrow().drag_gesture.clone() {
            return gesture;
        }
        let weak = self.downgrade();
        let gesture = Rc::new(RefCell::new(DragGesture::new(self)));
        gesture.borrow_mut().threshold(4.0);
        gesture.borrow_mut().set_started({
            let weak = weak.clone();
            move |_event: DragStartedEvent| {
                if let Some(node) = weak.upgrade() {
                    node.handle_drag_gesture_started();
                }
            }
        });
        gesture
            .borrow_mut()
            .set_completed(move |event: DragCompletedEvent| {
                if let Some(node) = weak.upgrade() {
                    node.handle_drag_gesture_completed(event);
                }
            });
        self.inner.borrow_mut().drag_gesture = Some(gesture.clone());
        gesture
    }

    fn handle_drag_gesture_started(&self) {
        if !crate::event::begin_drag_session(self.clone()) {
            if let Some(gesture) = self.inner.borrow().drag_gesture.clone() {
                gesture.borrow_mut().cancel();
            }
        }
    }

    fn handle_drag_gesture_completed(&self, event: DragCompletedEvent) {
        let mut core = self.inner.borrow_mut();
        core.drag_click_pending = false;
        core.drag_click_pending_count = 0;
        drop(core);
        if event.cancelled {
            crate::event::cancel_drag_session_for_source(self);
        }
    }

    pub(crate) fn has_drag_source(&self) -> bool {
        self.inner.borrow().drag_data_callback.is_some()
    }

    pub(crate) fn create_drag_data_object(&self) -> Option<DragDataObject> {
        let callback = self.inner.borrow().drag_data_callback.clone();
        callback.and_then(|callback| callback())
    }

    pub(crate) fn get_drag_allowed_effects(&self) -> DragDropEffects {
        self.inner.borrow().drag_allowed_effects
    }

    pub(crate) fn notify_drag_completed(&self, effect: DragDropEffects) {
        let callback = self.inner.borrow().drag_completed_callback.clone();
        if let Some(callback) = callback {
            callback(DragCompletedEventArgs { effect });
        }
    }

    pub(crate) fn allows_drop(&self) -> bool {
        self.inner.borrow().drop_allowed
    }

    pub(crate) fn has_drag_enter_handler(&self) -> bool {
        self.inner.borrow().drag_enter_callback.is_some()
    }

    pub(crate) fn has_drag_over_handler(&self) -> bool {
        self.inner.borrow().drag_over_callback.is_some()
    }

    pub(crate) fn handle_drag_enter(&self, args: DragEventArgs) -> DropProposal {
        let callback = self.inner.borrow().drag_enter_callback.clone();
        callback
            .map(|callback| callback(args))
            .unwrap_or_else(DropProposal::none)
    }

    pub(crate) fn handle_drag_over(&self, args: DragEventArgs) -> DropProposal {
        let callback = self.inner.borrow().drag_over_callback.clone();
        callback
            .map(|callback| callback(args))
            .unwrap_or_else(DropProposal::none)
    }

    pub(crate) fn handle_drag_leave(&self, args: DragEventArgs) {
        let callback = self.inner.borrow().drag_leave_callback.clone();
        if let Some(callback) = callback {
            callback(args);
        }
    }

    pub(crate) fn handle_drop_event(&self, args: DragEventArgs) {
        let callback = self.inner.borrow().drop_callback.clone();
        if let Some(callback) = callback {
            callback(args);
        }
    }

    pub(crate) fn allows_external_drop(&self) -> bool {
        self.inner.borrow().behavior.external_drop_allowed
    }

    pub(crate) fn has_external_drag_enter_handler(&self) -> bool {
        self.inner.borrow().external_drag_enter_callback.is_some()
    }

    pub(crate) fn has_external_drag_over_handler(&self) -> bool {
        self.inner.borrow().external_drag_over_callback.is_some()
    }

    pub(crate) fn handle_external_drag_enter(&self, args: ExternalDropEventArgs) -> DropProposal {
        let callback = self.inner.borrow().external_drag_enter_callback.clone();
        callback
            .map(|callback| callback(args))
            .unwrap_or_else(DropProposal::none)
    }

    pub(crate) fn handle_external_drag_over(&self, args: ExternalDropEventArgs) -> DropProposal {
        let callback = self.inner.borrow().external_drag_over_callback.clone();
        callback
            .map(|callback| callback(args))
            .unwrap_or_else(DropProposal::none)
    }

    pub(crate) fn handle_external_drag_leave(&self, args: ExternalDropEventArgs) {
        let callback = self.inner.borrow().external_drag_leave_callback.clone();
        if let Some(callback) = callback {
            callback(args);
        }
    }

    pub(crate) fn handle_external_drop_event(&self, args: ExternalDropEventArgs) {
        let callback = self.inner.borrow().external_drop_callback.clone();
        if let Some(callback) = callback {
            callback(args);
        }
    }
}

pub trait Node: Clone + 'static {
    #[doc(hidden)]
    fn node_ref(&self) -> NodeRef {
        self.retained_node_ref()
    }
    #[doc(hidden)]
    fn retained_node_ref(&self) -> NodeRef;
    #[doc(hidden)]
    fn retained_owner_attachment(&self) -> Option<Rc<dyn Any>> {
        None
    }
    #[doc(hidden)]
    fn build_self(&self);

    fn node_id(&self, node_id: impl Into<String>) -> &Self
    where
        Self: Sized,
    {
        let node_ref = self.node_ref();
        node_ref.set_node_id(node_id);
        if self.has_built_handle() {
            self.notify_retained_mutation();
        }
        self
    }

    fn semantic_role(&self, role: SemanticRole) -> &Self
    where
        Self: Sized,
    {
        let node_ref = self.node_ref();
        node_ref.inner.borrow_mut().behavior.semantic_role = Some(role);
        if self.has_built_handle() {
            ui::set_semantic_role(self.handle().raw(), role as u32);
            self.notify_retained_mutation();
        }
        self
    }

    fn semantic_label(&self, label: impl Into<String>) -> &Self
    where
        Self: Sized,
    {
        let label = label.into();
        let node_ref = self.node_ref();
        node_ref.inner.borrow_mut().behavior.semantic_label = Some(label.clone());
        if self.has_built_handle() {
            ui::set_semantic_label(self.handle().raw(), &label);
            self.notify_retained_mutation();
        }
        self
    }

    fn persist_state(&self, adapter: Rc<dyn PersistedStateAdapter>) -> &Self
    where
        Self: Sized,
    {
        self.node_ref().register_persisted_state_adapter(adapter);
        self
    }

    fn handle(&self) -> NodeHandle {
        self.node_ref().inner.borrow().handle
    }

    fn has_built_handle(&self) -> bool {
        self.handle() != NodeHandle::INVALID
    }

    fn on_context_menu(&self, handler: impl Fn(ContextMenuEventArgs) + 'static) -> &Self
    where
        Self: Sized,
    {
        self.node_ref()
            .inner
            .borrow_mut()
            .behavior
            .context_menu_handler = Some(Rc::new(handler));
        self
    }

    fn clear_context_menu(&self) -> &Self
    where
        Self: Sized,
    {
        self.node_ref()
            .inner
            .borrow_mut()
            .behavior
            .context_menu_handler = None;
        self
    }

    fn disable_context_menu(&self, flag: bool) -> &Self
    where
        Self: Sized,
    {
        self.node_ref()
            .inner
            .borrow_mut()
            .behavior
            .context_menu_disabled = flag;
        self
    }

    fn on_click(&self, handler: impl Fn(&mut PointerEventArgs) + 'static) -> &Self
    where
        Self: Sized,
    {
        let node_ref = self.node_ref();
        node_ref.inner.borrow_mut().handlers.pointer_click = Some(Rc::new(handler));
        node_ref.require_interactive();
        self
    }

    fn on_pointer_down(&self, handler: impl Fn(&mut PointerEventArgs) + 'static) -> &Self
    where
        Self: Sized,
    {
        let node_ref = self.node_ref();
        node_ref.inner.borrow_mut().handlers.pointer_down = Some(Rc::new(handler));
        node_ref.require_interactive();
        self
    }

    fn on_pointer_move(&self, handler: impl Fn(&mut PointerEventArgs) + 'static) -> &Self
    where
        Self: Sized,
    {
        let node_ref = self.node_ref();
        node_ref.inner.borrow_mut().handlers.pointer_move = Some(Rc::new(handler));
        node_ref.require_interactive();
        self
    }

    fn on_pointer_up(&self, handler: impl Fn(&mut PointerEventArgs) + 'static) -> &Self
    where
        Self: Sized,
    {
        let node_ref = self.node_ref();
        node_ref.inner.borrow_mut().handlers.pointer_up = Some(Rc::new(handler));
        node_ref.require_interactive();
        self
    }

    fn on_pointer_enter(&self, handler: impl Fn(&mut PointerEventArgs) + 'static) -> &Self
    where
        Self: Sized,
    {
        let node_ref = self.node_ref();
        node_ref.inner.borrow_mut().handlers.pointer_enter = Some(Rc::new(handler));
        node_ref.require_interactive();
        self
    }

    fn on_pointer_leave(&self, handler: impl Fn(&mut PointerEventArgs) + 'static) -> &Self
    where
        Self: Sized,
    {
        let node_ref = self.node_ref();
        node_ref.inner.borrow_mut().handlers.pointer_leave = Some(Rc::new(handler));
        node_ref.require_interactive();
        self
    }

    fn on_pointer_cancel(&self, handler: impl Fn(&mut PointerEventArgs) + 'static) -> &Self
    where
        Self: Sized,
    {
        let node_ref = self.node_ref();
        node_ref.inner.borrow_mut().handlers.pointer_cancel = Some(Rc::new(handler));
        node_ref.require_interactive();
        self
    }

    fn on_wheel(&self, handler: impl Fn(&mut WheelEventArgs) + 'static) -> &Self
    where
        Self: Sized,
    {
        let node_ref = self.node_ref();
        node_ref.inner.borrow_mut().handlers.wheel = Some(Rc::new(handler));
        node_ref.require_interactive();
        self
    }

    fn on_pan_gesture(&self, handler: impl Fn(&mut GestureEventArgs) + 'static) -> &Self
    where
        Self: Sized,
    {
        self.node_ref().inner.borrow_mut().handlers.pan_gesture = Some(Rc::new(handler));
        self
    }

    fn on_pinch_gesture(&self, handler: impl Fn(&mut GestureEventArgs) + 'static) -> &Self
    where
        Self: Sized,
    {
        self.node_ref().inner.borrow_mut().handlers.pinch_gesture = Some(Rc::new(handler));
        self
    }

    fn long_press_options(&self, minimum_duration_ms: i32, movement_tolerance: f32) -> &Self
    where
        Self: Sized,
    {
        let node_ref = self.node_ref();
        let mut core = node_ref.inner.borrow_mut();
        let handlers = &mut core.handlers;
        handlers.long_press_minimum_duration_ms = minimum_duration_ms.max(0);
        handlers.long_press_movement_tolerance = movement_tolerance.max(0.0);
        self
    }

    fn on_long_press(&self, handler: impl Fn(&mut LongPressEventArgs) + 'static) -> &Self
    where
        Self: Sized,
    {
        let node_ref = self.node_ref();
        node_ref.inner.borrow_mut().handlers.long_press = Some(Rc::new(handler));
        node_ref.require_interactive();
        self
    }

    fn focusable(&self, enabled: bool, tab_index: i32) -> &Self
    where
        Self: Sized,
    {
        let node_ref = self.node_ref();
        let mut core = node_ref.inner.borrow_mut();
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

    fn on_key_down(&self, handler: impl Fn(&mut KeyEventArgs) + 'static) -> &Self
    where
        Self: Sized,
    {
        self.node_ref().inner.borrow_mut().handlers.key_down = Some(Rc::new(handler));
        self
    }

    fn on_key_up(&self, handler: impl Fn(&mut KeyEventArgs) + 'static) -> &Self
    where
        Self: Sized,
    {
        self.node_ref().inner.borrow_mut().handlers.key_up = Some(Rc::new(handler));
        self
    }

    fn on_focus_changed(&self, handler: impl Fn(FocusChangedEventArgs) + 'static) -> &Self
    where
        Self: Sized,
    {
        self.node_ref().inner.borrow_mut().handlers.focus_changed = Some(Rc::new(handler));
        self
    }

    fn drag_allowed_effects(&self, effects: DragDropEffects) -> &Self
    where
        Self: Sized,
    {
        self.node_ref().inner.borrow_mut().drag_allowed_effects = effects;
        self
    }

    fn drag_data(&self, handler: impl Fn() -> Option<DragDataObject> + 'static) -> &Self
    where
        Self: Sized,
    {
        let node_ref = self.node_ref();
        node_ref.inner.borrow_mut().drag_data_callback = Some(Rc::new(handler));
        node_ref.require_interactive();
        node_ref.ensure_drag_gesture();
        self
    }

    fn clear_drag_data(&self) -> &Self
    where
        Self: Sized,
    {
        self.node_ref().inner.borrow_mut().drag_data_callback = None;
        self
    }

    fn allow_drop(&self, flag: bool) -> &Self
    where
        Self: Sized,
    {
        let node_ref = self.node_ref();
        node_ref.inner.borrow_mut().drop_allowed = flag;
        if flag {
            node_ref.require_interactive();
        }
        self
    }

    fn on_drag_completed(&self, handler: impl Fn(DragCompletedEventArgs) + 'static) -> &Self
    where
        Self: Sized,
    {
        self.node_ref().inner.borrow_mut().drag_completed_callback = Some(Rc::new(handler));
        self
    }

    fn on_drag_enter(&self, handler: impl Fn(DragEventArgs) -> DropProposal + 'static) -> &Self
    where
        Self: Sized,
    {
        let node_ref = self.node_ref();
        node_ref.inner.borrow_mut().drag_enter_callback = Some(Rc::new(handler));
        self.allow_drop(true);
        node_ref.require_interactive();
        self
    }

    fn on_drag_over(&self, handler: impl Fn(DragEventArgs) -> DropProposal + 'static) -> &Self
    where
        Self: Sized,
    {
        let node_ref = self.node_ref();
        node_ref.inner.borrow_mut().drag_over_callback = Some(Rc::new(handler));
        self.allow_drop(true);
        node_ref.require_interactive();
        self
    }

    fn on_drag_leave(&self, handler: impl Fn(DragEventArgs) + 'static) -> &Self
    where
        Self: Sized,
    {
        let node_ref = self.node_ref();
        node_ref.inner.borrow_mut().drag_leave_callback = Some(Rc::new(handler));
        self.allow_drop(true);
        node_ref.require_interactive();
        self
    }

    fn on_drop(&self, handler: impl Fn(DragEventArgs) + 'static) -> &Self
    where
        Self: Sized,
    {
        let node_ref = self.node_ref();
        node_ref.inner.borrow_mut().drop_callback = Some(Rc::new(handler));
        self.allow_drop(true);
        node_ref.require_interactive();
        self
    }

    fn allow_external_drop(&self, flag: bool) -> &Self
    where
        Self: Sized,
    {
        let node_ref = self.node_ref();
        node_ref.inner.borrow_mut().behavior.external_drop_allowed = flag;
        if flag {
            node_ref.require_interactive();
        }
        self
    }

    fn on_external_drag_enter(
        &self,
        handler: impl Fn(ExternalDropEventArgs) -> DropProposal + 'static,
    ) -> &Self
    where
        Self: Sized,
    {
        let node_ref = self.node_ref();
        node_ref.inner.borrow_mut().external_drag_enter_callback = Some(Rc::new(handler));
        self.allow_external_drop(true);
        node_ref.require_interactive();
        self
    }

    fn on_external_drag_over(
        &self,
        handler: impl Fn(ExternalDropEventArgs) -> DropProposal + 'static,
    ) -> &Self
    where
        Self: Sized,
    {
        let node_ref = self.node_ref();
        node_ref.inner.borrow_mut().external_drag_over_callback = Some(Rc::new(handler));
        self.allow_external_drop(true);
        node_ref.require_interactive();
        self
    }

    fn on_external_drag_leave(&self, handler: impl Fn(ExternalDropEventArgs) + 'static) -> &Self
    where
        Self: Sized,
    {
        let node_ref = self.node_ref();
        node_ref.inner.borrow_mut().external_drag_leave_callback = Some(Rc::new(handler));
        self.allow_external_drop(true);
        node_ref.require_interactive();
        self
    }

    fn on_external_drop(&self, handler: impl Fn(ExternalDropEventArgs) + 'static) -> &Self
    where
        Self: Sized,
    {
        let node_ref = self.node_ref();
        node_ref.inner.borrow_mut().external_drop_callback = Some(Rc::new(handler));
        self.allow_external_drop(true);
        node_ref.require_interactive();
        self
    }

    fn get_bounds(&self) -> [f32; 4] {
        if !self.has_built_handle() {
            return [0.0; 4];
        }
        ui::get_bounds(self.handle().raw()).unwrap_or([0.0; 4])
    }

    fn absolute_to_local_position(&self, absolute_x: f32, absolute_y: f32) -> [f32; 2] {
        let bounds = self.get_bounds();
        [absolute_x - bounds[0], absolute_y - bounds[1]]
    }

    fn local_to_absolute_position(&self, local_x: f32, local_y: f32) -> [f32; 2] {
        let bounds = self.get_bounds();
        [bounds[0] + local_x, bounds[1] + local_y]
    }

    fn ensure_handle(&self) {
        if self.has_built_handle() {
            return;
        }
        let node_ref = self.node_ref();
        let node_type = node_ref.inner.borrow().kind.node_type();
        let handle = NodeHandle(ui::create_node(node_type as u32));
        {
            let mut core = node_ref.inner.borrow_mut();
            core.handle = handle;
            core.mounted = true;
        }
        crate::event::register_node(&node_ref);
    }

    fn build(&self) {
        self.ensure_handle();
        self.build_self();
        self.build_children();
    }

    fn dispose(&self) {
        self.node_ref().dispose();
    }

    fn build_children(&self) {
        let node_ref = self.node_ref();
        {
            let mut core = node_ref.inner.borrow_mut();
            if core.children_built {
                return;
            }
            core.children_built = true;
        }
        let children = node_ref.inner.borrow().children.clone();
        for child in children {
            child.build();
            ui::add_child(self.handle().raw(), child.handle().raw());
        }
    }

    fn append_child<T: Node>(&self, child: &T) {
        let self_node_ref = self.node_ref();
        let child_node_ref = child.node_ref();
        if Rc::ptr_eq(&self_node_ref.inner, &child_node_ref.inner) {
            return;
        }
        if self_node_ref
            .inner
            .borrow()
            .children
            .iter()
            .any(|candidate| Rc::ptr_eq(&candidate.inner, &child_node_ref.inner))
        {
            return;
        }
        if let Some(previous_parent) = child_node_ref.inner.borrow().parent.upgrade() {
            if Rc::ptr_eq(&previous_parent, &self_node_ref.inner) {
                return;
            }
            let child_handle = child.handle();
            {
                let mut previous_parent_core = previous_parent.borrow_mut();
                previous_parent_core
                    .children
                    .retain(|candidate| !Rc::ptr_eq(&candidate.inner, &child_node_ref.inner));
            }
            if previous_parent.borrow().handle != NodeHandle::INVALID
                && child_handle != NodeHandle::INVALID
            {
                ui::remove_child(previous_parent.borrow().handle.raw(), child_handle.raw());
            }
        }

        {
            let mut child_core = child_node_ref.inner.borrow_mut();
            child_core.parent = Rc::downgrade(&self_node_ref.inner);
        }
        child_node_ref.set_inherited_enabled(self_node_ref.is_enabled_for_routing());
        self_node_ref
            .inner
            .borrow_mut()
            .children
            .push(child_node_ref.clone());
        if let Some(owner) = child.retained_owner_attachment() {
            self_node_ref
                .inner
                .borrow_mut()
                .retained_attachments
                .push(owner);
        }
        self_node_ref.inner.borrow_mut().children_built = false;
        if self.has_built_handle() {
            child.build();
            ui::add_child(self.handle().raw(), child.handle().raw());
            self_node_ref.inner.borrow_mut().children_built = true;
            self.notify_retained_mutation();
            self.notify_retained_child_layout_changed();
        }
    }

    fn remove_child<T: Node>(&self, child: &T) -> bool {
        let child_handle = child.handle();
        let child_node_ref = child.node_ref();
        let removed = {
            let self_node_ref = self.node_ref();
            let mut core = self_node_ref.inner.borrow_mut();
            let Some(index) = core
                .children
                .iter()
                .position(|candidate| Rc::ptr_eq(&candidate.inner, &child_node_ref.inner))
            else {
                return false;
            };
            core.children.remove(index)
        };
        removed.inner.borrow_mut().parent = Weak::new();
        removed.set_inherited_enabled(true);
        self.node_ref().inner.borrow_mut().children_built = false;
        if self.has_built_handle() && child.has_built_handle() {
            ui::remove_child(self.handle().raw(), child_handle.raw());
            self.node_ref().inner.borrow_mut().children_built = true;
            self.notify_retained_mutation();
            self.notify_retained_child_layout_changed();
        }
        true
    }

    fn parent_handle(&self) -> Option<NodeHandle> {
        self.node_ref()
            .inner
            .borrow()
            .parent
            .upgrade()
            .map(|parent| parent.borrow().handle)
    }

    fn notify_retained_layout_mutation(&self) {
        self.notify_retained_mutation();
    }

    fn notify_retained_mutation(&self) {
        crate::frame_scheduler::mark_needs_commit();
    }

    fn notify_retained_child_layout_changed(&self) {}

    #[doc(hidden)]
    fn bind_scroll_proxy_target_handle(&self, scroll_handle: u64) -> &Self
    where
        Self: Sized,
    {
        let node_ref = self.node_ref();
        node_ref.inner.borrow_mut().behavior.scroll_proxy_target = Some(scroll_handle);
        if self.has_built_handle() {
            ui::set_scroll_proxy_target(self.handle().raw(), scroll_handle);
            self.notify_retained_mutation();
        }
        self
    }

    fn preserve_selection_on_pointer_down(&self, preserve: bool) -> &Self
    where
        Self: Sized,
    {
        let node_ref = self.node_ref();
        node_ref
            .inner
            .borrow_mut()
            .behavior
            .preserve_selection_on_pointer_down = preserve;
        if self.has_built_handle() {
            ui::set_preserve_selection_on_pointer_down(self.handle().raw(), preserve);
            self.notify_retained_mutation();
        }
        self
    }

    fn tool_tip(&self, tool_tip: ToolTip) -> &Self
    where
        Self: Sized,
    {
        let node_ref = self.node_ref();
        node_ref.inner.borrow_mut().behavior.tool_tip = Some(tool_tip.clone());
        node_ref.require_interactive();
        if self.has_built_handle()
            && node_ref.is_enabled_for_routing()
            && node_ref.is_visible_for_routing()
        {
            ui::set_interactive(self.handle().raw(), true);
            self.notify_retained_mutation();
        }
        crate::tool_tip_manager::ToolTipManager::handle_tool_tip_changed(&node_ref, Some(tool_tip));
        self
    }

    fn tool_tip_text(&self, text: impl Into<String>) -> &Self
    where
        Self: Sized,
    {
        self.tool_tip(ToolTip::text(text))
    }

    fn clear_tool_tip(&self) -> &Self
    where
        Self: Sized,
    {
        let node_ref = self.node_ref();
        node_ref.inner.borrow_mut().behavior.tool_tip = None;
        crate::tool_tip_manager::ToolTipManager::handle_tool_tip_changed(&node_ref, None);
        self
    }
}
