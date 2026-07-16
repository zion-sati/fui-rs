use super::*;
use crate::animation::AnimationTiming;
use crate::ffi::{Orientation, Unit};
use crate::persisted::{store_scroll_offset, try_load_scroll_offset};
use crate::signal::Subscription;
use crate::transitions::NodeTransitions;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

const DEFAULT_SCROLLBAR_GUTTER: f32 = 0.0;
const OVERFLOW_TOLERANCE: f32 = 0.5;

fn clamp(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        return min;
    }
    if value > max {
        return max;
    }
    value
}

struct ScrollBoxInner {
    root: FlexBox,
    scroll_state: ScrollState,
    viewport: ScrollView,
    top_row: FlexBox,
    bottom_row: FlexBox,
    vertical_gutter: FlexBox,
    corner: FlexBox,
    vertical_scrollbar: ScrollBar,
    horizontal_scrollbar: ScrollBar,
    vertical_visibility: Cell<ScrollBarVisibility>,
    horizontal_visibility: Cell<ScrollBarVisibility>,
    vertical_scroll_enabled: Cell<bool>,
    horizontal_scroll_enabled: Cell<bool>,
    scrollbar_gutter: Cell<f32>,
    persist_scroll: Cell<bool>,
    persisted_restore_pending: Cell<bool>,
    subscriptions: RefCell<Vec<Subscription>>,
}

#[derive(Clone)]
pub struct ScrollBox {
    inner: Rc<ScrollBoxInner>,
}

impl ScrollBox {
    pub fn new() -> Self {
        Self::with_parts(ScrollState::new(), ScrollView::new())
    }

    pub fn with_parts(scroll_state: ScrollState, viewport: ScrollView) -> Self {
        let root = column();
        let viewport = viewport
            .bind_scroll_state(scroll_state.clone())
            .fill_size()
            .clone();
        let vertical_gutter = flex_box();
        vertical_gutter
            .width(DEFAULT_SCROLLBAR_GUTTER, Unit::Pixel)
            .fill_height()
            .on_pointer_down(|_event| {});
        let vertical_scrollbar = ScrollBar::new(scroll_state.clone(), Orientation::Vertical);
        let corner = flex_box();
        corner
            .width(0.0, Unit::Pixel)
            .height(0.0, Unit::Pixel)
            .on_pointer_down(|_event| {});
        let horizontal_scrollbar = ScrollBar::new(scroll_state.clone(), Orientation::Horizontal);
        let top_row = row();
        top_row
            .fill_size()
            .on_pointer_down(|_event| {})
            .child(&viewport)
            .child(&vertical_gutter)
            .child(&vertical_scrollbar.render());
        let bottom_row = row();
        bottom_row
            .fill_width()
            .on_pointer_down(|_event| {})
            .child(&horizontal_scrollbar.render())
            .child(&corner);
        root.child(&top_row).child(&bottom_row);

        let inner = Rc::new(ScrollBoxInner {
            root,
            scroll_state,
            viewport,
            top_row,
            bottom_row,
            vertical_gutter,
            corner,
            vertical_scrollbar,
            horizontal_scrollbar,
            vertical_visibility: Cell::new(ScrollBarVisibility::Auto),
            horizontal_visibility: Cell::new(ScrollBarVisibility::Auto),
            vertical_scroll_enabled: Cell::new(true),
            horizontal_scroll_enabled: Cell::new(true),
            scrollbar_gutter: Cell::new(DEFAULT_SCROLLBAR_GUTTER),
            persist_scroll: Cell::new(true),
            persisted_restore_pending: Cell::new(false),
            subscriptions: RefCell::new(Vec::new()),
        });
        let scroll_box = Self { inner };
        scroll_box.attach_listeners();
        scroll_box.refresh_chrome();
        scroll_box
    }

    pub fn scroll_state(&self) -> ScrollState {
        self.inner.scroll_state.clone()
    }

    pub fn viewport(&self) -> ScrollView {
        self.inner.viewport.clone()
    }

    pub fn vertical_scrollbar(&self) -> ScrollBar {
        self.inner.vertical_scrollbar.clone()
    }

    pub fn horizontal_scrollbar(&self) -> ScrollBar {
        self.inner.horizontal_scrollbar.clone()
    }

    pub fn child<T: Node>(&self, node: &T) -> &Self {
        self.inner.viewport.child(node);
        self.bind_content_scroll_proxy_targets();
        self
    }

    pub fn children<I, C>(&self, nodes: I) -> &Self
    where
        I: IntoIterator<Item = C>,
        C: Into<Child>,
    {
        for node in nodes {
            self.inner
                .viewport
                .retained_node_ref()
                .append_child_ref(&node.into().node_ref);
        }
        self.bind_content_scroll_proxy_targets();
        self
    }

    pub fn scroll_enabled_x(&self, enabled: bool) -> &Self {
        self.inner.horizontal_scroll_enabled.set(enabled);
        self.inner.viewport.scroll_enabled_x(enabled);
        self.refresh_chrome();
        self
    }

    pub fn scroll_enabled_y(&self, enabled: bool) -> &Self {
        self.inner.vertical_scroll_enabled.set(enabled);
        self.inner.viewport.scroll_enabled_y(enabled);
        self.refresh_chrome();
        self
    }

    pub fn smooth_scrolling(&self, smooth_scrolling: bool) -> &Self {
        self.inner.viewport.smooth_scrolling(smooth_scrolling);
        self
    }

    pub fn persist_scroll(&self, persist: bool) -> &Self {
        self.inner.persist_scroll.set(persist);
        if !persist {
            self.inner.persisted_restore_pending.set(false);
        }
        self
    }

    pub fn scroll_offset(&self, x: f32, y: f32) -> &Self {
        self.inner.viewport.scroll_offset(x, y);
        self
    }

    pub fn transitions(&self, transitions: Option<NodeTransitions>) -> &Self {
        self.inner.viewport.transitions(transitions);
        self
    }

    pub fn scroll_content_size(&self, width: f32, height: f32) -> &Self {
        self.inner.viewport.scroll_content_size(width, height);
        self
    }

    pub fn scroll_to(&self, x: f32, y: f32) -> &Self {
        self.inner.viewport.scroll_to(x, y);
        self
    }

    pub fn scroll_to_animated(&self, x: f32, y: f32, timing: AnimationTiming) -> &Self {
        self.inner.viewport.scroll_to_animated(x, y, timing);
        self
    }

    pub fn set_runtime_scroll_offset(&self, x: f32, y: f32) {
        self.inner.viewport.set_runtime_scroll_offset(x, y);
    }

    pub fn vertical_scrollbar_visibility(&self, mode: ScrollBarVisibility) -> &Self {
        self.inner.vertical_visibility.set(mode);
        self.refresh_chrome();
        self
    }

    pub fn horizontal_scrollbar_visibility(&self, mode: ScrollBarVisibility) -> &Self {
        self.inner.horizontal_visibility.set(mode);
        self.refresh_chrome();
        self
    }

    pub fn scrollbar_gutter(&self, value: f32) -> &Self {
        self.inner.scrollbar_gutter.set(value.max(0.0));
        self.refresh_chrome();
        self
    }

    fn attach_listeners(&self) {
        let weak = Rc::downgrade(&self.inner);
        self.inner
            .vertical_scrollbar
            .set_chrome_metric_changed_handler(move || {
                if let Some(inner) = Weak::upgrade(&weak) {
                    ScrollBox { inner }.refresh_chrome();
                }
            });
        let weak = Rc::downgrade(&self.inner);
        self.inner
            .horizontal_scrollbar
            .set_chrome_metric_changed_handler(move || {
                if let Some(inner) = Weak::upgrade(&weak) {
                    ScrollBox { inner }.refresh_chrome();
                }
            });

        let weak = Rc::downgrade(&self.inner);
        self.inner
            .subscriptions
            .borrow_mut()
            .push(self.inner.scroll_state.subscribe_offset_x(move || {
                if let Some(inner) = Weak::upgrade(&weak) {
                    let scroll_box = ScrollBox { inner };
                    scroll_box.store_persisted_scroll_offset_if_needed();
                    scroll_box.try_restore_persisted_scroll_offset();
                }
            }));
        let weak = Rc::downgrade(&self.inner);
        self.inner
            .subscriptions
            .borrow_mut()
            .push(self.inner.scroll_state.subscribe_offset_y(move || {
                if let Some(inner) = Weak::upgrade(&weak) {
                    let scroll_box = ScrollBox { inner };
                    scroll_box.store_persisted_scroll_offset_if_needed();
                    scroll_box.try_restore_persisted_scroll_offset();
                }
            }));
        let weak = Rc::downgrade(&self.inner);
        self.inner.subscriptions.borrow_mut().push(
            self.inner.scroll_state.subscribe_content_width(move || {
                if let Some(inner) = Weak::upgrade(&weak) {
                    let scroll_box = ScrollBox { inner };
                    scroll_box.refresh_chrome();
                    scroll_box.try_restore_persisted_scroll_offset();
                }
            }),
        );
        let weak = Rc::downgrade(&self.inner);
        self.inner.subscriptions.borrow_mut().push(
            self.inner.scroll_state.subscribe_content_height(move || {
                if let Some(inner) = Weak::upgrade(&weak) {
                    let scroll_box = ScrollBox { inner };
                    scroll_box.refresh_chrome();
                    scroll_box.try_restore_persisted_scroll_offset();
                }
            }),
        );
        let weak = Rc::downgrade(&self.inner);
        self.inner.subscriptions.borrow_mut().push(
            self.inner.scroll_state.subscribe_viewport_width(move || {
                if let Some(inner) = Weak::upgrade(&weak) {
                    let scroll_box = ScrollBox { inner };
                    scroll_box.refresh_chrome();
                    scroll_box.bind_scroll_chrome();
                    scroll_box.try_restore_persisted_scroll_offset();
                }
            }),
        );
        let weak = Rc::downgrade(&self.inner);
        self.inner.subscriptions.borrow_mut().push(
            self.inner.scroll_state.subscribe_viewport_height(move || {
                if let Some(inner) = Weak::upgrade(&weak) {
                    let scroll_box = ScrollBox { inner };
                    scroll_box.refresh_chrome();
                    scroll_box.bind_scroll_chrome();
                    scroll_box.try_restore_persisted_scroll_offset();
                }
            }),
        );
    }

    fn bind_scroll_chrome(&self) {
        let viewport_handle = self.inner.viewport.handle().raw();
        self.inner
            .vertical_scrollbar
            .bind_scroll_handle(viewport_handle);
        self.inner
            .horizontal_scrollbar
            .bind_scroll_handle(viewport_handle);
        self.inner
            .root
            .bind_scroll_proxy_target_handle(viewport_handle);
        self.inner
            .top_row
            .bind_scroll_proxy_target_handle(viewport_handle);
        self.inner
            .bottom_row
            .bind_scroll_proxy_target_handle(viewport_handle);
        self.inner
            .vertical_gutter
            .bind_scroll_proxy_target_handle(viewport_handle);
        self.inner
            .corner
            .bind_scroll_proxy_target_handle(viewport_handle);
        self.bind_content_scroll_proxy_targets();
    }

    fn bind_content_scroll_proxy_targets(&self) {
        let viewport_handle = self.inner.viewport.handle().raw();
        let children = self.inner.viewport.retained_node_ref().children();
        for child in children {
            child.bind_scroll_proxy_target_handle(viewport_handle);
        }
    }

    fn refresh_chrome(&self) {
        let vertical_rail_thickness =
            self.inner.scrollbar_gutter.get() + self.inner.vertical_scrollbar.thickness();
        let horizontal_rail_thickness = self.inner.horizontal_scrollbar.thickness();
        let show_vertical = self.should_show(
            self.inner.vertical_visibility.get(),
            self.inner.vertical_scroll_enabled.get(),
            self.inner.scroll_state.content_height(),
            self.inner.scroll_state.viewport_height(),
        );
        let show_horizontal = self.should_show(
            self.inner.horizontal_visibility.get(),
            self.inner.horizontal_scroll_enabled.get(),
            self.inner.scroll_state.content_width(),
            self.inner.scroll_state.viewport_width(),
        );
        self.inner.vertical_scrollbar.chrome_visible(show_vertical);
        self.inner
            .horizontal_scrollbar
            .chrome_visible(show_horizontal);

        self.inner.vertical_gutter.width(
            if show_vertical {
                self.inner.scrollbar_gutter.get()
            } else {
                0.0
            },
            Unit::Pixel,
        );
        self.inner.vertical_scrollbar.render().width(
            if show_vertical {
                self.inner.vertical_scrollbar.thickness()
            } else {
                0.0
            },
            Unit::Pixel,
        );
        let horizontal_rail_height = if show_horizontal {
            horizontal_rail_thickness
        } else {
            0.0
        };
        self.inner
            .bottom_row
            .height(horizontal_rail_height, Unit::Pixel);
        self.inner.horizontal_scrollbar.render().height(
            if show_horizontal {
                self.inner.horizontal_scrollbar.thickness()
            } else {
                0.0
            },
            Unit::Pixel,
        );
        let vertical_rail_width = if show_vertical {
            vertical_rail_thickness
        } else {
            0.0
        };
        self.inner.corner.width(vertical_rail_width, Unit::Pixel);
        self.inner
            .corner
            .height(horizontal_rail_height, Unit::Pixel);
    }

    fn should_show(
        &self,
        mode: ScrollBarVisibility,
        enabled: bool,
        content_size: f32,
        viewport_size: f32,
    ) -> bool {
        if !enabled || mode == ScrollBarVisibility::Never {
            return false;
        }
        if mode == ScrollBarVisibility::Always {
            return true;
        }
        if viewport_size <= 0.0 || content_size <= 0.0 {
            return false;
        }
        content_size > viewport_size + OVERFLOW_TOLERANCE
    }

    fn store_persisted_scroll_offset_if_needed(&self) {
        if !self.inner.persist_scroll.get() {
            return;
        }
        let Some(node_id) = self.inner.root.retained_node_ref().node_id() else {
            return;
        };
        if node_id.is_empty() {
            return;
        }
        store_scroll_offset(
            &node_id,
            self.inner.scroll_state.offset_x(),
            self.inner.scroll_state.offset_y(),
        );
    }

    fn try_restore_persisted_scroll_offset(&self) -> bool {
        if !self.inner.persisted_restore_pending.get() || !self.inner.persist_scroll.get() {
            return false;
        }
        let Some(node_id) = self.inner.root.retained_node_ref().node_id() else {
            self.inner.persisted_restore_pending.set(false);
            return false;
        };
        if node_id.is_empty() {
            self.inner.persisted_restore_pending.set(false);
            return false;
        }
        let Some(restored) = try_load_scroll_offset(&node_id) else {
            self.inner.persisted_restore_pending.set(false);
            return false;
        };
        let max_x = (self.inner.scroll_state.content_width()
            - self.inner.scroll_state.viewport_width())
        .max(0.0);
        let max_y = (self.inner.scroll_state.content_height()
            - self.inner.scroll_state.viewport_height())
        .max(0.0);
        let can_restore_x = !self.inner.horizontal_scroll_enabled.get()
            || self.inner.scroll_state.viewport_width() > 0.0;
        let can_restore_y = !self.inner.vertical_scroll_enabled.get()
            || self.inner.scroll_state.viewport_height() > 0.0;
        if !can_restore_x || !can_restore_y {
            return false;
        }
        self.inner.persisted_restore_pending.set(false);
        self.set_runtime_scroll_offset(
            clamp(restored.x, 0.0, max_x),
            clamp(restored.y, 0.0, max_y),
        );
        true
    }
}

impl Default for ScrollBox {
    fn default() -> Self {
        Self::new()
    }
}

impl Node for ScrollBox {
    fn retained_node_ref(&self) -> NodeRef {
        let scroll_box = self.clone();
        self.inner
            .root
            .retained_node_ref()
            .with_build_callback(move || scroll_box.build())
    }

    fn build(&self) {
        self.ensure_handle();
        self.inner.root.build_self();
        self.build_children();
        self.inner
            .persisted_restore_pending
            .set(self.inner.persist_scroll.get());
        self.bind_scroll_chrome();
        self.refresh_chrome();
        let _ = self.try_restore_persisted_scroll_offset();
    }

    fn build_self(&self) {
        self.inner.root.build_self();
    }
}

impl HasFlexBoxRoot for ScrollBox {
    fn flex_box_root(&self) -> &FlexBox {
        &self.inner.root
    }
}

impl ThemeBindable for ScrollBox {
    fn theme_binding_node(&self) -> NodeRef {
        self.inner.root.retained_node_ref()
    }

    fn weak_theme_target(&self) -> Box<dyn Fn() -> Option<Self>> {
        let weak = Rc::downgrade(&self.inner);
        Box::new(move || {
            Some(ScrollBox {
                inner: weak.upgrade()?,
            })
        })
    }
}
