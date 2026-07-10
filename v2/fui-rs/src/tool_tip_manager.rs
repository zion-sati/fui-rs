use crate::app::Application;
use crate::bindings::ui;
use crate::ffi::PositionType;
use crate::focus_visibility;
use crate::generated::framework_host_services::fui_now_ms;
use crate::node::{portal, text, FlexBox, Node, NodeHandle, NodeRef, TextNode, WeakNodeRef};
use crate::theme::current_theme;
use crate::timers;
use crate::{FlexDirection, PopupPlacement, PopupPresenter, ToolTip, Unit};
use std::cell::RefCell;

const SHOW_TIMER_ID: u32 = 0x5454_5001;
const HIDE_TIMER_ID: u32 = 0x5454_5002;
const MIN_TOOLTIP_SURFACE_SIZE: f32 = 1.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ToolTipAnchorKind {
    None = 0,
    Owner = 1,
    Pointer = 2,
}

struct ToolTipManagerState {
    host_root: Option<FlexBox>,
    panel_node: Option<FlexBox>,
    label_node: Option<TextNode>,
    presenter: Option<PopupPresenter>,
    active_owner: Option<WeakNodeRef>,
    active_owner_handle: Option<NodeHandle>,
    active_tool_tip: Option<ToolTip>,
    pending_owner: Option<WeakNodeRef>,
    pending_owner_handle: Option<NodeHandle>,
    pending_tool_tip: Option<ToolTip>,
    hovered_owner: Option<WeakNodeRef>,
    hovered_owner_handle: Option<NodeHandle>,
    hovered_tool_tip: Option<ToolTip>,
    suppressed_hover_owner_handle: Option<NodeHandle>,
    focused_owner: Option<WeakNodeRef>,
    focused_owner_handle: Option<NodeHandle>,
    focused_tool_tip: Option<ToolTip>,
    quick_show_until_ms: f64,
    hovered_pointer_x: f32,
    hovered_pointer_y: f32,
    pending_anchor_kind: ToolTipAnchorKind,
    pending_popup_x: f32,
    pending_popup_y: f32,
    active_anchor_kind: ToolTipAnchorKind,
    active_anchor_x: f32,
    active_anchor_y: f32,
    active_anchor_width: f32,
    active_anchor_height: f32,
    active_popup_x: f32,
    active_popup_y: f32,
    focus_visibility_subscription: Option<crate::signal::SubscriptionGuard>,
}

impl Default for ToolTipManagerState {
    fn default() -> Self {
        Self {
            host_root: None,
            panel_node: None,
            label_node: None,
            presenter: None,
            active_owner: None,
            active_owner_handle: None,
            active_tool_tip: None,
            pending_owner: None,
            pending_owner_handle: None,
            pending_tool_tip: None,
            hovered_owner: None,
            hovered_owner_handle: None,
            hovered_tool_tip: None,
            suppressed_hover_owner_handle: None,
            focused_owner: None,
            focused_owner_handle: None,
            focused_tool_tip: None,
            quick_show_until_ms: -1.0,
            hovered_pointer_x: f32::NAN,
            hovered_pointer_y: f32::NAN,
            pending_anchor_kind: ToolTipAnchorKind::None,
            pending_popup_x: f32::NAN,
            pending_popup_y: f32::NAN,
            active_anchor_kind: ToolTipAnchorKind::None,
            active_anchor_x: f32::NAN,
            active_anchor_y: f32::NAN,
            active_anchor_width: f32::NAN,
            active_anchor_height: f32::NAN,
            active_popup_x: f32::NAN,
            active_popup_y: f32::NAN,
            focus_visibility_subscription: None,
        }
    }
}

thread_local! {
    static TOOL_TIP_MANAGER: RefCell<ToolTipManagerState> = RefCell::new(ToolTipManagerState::default());
}

pub(crate) struct ToolTipManager;

impl ToolTipManager {
    pub(crate) fn create_default_host() -> FlexBox {
        let (host_root, needs_focus_visibility_subscription) = TOOL_TIP_MANAGER.with(|slot| {
            let mut state = slot.borrow_mut();
            if let Some(host_root) = state.host_root.as_ref() {
                return (
                    host_root.clone(),
                    state.focus_visibility_subscription.is_none(),
                );
            }
            let label_node = text("");
            let panel_node = FlexBox::default();
            panel_node
                .position_type(PositionType::Absolute)
                .flex_direction(FlexDirection::Column)
                .child(&label_node);
            let host_root = portal();
            host_root
                .position_type(PositionType::Absolute)
                .position(0.0, 0.0)
                .width(100.0, Unit::Percent)
                .height(100.0, Unit::Percent);
            let presenter = PopupPresenter::new_with_semantic_scope(
                host_root.clone(),
                panel_node.clone(),
                None,
            );
            state.host_root = Some(host_root.clone());
            state.panel_node = Some(panel_node);
            state.label_node = Some(label_node);
            state.presenter = Some(presenter);
            (host_root, state.focus_visibility_subscription.is_none())
        });
        if needs_focus_visibility_subscription {
            let guard = focus_visibility::subscribe(|_| ToolTipManager::activate_best_candidate());
            TOOL_TIP_MANAGER.with(|slot| {
                let mut state = slot.borrow_mut();
                if state.focus_visibility_subscription.is_none() {
                    state.focus_visibility_subscription = Some(guard);
                }
            });
        }
        host_root
    }

    pub(crate) fn clear() {
        timers::cancel_internal_timer(SHOW_TIMER_ID);
        TOOL_TIP_MANAGER.with(|slot| {
            let mut state = slot.borrow_mut();
            state.pending_owner = None;
            state.pending_owner_handle = None;
            state.pending_tool_tip = None;
            state.hovered_owner = None;
            state.hovered_owner_handle = None;
            state.hovered_tool_tip = None;
            state.suppressed_hover_owner_handle = None;
            state.hovered_pointer_x = f32::NAN;
            state.hovered_pointer_y = f32::NAN;
            state.focused_owner = None;
            state.focused_owner_handle = None;
            state.focused_tool_tip = None;
            state.pending_anchor_kind = ToolTipAnchorKind::None;
            state.pending_popup_x = f32::NAN;
            state.pending_popup_y = f32::NAN;
            state.quick_show_until_ms = -1.0;
        });
        Self::hide_current();
    }

    pub(crate) fn handle_tool_tip_changed(owner: &NodeRef, tool_tip: Option<ToolTip>) {
        if tool_tip.is_none() {
            Self::clear_owner(owner.handle());
            Self::activate_best_candidate();
            return;
        }
        let owner_handle = owner.handle();
        let show_active_now = TOOL_TIP_MANAGER.with(|slot| {
            let mut state = slot.borrow_mut();
            if state.hovered_owner_handle == Some(owner_handle) {
                state.hovered_tool_tip = tool_tip.clone();
            }
            if state.focused_owner_handle == Some(owner_handle) {
                state.focused_tool_tip = tool_tip.clone();
            }
            let show_active_now = state.active_owner_handle == Some(owner_handle);
            if show_active_now {
                state.active_tool_tip = tool_tip.clone();
            }
            if state.pending_owner_handle == Some(owner_handle) {
                state.pending_tool_tip = tool_tip;
            }
            show_active_now
        });
        if show_active_now {
            if let Some(tool_tip) = owner.tool_tip_for_routing() {
                Self::show_now(owner_handle, tool_tip, true, ToolTipAnchorKind::Owner);
            }
            return;
        }
        Self::activate_best_candidate();
    }

    pub(crate) fn handle_pointer_enter(owner: &NodeRef, tool_tip: Option<ToolTip>, x: f32, y: f32) {
        let Some(tool_tip) = tool_tip else {
            return;
        };
        if tool_tip.content_text().is_empty() {
            return;
        }
        TOOL_TIP_MANAGER.with(|slot| {
            let mut state = slot.borrow_mut();
            if state.suppressed_hover_owner_handle == Some(owner.handle()) {
                state.suppressed_hover_owner_handle = None;
            }
            state.hovered_owner = Some(owner.downgrade());
            state.hovered_owner_handle = Some(owner.handle());
            state.hovered_tool_tip = Some(tool_tip);
            state.hovered_pointer_x = x;
            state.hovered_pointer_y = y;
        });
        Self::activate_best_candidate();
    }

    pub(crate) fn handle_pointer_move(owner: &NodeRef, x: f32, y: f32) {
        TOOL_TIP_MANAGER.with(|slot| {
            let mut state = slot.borrow_mut();
            if state.hovered_owner_handle != Some(owner.handle()) {
                return;
            }
            state.hovered_pointer_x = x;
            state.hovered_pointer_y = y;
            if state.pending_owner_handle == Some(owner.handle())
                && state.pending_anchor_kind == ToolTipAnchorKind::Pointer
            {
                state.pending_popup_x = x;
                state.pending_popup_y = y;
            }
        });
    }

    pub(crate) fn handle_pointer_leave(owner: &NodeRef) {
        TOOL_TIP_MANAGER.with(|slot| {
            let mut state = slot.borrow_mut();
            if state.hovered_owner_handle == Some(owner.handle()) {
                state.hovered_owner = None;
                state.hovered_owner_handle = None;
                state.hovered_tool_tip = None;
                state.hovered_pointer_x = f32::NAN;
                state.hovered_pointer_y = f32::NAN;
            }
            if state.suppressed_hover_owner_handle == Some(owner.handle()) {
                state.suppressed_hover_owner_handle = None;
            }
        });
        Self::activate_best_candidate();
    }

    pub(crate) fn handle_pointer_down(owner: &NodeRef) {
        let should_hide = TOOL_TIP_MANAGER.with(|slot| {
            let state = slot.borrow();
            state.active_owner_handle == Some(owner.handle())
                || state.pending_owner_handle == Some(owner.handle())
        });
        if !should_hide {
            return;
        }
        timers::cancel_internal_timer(SHOW_TIMER_ID);
        Self::hide_current();
        TOOL_TIP_MANAGER.with(|slot| {
            let mut state = slot.borrow_mut();
            state.pending_owner = None;
            state.pending_owner_handle = None;
            state.pending_tool_tip = None;
            state.pending_anchor_kind = ToolTipAnchorKind::None;
            state.pending_popup_x = f32::NAN;
            state.pending_popup_y = f32::NAN;
        });
    }

    pub(crate) fn handle_focus_changed(owner: &NodeRef, tool_tip: Option<ToolTip>, focused: bool) {
        TOOL_TIP_MANAGER.with(|slot| {
            let mut state = slot.borrow_mut();
            if focused {
                if let Some(tool_tip) = tool_tip {
                    if tool_tip.opens_on_focus() && !tool_tip.content_text().is_empty() {
                        state.focused_owner = Some(owner.downgrade());
                        state.focused_owner_handle = Some(owner.handle());
                        state.focused_tool_tip = Some(tool_tip);
                    }
                }
            } else if state.focused_owner_handle == Some(owner.handle()) {
                state.focused_owner = None;
                state.focused_owner_handle = None;
                state.focused_tool_tip = None;
            }
        });
        Self::activate_best_candidate();
    }

    pub(crate) fn handle_owner_destroyed(handle: NodeHandle) {
        Self::clear_owner(handle);
        Self::activate_best_candidate();
    }

    pub(crate) fn handle_scroll() {
        let should_hide = TOOL_TIP_MANAGER.with(|slot| {
            let state = slot.borrow();
            state.active_anchor_kind == ToolTipAnchorKind::Pointer
                && state.active_owner_handle.is_some()
        });
        if !should_hide {
            return;
        }
        TOOL_TIP_MANAGER.with(|slot| {
            let mut state = slot.borrow_mut();
            state.suppressed_hover_owner_handle = state.active_owner_handle;
            if state.hovered_owner_handle == state.active_owner_handle {
                state.hovered_owner = None;
                state.hovered_owner_handle = None;
                state.hovered_tool_tip = None;
                state.hovered_pointer_x = f32::NAN;
                state.hovered_pointer_y = f32::NAN;
            }
            state.pending_owner = None;
            state.pending_owner_handle = None;
            state.pending_tool_tip = None;
            state.pending_anchor_kind = ToolTipAnchorKind::None;
            state.pending_popup_x = f32::NAN;
            state.pending_popup_y = f32::NAN;
        });
        timers::cancel_internal_timer(SHOW_TIMER_ID);
        Self::hide_current();
        Self::activate_best_candidate();
    }

    pub(crate) fn commit_pending_show() {
        let (owner, tool_tip, anchor_kind) = TOOL_TIP_MANAGER.with(|slot| {
            let mut state = slot.borrow_mut();
            let owner = state.pending_owner.clone();
            let tool_tip = state.pending_tool_tip.clone();
            let anchor_kind = state.pending_anchor_kind;
            state.pending_owner = None;
            state.pending_owner_handle = None;
            state.pending_tool_tip = None;
            state.pending_anchor_kind = ToolTipAnchorKind::None;
            (owner, tool_tip, anchor_kind)
        });
        let Some(tool_tip) = tool_tip else {
            return;
        };
        let Some(owner) = owner.and_then(|weak| weak.upgrade()) else {
            return;
        };
        Self::show_now(owner.handle(), tool_tip, false, anchor_kind);
    }

    fn clear_owner(handle: NodeHandle) {
        TOOL_TIP_MANAGER.with(|slot| {
            let mut state = slot.borrow_mut();
            if state.hovered_owner_handle == Some(handle) {
                state.hovered_owner = None;
                state.hovered_owner_handle = None;
                state.hovered_tool_tip = None;
                state.hovered_pointer_x = f32::NAN;
                state.hovered_pointer_y = f32::NAN;
            }
            if state.suppressed_hover_owner_handle == Some(handle) {
                state.suppressed_hover_owner_handle = None;
            }
            if state.focused_owner_handle == Some(handle) {
                state.focused_owner = None;
                state.focused_owner_handle = None;
                state.focused_tool_tip = None;
            }
            if state.pending_owner_handle == Some(handle) {
                state.pending_owner = None;
                state.pending_owner_handle = None;
                state.pending_tool_tip = None;
                timers::cancel_internal_timer(SHOW_TIMER_ID);
            }
        });
        let is_active =
            TOOL_TIP_MANAGER.with(|slot| slot.borrow().active_owner_handle == Some(handle));
        if is_active {
            Self::hide_current();
        }
    }

    fn activate_best_candidate() {
        let (candidate_owner, candidate_tool_tip, candidate_anchor_kind, same_as_active) =
            TOOL_TIP_MANAGER.with(|slot| {
                let state = slot.borrow();
                let hovered_candidate =
                    if state.hovered_owner_handle != state.suppressed_hover_owner_handle {
                        state.hovered_owner.clone()
                    } else {
                        None
                    };
                let candidate_owner = if hovered_candidate.is_some() {
                    hovered_candidate
                } else if focus_visibility::keyboard_focus_visible() {
                    state.focused_owner.clone()
                } else {
                    None
                };
                let candidate_tool_tip = if state.hovered_owner_handle
                    != state.suppressed_hover_owner_handle
                    && state.hovered_owner.is_some()
                {
                    state.hovered_tool_tip.clone()
                } else if focus_visibility::keyboard_focus_visible() {
                    state.focused_tool_tip.clone()
                } else {
                    None
                };
                let candidate_anchor_kind = if state.hovered_owner_handle
                    != state.suppressed_hover_owner_handle
                    && state.hovered_owner.is_some()
                {
                    ToolTipAnchorKind::Pointer
                } else {
                    ToolTipAnchorKind::Owner
                };
                let candidate_handle = candidate_owner
                    .as_ref()
                    .and_then(|weak| weak.upgrade())
                    .map(|owner| owner.handle());
                let same_as_active = state.active_owner_handle == candidate_handle
                    && state.active_tool_tip == candidate_tool_tip;
                (
                    candidate_owner,
                    candidate_tool_tip,
                    candidate_anchor_kind,
                    same_as_active,
                )
            });

        let Some(tool_tip) = candidate_tool_tip else {
            TOOL_TIP_MANAGER.with(|slot| {
                let mut state = slot.borrow_mut();
                state.pending_owner = None;
                state.pending_owner_handle = None;
                state.pending_tool_tip = None;
                state.pending_anchor_kind = ToolTipAnchorKind::None;
                state.pending_popup_x = f32::NAN;
                state.pending_popup_y = f32::NAN;
            });
            timers::cancel_internal_timer(SHOW_TIMER_ID);
            Self::hide_current();
            return;
        };
        if tool_tip.content_text().is_empty() {
            timers::cancel_internal_timer(SHOW_TIMER_ID);
            Self::hide_current();
            return;
        }
        if same_as_active {
            return;
        }
        let Some(owner) = candidate_owner.and_then(|weak| weak.upgrade()) else {
            return;
        };
        Self::request_show(owner, tool_tip, candidate_anchor_kind);
    }

    fn request_show(owner: NodeRef, tool_tip: ToolTip, anchor_kind: ToolTipAnchorKind) {
        TOOL_TIP_MANAGER.with(|slot| {
            let mut state = slot.borrow_mut();
            state.pending_owner = Some(owner.downgrade());
            state.pending_owner_handle = Some(owner.handle());
            state.pending_tool_tip = Some(tool_tip.clone());
            state.pending_anchor_kind = anchor_kind;
            state.pending_popup_x = if anchor_kind == ToolTipAnchorKind::Pointer {
                state.hovered_pointer_x
            } else {
                f32::NAN
            };
            state.pending_popup_y = if anchor_kind == ToolTipAnchorKind::Pointer {
                state.hovered_pointer_y
            } else {
                f32::NAN
            };
        });
        timers::cancel_internal_timer(HIDE_TIMER_ID);
        let now = fui_now_ms();
        let delay_ms = TOOL_TIP_MANAGER.with(|slot| {
            let state = slot.borrow();
            if now <= state.quick_show_until_ms {
                0
            } else {
                tool_tip.initial_show_delay_ms()
            }
        });
        timers::cancel_internal_timer(SHOW_TIMER_ID);
        if delay_ms <= 0 {
            Self::commit_pending_show();
            return;
        }
        timers::schedule_internal_timer(SHOW_TIMER_ID, delay_ms, || {
            ToolTipManager::commit_pending_show();
        });
    }

    fn show_now(
        owner_handle: NodeHandle,
        tool_tip: ToolTip,
        preserve_current_popup_anchor: bool,
        anchor_kind: ToolTipAnchorKind,
    ) {
        let Some(owner) = Application::resolve_mounted_node(owner_handle) else {
            return;
        };
        let (presenter, panel_node, label_node, host_root) = TOOL_TIP_MANAGER.with(|slot| {
            let state = slot.borrow();
            (
                state.presenter.clone(),
                state.panel_node.clone(),
                state.label_node.clone(),
                state.host_root.clone(),
            )
        });
        let (Some(presenter), Some(panel_node), Some(label_node), Some(host_root)) =
            (presenter, panel_node, label_node, host_root)
        else {
            return;
        };
        if owner.handle() == NodeHandle::INVALID || host_root.handle() == NodeHandle::INVALID {
            return;
        }

        owner.append_child_ref(&host_root.node_ref());

        let theme = current_theme().tool_tip;
        let panel_background = if tool_tip.has_panel_color_override() {
            tool_tip.panel_background_color()
        } else {
            theme.panel_background
        };
        let text_color = if tool_tip.has_text_color_override() {
            tool_tip.tooltip_text_color()
        } else {
            theme.text_color
        };
        label_node
            .text(tool_tip.content_text())
            .font_family(theme.font_family.clone())
            .font_size(theme.font_size)
            .text_color(text_color)
            .wrapping(true);
        panel_node
            .padding(
                theme.padding_left,
                theme.padding_top,
                theme.padding_right,
                theme.padding_bottom,
            )
            .corner_radius(theme.panel_corner_radius)
            .border(1.0, theme.panel_border_color)
            .drop_shadow(
                theme.panel_shadow_color,
                0.0,
                theme.shadow_offset_y,
                theme.shadow_blur,
                theme.shadow_spread,
            )
            .bg_color(panel_background)
            .width(0.0, Unit::Auto)
            .height(0.0, Unit::Auto)
            .max_width(theme.max_width, Unit::Pixel);
        presenter.placement(tool_tip.popup_placement());
        presenter.anchor_gap(8.0);

        TOOL_TIP_MANAGER.with(|slot| {
            let mut state = slot.borrow_mut();
            if !preserve_current_popup_anchor || state.active_owner_handle != Some(owner_handle) {
                if anchor_kind == ToolTipAnchorKind::Pointer
                    && !state.pending_popup_x.is_nan()
                    && !state.pending_popup_y.is_nan()
                {
                    state.active_anchor_kind = ToolTipAnchorKind::Pointer;
                    state.active_popup_x = state.pending_popup_x + tool_tip.horizontal_offset_px();
                    state.active_popup_y = state.pending_popup_y + tool_tip.vertical_offset_px();
                    state.active_anchor_x = f32::NAN;
                    state.active_anchor_y = f32::NAN;
                    state.active_anchor_width = f32::NAN;
                    state.active_anchor_height = f32::NAN;
                } else {
                    let bounds = ui::get_bounds(owner_handle.raw()).unwrap_or([0.0, 0.0, 1.0, 1.0]);
                    state.active_anchor_kind = ToolTipAnchorKind::Owner;
                    state.active_anchor_x = bounds[0] + tool_tip.horizontal_offset_px();
                    state.active_anchor_y = bounds[1] + tool_tip.vertical_offset_px();
                    state.active_anchor_width = bounds[2];
                    state.active_anchor_height = bounds[3];
                    state.active_popup_x = f32::NAN;
                    state.active_popup_y = f32::NAN;
                }
            }
        });

        Self::show_at_resolved_anchor(
            &presenter,
            tool_tip.popup_placement(),
            MIN_TOOLTIP_SURFACE_SIZE,
            MIN_TOOLTIP_SURFACE_SIZE,
        );
        Application::flush_renders();
        let measured_bounds = if panel_node.handle() != NodeHandle::INVALID {
            ui::get_bounds(panel_node.handle().raw())
        } else {
            None
        };
        let measured_width = measured_bounds
            .map(|bounds| bounds[2].max(MIN_TOOLTIP_SURFACE_SIZE))
            .unwrap_or(MIN_TOOLTIP_SURFACE_SIZE);
        let measured_height = measured_bounds
            .map(|bounds| bounds[3].max(MIN_TOOLTIP_SURFACE_SIZE))
            .unwrap_or(MIN_TOOLTIP_SURFACE_SIZE);
        Self::show_at_resolved_anchor(
            &presenter,
            tool_tip.popup_placement(),
            measured_width,
            measured_height,
        );
        Application::flush_renders();

        TOOL_TIP_MANAGER.with(|slot| {
            let mut state = slot.borrow_mut();
            state.active_owner = Some(owner.downgrade());
            state.active_owner_handle = Some(owner_handle);
            state.active_tool_tip = Some(tool_tip.clone());
            state.quick_show_until_ms = fui_now_ms() + f64::from(tool_tip.between_show_delay_ms());
        });
        timers::cancel_internal_timer(HIDE_TIMER_ID);
        if tool_tip.show_duration_ms() > 0 {
            timers::schedule_internal_timer(HIDE_TIMER_ID, tool_tip.show_duration_ms(), || {
                ToolTipManager::hide_current();
                ToolTipManager::activate_best_candidate();
            });
        }
    }

    fn show_at_resolved_anchor(
        presenter: &PopupPresenter,
        placement: PopupPlacement,
        width: f32,
        height: f32,
    ) {
        TOOL_TIP_MANAGER.with(|slot| {
            let state = slot.borrow();
            if state.active_anchor_kind == ToolTipAnchorKind::Pointer
                && !state.active_popup_x.is_nan()
                && !state.active_popup_y.is_nan()
            {
                presenter.show_at_point(state.active_popup_x, state.active_popup_y, width, height);
            } else {
                presenter.show_anchored_with_placement(
                    state.active_anchor_x,
                    state.active_anchor_y,
                    state.active_anchor_width,
                    state.active_anchor_height,
                    width,
                    height,
                    placement,
                );
            }
        });
    }

    fn hide_current() {
        timers::cancel_internal_timer(HIDE_TIMER_ID);
        TOOL_TIP_MANAGER.with(|slot| {
            let mut state = slot.borrow_mut();
            if let Some(presenter) = state.presenter.as_ref() {
                presenter.hide();
            }
            if let Some(host_root) = state.host_root.as_ref() {
                host_root.node_ref().detach_from_parent();
            }
            state.active_owner = None;
            state.active_owner_handle = None;
            state.active_tool_tip = None;
            state.active_anchor_kind = ToolTipAnchorKind::None;
            state.active_anchor_x = f32::NAN;
            state.active_anchor_y = f32::NAN;
            state.active_anchor_width = f32::NAN;
            state.active_anchor_height = f32::NAN;
            state.active_popup_x = f32::NAN;
            state.active_popup_y = f32::NAN;
        });
    }
}
