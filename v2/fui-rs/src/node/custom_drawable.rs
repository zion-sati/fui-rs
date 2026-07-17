use super::core::*;
use super::*;

#[derive(Clone)]
pub struct CustomDrawable {
    base: FlexBox,
    draw_callback: DrawCallback,
}

#[derive(Clone)]
pub struct DrawableInvalidator {
    base: WeakFlexBox,
}

impl DrawableInvalidator {
    pub fn mark_dirty(&self) {
        if let Some(base) = self.base.upgrade() {
            mark_base_dirty(&base);
        }
    }
}

impl CustomDrawable {
    pub fn new(handler: impl Fn(&mut DrawContext) + 'static) -> Self {
        let base = FlexBox::default();
        base.custom_drawable(true);
        Self {
            base,
            draw_callback: Rc::new(handler),
        }
    }

    pub fn mark_dirty(&self) {
        mark_base_dirty(&self.base);
    }

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
}
