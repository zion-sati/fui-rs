use crate::controls::DropdownSizing;
use crate::ffi::{AlignItems, JustifyContent, Unit};
use crate::node::{flex_box, svg, FlexBox, SvgNode};
use crate::theme::Theme;
use std::rc::Rc;

const DROPDOWN_CHEVRON_COLLAPSED_SVG: &str = "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'><path d='M3 4.5 6 7.5 9 4.5' fill='none' stroke='%23000000' stroke-width='1.6' stroke-linecap='round' stroke-linejoin='round'/></svg>";
const DROPDOWN_CHEVRON_EXPANDED_SVG: &str = "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'><path d='M3 7.5 6 4.5 9 7.5' fill='none' stroke='%23000000' stroke-width='1.6' stroke-linecap='round' stroke-linejoin='round'/></svg>";
const DEFAULT_DROPDOWN_CHEVRON_ICON_SIZE: f32 = 12.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DropdownChevronMetrics {
    pub icon_size: f32,
}

impl DropdownChevronMetrics {
    pub const fn new(icon_size: f32) -> Self {
        Self { icon_size }
    }
}

pub const DEFAULT_DROPDOWN_CHEVRON_METRICS: DropdownChevronMetrics =
    DropdownChevronMetrics::new(DEFAULT_DROPDOWN_CHEVRON_ICON_SIZE);

fn resolve_chevron_metrics(sizing: Option<DropdownSizing>) -> DropdownChevronMetrics {
    let Some(sizing) = sizing else {
        return DEFAULT_DROPDOWN_CHEVRON_METRICS;
    };
    if !sizing.has_chevron_icon_size() {
        return DEFAULT_DROPDOWN_CHEVRON_METRICS;
    }
    DropdownChevronMetrics::new(sizing.chevron_icon_size_px())
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DropdownChevronVisualState {
    pub open: bool,
    pub hovered: bool,
    pub enabled: bool,
}

impl DropdownChevronVisualState {
    pub const fn new(open: bool, hovered: bool, enabled: bool) -> Self {
        Self {
            open,
            hovered,
            enabled,
        }
    }
}

pub trait DropdownChevronPresenter {
    fn root(&self) -> FlexBox;
    fn apply(&self, theme: Theme, state: DropdownChevronVisualState);
}

pub trait DropdownChevronTemplate {
    fn create(&self, sizing: Option<DropdownSizing>) -> Rc<dyn DropdownChevronPresenter>;
}

#[derive(Clone)]
pub struct DefaultDropdownChevronPresenter {
    root: FlexBox,
    metrics: DropdownChevronMetrics,
    icon_node: SvgNode,
}

impl DefaultDropdownChevronPresenter {
    pub fn new(metrics: DropdownChevronMetrics) -> Self {
        let root = flex_box();
        root.fill_size()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center);
        let icon_node = svg(0);
        icon_node
            .width(metrics.icon_size, Unit::Pixel)
            .height(metrics.icon_size, Unit::Pixel);
        root.child(&icon_node);
        Self {
            root,
            metrics,
            icon_node: icon_node.clone(),
        }
    }
}

impl DropdownChevronPresenter for DefaultDropdownChevronPresenter {
    fn root(&self) -> FlexBox {
        self.root.clone()
    }

    fn apply(&self, theme: Theme, state: DropdownChevronVisualState) {
        let metrics = self.metrics;
        self.root
            .fill_size()
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center);
        self.icon_node
            .width(metrics.icon_size, Unit::Pixel)
            .height(metrics.icon_size, Unit::Pixel)
            .source(if state.open {
                DROPDOWN_CHEVRON_EXPANDED_SVG
            } else {
                DROPDOWN_CHEVRON_COLLAPSED_SVG
            })
            .tint(if !state.enabled {
                theme.colors.text_muted
            } else if state.hovered {
                theme.colors.text_primary
            } else {
                theme.colors.text_muted
            });
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DefaultDropdownChevronTemplate;

impl DropdownChevronTemplate for DefaultDropdownChevronTemplate {
    fn create(&self, sizing: Option<DropdownSizing>) -> Rc<dyn DropdownChevronPresenter> {
        create_default_dropdown_chevron_presenter(sizing)
    }
}

pub const DEFAULT_DROPDOWN_CHEVRON_TEMPLATE: DefaultDropdownChevronTemplate =
    DefaultDropdownChevronTemplate;

pub fn create_default_dropdown_chevron_presenter(
    sizing: Option<DropdownSizing>,
) -> Rc<dyn DropdownChevronPresenter> {
    Rc::new(DefaultDropdownChevronPresenter::new(
        resolve_chevron_metrics(sizing),
    ))
}
