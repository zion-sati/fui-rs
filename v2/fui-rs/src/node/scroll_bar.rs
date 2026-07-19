use super::*;
use crate::bindings::ui;
use crate::event::PointerEventArgs;
use crate::ffi::{CursorStyle, Orientation, Unit};
use crate::platform;
use crate::signal::Subscription;
use crate::theme::{current_theme, subscribe, Theme};
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

const DEFAULT_TRACK_THICKNESS: f32 = 8.0;
const DEFAULT_THUMB_THICKNESS: f32 = 8.0;
const DEFAULT_MIN_THUMB_SIZE: f32 = 18.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollBarStyle {
    track_width: f32,
    thumb_width: f32,
    thumb_min_height: f32,
    track_corner_radius: f32,
    thumb_corner_radius: f32,
    track_color: Option<u32>,
    thumb_color: Option<u32>,
}

impl ScrollBarStyle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn track_width(mut self, value: f32) -> Self {
        self.track_width = value;
        self
    }

    pub fn thumb_width(mut self, value: f32) -> Self {
        self.thumb_width = value;
        self
    }

    pub fn thumb_min_height(mut self, value: f32) -> Self {
        self.thumb_min_height = value;
        self
    }

    pub fn track_corner_radius(mut self, value: f32) -> Self {
        self.track_corner_radius = value;
        self
    }

    pub fn thumb_corner_radius(mut self, value: f32) -> Self {
        self.thumb_corner_radius = value;
        self
    }

    pub fn track_color(mut self, value: u32) -> Self {
        self.track_color = Some(value);
        self
    }

    pub fn thumb_color(mut self, value: u32) -> Self {
        self.thumb_color = Some(value);
        self
    }

    pub(crate) fn apply_to(self, scroll_bar: &ScrollBar) {
        scroll_bar
            .track_width(self.track_width)
            .thumb_width(self.thumb_width)
            .thumb_min_height(self.thumb_min_height)
            .track_corner_radius(self.track_corner_radius)
            .thumb_corner_radius(self.thumb_corner_radius);
        if let Some(color) = self.track_color {
            scroll_bar.track_color(color);
        } else {
            scroll_bar.clear_track_color();
        }
        if let Some(color) = self.thumb_color {
            scroll_bar.thumb_color(color);
        } else {
            scroll_bar.clear_thumb_color();
        }
    }
}

impl Default for ScrollBarStyle {
    fn default() -> Self {
        Self {
            track_width: DEFAULT_TRACK_THICKNESS,
            thumb_width: DEFAULT_THUMB_THICKNESS,
            thumb_min_height: DEFAULT_MIN_THUMB_SIZE,
            track_corner_radius: 0.0,
            thumb_corner_radius: 0.0,
            track_color: None,
            thumb_color: None,
        }
    }
}

fn clamp(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        return min;
    }
    if value > max {
        return max;
    }
    value
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollBarVisibility {
    Always,
    Auto,
    Never,
}

#[derive(Clone, Copy, Default)]
struct ScrollMetrics {
    viewport_size: f32,
    max_offset: f32,
    thumb_size: f32,
    max_thumb_offset: f32,
    thumb_offset: f32,
    trailing_spacer_size: f32,
}

struct ScrollBarInner {
    scroll_state: ScrollState,
    orientation: Orientation,
    target_handle: Cell<u64>,
    drag_start_axis: Cell<f32>,
    drag_start_offset: Cell<f32>,
    dragging: Cell<bool>,
    chrome_visible: Cell<bool>,
    track_thickness: Cell<f32>,
    thumb_thickness: Cell<f32>,
    min_thumb_size: Cell<f32>,
    track_corner_radius: Cell<f32>,
    thumb_corner_radius: Cell<f32>,
    track_color: Cell<u32>,
    thumb_color: Cell<u32>,
    track_color_overridden: Cell<bool>,
    thumb_color_overridden: Cell<bool>,
    track_node: FlexBox,
    track_strip: FlexBox,
    leading_spacer_node: FlexBox,
    thumb_node: FlexBox,
    trailing_spacer_node: FlexBox,
    subscriptions: RefCell<Vec<Subscription>>,
    chrome_metric_changed: RefCell<Option<Rc<dyn Fn()>>>,
}

#[derive(Clone)]
/// Scroll chrome helper matching FUI-AS `ScrollBar`.
///
/// This is not a retained `Node`; attach [`render`](Self::render) to a retained
/// tree. It follows the active theme automatically. Per-instance theme
/// callbacks belong on the rendered `FlexBox`, avoiding a root/state ownership
/// cycle in this helper.
pub struct ScrollBar {
    inner: Rc<ScrollBarInner>,
}

impl ScrollBar {
    pub fn new(scroll_state: ScrollState, orientation: Orientation) -> Self {
        let orientation = if orientation == Orientation::Horizontal {
            Orientation::Horizontal
        } else {
            Orientation::Vertical
        };
        let track_node = flex_box();
        let track_strip = if orientation == Orientation::Horizontal {
            row()
        } else {
            column()
        };
        let leading_spacer_node = flex_box();
        let thumb_node = flex_box();
        let trailing_spacer_node = flex_box();
        track_strip
            .align_items(crate::ffi::AlignItems::Center)
            .child(&leading_spacer_node)
            .child(&thumb_node)
            .child(&trailing_spacer_node);
        let interactive = !platform::is_coarse_pointer();
        track_node.clip_to_bounds(true).child(&track_strip);
        if interactive {
            track_node.on_pointer_down(|_event: &mut PointerEventArgs| {});
            thumb_node
                .on_pointer_down(|_event: &mut PointerEventArgs| {})
                .on_pointer_enter(|_event: &mut PointerEventArgs| {})
                .on_pointer_leave(|_event: &mut PointerEventArgs| {})
                .on_pointer_move(|_event: &mut PointerEventArgs| {})
                .on_pointer_up(|_event: &mut PointerEventArgs| {});
        }

        let theme = current_theme();
        let inner = Rc::new(ScrollBarInner {
            scroll_state,
            orientation,
            target_handle: Cell::new(HandleValue::Invalid as u64),
            drag_start_axis: Cell::new(0.0),
            drag_start_offset: Cell::new(0.0),
            dragging: Cell::new(false),
            chrome_visible: Cell::new(true),
            track_thickness: Cell::new(DEFAULT_TRACK_THICKNESS),
            thumb_thickness: Cell::new(DEFAULT_THUMB_THICKNESS),
            min_thumb_size: Cell::new(DEFAULT_MIN_THUMB_SIZE),
            track_corner_radius: Cell::new(0.0),
            thumb_corner_radius: Cell::new(0.0),
            track_color: Cell::new(theme.colors.scrollbar_track),
            thumb_color: Cell::new(theme.colors.scrollbar_thumb),
            track_color_overridden: Cell::new(false),
            thumb_color_overridden: Cell::new(false),
            track_node,
            track_strip,
            leading_spacer_node,
            thumb_node,
            trailing_spacer_node,
            subscriptions: RefCell::new(Vec::new()),
            chrome_metric_changed: RefCell::new(None),
        });
        let bar = Self { inner };
        if interactive {
            bar.attach_handlers();
        }
        bar.attach_listeners();
        bar.apply_geometry_style();
        bar.apply_color_style();
        bar.sync_visual_state();
        bar.apply_thumb_cursor();
        bar
    }

    pub fn thickness(&self) -> f32 {
        self.inner.track_thickness.get()
    }

    pub fn render(&self) -> FlexBox {
        self.inner.track_node.clone()
    }

    pub fn track_width(&self, value: f32) -> &Self {
        self.set_track_thickness(value);
        self
    }

    pub fn track_thickness(&self, value: f32) -> &Self {
        self.set_track_thickness(value);
        self
    }

    pub fn thumb_width(&self, value: f32) -> &Self {
        self.set_thumb_thickness(value);
        self
    }

    pub fn thumb_thickness(&self, value: f32) -> &Self {
        self.set_thumb_thickness(value);
        self
    }

    pub fn thumb_min_height(&self, value: f32) -> &Self {
        self.set_thumb_min_size(value);
        self
    }

    fn set_track_thickness(&self, value: f32) {
        let next = if value > 0.0 { value } else { 1.0 };
        if value <= 0.0 {
            crate::logger::warn(
                "Layout",
                &format!("ScrollBar.trackThickness() received {value}; clamping to 1.0."),
            );
        }
        if self.inner.track_thickness.get() == next {
            return;
        }
        self.inner.track_thickness.set(next);
        self.apply_geometry_style();
        self.notify_chrome_metric_changed();
    }

    fn set_thumb_thickness(&self, value: f32) {
        let next = if value > 0.0 { value } else { 1.0 };
        if value <= 0.0 {
            crate::logger::warn(
                "Layout",
                &format!("ScrollBar.thumbThickness() received {value}; clamping to 1.0."),
            );
        }
        self.inner.thumb_thickness.set(next);
        self.apply_geometry_style();
    }

    fn set_thumb_min_size(&self, value: f32) {
        let next = if value > 0.0 { value } else { 1.0 };
        if value <= 0.0 {
            crate::logger::warn(
                "Layout",
                &format!("ScrollBar.thumbMinHeight() received {value}; clamping to 1.0."),
            );
        }
        self.inner.min_thumb_size.set(next);
        self.sync_visual_state();
    }

    pub fn track_corner_radius(&self, radius: f32) -> &Self {
        self.inner.track_corner_radius.set(radius.max(0.0));
        self.inner
            .track_node
            .corner_radius(self.inner.track_corner_radius.get());
        self
    }

    pub fn thumb_corner_radius(&self, radius: f32) -> &Self {
        self.inner.thumb_corner_radius.set(radius.max(0.0));
        self.inner
            .thumb_node
            .corner_radius(self.inner.thumb_corner_radius.get());
        self
    }

    pub fn track_color(&self, color: u32) -> &Self {
        self.inner.track_color_overridden.set(true);
        self.inner.track_color.set(color);
        self.apply_color_style();
        self
    }

    fn clear_track_color(&self) {
        self.inner.track_color_overridden.set(false);
        self.inner
            .track_color
            .set(current_theme().colors.scrollbar_track);
        self.apply_color_style();
    }

    pub fn thumb_color(&self, color: u32) -> &Self {
        self.inner.thumb_color_overridden.set(true);
        self.inner.thumb_color.set(color);
        self.apply_color_style();
        self
    }

    fn clear_thumb_color(&self) {
        self.inner.thumb_color_overridden.set(false);
        self.inner
            .thumb_color
            .set(current_theme().colors.scrollbar_thumb);
        self.apply_color_style();
    }

    pub fn bind_scroll_handle(&self, handle: u64) {
        self.inner.target_handle.set(handle);
        self.inner
            .track_node
            .bind_scroll_proxy_target_handle(handle);
        self.inner
            .track_strip
            .bind_scroll_proxy_target_handle(handle);
        self.inner
            .leading_spacer_node
            .bind_scroll_proxy_target_handle(handle);
        self.inner
            .thumb_node
            .bind_scroll_proxy_target_handle(handle);
        self.inner
            .trailing_spacer_node
            .bind_scroll_proxy_target_handle(handle);
    }

    pub fn clear_scroll_handle(&self, handle: u64) {
        if self.inner.target_handle.get() == handle {
            self.bind_scroll_handle(HandleValue::Invalid as u64);
        }
    }

    pub fn chrome_visible(&self, visible: bool) {
        if self.inner.chrome_visible.get() == visible {
            return;
        }
        self.inner.chrome_visible.set(visible);
        if !visible {
            self.end_drag();
        }
        self.apply_geometry_style();
        self.sync_visual_state();
        self.apply_thumb_cursor();
    }

    pub fn refresh_now(&self) {
        self.sync_visual_state();
    }

    pub fn can_start_thumb_drag(&self) -> bool {
        let metrics = self.compute_metrics();
        self.inner.chrome_visible.get()
            && metrics.max_offset > 0.0
            && metrics.max_thumb_offset > 0.0
    }

    pub(crate) fn set_chrome_metric_changed_handler(&self, handler: impl Fn() + 'static) {
        *self.inner.chrome_metric_changed.borrow_mut() = Some(Rc::new(handler));
    }

    fn attach_handlers(&self) {
        let weak = Rc::downgrade(&self.inner);
        self.inner.track_node.on_pointer_down(move |event| {
            if let Some(inner) = Weak::upgrade(&weak) {
                ScrollBar { inner }.handle_track_pointer_down(event.scene_x, event.scene_y);
            }
        });

        let weak = Rc::downgrade(&self.inner);
        self.inner.thumb_node.on_pointer_down(move |event| {
            if let Some(inner) = Weak::upgrade(&weak) {
                let bar = ScrollBar { inner };
                if bar.can_start_thumb_drag() {
                    bar.inner.dragging.set(true);
                    bar.inner
                        .drag_start_axis
                        .set(bar.axis_position(event.scene_x, event.scene_y));
                    bar.inner.drag_start_offset.set(bar.axis_offset());
                    event.capture_pointer();
                    event.handled = true;
                    bar.apply_thumb_cursor();
                }
            }
        });

        let weak = Rc::downgrade(&self.inner);
        self.inner.thumb_node.on_pointer_move(move |event| {
            if let Some(inner) = Weak::upgrade(&weak) {
                let bar = ScrollBar { inner };
                if !bar.inner.dragging.get() {
                    return;
                }
                let metrics = bar.compute_metrics();
                if metrics.max_offset <= 0.0 || metrics.max_thumb_offset <= 0.0 {
                    return;
                }
                let delta = bar.axis_position(event.scene_x, event.scene_y)
                    - bar.inner.drag_start_axis.get();
                let offset_per_thumb_pixel = metrics.max_offset / metrics.max_thumb_offset;
                bar.set_scroll_offset(
                    bar.inner.drag_start_offset.get() + (delta * offset_per_thumb_pixel),
                    metrics.max_offset,
                );
                event.handled = true;
            }
        });

        let weak = Rc::downgrade(&self.inner);
        self.inner.thumb_node.on_pointer_up(move |event| {
            if let Some(inner) = Weak::upgrade(&weak) {
                let bar = ScrollBar { inner };
                if bar.inner.dragging.get() {
                    bar.end_drag();
                    event.handled = true;
                }
            }
        });
    }

    fn attach_listeners(&self) {
        let weak = Rc::downgrade(&self.inner);
        self.inner
            .subscriptions
            .borrow_mut()
            .push(self.inner.scroll_state.subscribe_offset_x(move || {
                if let Some(inner) = Weak::upgrade(&weak) {
                    ScrollBar { inner }.sync_visual_state();
                }
            }));
        let weak = Rc::downgrade(&self.inner);
        self.inner
            .subscriptions
            .borrow_mut()
            .push(self.inner.scroll_state.subscribe_offset_y(move || {
                if let Some(inner) = Weak::upgrade(&weak) {
                    ScrollBar { inner }.sync_visual_state();
                }
            }));
        let weak = Rc::downgrade(&self.inner);
        self.inner.subscriptions.borrow_mut().push(
            self.inner.scroll_state.subscribe_content_width(move || {
                if let Some(inner) = Weak::upgrade(&weak) {
                    ScrollBar { inner }.sync_visual_state();
                }
            }),
        );
        let weak = Rc::downgrade(&self.inner);
        self.inner.subscriptions.borrow_mut().push(
            self.inner.scroll_state.subscribe_content_height(move || {
                if let Some(inner) = Weak::upgrade(&weak) {
                    ScrollBar { inner }.sync_visual_state();
                }
            }),
        );
        let weak = Rc::downgrade(&self.inner);
        self.inner.subscriptions.borrow_mut().push(
            self.inner.scroll_state.subscribe_viewport_width(move || {
                if let Some(inner) = Weak::upgrade(&weak) {
                    ScrollBar { inner }.sync_visual_state();
                }
            }),
        );
        let weak = Rc::downgrade(&self.inner);
        self.inner.subscriptions.borrow_mut().push(
            self.inner.scroll_state.subscribe_viewport_height(move || {
                if let Some(inner) = Weak::upgrade(&weak) {
                    ScrollBar { inner }.sync_visual_state();
                }
            }),
        );
        let weak = Rc::downgrade(&self.inner);
        self.inner
            .subscriptions
            .borrow_mut()
            .push(subscribe(move |theme| {
                if let Some(inner) = Weak::upgrade(&weak) {
                    ScrollBar { inner }.handle_theme_changed(theme);
                }
            }));
    }

    fn handle_theme_changed(&self, theme: Theme) {
        if !self.inner.track_color_overridden.get() {
            self.inner.track_color.set(theme.colors.scrollbar_track);
        }
        if !self.inner.thumb_color_overridden.get() {
            self.inner.thumb_color.set(theme.colors.scrollbar_thumb);
        }
        self.apply_color_style();
    }

    fn notify_chrome_metric_changed(&self) {
        if let Some(handler) = self.inner.chrome_metric_changed.borrow().clone() {
            handler();
        }
    }

    fn end_drag(&self) {
        self.inner.dragging.set(false);
        crate::event::release_pointer(self.inner.thumb_node.handle());
        unsafe { crate::ffi::fui_release_pointer_capture() };
        self.apply_thumb_cursor();
    }

    fn compute_metrics(&self) -> ScrollMetrics {
        let viewport_size = if self.inner.orientation == Orientation::Horizontal {
            self.inner.scroll_state.viewport_width()
        } else {
            self.inner.scroll_state.viewport_height()
        }
        .max(0.0);
        let content_size = if self.inner.orientation == Orientation::Horizontal {
            self.inner.scroll_state.content_width()
        } else {
            self.inner.scroll_state.content_height()
        }
        .max(viewport_size);
        let max_offset = content_size - viewport_size;
        let raw_thumb_size = if content_size > 0.0 {
            viewport_size * (viewport_size / content_size)
        } else {
            0.0
        };
        let thumb_size = if viewport_size > 0.0 {
            clamp(
                raw_thumb_size,
                self.inner.min_thumb_size.get(),
                viewport_size,
            )
        } else {
            0.0
        };
        let max_thumb_offset = if viewport_size > thumb_size {
            viewport_size - thumb_size
        } else {
            0.0
        };
        let offset = self.axis_offset();
        let thumb_offset = if max_offset > 0.0 && max_thumb_offset > 0.0 {
            clamp(
                (offset / max_offset) * max_thumb_offset,
                0.0,
                max_thumb_offset,
            )
        } else {
            0.0
        };
        ScrollMetrics {
            viewport_size,
            max_offset,
            thumb_size,
            max_thumb_offset,
            thumb_offset,
            trailing_spacer_size: (viewport_size - thumb_size - thumb_offset).max(0.0),
        }
    }

    fn sync_visual_state(&self) {
        if !self.inner.chrome_visible.get() {
            if self.inner.orientation == Orientation::Horizontal {
                self.inner.track_node.width(0.0, Unit::Pixel);
                self.inner.track_strip.width(0.0, Unit::Pixel);
                self.inner.leading_spacer_node.width(0.0, Unit::Pixel);
                self.inner.thumb_node.width(0.0, Unit::Pixel);
                self.inner.trailing_spacer_node.width(0.0, Unit::Pixel);
            } else {
                self.inner.track_node.height(0.0, Unit::Pixel);
                self.inner.track_strip.height(0.0, Unit::Pixel);
                self.inner.leading_spacer_node.height(0.0, Unit::Pixel);
                self.inner.thumb_node.height(0.0, Unit::Pixel);
                self.inner.trailing_spacer_node.height(0.0, Unit::Pixel);
            }
            return;
        }

        let metrics = self.compute_metrics();
        if self.inner.orientation == Orientation::Horizontal {
            self.inner
                .track_node
                .width(metrics.viewport_size, Unit::Pixel);
            self.inner
                .track_strip
                .width(metrics.viewport_size, Unit::Pixel);
            self.inner
                .leading_spacer_node
                .width(metrics.thumb_offset, Unit::Pixel);
            self.inner.thumb_node.width(metrics.thumb_size, Unit::Pixel);
            self.inner
                .trailing_spacer_node
                .width(metrics.trailing_spacer_size, Unit::Pixel);
        } else {
            self.inner
                .track_node
                .height(metrics.viewport_size, Unit::Pixel);
            self.inner
                .track_strip
                .height(metrics.viewport_size, Unit::Pixel);
            self.inner
                .leading_spacer_node
                .height(metrics.thumb_offset, Unit::Pixel);
            self.inner
                .thumb_node
                .height(metrics.thumb_size, Unit::Pixel);
            self.inner
                .trailing_spacer_node
                .height(metrics.trailing_spacer_size, Unit::Pixel);
        }
    }

    fn apply_geometry_style(&self) {
        let track_thickness = if self.inner.chrome_visible.get() {
            self.inner.track_thickness.get()
        } else {
            0.0
        };
        let thumb_thickness = if self.inner.chrome_visible.get() {
            self.inner.thumb_thickness.get()
        } else {
            0.0
        };
        if self.inner.orientation == Orientation::Horizontal {
            self.inner.track_node.height(track_thickness, Unit::Pixel);
            self.inner.track_strip.height(track_thickness, Unit::Pixel);
            self.inner
                .leading_spacer_node
                .height(track_thickness, Unit::Pixel);
            self.inner
                .trailing_spacer_node
                .height(track_thickness, Unit::Pixel);
            self.inner.thumb_node.height(thumb_thickness, Unit::Pixel);
        } else {
            self.inner.track_node.width(track_thickness, Unit::Pixel);
            self.inner.track_strip.width(track_thickness, Unit::Pixel);
            self.inner
                .leading_spacer_node
                .width(track_thickness, Unit::Pixel);
            self.inner
                .trailing_spacer_node
                .width(track_thickness, Unit::Pixel);
            self.inner.thumb_node.width(thumb_thickness, Unit::Pixel);
        }
        self.inner
            .track_node
            .corner_radius(self.inner.track_corner_radius.get());
        self.inner
            .thumb_node
            .corner_radius(self.inner.thumb_corner_radius.get());
    }

    fn apply_color_style(&self) {
        self.inner.track_node.bg_color(self.inner.track_color.get());
        self.inner.thumb_node.bg_color(self.inner.thumb_color.get());
    }

    fn apply_thumb_cursor(&self) {
        if platform::is_coarse_pointer() {
            return;
        }
        self.inner
            .thumb_node
            .cursor(if !self.inner.chrome_visible.get() {
                CursorStyle::Default
            } else if self.inner.dragging.get() {
                CursorStyle::Grabbing
            } else {
                CursorStyle::Grab
            });
    }

    fn axis_position(&self, x: f32, y: f32) -> f32 {
        if self.inner.orientation == Orientation::Horizontal {
            x
        } else {
            y
        }
    }

    fn axis_offset(&self) -> f32 {
        if self.inner.orientation == Orientation::Horizontal {
            self.inner.scroll_state.offset_x()
        } else {
            self.inner.scroll_state.offset_y()
        }
    }

    fn handle_track_pointer_down(&self, pointer_x: f32, pointer_y: f32) {
        if self.inner.dragging.get() {
            return;
        }
        let metrics = self.compute_metrics();
        if metrics.max_offset <= 0.0 || metrics.max_thumb_offset <= 0.0 {
            return;
        }
        let Some(bounds) = ui::get_bounds(self.inner.track_node.handle().raw()) else {
            return;
        };
        let local_pointer = if self.inner.orientation == Orientation::Horizontal {
            pointer_x - bounds[0]
        } else {
            pointer_y - bounds[1]
        };
        let target_thumb_offset = clamp(
            local_pointer - (metrics.thumb_size * 0.5),
            0.0,
            metrics.max_thumb_offset,
        );
        let next_offset = (target_thumb_offset / metrics.max_thumb_offset) * metrics.max_offset;
        self.set_scroll_offset(next_offset, metrics.max_offset);
    }

    fn set_scroll_offset(&self, offset: f32, max_offset: f32) {
        let clamped_offset = clamp(offset, 0.0, max_offset);
        if self.inner.orientation == Orientation::Horizontal {
            self.inner.scroll_state.set_offset_x(clamped_offset);
            if self.inner.target_handle.get() != HandleValue::Invalid as u64 {
                ui::set_scroll_offset(
                    self.inner.target_handle.get(),
                    clamped_offset,
                    self.inner.scroll_state.offset_y(),
                );
                crate::frame_scheduler::mark_needs_commit();
            }
        } else {
            self.inner.scroll_state.set_offset_y(clamped_offset);
            if self.inner.target_handle.get() != HandleValue::Invalid as u64 {
                ui::set_scroll_offset(
                    self.inner.target_handle.get(),
                    self.inner.scroll_state.offset_x(),
                    clamped_offset,
                );
                crate::frame_scheduler::mark_needs_commit();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::{self, Call};
    use crate::theme::{generate_theme, use_custom_theme, use_system_theme};
    use crate::Application;

    #[test]
    fn scrollbar_style_surface_matches_fui_as_vertical_and_horizontal_geometry() {
        ffi::test::reset();
        let vertical_state = ScrollState::new();
        vertical_state.set_viewport_height(100.0);
        vertical_state.set_content_height(400.0);
        let vertical = ScrollBar::new(vertical_state, Orientation::Vertical);
        vertical
            .track_width(12.0)
            .thumb_width(6.0)
            .thumb_min_height(30.0);
        Application::mount(vertical.render());
        let vertical_track_handle = vertical.inner.track_node.handle().raw();
        let vertical_thumb_handle = vertical.inner.thumb_node.handle().raw();
        let calls = ffi::test::take_calls();

        assert!(calls.iter().any(|call| matches!(
            call,
            Call::SetWidth { handle, value, .. } if *handle == vertical_track_handle && (*value - 12.0).abs() < f32::EPSILON
        )));
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::SetWidth { handle, value, .. } if *handle == vertical_thumb_handle && (*value - 6.0).abs() < f32::EPSILON
        )));
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::SetHeight { handle, value, .. } if *handle == vertical_thumb_handle && (*value - 30.0).abs() < f32::EPSILON
        )));

        ffi::test::reset();
        let horizontal_state = ScrollState::new();
        horizontal_state.set_viewport_width(100.0);
        horizontal_state.set_content_width(400.0);
        let horizontal = ScrollBar::new(horizontal_state, Orientation::Horizontal);
        horizontal
            .track_width(14.0)
            .thumb_width(9.0)
            .thumb_min_height(32.0);
        Application::mount(horizontal.render());
        let horizontal_track_handle = horizontal.inner.track_node.handle().raw();
        let horizontal_thumb_handle = horizontal.inner.thumb_node.handle().raw();
        let calls = ffi::test::take_calls();

        assert!(calls.iter().any(|call| matches!(
            call,
            Call::SetHeight { handle, value, .. } if *handle == horizontal_track_handle && (*value - 14.0).abs() < f32::EPSILON
        )));
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::SetHeight { handle, value, .. } if *handle == horizontal_thumb_handle && (*value - 9.0).abs() < f32::EPSILON
        )));
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::SetWidth { handle, value, .. } if *handle == horizontal_thumb_handle && (*value - 32.0).abs() < f32::EPSILON
        )));
    }

    #[test]
    fn scrollbar_theme_changes_update_default_colors_and_overrides_survive() {
        ffi::test::reset();
        use_custom_theme(generate_theme(true, 0x112233FF));
        let state = ScrollState::new();
        state.set_viewport_height(100.0);
        state.set_content_height(400.0);
        let scrollbar = ScrollBar::new(state, Orientation::Vertical);
        Application::mount(scrollbar.render());
        let _ = ffi::test::take_calls();

        use_custom_theme(generate_theme(false, 0x445566FF));
        let theme = current_theme();
        let calls = ffi::test::take_calls();
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::SetBgColor { color, .. } if *color == theme.colors.scrollbar_track
        )));
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::SetBgColor { color, .. } if *color == theme.colors.scrollbar_thumb
        )));

        ffi::test::reset();
        let state = ScrollState::new();
        state.set_viewport_height(100.0);
        state.set_content_height(400.0);
        let scrollbar = ScrollBar::new(state, Orientation::Vertical);
        scrollbar
            .track_width(10.0)
            .thumb_width(7.0)
            .track_corner_radius(5.0)
            .thumb_corner_radius(4.0)
            .track_color(0x123456FF)
            .thumb_color(0xABCDEF88);
        Application::mount(scrollbar.render());
        let mount_calls = ffi::test::take_calls();
        assert!(mount_calls.iter().any(|call| matches!(
            call,
            Call::SetBoxStyle { radius_tl, radius_tr, radius_br, radius_bl, .. }
                if *radius_tl == 5.0
                    && *radius_tr == 5.0
                    && *radius_br == 5.0
                    && *radius_bl == 5.0
        )));
        assert!(mount_calls.iter().any(|call| matches!(
            call,
            Call::SetBoxStyle { radius_tl, radius_tr, radius_br, radius_bl, .. }
                if *radius_tl == 4.0
                    && *radius_tr == 4.0
                    && *radius_br == 4.0
                    && *radius_bl == 4.0
        )));

        use_custom_theme(generate_theme(true, 0x556677FF));
        let calls = ffi::test::take_calls();
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::SetBgColor { color, .. } if *color == 0x123456FF
        )));
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::SetBgColor { color, .. } if *color == 0xABCDEF88
        )));

        use_system_theme();
    }

    #[test]
    fn scrollbar_invalid_recipe_values_warn_and_clamp_like_fui_as() {
        ffi::test::reset();
        let state = ScrollState::new();
        state.set_viewport_height(100.0);
        state.set_content_height(400.0);
        let scrollbar = ScrollBar::new(state, Orientation::Vertical);
        scrollbar
            .track_width(0.0)
            .thumb_width(-2.0)
            .thumb_min_height(0.0);
        Application::mount(scrollbar.render());
        let calls = ffi::test::take_calls();

        assert!(calls.iter().any(|call| matches!(
            call,
            Call::Log { category, message }
                if category == "Warning/Layout"
                    && message == "ScrollBar.trackThickness() received 0; clamping to 1.0."
        )));
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::Log { category, message }
                if category == "Warning/Layout"
                    && message == "ScrollBar.thumbThickness() received -2; clamping to 1.0."
        )));
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::Log { category, message }
                if category == "Warning/Layout"
                    && message == "ScrollBar.thumbMinHeight() received 0; clamping to 1.0."
        )));
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::SetWidth { value, .. } if (*value - 1.0).abs() < f32::EPSILON
        )));
    }
}
