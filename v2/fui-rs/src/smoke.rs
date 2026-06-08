use crate::app::Application;
use crate::component::Component;
use crate::ffi::Unit;
use crate::node::{column, flex_box, row, text, Node};

const FONT_REGULAR: u32 = 1;
const PANEL_TEXT: u32 = 0xE2E8F0FF;
const SPACING: f32 = 32.0;

struct SmokeApp;

impl Component for SmokeApp {
    fn render(&self) -> Box<dyn Node> {
        Box::new(
            column()
                .padding(24.0, 24.0, 24.0, 24.0)
                .child(
                    row()
                        .child(
                            text("left")
                                .font(FONT_REGULAR, 28.0)
                                .text_color(PANEL_TEXT),
                        )
                        .child(
                            flex_box()
                                .width(SPACING, Unit::Pixel)
                                .height(1.0, Unit::Pixel),
                        )
                        .child(
                            text("right")
                                .font(FONT_REGULAR, 28.0)
                                .text_color(PANEL_TEXT),
                        ),
                )
                .child(flex_box().height(24.0, Unit::Pixel))
                .child(
                    flex_box()
                        .width(120.0, Unit::Pixel)
                        .height(96.0, Unit::Pixel)
                        .bg_color(0x006CFFFF),
                ),
        )
    }
}

#[no_mangle]
pub extern "C" fn __runSmokeApp() {
    Application::run(|| SmokeApp);
}

#[no_mangle]
pub extern "C" fn __flushRenders() {
    Application::flush_renders();
}
