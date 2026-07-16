use super::*;
use crate::node::WeakFlexBox;
use crate::Border;

#[derive(Clone, Copy)]
struct ResolvedProgressBarSizing {
    length: f32,
    thickness: f32,
}

fn resolve_sizing(sizing: Option<ProgressBarSizing>) -> ResolvedProgressBarSizing {
    let sizing = sizing.unwrap_or_default();
    ResolvedProgressBarSizing {
        length: if sizing.has_length() {
            sizing.length_px()
        } else {
            PROGRESS_LENGTH
        },
        thickness: if sizing.has_thickness() {
            sizing.thickness_px()
        } else {
            PROGRESS_THICKNESS
        },
    }
}

#[derive(Clone)]
pub struct ProgressBar {
    root: FlexBox,
    fill: FlexBox,
    min: Rc<Cell<f32>>,
    max: Rc<Cell<f32>>,
    value: Rc<Cell<f32>>,
    sizing_value: Rc<Cell<Option<ProgressBarSizing>>>,
    colors_value: Rc<Cell<Option<ProgressBarColors>>>,
    weak_root: WeakFlexBox,
    weak_fill: WeakFlexBox,
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
            sizing_value: Rc::new(Cell::new(None)),
            colors_value: Rc::new(Cell::new(None)),
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

    pub fn sizing(&self, sizing: ProgressBarSizing) -> &Self {
        self.sizing_value.set(Some(sizing));
        self.sync_geometry();
        self.sync_visual_state();
        self
    }

    pub fn clear_sizing(&self) -> &Self {
        self.sizing_value.set(None);
        self.sync_geometry();
        self.sync_visual_state();
        self
    }

    pub fn colors(&self, colors: ProgressBarColors) -> &Self {
        self.colors_value.set(Some(colors));
        self.sync_visual_state();
        self
    }

    pub fn clear_colors(&self) -> &Self {
        self.colors_value.set(None);
        self.sync_visual_state();
        self
    }

    pub fn corner_radius(&self, radius: f32) -> &Self {
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
        let sizing_value = self.sizing_value.clone();
        let colors_value = self.colors_value.clone();
        let guard = subscribe(move |_theme| {
            let Some(root) = weak_root.upgrade() else {
                return;
            };
            let Some(fill) = weak_fill.upgrade() else {
                return;
            };
            sync_progress_visual_state(
                &root,
                &fill,
                resolve_sizing(sizing_value.get()),
                colors_value.get(),
            );
        });
        self.root
            .retained_node_ref()
            .retain_attachment(Rc::new(guard));
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
            resolve_sizing(self.sizing_value.get()),
        );
    }

    fn sync_visual_state(&self) {
        sync_progress_visual_state(
            &self.root,
            &self.fill,
            resolve_sizing(self.sizing_value.get()),
            self.colors_value.get(),
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
    sizing: ResolvedProgressBarSizing,
) {
    let range = max - min;
    let fraction = if range > 0.0 {
        ((current_value - min) / range).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let fill_length = sizing.length * fraction;
    root.width(sizing.length, Unit::Pixel)
        .height(sizing.thickness, Unit::Pixel)
        .semantic_value_range(current_value, min, max)
        .default_semantic_label(format!(
            "Progress bar, value {}, range {} to {}",
            current_value, min, max
        ));
    fill.width(fill_length, Unit::Pixel)
        .height(sizing.thickness, Unit::Pixel);
}

fn sync_progress_visual_state(
    root: &FlexBox,
    fill: &FlexBox,
    sizing: ResolvedProgressBarSizing,
    colors: Option<ProgressBarColors>,
) {
    let theme = current_theme();
    let track_color = colors
        .filter(ProgressBarColors::has_track)
        .map(|colors| colors.track_color())
        .unwrap_or(theme.colors.scrollbar_track);
    let fill_color = colors
        .filter(ProgressBarColors::has_fill)
        .map(|colors| colors.fill_color())
        .unwrap_or(theme.colors.accent);
    root.apply_presenter_style(
        crate::PresenterHostStyle::new()
            .background(track_color)
            .border(Border::solid(1.0, theme.colors.border))
            .corners(crate::Corners::all(sizing.thickness * 0.5)),
    );
    let corners = root
        .resolved_host_style()
        .corners
        .unwrap_or_else(|| crate::Corners::all(sizing.thickness * 0.5));
    fill.corners(
        corners.top_left,
        corners.top_right,
        corners.bottom_right,
        corners.bottom_left,
    )
    .bg_color(fill_color);
}
