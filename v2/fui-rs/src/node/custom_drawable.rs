use super::core::*;
use super::*;

#[derive(Clone)]
pub struct CustomDrawable {
    base: FlexBox,
    draw_callback: DrawCallback,
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

    pub fn width(&self, width: f32, unit: Unit) -> &Self {
        self.base.width(width, unit);
        self
    }

    pub fn width_len(&self, length: Length) -> &Self {
        self.base.width_len(length);
        self
    }

    pub fn height(&self, height: f32, unit: Unit) -> &Self {
        self.base.height(height, unit);
        self
    }

    pub fn height_len(&self, length: Length) -> &Self {
        self.base.height_len(length);
        self
    }

    pub fn bg_color(&self, color: u32) -> &Self {
        self.base.bg_color(color);
        self
    }

    pub fn corner_radius(&self, radius: f32) -> &Self {
        self.base.corner_radius(radius);
        self
    }

    pub fn border(&self, width: f32, color: u32) -> &Self {
        self.base.border(width, color);
        self
    }

    pub fn border_config(&self, border: Border) -> &Self {
        self.base.border_config(border);
        self
    }

    pub fn opacity(&self, value: f32) -> &Self {
        self.base.opacity(value);
        self
    }

    pub fn drop_shadow(
        &self,
        color: u32,
        offset_x: f32,
        offset_y: f32,
        blur_sigma: f32,
        spread: f32,
    ) -> &Self {
        self.base
            .drop_shadow(color, offset_x, offset_y, blur_sigma, spread);
        self
    }

    pub fn linear_gradient(
        &self,
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        offsets: Vec<f32>,
        colors: Vec<u32>,
    ) -> &Self {
        self.base.linear_gradient(sx, sy, ex, ey, offsets, colors);
        self
    }

    pub fn child<T: Node>(&self, child: &T) -> &Self {
        self.base.child(child);
        self
    }

    pub fn mark_dirty(&self) {
        let handle = self.handle();
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
