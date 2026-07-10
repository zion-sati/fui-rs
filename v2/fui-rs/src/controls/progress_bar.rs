use super::*;
use crate::logger;
use crate::node::WeakFlexBox;
use crate::signal::SubscriptionGuard;

#[derive(Clone)]
pub struct ProgressBar {
    root: FlexBox,
    fill: FlexBox,
    min: Rc<Cell<f32>>,
    max: Rc<Cell<f32>>,
    value: Rc<Cell<f32>>,
    length: Rc<Cell<f32>>,
    thickness: Rc<Cell<f32>>,
    track_color_value: Rc<Cell<u32>>,
    fill_color_value: Rc<Cell<u32>>,
    track_color_overridden: Rc<Cell<bool>>,
    fill_color_overridden: Rc<Cell<bool>>,
    corner_radius_overridden: Rc<Cell<bool>>,
    weak_root: WeakFlexBox,
    weak_fill: WeakFlexBox,
    theme_guard: Rc<RefCell<Option<SubscriptionGuard>>>,
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressBar {
    pub fn new() -> Self {
        let root = flex_box();
        let fill = flex_box();
        root.semantic_value_range(0.0, 0.0, 100.0).child(&fill);
        let control = Self {
            weak_root: root.downgrade(),
            weak_fill: fill.downgrade(),
            root,
            fill,
            min: Rc::new(Cell::new(0.0)),
            max: Rc::new(Cell::new(100.0)),
            value: Rc::new(Cell::new(0.0)),
            length: Rc::new(Cell::new(PROGRESS_LENGTH)),
            thickness: Rc::new(Cell::new(PROGRESS_THICKNESS)),
            track_color_value: Rc::new(Cell::new(0)),
            fill_color_value: Rc::new(Cell::new(0)),
            track_color_overridden: Rc::new(Cell::new(false)),
            fill_color_overridden: Rc::new(Cell::new(false)),
            corner_radius_overridden: Rc::new(Cell::new(false)),
            theme_guard: Rc::new(RefCell::new(None)),
        };
        control.install_visual_subscriptions();
        control.set_value(0.0);
        control.handle_theme_changed();
        control
    }

    pub fn min(&self, value: f32) -> &Self {
        self.min.set(value);
        if self.max.get() < value {
            self.max.set(value);
        }
        self.set_value(self.value.get());
        self
    }

    pub fn max(&self, value: f32) -> &Self {
        self.max.set(value);
        if self.min.get() > value {
            self.min.set(value);
        }
        self.set_value(self.value.get());
        self
    }

    pub fn value(&self, value: f32) -> &Self {
        self.set_value(value);
        self
    }

    pub fn length(&self, value: f32) -> &Self {
        if value <= 0.0 {
            logger::warn(
                "Layout",
                &format!("ProgressBar.length() received {value}; clamping to 1.0."),
            );
        }
        self.length.set(if value > 0.0 { value } else { 1.0 });
        self.sync_geometry();
        self
    }

    pub fn thickness(&self, value: f32) -> &Self {
        if value <= 0.0 {
            logger::warn(
                "Layout",
                &format!("ProgressBar.thickness() received {value}; clamping to 1.0."),
            );
        }
        self.thickness.set(if value > 0.0 { value } else { 1.0 });
        if !self.corner_radius_overridden.get() {
            let radius = self.thickness.get() * 0.5;
            self.root.corner_radius(radius);
        }
        self.sync_geometry();
        self.sync_visual_state();
        self
    }

    pub fn track_color(&self, color: u32) -> &Self {
        self.track_color_overridden.set(true);
        self.track_color_value.set(color);
        self.sync_visual_state();
        self
    }

    pub fn fill_color(&self, color: u32) -> &Self {
        self.fill_color_overridden.set(true);
        self.fill_color_value.set(color);
        self.sync_visual_state();
        self
    }

    pub fn corner_radius(&self, radius: f32) -> &Self {
        self.corner_radius_overridden.set(true);
        self.root.corner_radius(radius);
        self.fill.corner_radius(radius);
        self
    }

    pub fn current_value(&self) -> f32 {
        self.value.get()
    }

    fn install_visual_subscriptions(&self) {
        let weak_root = self.weak_root.clone();
        let weak_fill = self.weak_fill.clone();
        let min = self.min.clone();
        let max = self.max.clone();
        let value = self.value.clone();
        let length = self.length.clone();
        let thickness = self.thickness.clone();
        let track_color_value = self.track_color_value.clone();
        let fill_color_value = self.fill_color_value.clone();
        let track_color_overridden = self.track_color_overridden.clone();
        let fill_color_overridden = self.fill_color_overridden.clone();
        let corner_radius_overridden = self.corner_radius_overridden.clone();
        *self.theme_guard.borrow_mut() = Some(subscribe(move |_theme| {
            let Some(root) = weak_root.upgrade() else {
                return;
            };
            let Some(fill) = weak_fill.upgrade() else {
                return;
            };
            sync_progress_visual_state(
                &root,
                &fill,
                thickness.get(),
                track_color_value.get(),
                fill_color_value.get(),
                track_color_overridden.get(),
                fill_color_overridden.get(),
                corner_radius_overridden.get(),
            );
            sync_progress_geometry(
                &root,
                &fill,
                min.get(),
                max.get(),
                value.get(),
                length.get(),
                thickness.get(),
            );
        }));
    }

    fn handle_theme_changed(&self) {
        self.sync_visual_state();
        self.sync_geometry();
    }

    fn set_value(&self, value: f32) {
        self.value.set(value.clamp(self.min.get(), self.max.get()));
        self.sync_geometry();
    }

    fn sync_geometry(&self) {
        sync_progress_geometry(
            &self.root,
            &self.fill,
            self.min.get(),
            self.max.get(),
            self.value.get(),
            self.length.get(),
            self.thickness.get(),
        );
    }

    fn sync_visual_state(&self) {
        sync_progress_visual_state(
            &self.root,
            &self.fill,
            self.thickness.get(),
            self.track_color_value.get(),
            self.fill_color_value.get(),
            self.track_color_overridden.get(),
            self.fill_color_overridden.get(),
            self.corner_radius_overridden.get(),
        );
    }
}

impl Node for ProgressBar {
    fn retained_node_ref(&self) -> NodeRef {
        self.root.retained_node_ref()
    }

    fn build_self(&self) {
        self.sync_visual_state();
        self.sync_geometry();
        self.root.build_self();
    }
}

impl HasFlexBoxRoot for ProgressBar {
    fn flex_box_root(&self) -> &FlexBox {
        &self.root
    }
}

fn sync_progress_geometry(
    root: &FlexBox,
    fill: &FlexBox,
    min: f32,
    max: f32,
    current_value: f32,
    length: f32,
    thickness: f32,
) {
    let range = max - min;
    let fraction = if range > 0.0 {
        ((current_value - min) / range).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let fill_length = length * fraction;
    root.width(length, Unit::Pixel)
        .height(thickness, Unit::Pixel)
        .semantic_value_range(current_value, min, max)
        .default_semantic_label(format!(
            "Progress bar, value {}, range {} to {}",
            current_value, min, max
        ));
    fill.width(fill_length, Unit::Pixel)
        .height(thickness, Unit::Pixel);
}

fn sync_progress_visual_state(
    root: &FlexBox,
    fill: &FlexBox,
    thickness: f32,
    track_color_value: u32,
    fill_color_value: u32,
    track_color_overridden: bool,
    fill_color_overridden: bool,
    corner_radius_overridden: bool,
) {
    let theme = current_theme();
    let track_color = if track_color_overridden {
        track_color_value
    } else {
        theme.colors.scrollbar_track
    };
    let fill_color = if fill_color_overridden {
        fill_color_value
    } else {
        theme.colors.accent
    };
    if !corner_radius_overridden {
        let radius = thickness * 0.5;
        root.corner_radius(radius);
        fill.corner_radius(radius);
    }
    root.bg_color(track_color).border(1.0, theme.colors.border);
    fill.bg_color(fill_color);
}
