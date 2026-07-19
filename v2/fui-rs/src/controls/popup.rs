use super::*;
use crate::ffi::{FlexDirection, PositionType, Unit};
use crate::node::portal;
use crate::popup_presenter::{PopupPlacement, PopupPresenter};
use std::cell::Cell;

#[derive(Clone)]
pub struct Popup {
    root: FlexBox,
    surface_node: FlexBox,
    presenter: PopupPresenter,
    dismiss_on_backdrop_click: Rc<Cell<bool>>,
    appearance_value: Rc<RefCell<Option<PopupAppearance>>>,
}

impl Default for Popup {
    fn default() -> Self {
        Self::new()
    }
}

impl Popup {
    pub fn new() -> Self {
        let root = portal();
        let surface_node = flex_box();
        surface_node
            .position_type(PositionType::Absolute)
            .flex_direction(FlexDirection::Column);
        let presenter = PopupPresenter::new(root.clone(), surface_node.clone());
        root.position_type(PositionType::Absolute)
            .position(0.0, 0.0)
            .width(100.0, Unit::Percent)
            .height(100.0, Unit::Percent);
        let dismiss_on_backdrop_click = Rc::new(Cell::new(true));
        let dismiss_flag = dismiss_on_backdrop_click.clone();
        let presenter_target = presenter.event_target();
        presenter
            .overlay_node()
            .interactive(true)
            .on_pointer_click(move |_event| {
                if dismiss_flag.get() {
                    presenter_target.hide();
                }
            });
        Self {
            root,
            surface_node,
            presenter,
            dismiss_on_backdrop_click,
            appearance_value: Rc::new(RefCell::new(None)),
        }
    }

    pub fn is_open(&self) -> bool {
        self.presenter.is_open()
    }

    /// Returns the presented content panel. Inherited `child`/`children` calls
    /// are routed here rather than into the portal's overlay root.
    pub fn surface(&self) -> FlexBox {
        self.surface_node.clone()
    }

    pub fn placement(&self, value: PopupPlacement) -> &Self {
        self.presenter.placement(value);
        self
    }

    pub fn edge_padding(&self, value: f32) -> &Self {
        self.presenter.edge_padding(value);
        self
    }

    pub fn anchor_gap(&self, value: f32) -> &Self {
        self.presenter.anchor_gap(value);
        self
    }

    pub fn dismiss_on_backdrop_click(&self, flag: bool) -> &Self {
        self.dismiss_on_backdrop_click.set(flag);
        self
    }

    pub fn appearance(&self, appearance: PopupAppearance) -> &Self {
        self.appearance_value.replace(Some(appearance));
        self.sync_appearance();
        self
    }

    pub fn clear_appearance(&self) -> &Self {
        self.appearance_value.replace(None);
        self.sync_appearance();
        self
    }

    fn sync_appearance(&self) {
        let appearance = self.appearance_value.borrow().clone().unwrap_or_default();
        let panel = appearance.panel.unwrap_or_default();
        let backdrop = appearance.backdrop.unwrap_or_default();
        self.surface_node
            .apply_presenter_style(panel.presenter_host_style())
            .background_blur(panel.background_blur.unwrap_or(0.0));
        self.presenter
            .backdrop_color(backdrop.color.unwrap_or(0x00000000))
            .background_blur(backdrop.blur.unwrap_or(0.0));
    }

    pub fn show_anchored(
        &self,
        anchor_x: f32,
        anchor_y: f32,
        anchor_width: f32,
        anchor_height: f32,
        width: f32,
        height: f32,
    ) {
        self.surface_node.width(width, Unit::Pixel);
        self.surface_node.height(height, Unit::Pixel);
        self.presenter.show_anchored(
            anchor_x,
            anchor_y,
            anchor_width,
            anchor_height,
            width,
            height,
        );
    }

    pub fn show_at_point(&self, x: f32, y: f32, width: f32, height: f32) {
        self.surface_node.width(width, Unit::Pixel);
        self.surface_node.height(height, Unit::Pixel);
        self.presenter.show_at_point(x, y, width, height);
    }

    pub fn hide(&self) {
        self.presenter.hide();
    }
}

impl Node for Popup {
    fn retained_node_ref(&self) -> NodeRef {
        self.root.retained_node_ref()
    }

    fn build_self(&self) {
        self.root.build_self();
    }

    fn dispose(&self) {
        self.presenter.dispose();
        self.root.dispose();
    }
}

impl crate::node::HasFlexBoxRoot for Popup {
    fn flex_box_root(&self) -> &FlexBox {
        &self.root
    }

    fn append_flex_box_surface_child(&self, child: NodeRef) {
        self.surface_node
            .retained_node_ref()
            .append_child_ref(&child);
    }
}

impl crate::node::ThemeBindable for Popup {
    fn theme_binding_node(&self) -> NodeRef {
        self.root.retained_node_ref()
    }

    fn weak_theme_target(&self) -> Box<dyn Fn() -> Option<Self>> {
        let root = self.root.downgrade();
        let surface_node = self.surface_node.downgrade();
        let presenter = self.presenter.downgrade();
        let dismiss_on_backdrop_click = Rc::downgrade(&self.dismiss_on_backdrop_click);
        // Appearance state does not own the portal root. Keep it alive with the
        // retained theme binding so a dropped wrapper remains reconstructible.
        let appearance_value = self.appearance_value.clone();
        Box::new(move || {
            Some(Self {
                root: root.upgrade()?,
                surface_node: surface_node.upgrade()?,
                presenter: presenter.upgrade()?,
                dismiss_on_backdrop_click: dismiss_on_backdrop_click.upgrade()?,
                appearance_value: appearance_value.clone(),
            })
        })
    }
}
