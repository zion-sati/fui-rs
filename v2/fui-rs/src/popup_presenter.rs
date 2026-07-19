use crate::bindings::ui;
use crate::ffi::{PositionType, Unit};
use crate::node::{portal, FlexBox, Node, NodeHandle, WeakFlexBox};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PopupPlacement {
    Auto = 0,
    Bottom = 1,
    Top = 2,
    Overlap = 3,
}

#[derive(Clone)]
#[doc(hidden)]
pub struct PopupPresenter {
    root: FlexBox,
    overlay_node: FlexBox,
    surface_node: FlexBox,
    semantic_scope_node: Option<FlexBox>,
    state: Rc<RefCell<PopupPresenterState>>,
}

#[derive(Clone)]
pub(crate) struct WeakPopupPresenter {
    root: WeakFlexBox,
    overlay_node: FlexBox,
    surface_node: FlexBox,
    semantic_scope_node: Option<FlexBox>,
    state: Rc<RefCell<PopupPresenterState>>,
}

#[derive(Clone)]
pub(crate) struct PopupPresenterEventTarget {
    root: WeakFlexBox,
    overlay_node: WeakFlexBox,
    state: Rc<RefCell<PopupPresenterState>>,
}

#[derive(Clone, Copy)]
struct PopupPresenterState {
    open: bool,
    semantic_scope_token: u32,
    edge_padding: f32,
    anchor_gap: f32,
    placement: PopupPlacement,
    backdrop_color: u32,
    background_blur_sigma: f32,
    surface_x: f32,
    surface_y: f32,
}

impl Default for PopupPresenterState {
    fn default() -> Self {
        Self {
            open: false,
            semantic_scope_token: 0,
            edge_padding: 8.0,
            anchor_gap: 4.0,
            placement: PopupPlacement::Auto,
            backdrop_color: 0x00000000,
            background_blur_sigma: 0.0,
            surface_x: 0.0,
            surface_y: 0.0,
        }
    }
}

impl PopupPresenter {
    pub fn new(root: FlexBox, surface_node: FlexBox) -> Self {
        Self::new_with_semantic_scope(root, surface_node.clone(), Some(surface_node))
    }

    pub fn new_with_semantic_scope(
        root: FlexBox,
        surface_node: FlexBox,
        semantic_scope_node: Option<FlexBox>,
    ) -> Self {
        let overlay_node = FlexBox::default();
        overlay_node
            .position_type(PositionType::Absolute)
            .position(0.0, 0.0)
            .width(100.0, Unit::Percent)
            .height(100.0, Unit::Percent)
            .child(&surface_node);
        surface_node.position_type(PositionType::Absolute);
        let presenter = Self {
            root,
            overlay_node,
            surface_node,
            semantic_scope_node,
            state: Rc::new(RefCell::new(PopupPresenterState::default())),
        };
        presenter.apply_backdrop_style();
        presenter
    }

    pub fn with_default_root(surface_node: FlexBox) -> Self {
        Self::new(portal(), surface_node)
    }

    pub fn root(&self) -> FlexBox {
        self.root.clone()
    }

    pub fn overlay_node(&self) -> FlexBox {
        self.overlay_node.clone()
    }

    pub fn surface_node(&self) -> FlexBox {
        self.surface_node.clone()
    }

    pub fn is_open(&self) -> bool {
        self.state.borrow().open
    }

    pub(crate) fn downgrade(&self) -> WeakPopupPresenter {
        WeakPopupPresenter {
            root: self.root.downgrade(),
            overlay_node: self.overlay_node.clone(),
            surface_node: self.surface_node.clone(),
            semantic_scope_node: self.semantic_scope_node.clone(),
            state: self.state.clone(),
        }
    }

    pub fn surface_x(&self) -> f32 {
        self.state.borrow().surface_x
    }

    pub fn surface_y(&self) -> f32 {
        self.state.borrow().surface_y
    }

    pub fn semantic_scope_token(&self) -> u32 {
        self.state.borrow().semantic_scope_token
    }

    pub(crate) fn event_target(&self) -> PopupPresenterEventTarget {
        PopupPresenterEventTarget {
            root: self.root.downgrade(),
            overlay_node: self.overlay_node.downgrade(),
            state: self.state.clone(),
        }
    }

    pub fn placement(&self, value: PopupPlacement) -> &Self {
        self.state.borrow_mut().placement = value;
        self
    }

    pub fn edge_padding(&self, value: f32) -> &Self {
        self.state.borrow_mut().edge_padding = value.max(0.0);
        self
    }

    pub fn anchor_gap(&self, value: f32) -> &Self {
        self.state.borrow_mut().anchor_gap = value.max(0.0);
        self
    }

    pub fn backdrop_color(&self, color: u32) -> &Self {
        self.state.borrow_mut().backdrop_color = color;
        self.apply_backdrop_style();
        self
    }

    pub fn background_blur(&self, sigma: f32) -> &Self {
        self.state.borrow_mut().background_blur_sigma = sigma.max(0.0);
        self.apply_backdrop_style();
        self
    }

    pub fn sync_overlay_bounds(&self) {
        let popup_bounds = if self.root.handle() != NodeHandle::INVALID {
            ui::get_bounds(self.root.handle().raw())
        } else {
            None
        };
        let overlay_x = popup_bounds.map(|bounds| -bounds[0]).unwrap_or(0.0);
        let overlay_y = popup_bounds.map(|bounds| -bounds[1]).unwrap_or(0.0);
        self.overlay_node
            .position(overlay_x, overlay_y)
            .width(ui::get_viewport_width(), Unit::Pixel)
            .height(ui::get_viewport_height(), Unit::Pixel);
    }

    pub fn show_anchored(
        &self,
        anchor_x: f32,
        anchor_y: f32,
        anchor_width: f32,
        anchor_height: f32,
        surface_width: f32,
        surface_height: f32,
    ) {
        let placement = self.state.borrow().placement;
        self.show_anchored_with_placement(
            anchor_x,
            anchor_y,
            anchor_width,
            anchor_height,
            surface_width,
            surface_height,
            placement,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn show_anchored_with_placement(
        &self,
        anchor_x: f32,
        anchor_y: f32,
        _anchor_width: f32,
        anchor_height: f32,
        surface_width: f32,
        surface_height: f32,
        placement: PopupPlacement,
    ) {
        if self.root.handle() == NodeHandle::INVALID {
            return;
        }
        self.sync_overlay_bounds();
        let state = *self.state.borrow();
        let clamped_width = if surface_width > 0.0 {
            surface_width
        } else {
            1.0
        };
        let clamped_height = if surface_height > 0.0 {
            surface_height
        } else {
            1.0
        };
        let viewport_width = ui::get_viewport_width();
        let viewport_height = ui::get_viewport_height();
        let max_x = state
            .edge_padding
            .max(viewport_width - clamped_width - state.edge_padding);
        let max_y = state
            .edge_padding
            .max(viewport_height - clamped_height - state.edge_padding);
        let below_y = anchor_y + anchor_height + state.anchor_gap;
        let above_y = anchor_y - clamped_height - state.anchor_gap;
        let fits_below = below_y <= max_y;
        let fits_above = above_y >= state.edge_padding;
        let mut panel_y = below_y;
        if placement == PopupPlacement::Top {
            panel_y = above_y;
        } else if placement == PopupPlacement::Overlap {
            panel_y = anchor_y;
        } else if placement == PopupPlacement::Auto && !fits_below && fits_above {
            panel_y = above_y;
        }
        self.set_surface_position(
            anchor_x.clamp(state.edge_padding, max_x),
            panel_y.clamp(state.edge_padding, max_y),
        );
        self.attach();
    }

    pub fn show_at_point(&self, x: f32, y: f32, surface_width: f32, surface_height: f32) {
        if self.root.handle() == NodeHandle::INVALID {
            return;
        }
        self.sync_overlay_bounds();
        let state = *self.state.borrow();
        let clamped_width = if surface_width > 0.0 {
            surface_width
        } else {
            1.0
        };
        let clamped_height = if surface_height > 0.0 {
            surface_height
        } else {
            1.0
        };
        let max_x = state
            .edge_padding
            .max(ui::get_viewport_width() - clamped_width - state.edge_padding);
        let max_y = state
            .edge_padding
            .max(ui::get_viewport_height() - clamped_height - state.edge_padding);
        self.set_surface_position(
            x.clamp(state.edge_padding, max_x),
            y.clamp(state.edge_padding, max_y),
        );
        self.attach();
    }

    pub fn hide(&self) {
        if !self.is_open() && self.overlay_node.parent_handle().is_none() {
            return;
        }
        self.root.remove_child(&self.overlay_node);
        let mut state = self.state.borrow_mut();
        state.open = false;
        if state.semantic_scope_token != 0 {
            ui::remove_semantic_scope(state.semantic_scope_token);
            state.semantic_scope_token = 0;
        }
    }

    pub fn dispose(&self) {
        self.hide();
        if self.overlay_node.handle() != NodeHandle::INVALID {
            self.overlay_node.dispose();
        }
    }

    fn attach(&self) {
        self.root.child(&self.overlay_node);
        self.state.borrow_mut().open = true;
        let semantic_scope_handle = self
            .semantic_scope_node
            .as_ref()
            .map(|node| node.handle())
            .unwrap_or(NodeHandle::INVALID);
        let mut state = self.state.borrow_mut();
        if state.semantic_scope_token == 0 && semantic_scope_handle != NodeHandle::INVALID {
            state.semantic_scope_token = ui::push_semantic_scope(semantic_scope_handle.raw());
        }
    }

    fn apply_backdrop_style(&self) {
        let state = *self.state.borrow();
        self.overlay_node
            .bg_color(state.backdrop_color)
            .background_blur(state.background_blur_sigma);
    }

    fn set_surface_position(&self, x: f32, y: f32) {
        {
            let mut state = self.state.borrow_mut();
            state.surface_x = x;
            state.surface_y = y;
        }
        self.surface_node.position(x, y);
    }
}

impl WeakPopupPresenter {
    pub(crate) fn upgrade(&self) -> Option<PopupPresenter> {
        Some(PopupPresenter {
            root: self.root.upgrade()?,
            overlay_node: self.overlay_node.clone(),
            surface_node: self.surface_node.clone(),
            semantic_scope_node: self.semantic_scope_node.clone(),
            state: self.state.clone(),
        })
    }
}

impl PopupPresenterEventTarget {
    pub(crate) fn is_open(&self) -> bool {
        self.state.borrow().open
    }

    pub(crate) fn surface_x(&self) -> f32 {
        self.state.borrow().surface_x
    }

    pub(crate) fn surface_y(&self) -> f32 {
        self.state.borrow().surface_y
    }

    pub(crate) fn backdrop_color(&self, color: u32) {
        self.state.borrow_mut().backdrop_color = color;
        self.apply_backdrop_style();
    }

    pub(crate) fn background_blur(&self, sigma: f32) {
        self.state.borrow_mut().background_blur_sigma = sigma.max(0.0);
        self.apply_backdrop_style();
    }

    fn apply_backdrop_style(&self) {
        let Some(overlay) = self.overlay_node.upgrade() else {
            return;
        };
        let state = *self.state.borrow();
        overlay
            .bg_color(state.backdrop_color)
            .background_blur(state.background_blur_sigma);
    }

    pub(crate) fn hide(&self) {
        let Some(root) = self.root.upgrade() else {
            return;
        };
        let Some(overlay_node) = self.overlay_node.upgrade() else {
            return;
        };
        let open = self.state.borrow().open;
        if !open && overlay_node.parent_handle().is_none() {
            return;
        }
        root.remove_child(&overlay_node);
        let mut state = self.state.borrow_mut();
        state.open = false;
        if state.semantic_scope_token != 0 {
            ui::remove_semantic_scope(state.semantic_scope_token);
            state.semantic_scope_token = 0;
        }
    }
}
