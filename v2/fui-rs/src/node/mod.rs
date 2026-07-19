use crate::bindings::ui;
use crate::drawing::DrawContext;
use crate::event::{
    FocusChangedEventArgs, GestureEventArgs, KeyEventArgs, LongPressEventArgs, PointerEventArgs,
    PointerType, SelectionChangedEventArgs, TextChangedEventArgs, WheelEventArgs,
    DEFAULT_LONG_PRESS_MINIMUM_DURATION_MS, DEFAULT_LONG_PRESS_MOVEMENT_TOLERANCE,
};
use crate::ffi::{
    AlignItems, AlignSelf, BorderStyle, CursorStyle, FlexDirection, FlexWrap, GridUnit,
    HandleValue, ImageSamplingKind, JustifyContent, NodeType, ObjectFit, Orientation, PositionType,
    SemanticCheckedState, SemanticRole, TextAlign, TextOverflow, TextVerticalAlign, Unit,
    Visibility,
};
use crate::theme;
use crate::transitions::NodeTransitions;
use crate::typography::{FontFamily, FontStack, FontStyle, FontWeight};
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
pub use text_node::TextNode;
pub use virtual_list::VirtualList;

pub type Image = ImageNode;
pub type Portal = FlexBox;
pub type Svg = SvgNode;
pub type Text = TextNode;

pub trait HasFlexBoxRoot {
    fn flex_box_root(&self) -> &FlexBox;

    #[doc(hidden)]
    fn set_flex_box_surface_width(&self, value: f32, unit: Unit) {
        self.flex_box_root().width(value, unit);
    }

    #[doc(hidden)]
    fn set_flex_box_surface_height(&self, value: f32, unit: Unit) {
        self.flex_box_root().height(value, unit);
    }

    #[doc(hidden)]
    fn append_flex_box_surface_child(&self, child: NodeRef) {
        self.flex_box_root()
            .retained_node_ref()
            .append_child_ref(&child);
    }
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

pub trait HasTextNode {
    fn text_node(&self) -> &TextNode;

    #[doc(hidden)]
    fn set_text_surface_content(&self, content: String) {
        self.text_node().text(content);
    }

    #[doc(hidden)]
    fn set_text_surface_font_stack(&self, stack: FontStack, size: f32) {
        self.text_node().font_stack(stack, size);
    }

    #[doc(hidden)]
    fn set_text_surface_font_family(&self, family: FontFamily) {
        self.text_node().font_family(family);
    }

    #[doc(hidden)]
    fn set_text_surface_font_weight(&self, weight: FontWeight) {
        self.text_node().font_weight(weight);
    }

    #[doc(hidden)]
    fn set_text_surface_font_style(&self, style: FontStyle) {
        self.text_node().font_style(style);
    }

    #[doc(hidden)]
    fn set_text_surface_font_size(&self, size: f32) {
        self.text_node().font_size(size);
    }

    #[doc(hidden)]
    fn set_text_surface_color(&self, color: u32) {
        self.text_node().text_color(color);
    }
}

pub trait TextLayoutSurface: HasTextNode {
    fn width(&self, width: f32, unit: Unit) -> &Self {
        self.text_node().width(width, unit);
        self
    }

    fn height(&self, height: f32, unit: Unit) -> &Self {
        self.text_node().height(height, unit);
        self
    }

    fn width_len(&self, length: Length) -> &Self {
        let (value, unit) = length;
        self.text_node().width(value, unit);
        self
    }

    fn height_len(&self, length: Length) -> &Self {
        let (value, unit) = length;
        self.text_node().height(value, unit);
        self
    }

    fn fill_width(&self) -> &Self {
        self.text_node().fill_width();
        self
    }

    fn fill_height(&self) -> &Self {
        self.text_node().fill_height();
        self
    }

    fn fill_size(&self) -> &Self {
        self.text_node().fill_size();
        self
    }

    fn fill_width_percent(&self, percent: f32) -> &Self {
        self.text_node().fill_width_percent(percent);
        self
    }

    fn fill_height_percent(&self, percent: f32) -> &Self {
        self.text_node().fill_height_percent(percent);
        self
    }

    fn min_width(&self, value: f32, unit: Unit) -> &Self {
        self.text_node().min_width(value, unit);
        self
    }

    fn max_width(&self, value: f32, unit: Unit) -> &Self {
        self.text_node().max_width(value, unit);
        self
    }

    fn min_height(&self, value: f32, unit: Unit) -> &Self {
        self.text_node().min_height(value, unit);
        self
    }

    fn max_height(&self, value: f32, unit: Unit) -> &Self {
        self.text_node().max_height(value, unit);
        self
    }

    fn text_align(&self, align: TextAlign) -> &Self {
        self.text_node().text_align(align);
        self
    }

    fn text_vertical_align(&self, align: TextVerticalAlign) -> &Self {
        self.text_node().text_vertical_align(align);
        self
    }

    fn text_limits(&self, max_chars: i32, max_lines: i32) -> &Self {
        self.text_node().text_limits(max_chars, max_lines);
        self
    }

    fn max_lines(&self, max_lines: i32) -> &Self {
        self.text_node().max_lines(max_lines);
        self
    }

    fn wrapping(&self, wrap: bool) -> &Self {
        self.text_node().wrapping(wrap);
        self
    }

    fn text_overflow(&self, overflow: TextOverflow) -> &Self {
        self.text_node().text_overflow(overflow);
        self
    }

    fn text_overflow_fade(&self, horizontal: bool, vertical: bool) -> &Self {
        self.text_node().text_overflow_fade(horizontal, vertical);
        self
    }
}

impl<T: HasTextNode> TextLayoutSurface for T {}

pub trait TextContentSurface: HasTextNode {
    fn text(&self, content: impl Into<String>) -> &Self {
        self.set_text_surface_content(content.into());
        self
    }

    fn content(&self) -> String {
        self.text_node().content()
    }
}

impl<T: HasTextNode> TextContentSurface for T {}

pub trait TextTypographySurface: HasTextNode {
    fn font_stack(&self, stack: FontStack, size: f32) -> &Self {
        self.set_text_surface_font_stack(stack, size);
        self
    }

    fn font_family(&self, family: FontFamily) -> &Self {
        self.set_text_surface_font_family(family);
        self
    }

    fn font_weight(&self, weight: FontWeight) -> &Self {
        self.set_text_surface_font_weight(weight);
        self
    }

    fn font_style(&self, style: FontStyle) -> &Self {
        self.set_text_surface_font_style(style);
        self
    }

    fn font_size(&self, size: f32) -> &Self {
        self.set_text_surface_font_size(size);
        self
    }

    fn line_height(&self, line_height: f32) -> &Self {
        self.text_node().line_height(line_height);
        self
    }

    fn text_color(&self, color: u32) -> &Self {
        self.set_text_surface_color(color);
        self
    }
}

impl<T: HasTextNode> TextTypographySurface for T {}

pub trait TextSelectionSurface: HasTextNode {
    fn uses_default_selection_behavior(&self) -> bool {
        self.text_node().uses_default_selection_behavior()
    }

    fn is_selectable_text(&self) -> bool {
        self.text_node().is_selectable_text()
    }

    fn selection_start(&self) -> u32 {
        self.text_node().selection_start()
    }

    fn selection_end(&self) -> u32 {
        self.text_node().selection_end()
    }

    fn selectable(&self, selectable: bool) -> &Self {
        self.text_node().selectable(selectable);
        self
    }

    fn selection_color(&self, color: u32) -> &Self {
        self.text_node().selection_color(color);
        self
    }

    fn selection_range(&self, start: u32, end: u32) -> &Self {
        self.text_node().selection_range(start, end);
        self
    }

    fn caret_color(&self, color: u32) -> &Self {
        self.text_node().caret_color(color);
        self
    }
}

impl<T: HasTextNode> TextSelectionSurface for T {}

pub trait TextEditingSurface: HasTextNode {
    fn is_editable_text(&self) -> bool {
        self.text_node().is_editable_text()
    }

    fn editable(&self, editable: bool) -> &Self {
        self.text_node().editable(editable);
        self
    }

    fn obscured(&self, obscured: bool) -> &Self {
        self.text_node().obscured(obscured);
        self
    }
}

impl<T: HasTextNode> TextEditingSurface for T {}

pub trait TextEventSurface: HasTextNode {
    fn on_text_changed(&self, handler: impl Fn(TextChangedEventArgs) + 'static) -> &Self {
        self.text_node().on_text_changed(handler);
        self
    }

    fn on_selection_changed(&self, handler: impl Fn(SelectionChangedEventArgs) + 'static) -> &Self {
        self.text_node().on_selection_changed(handler);
        self
    }
}

impl<T: HasTextNode> TextEventSurface for T {}

pub trait TextSurface:
    TextLayoutSurface
    + TextContentSurface
    + TextTypographySurface
    + TextSelectionSurface
    + TextEditingSurface
    + TextEventSurface
{
}

impl<T> TextSurface for T where
    T: TextLayoutSurface
        + TextContentSurface
        + TextTypographySurface
        + TextSelectionSurface
        + TextEditingSurface
        + TextEventSurface
{
}

impl HasTextNode for TextNode {
    fn text_node(&self) -> &TextNode {
        self
    }
}

pub trait LayoutSurface {
    #[doc(hidden)]
    fn layout_surface_root(&self) -> &FlexBox;

    #[doc(hidden)]
    fn set_layout_surface_width(&self, value: f32, unit: Unit);

    #[doc(hidden)]
    fn set_layout_surface_height(&self, value: f32, unit: Unit);

    fn width(&self, width: f32, unit: Unit) -> &Self {
        self.set_layout_surface_width(width, unit);
        self
    }

    fn width_len(&self, length: Length) -> &Self {
        let (value, unit) = length;
        self.set_layout_surface_width(value, unit);
        self
    }

    fn height(&self, height: f32, unit: Unit) -> &Self {
        self.set_layout_surface_height(height, unit);
        self
    }

    fn height_len(&self, length: Length) -> &Self {
        let (value, unit) = length;
        self.set_layout_surface_height(value, unit);
        self
    }

    fn fill_width(&self) -> &Self {
        self.layout_surface_root().fill_width();
        self
    }

    fn fill_height(&self) -> &Self {
        self.layout_surface_root().fill_height();
        self
    }

    fn fill_size(&self) -> &Self {
        self.layout_surface_root().fill_size();
        self
    }

    fn fill_width_percent(&self, percent: f32) -> &Self {
        self.layout_surface_root().fill_width_percent(percent);
        self
    }

    fn fill_height_percent(&self, percent: f32) -> &Self {
        self.layout_surface_root().fill_height_percent(percent);
        self
    }

    fn min_width(&self, value: f32, unit: Unit) -> &Self {
        self.layout_surface_root().min_width(value, unit);
        self
    }

    fn min_width_len(&self, length: Length) -> &Self {
        self.layout_surface_root().min_width_len(length);
        self
    }

    fn max_width(&self, value: f32, unit: Unit) -> &Self {
        self.layout_surface_root().max_width(value, unit);
        self
    }

    fn max_width_len(&self, length: Length) -> &Self {
        self.layout_surface_root().max_width_len(length);
        self
    }

    fn min_height(&self, value: f32, unit: Unit) -> &Self {
        self.layout_surface_root().min_height(value, unit);
        self
    }

    fn min_height_len(&self, length: Length) -> &Self {
        self.layout_surface_root().min_height_len(length);
        self
    }

    fn max_height(&self, value: f32, unit: Unit) -> &Self {
        self.layout_surface_root().max_height(value, unit);
        self
    }

    fn max_height_len(&self, length: Length) -> &Self {
        self.layout_surface_root().max_height_len(length);
        self
    }

    fn margin(&self, left: f32, top: f32, right: f32, bottom: f32) -> &Self {
        self.layout_surface_root().margin(left, top, right, bottom);
        self
    }

    fn position_type(&self, position_type: PositionType) -> &Self {
        self.layout_surface_root().position_type(position_type);
        self
    }

    fn position_absolute(&self) -> &Self {
        self.layout_surface_root()
            .position_type(PositionType::Absolute);
        self
    }

    fn position(&self, left: f32, top: f32) -> &Self {
        self.layout_surface_root().position(left, top);
        self
    }
}

impl<T: HasFlexBoxRoot> LayoutSurface for T {
    fn layout_surface_root(&self) -> &FlexBox {
        self.flex_box_root()
    }

    fn set_layout_surface_width(&self, value: f32, unit: Unit) {
        self.set_flex_box_surface_width(value, unit);
    }

    fn set_layout_surface_height(&self, value: f32, unit: Unit) {
        self.set_flex_box_surface_height(value, unit);
    }
}

pub trait BoxStyleSurface {
    #[doc(hidden)]
    fn box_style_surface_root(&self) -> &FlexBox;

    fn interactive(&self, interactive: bool) -> &Self {
        self.box_style_surface_root().interactive(interactive);
        self
    }

    fn padding(&self, left: f32, top: f32, right: f32, bottom: f32) -> &Self {
        self.box_style_surface_root()
            .padding(left, top, right, bottom);
        self
    }

    fn clear_padding(&self) -> &Self {
        self.box_style_surface_root().clear_padding();
        self
    }

    fn corner_radius(&self, radius: f32) -> &Self {
        self.box_style_surface_root().corner_radius(radius);
        self
    }

    fn clear_corners(&self) -> &Self {
        self.box_style_surface_root().clear_corners();
        self
    }

    fn corners(&self, tl: f32, tr: f32, br: f32, bl: f32) -> &Self {
        self.box_style_surface_root().corners(tl, tr, br, bl);
        self
    }

    fn border(&self, width: f32, color: u32) -> &Self {
        self.box_style_surface_root().border(width, color);
        self
    }

    fn border_config(&self, border: Border) -> &Self {
        self.box_style_surface_root().border_config(border);
        self
    }

    fn clear_border(&self) -> &Self {
        self.box_style_surface_root().clear_border();
        self
    }

    fn bg_color(&self, color: u32) -> &Self {
        self.box_style_surface_root().bg_color(color);
        self
    }

    fn clear_bg_color(&self) -> &Self {
        self.box_style_surface_root().clear_bg_color();
        self
    }

    fn opacity(&self, value: f32) -> &Self {
        self.box_style_surface_root().opacity(value);
        self
    }

    fn clear_opacity(&self) -> &Self {
        self.box_style_surface_root().clear_opacity();
        self
    }

    fn blur(&self, sigma: f32) -> &Self {
        self.box_style_surface_root().blur(sigma);
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
        self.box_style_surface_root()
            .drop_shadow(color, offset_x, offset_y, blur_sigma, spread);
        self
    }

    fn clear_drop_shadow(&self) -> &Self {
        self.box_style_surface_root().clear_drop_shadow();
        self
    }

    fn background_blur(&self, sigma: f32) -> &Self {
        self.box_style_surface_root().background_blur(sigma);
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
        self.box_style_surface_root()
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
        self.box_style_surface_root()
            .linear_gradient_stops(start_x, start_y, end_x, end_y, stops);
        self
    }

    fn transitions(&self, transitions: Option<NodeTransitions>) -> &Self {
        self.box_style_surface_root().transitions(transitions);
        self
    }

    fn clip_to_bounds(&self, clip: bool) -> &Self {
        self.box_style_surface_root().clip_to_bounds(clip);
        self
    }
}

impl<T: HasFlexBoxRoot> BoxStyleSurface for T {
    fn box_style_surface_root(&self) -> &FlexBox {
        self.flex_box_root()
    }
}

pub trait FlexLayoutSurface {
    #[doc(hidden)]
    fn flex_layout_surface_root(&self) -> &FlexBox;

    fn flex_direction(&self, direction: FlexDirection) -> &Self {
        self.flex_layout_surface_root().flex_direction(direction);
        self
    }

    fn flex_basis(&self, basis: f32) -> &Self {
        self.flex_layout_surface_root().flex_basis(basis);
        self
    }

    fn justify_content(&self, justify: JustifyContent) -> &Self {
        self.flex_layout_surface_root().justify_content(justify);
        self
    }

    fn clear_justify_content(&self) -> &Self {
        self.flex_layout_surface_root().clear_justify_content();
        self
    }

    fn align_items(&self, align: AlignItems) -> &Self {
        self.flex_layout_surface_root().align_items(align);
        self
    }

    fn clear_align_items(&self) -> &Self {
        self.flex_layout_surface_root().clear_align_items();
        self
    }

    fn align_self(&self, align: AlignSelf) -> &Self {
        self.flex_layout_surface_root().align_self(align);
        self
    }

    fn flex_wrap(&self, wrap: FlexWrap) -> &Self {
        self.flex_layout_surface_root().flex_wrap(wrap);
        self
    }
}

impl<T: HasFlexBoxRoot> FlexLayoutSurface for T {
    fn flex_layout_surface_root(&self) -> &FlexBox {
        self.flex_box_root()
    }
}

/// Fluent retained-child composition for FlexBox-backed nodes and controls.
///
/// Composed controls may install presenter-owned children before user content;
/// `child` and `children` append without replacing those presenter children.
/// A caller can remove a child it owns through [`Node::remove_child`] without
/// disturbing the control-owned prefix. Controls with a distinct public content
/// surface override the hidden append adapter to match their FUI-AS counterpart.
pub trait ChildContainerSurface {
    #[doc(hidden)]
    fn append_surface_child(&self, child: NodeRef);

    fn child<T: Node>(&self, child: &T) -> &Self {
        self.append_surface_child(child.retained_node_ref());
        self
    }

    fn children<I, C>(&self, children: I) -> &Self
    where
        I: IntoIterator<Item = C>,
        C: Into<Child>,
    {
        for child in children {
            self.append_surface_child(child.into().node_ref);
        }
        self
    }
}

impl<T: HasFlexBoxRoot> ChildContainerSurface for T {
    fn append_surface_child(&self, child: NodeRef) {
        self.append_flex_box_surface_child(child);
    }
}

pub trait FlexBoxSurface:
    LayoutSurface + BoxStyleSurface + FlexLayoutSurface + ChildContainerSurface
{
}

impl<T> FlexBoxSurface for T where
    T: LayoutSurface + BoxStyleSurface + FlexLayoutSurface + ChildContainerSurface
{
}

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
