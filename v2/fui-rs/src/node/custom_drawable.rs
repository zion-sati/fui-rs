use super::core::*;
use super::*;

#[derive(Clone)]
/// A retained visual surface whose callback records immediate drawing commands.
///
/// The same callback API is supported by browser/WebAssembly and native desktop
/// hosts. Keep expensive drawing resources outside the callback and invalidate
/// the drawable after retained state changes.
///
/// ```no_run
/// use fui::prelude::*;
///
/// let preview = custom_drawable(|context| {
///     context.draw_circle(32.0, 32.0, 20.0, Paint::fill(rgb(0x38, 0xbd, 0xf8)));
/// });
/// preview
///     .width(64.0, Unit::Pixel)
///     .height(64.0, Unit::Pixel)
///     .semantic_label("Drawing preview");
/// preview.mark_dirty();
/// ```
pub struct CustomDrawable {
    base: FlexBox,
    draw_callback: DrawCallback,
}

#[derive(Clone)]
/// A weak invalidation handle that does not keep its [`CustomDrawable`] alive.
pub struct DrawableInvalidator {
    base: WeakFlexBox,
}

impl DrawableInvalidator {
    /// Schedules a redraw if the associated drawable is still mounted.
    pub fn mark_dirty(&self) {
        if let Some(base) = self.base.upgrade() {
            mark_base_dirty(&base);
        }
    }
}

impl CustomDrawable {
    /// Creates a retained custom-drawing surface.
    ///
    /// The runtime saves canvas state, clips to the drawable bounds, invokes
    /// `handler`, restores state, and flushes the command batch.
    pub fn new(handler: impl Fn(&mut DrawContext) + 'static) -> Self {
        let base = FlexBox::default();
        base.custom_drawable(true);
        Self {
            base,
            draw_callback: Rc::new(handler),
        }
    }

    /// Schedules a commit when visible drawing state has changed.
    pub fn mark_dirty(&self) {
        mark_base_dirty(&self.base);
    }

    /// Returns a weak invalidator suitable for timers and readiness callbacks.
    pub fn invalidator(&self) -> DrawableInvalidator {
        DrawableInvalidator {
            base: self.base.downgrade(),
        }
    }
}

fn mark_base_dirty(base: &FlexBox) {
    let handle = base.handle();
    if handle != NodeHandle::INVALID {
        let Some(bounds) = ui::get_visible_bounds(handle.raw()) else {
            return;
        };
        if bounds[2] <= 0.0 || bounds[3] <= 0.0 {
            return;
        }
    }
    crate::frame_scheduler::mark_needs_commit();
}

impl Node for CustomDrawable {
    fn retained_node_ref(&self) -> NodeRef {
        NodeRef::from_node(self.base.core.clone(), self.clone())
    }

    fn build_self(&self) {
        self.base.build_self();
        let weak_base = self.base.downgrade();
        let draw_callback = self.draw_callback.clone();
        self.base.core.borrow_mut().draw_callback = Some(Rc::new(move |ctx| {
            let Some(base) = weak_base.upgrade() else {
                return;
            };
            let bounds = base.get_bounds();
            let (tl, tr, br, bl) = base
                .props
                .borrow()
                .box_style
                .map(|style| {
                    (
                        style.radius_tl,
                        style.radius_tr,
                        style.radius_br,
                        style.radius_bl,
                    )
                })
                .unwrap_or((0.0, 0.0, 0.0, 0.0));

            ctx.save();
            if tl > 0.0 || tr > 0.0 || br > 0.0 || bl > 0.0 {
                ctx.clip_round_rect(0.0, 0.0, bounds[2], bounds[3], tl, tr, br, bl);
            } else {
                ctx.clip_rect(0.0, 0.0, bounds[2], bounds[3]);
            }
            draw_callback(ctx);
            ctx.restore();
            ctx.flush();
        }));
    }
}

impl HasFlexBoxRoot for CustomDrawable {
    fn flex_box_root(&self) -> &FlexBox {
        &self.base
    }
}

impl ThemeBindable for CustomDrawable {
    fn theme_binding_node(&self) -> NodeRef {
        self.base.retained_node_ref()
    }

    fn weak_theme_target(&self) -> Box<dyn Fn() -> Option<Self>> {
        let weak_base = self.base.downgrade();
        let draw_callback = self.draw_callback.clone();
        Box::new(move || {
            weak_base.upgrade().map(|base| Self {
                base,
                draw_callback: draw_callback.clone(),
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Application;
    use crate::ffi::{self, Call};

    fn assert_flex_box_surface<T: FlexBoxSurface>() {}
    fn assert_theme_bindable<T: ThemeBindable>() {}

    #[test]
    fn custom_drawable_exposes_generic_retained_visual_surfaces() {
        assert_flex_box_surface::<CustomDrawable>();
        assert_theme_bindable::<CustomDrawable>();

        let drawable = CustomDrawable::new(|_| {});
        drawable
            .width(300.0, Unit::Pixel)
            .height(200.0, Unit::Pixel)
            .min_width(120.0, Unit::Pixel)
            .margin(1.0, 2.0, 3.0, 4.0)
            .padding(5.0, 6.0, 7.0, 8.0)
            .corner_radius(12.0)
            .bg_color(0x112233FF)
            .clip_to_bounds(true);

        let invalidator = drawable.invalidator();
        drop(drawable);
        invalidator.mark_dirty();
    }

    #[test]
    fn custom_drawable_requests_focus_through_node_trait() {
        ffi::test::reset();
        let drawable = CustomDrawable::new(|_| {});
        drawable.focusable(true, 0);
        Application::mount(drawable.clone());
        let handle = drawable.handle().raw();
        ffi::test::take_calls();

        drawable.focus_now();

        let calls = ffi::test::take_calls();
        assert!(calls.iter().any(
            |call| matches!(call, Call::RequestFocus { handle: requested } if *requested == handle)
        ));
        Application::unmount();
    }
}
