use crate::bindings::ui;
use crate::drawing::DrawContext;
use crate::event::{
    FocusChangedEventArgs, GestureEventArgs, KeyEventArgs, LongPressEventArgs, PointerEventArgs,
    PointerType, WheelEventArgs, DEFAULT_LONG_PRESS_MINIMUM_DURATION_MS,
    DEFAULT_LONG_PRESS_MOVEMENT_TOLERANCE,
};
use crate::ffi::{
    AlignItems, AlignSelf, BorderStyle, CursorStyle, FlexDirection, FlexWrap, GridUnit,
    HandleValue, ImageSamplingKind, JustifyContent, NodeType, ObjectFit, Orientation, PositionType,
    SemanticCheckedState, SemanticRole, TextAlign, TextOverflow, TextVerticalAlign, Unit,
    Visibility,
};
use crate::theme;
use crate::transitions::NodeTransitions;
use crate::typography::{FontFamily, FontStyle, FontWeight};
use std::any::Any;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

mod core;
mod custom_drawable;
mod flex_box;
mod grid;
mod helpers;
mod image;
mod presenter_host_style;
mod scroll_bar;
mod scroll_box;
mod scroll_state;
mod scroll_view;
mod svg_node;
mod text_node;
mod virtual_list;

#[doc(hidden)]
pub use core::NodeRef;
pub use core::{ContextMenuEventArgs, Node, NodeHandle};
pub(crate) use core::{WeakFlexBox, WeakNodeRef};
pub use custom_drawable::{CustomDrawable, DrawableInvalidator};
pub use flex_box::{Border, FlexBox, GradientStop};
pub use grid::{Grid, GridTrack};
pub(crate) use helpers::*;
pub use helpers::{
    auto, column, custom_drawable, fill, flex_box, grid, image, pct, portal, px, row, scroll_box,
    scroll_view, svg, text, viewport_height, viewport_width, virtual_list, Length,
};
pub use image::ImageNode;
pub(crate) use presenter_host_style::HostStyleLayers;
pub use presenter_host_style::{Corners, EdgeInsets, PresenterHostStyle, Shadow};
pub use scroll_bar::{ScrollBar, ScrollBarStyle, ScrollBarVisibility};
pub use scroll_box::ScrollBox;
pub use scroll_state::ScrollState;
pub use scroll_view::ScrollView;
pub use svg_node::SvgNode;
pub use text_node::{TextCore, TextNode};
pub use virtual_list::VirtualList;

pub type Image = ImageNode;
pub type Svg = SvgNode;
pub type Text = TextNode;

pub trait HasFlexBoxRoot {
    fn flex_box_root(&self) -> &FlexBox;
}

pub trait ThemeBindable: Sized + 'static {
    #[doc(hidden)]
    fn theme_binding_node(&self) -> NodeRef;

    #[doc(hidden)]
    fn weak_theme_target(&self) -> Box<dyn Fn() -> Option<Self>>;

    fn bind_theme(&self, handler: impl Fn(&Self, crate::theme::Theme) + 'static) -> &Self {
        let target = self.weak_theme_target();
        let guard = crate::theme::subscribe(move |theme| {
            if let Some(control) = target() {
                handler(&control, theme);
            }
        });
        self.theme_binding_node().retain_attachment(Rc::new(guard));
        self
    }
}

impl HasFlexBoxRoot for FlexBox {
    fn flex_box_root(&self) -> &FlexBox {
        self
    }
}

impl ThemeBindable for FlexBox {
    fn theme_binding_node(&self) -> NodeRef {
        self.retained_node_ref()
    }

    fn weak_theme_target(&self) -> Box<dyn Fn() -> Option<Self>> {
        let weak = self.downgrade();
        Box::new(move || weak.upgrade())
    }
}

pub trait FlexBoxSurface: HasFlexBoxRoot {
    fn width(&self, width: f32, unit: Unit) -> &Self {
        self.flex_box_root().width(width, unit);
        self
    }

    fn width_len(&self, length: Length) -> &Self {
        self.flex_box_root().width_len(length);
        self
    }

    fn height(&self, height: f32, unit: Unit) -> &Self {
        self.flex_box_root().height(height, unit);
        self
    }

    fn height_len(&self, length: Length) -> &Self {
        self.flex_box_root().height_len(length);
        self
    }

    fn fill_width(&self) -> &Self {
        self.flex_box_root().fill_width();
        self
    }

    fn fill_height(&self) -> &Self {
        self.flex_box_root().fill_height();
        self
    }

    fn fill_size(&self) -> &Self {
        self.flex_box_root().fill_size();
        self
    }

    fn fill_width_percent(&self, percent: f32) -> &Self {
        self.flex_box_root().fill_width_percent(percent);
        self
    }

    fn fill_height_percent(&self, percent: f32) -> &Self {
        self.flex_box_root().fill_height_percent(percent);
        self
    }

    fn min_width(&self, value: f32, unit: Unit) -> &Self {
        self.flex_box_root().min_width(value, unit);
        self
    }

    fn min_width_len(&self, length: Length) -> &Self {
        self.flex_box_root().min_width_len(length);
        self
    }

    fn max_width(&self, value: f32, unit: Unit) -> &Self {
        self.flex_box_root().max_width(value, unit);
        self
    }

    fn max_width_len(&self, length: Length) -> &Self {
        self.flex_box_root().max_width_len(length);
        self
    }

    fn min_height(&self, value: f32, unit: Unit) -> &Self {
        self.flex_box_root().min_height(value, unit);
        self
    }

    fn min_height_len(&self, length: Length) -> &Self {
        self.flex_box_root().min_height_len(length);
        self
    }

    fn max_height(&self, value: f32, unit: Unit) -> &Self {
        self.flex_box_root().max_height(value, unit);
        self
    }

    fn max_height_len(&self, length: Length) -> &Self {
        self.flex_box_root().max_height_len(length);
        self
    }

    fn padding(&self, left: f32, top: f32, right: f32, bottom: f32) -> &Self {
        self.flex_box_root().padding(left, top, right, bottom);
        self
    }

    fn clear_padding(&self) -> &Self {
        self.flex_box_root().clear_padding();
        self
    }

    fn margin(&self, left: f32, top: f32, right: f32, bottom: f32) -> &Self {
        self.flex_box_root().margin(left, top, right, bottom);
        self
    }

    fn corner_radius(&self, radius: f32) -> &Self {
        self.flex_box_root().corner_radius(radius);
        self
    }

    fn clear_corners(&self) -> &Self {
        self.flex_box_root().clear_corners();
        self
    }

    fn corners(&self, tl: f32, tr: f32, br: f32, bl: f32) -> &Self {
        self.flex_box_root().corners(tl, tr, br, bl);
        self
    }

    fn border(&self, width: f32, color: u32) -> &Self {
        self.flex_box_root().border(width, color);
        self
    }

    fn border_config(&self, border: Border) -> &Self {
        self.flex_box_root().border_config(border);
        self
    }

    fn clear_border(&self) -> &Self {
        self.flex_box_root().clear_border();
        self
    }

    fn bg_color(&self, color: u32) -> &Self {
        self.flex_box_root().bg_color(color);
        self
    }

    fn clear_bg_color(&self) -> &Self {
        self.flex_box_root().clear_bg_color();
        self
    }

    fn opacity(&self, value: f32) -> &Self {
        self.flex_box_root().opacity(value);
        self
    }

    fn clear_opacity(&self) -> &Self {
        self.flex_box_root().clear_opacity();
        self
    }

    fn blur(&self, sigma: f32) -> &Self {
        self.flex_box_root().blur(sigma);
        self
    }

    fn drop_shadow(
        &self,
        color: u32,
        offset_x: f32,
        offset_y: f32,
        blur_sigma: f32,
        spread: f32,
    ) -> &Self {
        self.flex_box_root()
            .drop_shadow(color, offset_x, offset_y, blur_sigma, spread);
        self
    }

    fn clear_drop_shadow(&self) -> &Self {
        self.flex_box_root().clear_drop_shadow();
        self
    }

    fn background_blur(&self, sigma: f32) -> &Self {
        self.flex_box_root().background_blur(sigma);
        self
    }

    fn linear_gradient(
        &self,
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
        offsets: Vec<f32>,
        colors: Vec<u32>,
    ) -> &Self {
        self.flex_box_root()
            .linear_gradient(start_x, start_y, end_x, end_y, offsets, colors);
        self
    }

    fn linear_gradient_stops(
        &self,
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
        stops: Vec<GradientStop>,
    ) -> &Self {
        self.flex_box_root()
            .linear_gradient_stops(start_x, start_y, end_x, end_y, stops);
        self
    }

    fn transitions(&self, transitions: Option<NodeTransitions>) -> &Self {
        self.flex_box_root().transitions(transitions);
        self
    }

    fn flex_basis(&self, basis: f32) -> &Self {
        self.flex_box_root().flex_basis(basis);
        self
    }

    fn justify_content(&self, justify: JustifyContent) -> &Self {
        self.flex_box_root().justify_content(justify);
        self
    }

    fn clear_justify_content(&self) -> &Self {
        self.flex_box_root().clear_justify_content();
        self
    }

    fn align_items(&self, align: AlignItems) -> &Self {
        self.flex_box_root().align_items(align);
        self
    }

    fn clear_align_items(&self) -> &Self {
        self.flex_box_root().clear_align_items();
        self
    }

    fn align_self(&self, align: AlignSelf) -> &Self {
        self.flex_box_root().align_self(align);
        self
    }

    fn flex_wrap(&self, wrap: FlexWrap) -> &Self {
        self.flex_box_root().flex_wrap(wrap);
        self
    }

    fn position_type(&self, position_type: PositionType) -> &Self {
        self.flex_box_root().position_type(position_type);
        self
    }

    fn position(&self, left: f32, top: f32) -> &Self {
        self.flex_box_root().position(left, top);
        self
    }

    fn clip_to_bounds(&self, clip: bool) -> &Self {
        self.flex_box_root().clip_to_bounds(clip);
        self
    }

    fn cursor(&self, style: CursorStyle) -> &Self {
        self.flex_box_root().cursor(style);
        self
    }

    fn clear_cursor(&self) -> &Self {
        self.flex_box_root().clear_cursor();
        self
    }

    fn visibility(&self, visibility: Visibility) -> &Self {
        self.flex_box_root().visibility(visibility);
        self
    }
}

impl<T: HasFlexBoxRoot> FlexBoxSurface for T {}

#[derive(Clone)]
pub struct Child {
    pub(crate) node_ref: NodeRef,
}

impl Child {
    pub fn from_node<T: Node>(value: &T) -> Self {
        Self {
            node_ref: value.retained_node_ref(),
        }
    }
}

impl<T: Node> From<T> for Child {
    fn from(value: T) -> Self {
        Self {
            node_ref: value.retained_node_ref(),
        }
    }
}
