mod bar_chart;
mod dancing_yarn;
mod gauge;
mod paint_canvas;
mod pie_chart;
mod scatter_plot;
mod shared;
mod sparkline;
mod waveform;

use bar_chart::BarChart;
use dancing_yarn::DancingYarn;
use fui::prelude::*;
use gauge::Gauge;
use paint_canvas::PaintCanvas;
use pie_chart::PieChart;
use scatter_plot::ScatterPlot;
use sparkline::Sparkline;
use std::cell::Cell;
use std::rc::{Rc, Weak};
use waveform::Waveform;

#[derive(Clone)]
pub struct DrawingGallery {
    root: FlexBox,
    _state: Rc<GalleryState>,
}

fui_component!(DrawingGallery => root, owner: _state);

impl DrawingGallery {
    pub fn new() -> Self {
        let theme = current_theme();
        let gauge = Gauge::new(&theme);
        let bar_chart = BarChart::new(&theme);
        let waveform = Waveform::new(&theme);
        let sparkline = Sparkline::new(&theme);
        let pie_chart = PieChart::new(&theme);
        let scatter_plot = ScatterPlot::new(&theme);
        let dancing_yarn = DancingYarn::new(&theme);
        let paint_canvas = PaintCanvas::new(&theme);
        let root = ui! {
            row().fill_width().flex_wrap(FlexWrap::Wrap) {
                gauge.node().clone(),
                flex_box().width(16.0, Unit::Pixel).height(1.0, Unit::Pixel),
                bar_chart.node().clone(),
                flex_box().width(16.0, Unit::Pixel).height(1.0, Unit::Pixel),
                waveform.node().clone(),
                flex_box().width(16.0, Unit::Pixel).height(1.0, Unit::Pixel),
                sparkline.node().clone(),
                flex_box().width(16.0, Unit::Pixel).height(1.0, Unit::Pixel),
                pie_chart.node().clone(),
                flex_box().width(16.0, Unit::Pixel).height(1.0, Unit::Pixel),
                scatter_plot.node().clone(),
                flex_box().width(16.0, Unit::Pixel).height(1.0, Unit::Pixel),
                dancing_yarn.node().clone(),
                flex_box().width(16.0, Unit::Pixel).height(1.0, Unit::Pixel),
                paint_canvas.node().clone(),
            }
        };
        let state = Rc::new(GalleryState {
            value: Cell::new(0.0),
            direction: Cell::new(1.0),
            gauge,
            bar_chart,
            waveform,
            sparkline,
            pie_chart,
            scatter_plot,
            dancing_yarn,
            _paint_canvas: paint_canvas,
        });
        let weak = Rc::downgrade(&state);
        on_loaded(move |_| schedule_tick(weak));
        Self {
            root,
            _state: state,
        }
    }
}

struct GalleryState {
    value: Cell<f32>,
    direction: Cell<f32>,
    gauge: Gauge,
    bar_chart: BarChart,
    waveform: Waveform,
    sparkline: Sparkline,
    pie_chart: PieChart,
    scatter_plot: ScatterPlot,
    dancing_yarn: DancingYarn,
    _paint_canvas: PaintCanvas,
}

impl GalleryState {
    fn tick(&self) {
        let mut value = self.value.get() + self.direction.get() * 2.0;
        if value >= 100.0 {
            value = 100.0;
            self.direction.set(-1.0);
        } else if value <= 0.0 {
            value = 0.0;
            self.direction.set(1.0);
        }
        self.value.set(value);
        self.gauge.set_value(value);
        self.bar_chart.push_values(
            value,
            (value - 50.0).abs() * 2.0,
            (value / 100.0 * std::f32::consts::PI).sin() * 80.0 + 20.0,
            (value / 100.0 * std::f32::consts::PI * 0.7).cos() * 60.0 + 40.0,
        );
        self.waveform.push_values(
            value,
            (value - 50.0).abs() * 2.0,
            (value / 100.0 * std::f32::consts::PI).sin() * 80.0 + 20.0,
            (value / 100.0 * std::f32::consts::PI * 0.7).cos() * 60.0 + 40.0,
        );
        self.sparkline.push(value);
        self.pie_chart.push_values(
            value,
            (value - 50.0).abs() * 2.0,
            (value / 100.0 * std::f32::consts::PI).sin() * 40.0 + 30.0,
            (value / 100.0 * std::f32::consts::PI * 0.7).cos() * 30.0 + 20.0,
        );
        self.scatter_plot.push_values(
            (value / 100.0 * std::f32::consts::PI * 2.0).sin(),
            (value / 100.0 * std::f32::consts::PI * 2.0).cos(),
            (value / 100.0 * std::f32::consts::PI * 3.0).sin(),
            (value / 100.0 * std::f32::consts::PI * 1.5).sin(),
        );
        self.dancing_yarn.tick();
    }
}

fn schedule_tick(state: Weak<GalleryState>) {
    set_timeout(25, move || {
        let Some(state) = state.upgrade() else {
            return;
        };
        state.tick();
        schedule_tick(Rc::downgrade(&state));
    });
}
