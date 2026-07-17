mod external_drop_demo;
pub mod generated;
mod reorder_demo;

use external_drop_demo::ExternalDropDemoPanel;
use fui::prelude::*;
use fui::{
    current_route, device_pixel_ratio, get_svg_asset_error, get_svg_asset_height,
    get_svg_asset_state, get_svg_asset_width, get_texture_asset_error, get_texture_asset_height,
    get_texture_asset_state, get_texture_asset_state_value, get_texture_asset_width, load_svg,
    load_texture, on_loaded, AssetLoadState,
};
use reorder_demo::ReorderDemoPanel;
use std::cell::RefCell;
use std::rc::Rc;

const STAGE4_SAMPLE_TEXTURE_ID: u32 = 4101;
const STAGE4_SAMPLE_SVG_ID: u32 = 4102;
const STAGE4_SAMPLE_SVG_URL: &str = "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='320' height='180' viewBox='0 0 320 180'><rect width='320' height='180' rx='24' fill='%23DBEAFE'/><path d='M58 124 120 62l48 48 42-42 52 56' fill='none' stroke='%231D4ED8' stroke-width='18' stroke-linecap='round' stroke-linejoin='round'/></svg>";
const STAGE4_ROUTE_RELATIVE_TEXTURE_ID: u32 = 4103;
const STAGE4_MISSING_TEXTURE_ID: u32 = 4104;
const STAGE4_MISSING_SVG_ID: u32 = 4105;
const STAGE4_ROUTE_RELATIVE_TEXTURE_URL: &str = "./demo-texture.png";
const STAGE4_MISSING_TEXTURE_URL: &str = "./missing-stage4-texture.png";
const STAGE4_MISSING_SVG_URL: &str = "./missing-stage4-icon.svg";
const STAGE4_FONT_BASE_URL: &str = "/v2/fui-rs/fonts";
const STAGE4_DRAW_TICK_MS: i32 = 25;
const SOURCE_DEMO_BASE: &str = "/v2/fui-rs/demo";
const SOURCE_HOME_ROUTE: &str = "/v2/fui-rs/demo/index.html";
const SOURCE_WORKBENCH_ROUTE: &str = "/v2/fui-rs/demo/workbench/";
const SOURCE_STAGE4_ROUTE: &str = "/v2/fui-rs/demo/stage4/";
const SOURCE_STAGE5_ROUTE: &str = "/v2/fui-rs/demo/stage5/";
const SOURCE_IMMEDIATE_DRAWING_ROUTE: &str = "/v2/fui-rs/demo/immediate-drawing/";
const PUBLISHED_HOME_ROUTE: &str = "/";
const PUBLISHED_WORKBENCH_ROUTE: &str = "/workbench/";
const PUBLISHED_STAGE4_ROUTE: &str = "/stage4/";
const PUBLISHED_STAGE5_ROUTE: &str = "/stage5/";
const PUBLISHED_IMMEDIATE_DRAWING_ROUTE: &str = "/immediate-drawing/";
const JSON_PLACEHOLDER_GET_URL: &str = "https://jsonplaceholder.typicode.com/posts/1";
const JSON_PLACEHOLDER_POST_URL: &str = "https://jsonplaceholder.typicode.com/posts";
const JSON_PLACEHOLDER_POST_BODY: &str =
    "{\"title\":\"EffinDom FUI-RS workbench\",\"body\":\"Posting through the shipped Fetch API.\",\"userId\":29}";

pub fn clear_demo_shared_state() {}

fn demo_card_color(theme: &Theme, color: u32) -> u32 {
    if is_dark_mode() {
        match color {
            0xE0F2FEFF | 0xD7EAFEFF => 0x0B2538FF,
            0xDCFCE7FF => 0x0B2A1AFF,
            0xEDE9FEFF | 0xF3E8FFFF => 0x241437FF,
            0xFDE68AFF => 0x352607FF,
            0xFEF3C7FF => 0x33230AFF,
            0xFCE7F3FF => 0x341528FF,
            _ => theme.colors.surface,
        }
    } else {
        color
    }
}

fn demo_subtle_surface(_theme: &Theme) -> u32 {
    if is_dark_mode() {
        0x111C2CFF
    } else {
        0xF8FAFCFF
    }
}

fn demo_probe_surface(_theme: &Theme) -> u32 {
    if is_dark_mode() {
        0x0B2538FF
    } else {
        0xDBEAFEFF
    }
}

fn demo_text_color(theme: &Theme, color: u32) -> u32 {
    match color {
        0x111827FF | 0x1F2937FF | 0x0F172AFF => theme.colors.text_primary,
        0x334155FF | 0x374151FF | 0x475569FF | 0x64748BFF => theme.colors.text_muted,
        _ => color,
    }
}

fn asset_state_name(state: AssetLoadState) -> &'static str {
    match state {
        AssetLoadState::Idle => "Idle",
        AssetLoadState::Loading => "Loading",
        AssetLoadState::Ready => "Ready",
        AssetLoadState::Failed => "Failed",
    }
}

fn is_source_demo_route(route: &str) -> bool {
    route.is_empty() || route.starts_with(SOURCE_DEMO_BASE)
}

fn demo_home_route() -> &'static str {
    let route = current_route();
    if is_source_demo_route(&route) {
        SOURCE_HOME_ROUTE
    } else {
        PUBLISHED_HOME_ROUTE
    }
}

fn demo_workbench_route() -> &'static str {
    let route = current_route();
    if is_source_demo_route(&route) {
        SOURCE_WORKBENCH_ROUTE
    } else {
        PUBLISHED_WORKBENCH_ROUTE
    }
}

fn demo_stage4_route() -> &'static str {
    let route = current_route();
    if is_source_demo_route(&route) {
        SOURCE_STAGE4_ROUTE
    } else {
        PUBLISHED_STAGE4_ROUTE
    }
}

fn demo_stage5_route() -> &'static str {
    let route = current_route();
    if is_source_demo_route(&route) {
        SOURCE_STAGE5_ROUTE
    } else {
        PUBLISHED_STAGE5_ROUTE
    }
}

fn demo_immediate_drawing_route() -> &'static str {
    let route = current_route();
    if is_source_demo_route(&route) {
        SOURCE_IMMEDIATE_DRAWING_ROUTE
    } else {
        PUBLISHED_IMMEDIATE_DRAWING_ROUTE
    }
}

pub struct Stage4Showcase {
    pub root: ScrollBox,
    pub worker_test_api: Stage4WorkerTestApi,
    _external_drop_panel: ExternalDropDemoPanel,
    _reorder_panel: ReorderDemoPanel,
    _guards: Vec<Subscription>,
}

#[derive(Clone)]
pub struct Stage4WorkerTestApi {
    pub start_prime: Rc<dyn Fn()>,
    pub start_fail: Rc<dyn Fn()>,
    pub status: Rc<dyn Fn() -> String>,
    pub detail: Rc<dyn Fn() -> String>,
}

impl Stage4Showcase {
    pub fn new(
        title: &str,
        on_breakpoint_changed: Rc<dyn Fn(bool)>,
        on_theme_accent_changed: Rc<dyn Fn(u32)>,
        on_animation_preview_opacity_changed: Rc<dyn Fn(i32)>,
        on_custom_draw: Rc<dyn Fn()>,
    ) -> Self {
        load_texture(
            STAGE4_ROUTE_RELATIVE_TEXTURE_ID,
            STAGE4_ROUTE_RELATIVE_TEXTURE_URL,
        );
        load_texture(STAGE4_MISSING_TEXTURE_ID, STAGE4_MISSING_TEXTURE_URL);
        load_svg(STAGE4_SAMPLE_SVG_ID, STAGE4_SAMPLE_SVG_URL);
        load_svg(STAGE4_MISSING_SVG_ID, STAGE4_MISSING_SVG_URL);
        let custom_emoji_face =
            FontFace::load(&format!("{}/NotoColorEmoji.ttf", STAGE4_FONT_BASE_URL));
        let custom_body_stack =
            FontStack::load(&format!("{}/DejaVuSans.ttf", STAGE4_FONT_BASE_URL))
                .fallback_face(custom_emoji_face.clone());
        let custom_heading_stack =
            FontStack::load(&format!("{}/DejaVuSans-Bold.ttf", STAGE4_FONT_BASE_URL))
                .fallback_face(custom_emoji_face.clone());
        let proof_mono_stack = FontStack::load(&format!(
            "{}/NotoSansMono-Regular.ttf",
            STAGE4_FONT_BASE_URL
        ))
        .fallback_face(custom_emoji_face.clone());
        let proof_mono_bold_stack =
            FontStack::load(&format!("{}/NotoSansMono-Bold.ttf", STAGE4_FONT_BASE_URL))
                .fallback_face(custom_emoji_face.clone());
        let custom_family =
            FontFamily::regular_bold_stacks(custom_body_stack.clone(), custom_heading_stack);
        let proof_mono_family =
            FontFamily::regular_bold_stacks(proof_mono_stack.clone(), proof_mono_bold_stack);

        let theme = current_theme();
        let page = ui! {
        demo_page_root(title).height_len(auto())
        };
        let root = ui! {
            scroll_box().fill_size()
            .bg_color(0xF7F4ECFF)
            .scrollbar_gutter(0.0)
            .persist_scroll(false)
            .child(&page)
        };
        root.vertical_scrollbar()
            .track_width(12.0)
            .thumb_width(8.0)
            .thumb_min_height(36.0)
            .track_corner_radius(6.0)
            .thumb_corner_radius(4.0)
            .track_color(0xCBD5E1FF)
            .thumb_color(0x64748BFF);
        root.vertical_scrollbar()
            .render()
            .semantic_label("Stage 4 outer vertical scrollbar");
        root.horizontal_scrollbar()
            .track_width(12.0)
            .thumb_width(8.0)
            .thumb_min_height(36.0)
            .track_corner_radius(6.0)
            .thumb_corner_radius(4.0)
            .track_color(0xCBD5E1FF)
            .thumb_color(0x64748BFF);
        root.horizontal_scrollbar()
            .render()
            .semantic_label("Stage 4 outer horizontal scrollbar");

        let intro = demo_card(
            "Stage 4.2 surface",
            "This workbench exercises retained layout, responsive mutations, theme binding, coordinate helpers, gradients, borders, shadows, blur, cursor, and custom drawing.",
            0xE0F2FEFF,
        );
        intro.margin(0.0, 18.0, 0.0, 0.0);
        page.child(&intro);

        let persisted_route_card = ui! {
        stage4_panel("Route persisted state", 0xFFFFFFFF)
            .fill_width()
            .margin(0.0, 18.0, 0.0, 0.0)
            .semantic_label("Stage 4 route persisted state card")
        };
        let persisted_route_switch = ui! {
        switch("Persisted route switch")
            .node_id("stage4-route-persisted-switch")
            .semantic_label("Stage 4 persisted route switch")
        };
        let persisted_route_status = ui! {
        demo_text("Stage 4 persisted switch: off", 14.0, 0x475569FF).semantic_label("Stage 4 persisted switch: off")
        };
        persisted_route_switch.on_changed({
            let persisted_route_status = persisted_route_status.clone();
            move |event| {
                let label = if event.checked {
                    "Stage 4 persisted switch: on"
                } else {
                    "Stage 4 persisted switch: off"
                };
                persisted_route_status.text(label);
                persisted_route_status.semantic_label(label);
            }
        });
        persisted_route_card
            .child(&demo_text("Back/forward restore", 18.0, 0x111827FF))
            .child(&spacer(8.0))
            .child(&demo_text(
                "This switch has a stable node id so back/forward route restores exercise the shared browser persisted-state path.",
                15.0,
                0x334155FF,
            ))
            .child(&spacer(12.0))
            .child(&persisted_route_switch)
            .child(&spacer(8.0))
            .child(&persisted_route_status);
        page.child(&persisted_route_card);

        let status_row = ui! {
            row().fill_width().margin(0.0, 18.0, 0.0, 0.0)
        };

        let responsive_card = ui! {
        stage4_panel("Responsive layout", 0xFFFFFFFF).semantic_label("Stage 4 responsive layout card")
        };
        responsive_card
            .opacity(0.82)
            .linear_gradient(
                0.0,
                0.0,
                1.0,
                1.0,
                vec![0.0, 1.0],
                vec![0xFFFFFFFF, 0xF8FAFCFF],
            )
            .border_config(Border {
                width: 1.0,
                color: 0xCBD5E1FF,
                style: BorderStyle::Dashed,
                dash_on: 7.0,
                dash_off: 5.0,
            })
            .drop_shadow(0x0000001C, 0.0, 12.0, 28.0, 0.0)
            .background_blur(6.0)
            .cursor(CursorStyle::Pointer);
        let responsive_title = demo_text("Breakpoint: narrow", 18.0, 0x111827FF);
        let viewport_label = demo_text("Viewport: 0 x 0", 14.0, 0x475569FF);
        responsive_card
            .child(&responsive_title)
            .child(&spacer(8.0))
            .child(&demo_text(
                "Resize above 940px to switch this lane to a row layout and rebalance card widths.",
                15.0,
                0x334155FF,
            ))
            .child(&spacer(8.0))
            .child(&viewport_label)
            .child(&spacer(12.0))
            .child(&ui! {
                flex_box()
                .width_len(fill())
                .height_len(px(40.0))
                .padding(10.0, 12.0, 10.0, 12.0)
                .corner_radius(12.0)
                .bg_color(0xE2E8F0FF)
                .border(1.0, 0xCBD5E1FF)
                .cursor(CursorStyle::Pointer)
                .interactive(true)
                .focusable(true, 0)
                .semantic_label("Stage 4 tooltip sample target")
                .tool_tip(
                    ToolTip::text(
                        "Shared retained tooltip host.\nHover or focus here to test delayed open.\nPress Escape to leave keyboard tab mode.",
                    )
                    .initial_show_delay(250)
                    .between_show_delay(75)
                    .show_duration(0)
                    .placement(PopupPlacement::Bottom),
                )
                .child(&demo_text("Hover or Tab here for tooltip", 14.0, 0x334155FF))
            });

        let theme_card = ui! {
        stage4_panel("Theme binding", 0xFFFFFFFF).semantic_label("Stage 4 accent binding card")
        };
        let theme_body = demo_text("Theme accent: #00000000", 15.0, 0x334155FF);
        let accent_chip = ui! {
            flex_box()
            .width(64.0, Unit::Pixel)
            .height(28.0, Unit::Pixel)
            .corner_radius(14.0)
            .bg_color(theme.colors.accent)
            .border(1.0, 0x00000018)
        };
        theme_card
            .child(&demo_text("Accent binding", 18.0, 0x111827FF))
            .child(&spacer(8.0))
            .child(&theme_body)
            .child(&spacer(12.0))
            .child(&accent_chip);

        status_row.child(&responsive_card).child(&theme_card);
        page.child(&status_row);

        let details_row = ui! {
            row().fill_width().margin(0.0, 18.0, 0.0, 0.0)
        };

        let metrics_card = ui! {
        stage4_panel("Coordinate helpers", 0xFFFFFFFF).semantic_label("Stage 4 pointer probe card")
        };
        let bounds_label = demo_text("Bounds: pending", 14.0, 0x475569FF);
        let local_label = demo_text("Local: pending", 14.0, 0x475569FF);
        let absolute_label = demo_text("Absolute: pending", 14.0, 0x475569FF);
        let probe = ui! {
            flex_box()
            .width_len(fill())
            .height_len(px(120.0))
            .corner_radius(16.0)
            .bg_color(0xDBEAFEFF)
            .border(1.0, 0x93C5FDFF)
        };
        let interaction_surface = ui! {
            column()
            .width_len(fill())
            .interactive(true)
            .cursor(CursorStyle::Pointer)
            .on_click({
                let probe_for_click = probe.clone();
                let bounds_for_click = bounds_label.clone();
                let local_for_click = local_label.clone();
                let absolute_for_click = absolute_label.clone();
                move |event| {
                    let bounds = probe_for_click.get_bounds();
                    bounds_for_click.text(format!(
                        "Bounds: x {:.0}, y {:.0}, w {:.0}, h {:.0}",
                        bounds[0], bounds[1], bounds[2], bounds[3]
                    ));
                    let local =
                        probe_for_click.absolute_to_local_position(event.scene_x, event.scene_y);
                    local_for_click.text(format!("Local: x {:.0}, y {:.0}", local[0], local[1]));
                    let absolute = probe_for_click.local_to_absolute_position(local[0], local[1]);
                    absolute_for_click.text(format!(
                        "Absolute: x {:.0}, y {:.0}",
                        absolute[0], absolute[1]
                    ));
                }
            })
        };
        interaction_surface.child(&probe);
        metrics_card
            .child(&demo_text("Pointer probe", 18.0, 0x111827FF))
            .child(&spacer(8.0))
            .child(&demo_text(
                "Click the probe to update bounds, local coordinates, and round-tripped absolute coordinates.",
                15.0,
                0x334155FF,
            ))
            .child(&spacer(12.0))
            .child(&interaction_surface)
            .child(&spacer(10.0))
            .child(&bounds_label)
            .child(&spacer(6.0))
            .child(&local_label)
            .child(&spacer(6.0))
            .child(&absolute_label);

        let drawing_card = ui! {
        stage4_panel("Custom drawing", 0xFFFFFFFF).semantic_label("Stage 4 immediate canvas card")
        };
        let drawing_text_status =
            demo_text("Stage 4 text layout status: Waiting", 14.0, 0x475569FF);
        drawing_text_status.semantic_label("Stage 4 text layout status: Waiting");
        let drawing_texture_status = demo_text(
            "Stage 4 direct texture draw status: Loading",
            14.0,
            0x475569FF,
        );
        drawing_texture_status.semantic_label("Stage 4 direct texture draw status: Loading");
        let drawing_svg_status =
            demo_text("Stage 4 direct SVG draw status: Loading", 14.0, 0x475569FF);
        drawing_svg_status.semantic_label("Stage 4 direct SVG draw status: Loading");
        let drawing_path_status = demo_text(
            "Stage 4 path clip transform status: Ready",
            14.0,
            0x475569FF,
        );
        drawing_path_status.semantic_label("Stage 4 path clip transform status: Ready");
        let drawing_bitmap = Bitmap::new(320, 176);
        let bitmap_canvas = drawing_bitmap.canvas();
        bitmap_canvas.draw_round_rect(0.0, 0.0, 320.0, 176.0, 22.0, 22.0, Paint::fill(0xE2E8F0FF));
        bitmap_canvas.draw_round_rect(
            18.0,
            20.0,
            138.0,
            92.0,
            18.0,
            18.0,
            Paint::filled_stroke(0x2563EBFF, 0x1D4ED8FF, 2.0),
        );
        bitmap_canvas.draw_circle(234.0, 68.0, 28.0, Paint::fill(0xF59E0BFF));
        bitmap_canvas.draw_line(28.0, 132.0, 286.0, 36.0, 0x0F172AFF, 3.0);
        drawing_bitmap.commit();
        let drawing_title_layout = TextLayout::text("Bitmap + TextLayout");
        drawing_title_layout
            .font_family(theme.fonts.heading_family.clone())
            .font_size(18.0)
            .text_color(theme.colors.text_primary)
            .width(220.0, Unit::Pixel)
            .height(28.0, Unit::Pixel);
        let drawing_value_layout = DynamicTextLayout::fixed_charset(
            "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ :.x+-",
        );
        drawing_value_layout
            .font_family(theme.fonts.body_family.clone())
            .font_size(14.0)
            .text_color(theme.colors.text_primary)
            .width(260.0, Unit::Pixel)
            .height(22.0, Unit::Pixel);
        drawing_value_layout.set_text("Draw batch ready");
        let mut drawing_path = Path::new();
        drawing_path
            .move_to(0.0, -18.0)
            .cubic_to(18.0, -32.0, 42.0, -30.0, 58.0, -12.0)
            .quad_to(74.0, 8.0, 54.0, 24.0)
            .line_to(8.0, 34.0)
            .line_to(-12.0, 6.0)
            .close();
        let drawing_count = Rc::new(RefCell::new(0u32));
        let drawing = custom_drawable({
            let on_custom_draw = on_custom_draw.clone();
            let drawing_bitmap = drawing_bitmap.clone();
            let drawing_title_layout = drawing_title_layout.clone();
            let drawing_value_layout = drawing_value_layout.clone();
            let drawing_path = drawing_path;
            let drawing_count = drawing_count.clone();
            move |ctx| {
                on_custom_draw();
                let next_count = {
                    let mut count = drawing_count.borrow_mut();
                    *count += 1;
                    *count
                };

                ctx.draw_round_rect(0.0, 0.0, 460.0, 176.0, 20.0, 20.0, Paint::fill(0xF8FAFCFF));
                ctx.draw_image(drawing_bitmap.texture_id(), 18.0, 18.0, 276.0, 140.0);
                ctx.save();
                ctx.clip_round_rect(306.0, 16.0, 132.0, 146.0, 16.0, 16.0, 16.0, 16.0);
                ctx.translate(370.0, 80.0);
                ctx.rotate((next_count % 360) as f32 * 0.8);
                ctx.scale(0.86, 0.86);
                ctx.draw_path(
                    &drawing_path,
                    Paint::filled_stroke(0xDB2777CC, 0x9D174DFF, 2.0),
                );
                ctx.restore();
                if drawing_title_layout.is_ready() {
                    ctx.draw_text_layout(&drawing_title_layout, 32.0, 34.0);
                }
                if drawing_value_layout.is_ready() {
                    let dynamic_node = drawing_value_layout.draw_node();
                    ctx.draw_text_node(&dynamic_node, 32.0, 136.0);
                }
                if get_texture_asset_state_value(STAGE4_ROUTE_RELATIVE_TEXTURE_ID)
                    == AssetLoadState::Ready
                {
                    ctx.draw_image(STAGE4_ROUTE_RELATIVE_TEXTURE_ID, 308.0, 22.0, 126.0, 74.0);
                }
                if get_svg_asset_state(STAGE4_SAMPLE_SVG_ID).get() == AssetLoadState::Ready {
                    ctx.draw_svg(STAGE4_SAMPLE_SVG_ID, 308.0, 104.0, 126.0, 54.0);
                }
            }
        });
        drawing
            .width_len(fill())
            .height_len(px(176.0))
            .corner_radius(22.0)
            .bg_color(0xF8FAFCFF)
            .border(1.0, 0xCBD5E1FF)
            .semantic_label("Stage 4 immediate drawing surface");
        drawing_title_layout.on_ready({
            let drawing = drawing.clone();
            let drawing_bitmap = drawing_bitmap.clone();
            let drawing_title_layout = drawing_title_layout.clone();
            let drawing_text_status = drawing_text_status.clone();
            move |_| {
                drawing_bitmap.render_text_layout(&drawing_title_layout, 26.0, 22.0, 1.0);
                drawing_bitmap.commit();
                let label = format!(
                    "Stage 4 text layout status: Ready {:.0}x{:.0}",
                    drawing_title_layout.measured_width(),
                    drawing_title_layout.measured_height()
                );
                drawing_text_status.text(&label);
                drawing_text_status.semantic_label(label);
                drawing.mark_dirty();
            }
        });
        drawing_value_layout.on_ready({
            let drawing = drawing.clone();
            move |_| {
                drawing.mark_dirty();
            }
        });
        drawing_card
            .child(&demo_text("Immediate canvas surface", 18.0, 0x111827FF))
            .child(&spacer(8.0))
            .child(&demo_text(
                "This sample exercises the FUI-AS immediate drawing resource stack: batched DrawContext commands, Bitmap offscreen commit, TextLayout readiness, DynamicTextLayout updates, and direct image/SVG draws.",
                15.0,
                0x334155FF,
            ))
            .child(&spacer(12.0))
            .child(&drawing)
            .child(&spacer(12.0))
            .child(&drawing_text_status)
            .child(&spacer(6.0))
            .child(&drawing_texture_status)
            .child(&spacer(6.0))
            .child(&drawing_svg_status)
            .child(&spacer(6.0))
            .child(&drawing_path_status);

        details_row.child(&metrics_card).child(&drawing_card);
        page.child(&details_row);

        let typography_card = ui! {
        stage4_panel("App-authored custom fonts", 0xFFFFFFFF)
            .fill_width()
            .margin(0.0, 18.0, 0.0, 0.0)
            .semantic_label("Stage 4 app authored custom fonts card")
        };
        let custom_font_heading = ui! {
        text("Custom DejaVu FontStack sample")
            .font_family(custom_family.clone())
            .font_weight(FontWeight::Bold)
            .font_size(22.0)
            .text_color(theme.colors.text_primary)
            .semantic_label("Stage 4 custom font heading")
        };
        let custom_font_body = ui! {
        text("Load DejaVu Sans through FontStack::load(...), use DejaVu Bold for heavier text, and keep fallback registration in the typed typography API without app-managed font IDs.")
            .font_family(custom_family.clone())
            .font_size(16.0)
            .wrapping(true)
            .text_limits(-1, 4)
            .text_color(theme.colors.text_muted)
            .semantic_label("Stage 4 custom font body")
        };
        let custom_font_direct_stack =
            text("Apply a stack directly: TextNode::font_stack(custom_body_stack, 17)");
        custom_font_direct_stack
            .font_stack(custom_body_stack.clone(), 17.0)
            .text_color(theme.colors.text_muted)
            .semantic_label("Stage 4 direct font stack text");
        let custom_font_comparison =
            text("Bold family resolution stays intact: DejaVu Bold + fallback stack");
        custom_font_comparison
            .font_family(custom_family.clone())
            .font_weight(FontWeight::Bold)
            .font_size(18.0)
            .text_color(theme.colors.text_primary)
            .semantic_label("Stage 4 custom font bold family resolution text");
        let rich_text_container = RichText::new(vec![
            span("Base family ")
                .underline()
                .text_color(theme.colors.text_primary),
            span("with ").text_color(theme.colors.text_primary),
            span("MONO OVERRIDE")
                .font_family(proof_mono_family.clone())
                .strikethrough()
                .text_color(theme.colors.accent),
        ]);
        rich_text_container
            .font_family(custom_family.clone())
            .font_weight(FontWeight::Bold)
            .font_size(20.0)
            .text_color(theme.colors.text_primary);
        rich_text_container
            .width(100.0, Unit::Percent)
            .line_height(28.0)
            .text_limits(-1, 1)
            .semantic_label("Stage 4 rich text container font family sample");
        let rich_text_helpers = RichText::new(vec![
            span("Rich ").bold().text_color(theme.colors.text_primary),
            span("text ").italic().text_color(0x60A5FAFF),
            span("underline ").underline().text_color(0xFBBF24FF),
            span("strike ").strikethrough().text_color(0xF87171FF),
            span("helpers")
                .bold()
                .italic()
                .underline()
                .strikethrough()
                .text_color(0xCBD5E1FF),
        ]);
        rich_text_helpers
            .font_family(custom_family.clone())
            .font_weight(FontWeight::Bold)
            .font_size(18.0)
            .text_color(theme.colors.text_primary);
        rich_text_helpers
            .width(100.0, Unit::Percent)
            .line_height(26.0)
            .text_limits(-1, 1)
            .semantic_label("Stage 4 rich text helper span sample");
        let custom_font_status = ui! {
        demo_text("Custom font status: waiting", 14.0, 0x475569FF).semantic_label("Stage 4 custom font status waiting")
        };
        custom_body_stack.on_loaded({
            let custom_font_status = custom_font_status.clone();
            move |event| {
                let label = format!(
                    "Custom font status: DejaVu stack ready for {}",
                    if event.stack.is_loaded() {
                        "rendering"
                    } else {
                        "layout"
                    }
                );
                custom_font_status.text(&label);
                custom_font_status.semantic_label(format!("Stage 4 {}", label));
            }
        });
        typography_card
            .child(&custom_font_heading)
            .child(&spacer(8.0))
            .child(&custom_font_body)
            .child(&spacer(10.0))
            .child(&custom_font_direct_stack)
            .child(&spacer(10.0))
            .child(&custom_font_comparison)
            .child(&spacer(16.0))
            .child(&rich_text_container)
            .child(&spacer(10.0))
            .child(&rich_text_helpers)
            .child(&spacer(12.0))
            .child(&custom_font_status);
        page.child(&typography_card);

        let animation_column = ui! {
            column().fill_width().margin(0.0, 18.0, 0.0, 0.0)
        };

        let animation_preview_card = ui! {
        stage4_panel("Transitions", 0xFFFFFFFF).semantic_label("Stage 4 animation preview card")
        };
        let animation_preview_title = demo_text("Calm transition target", 18.0, 0x111827FF);
        let animation_preview_body = demo_text(
            "Opacity and background transitions stay on the same retained node while the control layer keeps behavior ownership elsewhere.",
            15.0,
            0x334155FF,
        );
        let animation_preview_status =
            demo_text("Stage 4 animation preview status: calm", 14.0, 0x475569FF);
        animation_preview_status.semantic_label("Stage 4 animation preview status: calm");
        let loaded_status = ui! {
        demo_text("Stage 4 loaded status: waiting", 14.0, 0x475569FF).semantic_label("Stage 4 loaded status: waiting")
        };
        let animation_preview_surface = flex_box();
        animation_preview_surface
            .fill_width()
            .height_len(px(144.0))
            .padding(20.0, 18.0, 20.0, 18.0)
            .corner_radius(20.0)
            .transitions(Some(
                NodeTransitions::new()
                    .bg_color(AnimationTiming::with_easing(440.0, Easings::cubic_out()))
                    .opacity(AnimationTiming::with_easing(360.0, Easings::cubic_out())),
            ))
            .bg_color(theme.colors.surface)
            .opacity(0.7)
            .child(&ui! {
                column()
                .fill_width()
                .child(&animation_preview_title)
                .child(&spacer(8.0))
                .child(&animation_preview_body)
            });
        let animation_calm_button = button("Set calm preview");
        let animation_emphasis_button = button("Emphasize preview card");
        animation_preview_card
            .child(&animation_calm_button)
            .child(&spacer(8.0))
            .child(&animation_emphasis_button)
            .child(&spacer(12.0))
            .child(&animation_preview_surface)
            .child(&spacer(10.0))
            .child(&animation_preview_status)
            .child(&spacer(6.0))
            .child(&loaded_status);

        let animation_scroll_card = ui! {
        stage4_panel("Animated scrolling", 0xFFFFFFFF).semantic_label("Stage 4 animated scroll card")
        };
        let animation_scroll_status =
            demo_text("Stage 4 animated scroll target: top", 14.0, 0x475569FF);
        animation_scroll_status.semantic_label("Stage 4 animated scroll target: top");
        let animation_scroll_box = ui! {
            scroll_box()
            .fill_width()
            .height_len(px(300.0))
            .scroll_enabled_x(false)
            .scroll_enabled_y(true)
            .vertical_scrollbar_visibility(ScrollBarVisibility::Always)
            .horizontal_scrollbar_visibility(ScrollBarVisibility::Never)
            .scroll_content_size(-1.0, 960.0)
            .transitions(Some(NodeTransitions::new().scroll_offset(
                AnimationTiming::with_easing(420.0, Easings::cubic_out()),
            )))
        };
        let animation_scroll_content = ui! {
            column().fill_width()
        };
        for index in 0..13 {
            animation_scroll_content.child(&ui! {
                flex_box()
                .fill_width()
                .height_len(px(64.0))
                .padding(16.0, 12.0, 16.0, 12.0)
                .corner_radius(14.0)
                .bg_color(if index % 2 == 0 {
                    0xFFFFFFFF
                } else {
                    0xF8FAFCFF
                })
                .border(1.0, 0xCBD5E1FF)
                .semantic_label(format!("Stage 4 animation sample row {}", index + 1))
                .child(&demo_text(
                    &format!("Animation sample row {}", index + 1),
                    15.0,
                    0x0F172AFF,
                ))
            });
            if index < 12 {
                animation_scroll_content.child(&spacer(10.0));
            }
        }
        animation_scroll_box.child(&animation_scroll_content);
        let animation_top_button = button("Scroll to first sample");
        let animation_middle_button = button("Scroll to 7th sample");
        let animation_bottom_button = button("Scroll to 13th sample");
        let animation_tail_button = button("Scroll to logical tail");
        animation_scroll_card
            .child(&animation_top_button)
            .child(&spacer(8.0))
            .child(&animation_middle_button)
            .child(&spacer(8.0))
            .child(&animation_bottom_button)
            .child(&spacer(8.0))
            .child(&animation_tail_button)
            .child(&spacer(12.0))
            .child(&animation_scroll_box)
            .child(&spacer(10.0))
            .child(&animation_scroll_status);

        animation_column
            .child(&animation_preview_card)
            .child(&spacer(18.0))
            .child(&animation_scroll_card);
        page.child(&animation_column);

        let interaction_row = ui! {
            row().fill_width().margin(0.0, 18.0, 0.0, 0.0)
        };

        let scroll_card = ui! {
        stage4_panel("ScrollBox + SelectionArea", 0xFFFFFFFF).fill_width()
        };
        scroll_card.semantic_label("Stage 4 nested scrolling card");
        let selection_status = demo_text("Selected text: none", 14.0, 0x475569FF);
        let selection = ui! {
            selection_area().width(1500.0, Unit::Pixel)
        };
        let scroll = ui! {
            scroll_box()
            .height_len(px(240.0))
            .fill_width()
            .semantic_label("Stage 4 nested scrollbox")
            .border(1.0, 0xCBD5E1FF)
            .corner_radius(16.0)
            .bg_color(0xF8FAFCFF)
            .scrollbar_gutter(0.0)
            .scroll_content_size(1500.0, 560.0)
            .persist_scroll(false)
        };
        scroll
            .vertical_scrollbar()
            .track_width(12.0)
            .thumb_width(8.0)
            .thumb_min_height(36.0)
            .track_corner_radius(6.0)
            .thumb_corner_radius(4.0)
            .track_color(0xCBD5E1FF)
            .thumb_color(0x64748BFF);
        scroll
            .vertical_scrollbar()
            .render()
            .semantic_label("Stage 4 nested vertical scrollbar");
        scroll
            .horizontal_scrollbar()
            .track_width(12.0)
            .thumb_width(8.0)
            .thumb_min_height(36.0)
            .track_corner_radius(6.0)
            .thumb_corner_radius(4.0)
            .track_color(0xCBD5E1FF)
            .thumb_color(0x64748BFF);
        scroll
            .horizontal_scrollbar()
            .render()
            .semantic_label("Stage 4 nested horizontal scrollbar");

        let content = ui! {
            column()
            .width(1500.0, Unit::Pixel)
            .min_height(560.0, Unit::Pixel)
            .padding(18.0, 18.0, 18.0, 18.0)
            .child(&demo_text(
                "Drag the scrollbar thumb, wheel the viewport, and select this paragraph to confirm that the Rust SDK now owns the retained scroll chrome rather than a demo-only approximation.",
                15.0,
                0x0F172AFF,
            ))
            .child(&spacer(14.0))
            .child(&demo_text(
                "The explicit content width forces horizontal overflow so the bottom rail is visible too.",
                15.0,
                0x334155FF,
            ))
        };
        for index in 0..14 {
            let row_text = demo_text(
                &format!(
                    "Scrollable row {:02}: retained viewport metrics should keep the scrollbar thumb in sync with bridge scroll callbacks.",
                    index + 1
                ),
                14.0,
                0x1F2937FF,
            );
            row_text.semantic_label(format!("Stage 4 nested scroll row {}", index + 1));
            content.child(&spacer(10.0)).child(&row_text);
        }
        selection.child(&content);
        scroll.child(&selection);
        scroll_card
            .child(&demo_text("Nested retained scrolling", 18.0, 0x111827FF))
            .child(&spacer(8.0))
            .child(&demo_text(
                "This lane exercises the FUI-AS style retained scroll model: a viewport ScrollView, bound scroll metrics, chrome-owned scrollbars, and cross-selection inside the content surface.",
                15.0,
                0x334155FF,
            ))
            .child(&spacer(12.0))
            .child(&scroll)
            .child(&spacer(10.0))
            .child(&selection_status);
        interaction_row.child(&scroll_card);
        page.child(&interaction_row);

        let reorder_row = ui! {
            row().fill_width().margin(0.0, 18.0, 0.0, 0.0)
        };
        let reorder_panel = ReorderDemoPanel::new();
        reorder_row.child(&reorder_panel);
        page.child(&reorder_row);

        let context_row = ui! {
            row().fill_width().margin(0.0, 18.0, 0.0, 0.0)
        };

        let context_card = ui! {
        stage4_panel("Context targets", 0xFFFFFFFF).fill_width()
        };
        context_card.semantic_label("Stage 4 context targets card");
        let route_relative_state = demo_text(
            "Stage 4 route-relative image state: Loading",
            14.0,
            0x475569FF,
        );
        route_relative_state.semantic_label("Stage 4 route-relative image state: Loading");
        let explicit_svg_state = ui! {
        demo_text("Stage 4 explicit SVG state: Loading", 14.0, 0x475569FF).semantic_label("Stage 4 explicit SVG state: Loading")
        };
        let missing_texture_state =
            demo_text("Stage 4 missing texture state: Loading", 14.0, 0x475569FF);
        missing_texture_state.semantic_label("Stage 4 missing texture state: Loading");
        let missing_svg_state = ui! {
        demo_text("Stage 4 missing SVG state: Loading", 14.0, 0x475569FF).semantic_label("Stage 4 missing SVG state: Loading")
        };
        let selectable_context_text = demo_text(
            "Select this sentence, then right-click or long-press it to expose the non-editable text context actions.",
            15.0,
            0x0F172AFF,
        );
        selectable_context_text.semantic_label("Stage 4 context selectable text");
        context_card
            .child(&demo_text("Non-editable context targets", 18.0, 0x111827FF))
            .child(&spacer(8.0))
            .child(&demo_text(
                "Right-click or long-press the link, image, SVG, or selectable text below to confirm the built-in FUI-AS style menu generation for non-editable targets.",
                15.0,
                0x334155FF,
            ))
            .child(&spacer(12.0))
            .child(&ui! {
            nav_link("https://example.com/docs")
                .text("Stage 4 example docs")
                .semantic_label("Stage 4 example docs nav link")
            })
            .child(&spacer(16.0))
            .child(&ui! {
            image(STAGE4_SAMPLE_TEXTURE_ID)
                .width(320.0, Unit::Pixel)
                .height(180.0, Unit::Pixel)
                .source(STAGE4_ROUTE_RELATIVE_TEXTURE_URL)
                .alt_text("Stage 4 sample image")
            })
            .child(&spacer(16.0))
            .child(&ui! {
            svg(STAGE4_SAMPLE_SVG_ID)
                .width(320.0, Unit::Pixel)
                .height(180.0, Unit::Pixel)
                .source(STAGE4_SAMPLE_SVG_URL)
                .alt_text("Stage 4 sample SVG")
            })
            .child(&spacer(12.0))
            .child(&ui! {
            image(STAGE4_ROUTE_RELATIVE_TEXTURE_ID)
                .height(56.0, Unit::Pixel)
                .width(0.0, Unit::Auto)
                .sampling(ImageSampling::nearest())
                .alt_text("Stage 4 explicit texture ID image")
            })
            .child(&spacer(12.0))
            .child(&ui! {
            svg(STAGE4_SAMPLE_SVG_ID)
                .height(56.0, Unit::Pixel)
                .width(0.0, Unit::Auto)
                .tint(0x1D4ED8FF)
                .sampling(ImageSampling::cubic_catmull_rom())
                .alt_text("Stage 4 explicit SVG ID")
            })
            .child(&spacer(12.0))
            .child(&route_relative_state)
            .child(&spacer(6.0))
            .child(&explicit_svg_state)
            .child(&spacer(6.0))
            .child(&missing_texture_state)
            .child(&spacer(6.0))
            .child(&missing_svg_state)
            .child(&spacer(16.0))
            .child(&selectable_context_text);
        let tooling_card = ui! {
        stage4_panel("Browser tooling", 0xFFFFFFFF)
            .fill_width()
            .semantic_label("Stage 4 browser tooling card")
            .node_id("stage4-browser-tooling-card")
        };
        let tooling_body = demo_text(
            "Findable stage four phrase: bridge-tooled-text-anchor. This paragraph should be visible to browser find-on-page and debug inspection without a separate FUI-RS implementation.",
            15.0,
            0x334155FF,
        );
        tooling_body.semantic_label("Stage 4 findable browser tooling text");
        tooling_card
            .child(&demo_text("Find + inspect inheritance", 18.0, 0x111827FF))
            .child(&spacer(8.0))
            .child(&tooling_body)
            .child(&spacer(12.0))
            .child(&ui! {
                flex_box()
                .width_len(fill())
                .height_len(px(46.0))
                .padding(12.0, 14.0, 12.0, 14.0)
                .corner_radius(14.0)
                .bg_color(0xF1F5F9FF)
                .border(1.0, 0xCBD5E1FF)
                .interactive(true)
                .focusable(true, 0)
                .cursor(CursorStyle::Pointer)
                .node_id("stage4-inspectable-debug-target")
                .semantic_label("Stage 4 inspectable debug target")
                .child(&demo_text("Inspectable debug target", 14.0, 0x0F172AFF))
            });
        context_row.child(&context_card).child(&tooling_card);
        page.child(&context_row);

        let worker_row = ui! {
            row().fill_width().margin(0.0, 18.0, 0.0, 0.0)
        };

        let worker_card = ui! {
        stage4_panel("Workers", 0xFFFFFFFF)
            .fill_width()
            .semantic_label("Stage 4 workers card")
        };
        let worker_button_row = ui! {
            row().fill_width()
        };
        let worker_start_button = button("Start prime worker");
        let worker_fail_button = button("Start failing worker");
        let worker_cancel_button = button("Cancel worker");
        worker_button_row
            .child(&ui! {
                flex_box()
                .semantic_label("Stage 4 start worker button")
                .child(&worker_start_button)
            })
            .child(&spacer(14.0))
            .child(&ui! {
                flex_box()
                .semantic_label("Stage 4 failing worker button")
                .child(&worker_fail_button)
            })
            .child(&spacer(14.0))
            .child(&ui! {
                flex_box()
                .semantic_label("Stage 4 cancel worker button")
                .child(&worker_cancel_button)
            });
        let worker_progress = ui! {
            progress_bar()
            .sizing(ProgressBarSizing::new().length(360.0).thickness(18.0))
            .value(0.0)
            .semantic_label("Stage 4 worker progress bar")
        };
        let worker_status_value =
            Rc::new(RefCell::new(String::from("Stage 4 worker status: idle")));
        let worker_detail_value =
            Rc::new(RefCell::new(String::from("Stage 4 worker detail: waiting")));
        let worker_status = ui! {
        demo_text("Stage 4 worker status: idle", 14.0, 0x475569FF).semantic_label("Stage 4 worker status: idle")
        };
        let worker_detail = ui! {
        demo_text("Stage 4 worker detail: waiting", 14.0, 0x475569FF).semantic_label("Stage 4 worker detail: waiting")
        };
        worker_card
            .child(&demo_text("Background worker bridge", 18.0, 0x111827FF))
            .child(&spacer(8.0))
            .child(&demo_text(
                "This mirrors the FUI-AS worker shape: main-thread Worker controller, worker-side WorkerRuntime and WorkerJob, progress/complete/error callbacks, cancellation, and generated worker host services.",
                15.0,
                0x334155FF,
            ))
            .child(&spacer(12.0))
            .child(&worker_button_row)
            .child(&spacer(12.0))
            .child(&worker_progress)
            .child(&spacer(10.0))
            .child(&worker_status)
            .child(&spacer(6.0))
            .child(&worker_detail);
        worker_row.child(&worker_card);
        page.child(&worker_row);

        let fetch_row = ui! {
            row().fill_width().margin(0.0, 18.0, 0.0, 0.0)
        };
        let fetch_card = ui! {
        stage4_panel("Online Fetch sample", 0xFFFFFFFF).fill_width().semantic_label("Stage 4 fetch card")
        };
        let fetch_body = demo_text(
            "Send real GET and POST requests through the shipped Fetch API to JSONPlaceholder without dropping to browser-specific networking code.",
            15.0,
            0x334155FF,
        );
        fetch_body.wrapping(true).text_limits(i32::MAX, 4);
        let fetch_button_row = ui! {
            row().fill_width()
        };
        let fetch_get_button = ui! {
        button("GET /posts/1").width_len(px(156.0))
        };
        let fetch_post_button = ui! {
        button("POST /posts").width_len(px(156.0))
        };
        fetch_button_row
            .child(&ui! {
                flex_box()
                .width_len(px(156.0))
                .semantic_label("Stage 4 fetch GET button")
                .child(&fetch_get_button)
            })
            .child(&spacer(12.0))
            .child(&ui! {
                flex_box()
                .width_len(px(156.0))
                .semantic_label("Stage 4 fetch POST button")
                .child(&fetch_post_button)
            });
        let fetch_status_value = Rc::new(RefCell::new(String::from("Fetch status: idle")));
        let fetch_status = ui! {
        demo_text("Fetch status: idle", 14.0, 0x475569FF).semantic_label("Fetch status: idle")
        };
        let fetch_request = ui! {
        demo_text("Latest request: none", 14.0, 0x475569FF).wrapping(true).text_limits(i32::MAX, 3)
        };
        fetch_request.semantic_label("Latest request: none");
        let fetch_result = ui! {
        demo_text("Latest result: none yet", 14.0, 0x475569FF).wrapping(true).text_limits(i32::MAX, 5)
        };
        fetch_result.semantic_label("Latest result: none yet");
        let fetch_hint = demo_text(
            "This demo uses the shipped Fetch API against the live JSONPlaceholder service. The request is real and online; the current Fetch surface reports completion metadata rather than response bodies.",
            14.0,
            0x475569FF,
        );
        fetch_hint.wrapping(true).text_limits(i32::MAX, 6);
        let active_fetch_request = Rc::new(RefCell::new(None::<FetchRequest>));
        let active_fetch_label = Rc::new(RefCell::new(String::new()));
        fetch_card
            .child(&demo_text("HTTP fetch bridge", 18.0, 0x111827FF))
            .child(&spacer(8.0))
            .child(&fetch_body)
            .child(&spacer(12.0))
            .child(&fetch_button_row)
            .child(&spacer(12.0))
            .child(&fetch_status)
            .child(&spacer(6.0))
            .child(&fetch_request)
            .child(&spacer(6.0))
            .child(&fetch_result)
            .child(&spacer(10.0))
            .child(&fetch_hint);
        fetch_row.child(&fetch_card);
        page.child(&fetch_row);

        let file_row = ui! {
            row().fill_width().margin(0.0, 18.0, 0.0, 0.0)
        };
        let file_card = ui! {
        stage4_panel("File bridge", 0xFFFFFFFF)
            .fill_width()
            .semantic_label("Stage 4 file bridge card")
        };
        let file_capabilities = File::capabilities();
        let file_capability_label = format!(
            "Capabilities: open={} read={} save={} read-chunks={} write-chunks={} native-picker={} worker-process-save={}",
            if file_capabilities.can_pick_open { "yes" } else { "no" },
            if file_capabilities.can_read { "yes" } else { "no" },
            if file_capabilities.can_save { "yes" } else { "no" },
            if file_capabilities.can_read_chunks { "yes" } else { "no" },
            if file_capabilities.can_write_chunks { "yes" } else { "no" },
            if file_capabilities.can_use_native_save_picker { "yes" } else { "no" },
            if file_capabilities.can_process_in_worker_to_picked_file { "yes" } else { "no" },
        );
        let file_status = ui! {
        demo_text("Stage 4 file status: idle", 14.0, 0x475569FF).semantic_label("Stage 4 file status: idle")
        };
        let file_detail = ui! {
        demo_text("Stage 4 file detail: waiting", 14.0, 0x475569FF).semantic_label("Stage 4 file detail: waiting")
        };
        let file_picked = ui! {
        demo_text("Picked file: none", 14.0, 0x475569FF).semantic_label("Picked file: none")
        };
        let file_button_row = ui! {
            row().fill_width()
        };
        let pick_file_button = button("Pick file");
        let save_text_button = button("Save sample text");
        let save_bytes_button = button("Save sample bytes");
        let copy_file_button = button("Copy picked file");
        file_button_row
            .child(&pick_file_button)
            .child(&spacer(14.0))
            .child(&save_text_button)
            .child(&spacer(14.0))
            .child(&save_bytes_button)
            .child(&spacer(14.0))
            .child(&copy_file_button);
        file_card
            .child(&demo_text("File picker and worker copy", 18.0, 0x111827FF))
            .child(&spacer(8.0))
            .child(&demo_text(
                "This mirrors the FUI-AS file bridge shape: open picker, save requests, first-class BrowserFile handles, and worker-side chunk read plus picked-file write.",
                15.0,
                0x334155FF,
            ))
            .child(&spacer(12.0))
            .child(&file_button_row)
            .child(&spacer(12.0))
            .child(&ui! {
            demo_text(&file_capability_label, 14.0, 0x475569FF).semantic_label(file_capability_label)
            })
            .child(&spacer(10.0))
            .child(&file_picked)
            .child(&spacer(6.0))
            .child(&file_status)
            .child(&spacer(6.0))
            .child(&file_detail);
        file_row.child(&file_card);
        page.child(&file_row);

        let external_drop_row = ui! {
            row().fill_width().margin(0.0, 18.0, 0.0, 0.0)
        };
        let external_drop_panel = ExternalDropDemoPanel::new();
        external_drop_row.child(&external_drop_panel);
        page.child(&external_drop_row);

        let apply_breakpoint: Rc<dyn Fn(f32, f32)> = {
            let responsive_title = responsive_title.clone();
            let viewport_label = viewport_label.clone();
            let status_row = status_row.clone();
            let details_row = details_row.clone();
            let responsive_card = responsive_card.clone();
            let theme_card = theme_card.clone();
            let metrics_card = metrics_card.clone();
            let drawing_card = drawing_card.clone();
            let on_breakpoint_changed = on_breakpoint_changed.clone();
            Rc::new(move |width: f32, height: f32| {
                let is_wide = width >= 940.0;
                status_row.flex_direction(if is_wide {
                    FlexDirection::Row
                } else {
                    FlexDirection::Column
                });
                details_row.flex_direction(if is_wide {
                    FlexDirection::Row
                } else {
                    FlexDirection::Column
                });
                let paired_card_width = if is_wide { 48.5 } else { 100.0 };
                responsive_card.fill_width_percent(paired_card_width);
                theme_card.fill_width_percent(paired_card_width);
                metrics_card.fill_width_percent(paired_card_width);
                drawing_card.fill_width_percent(paired_card_width);
                theme_card.margin(
                    if is_wide { 18.0 } else { 0.0 },
                    if is_wide { 0.0 } else { 18.0 },
                    0.0,
                    0.0,
                );
                drawing_card.margin(
                    if is_wide { 18.0 } else { 0.0 },
                    if is_wide { 0.0 } else { 18.0 },
                    0.0,
                    0.0,
                );
                responsive_title.text(if is_wide {
                    "Breakpoint: wide"
                } else {
                    "Breakpoint: narrow"
                });
                viewport_label.text(format!("Viewport: {:.0} x {:.0}", width, height));
                on_breakpoint_changed(is_wide);
            })
        };
        apply_breakpoint(viewport_width(), viewport_height());

        {
            let animation_preview_title = animation_preview_title.clone();
            let animation_preview_body = animation_preview_body.clone();
            let animation_preview_status = animation_preview_status.clone();
            let animation_preview_surface = animation_preview_surface.clone();
            let on_animation_preview_opacity_changed = on_animation_preview_opacity_changed.clone();
            animation_calm_button.on_click(move |_event| {
                animation_preview_title.text("Calm transition target");
                animation_preview_body.text("Opacity and background transitions stay on the same retained node while the control layer keeps behavior ownership elsewhere.");
                animation_preview_status.text("Stage 4 animation preview status: calm");
                animation_preview_status.semantic_label("Stage 4 animation preview status: calm");
                animation_preview_surface
                    .bg_color(current_theme().colors.surface)
                    .opacity(0.7);
                on_animation_preview_opacity_changed(70);
            });
        }
        {
            let animation_preview_title = animation_preview_title.clone();
            let animation_preview_body = animation_preview_body.clone();
            let animation_preview_status = animation_preview_status.clone();
            let animation_preview_surface = animation_preview_surface.clone();
            let on_animation_preview_opacity_changed = on_animation_preview_opacity_changed.clone();
            animation_emphasis_button.on_click(move |_event| {
                animation_preview_title.text("Emphasized transition target");
                animation_preview_body.text("The preview card now drives both opacity and background transitions together from one typed slot set.");
                animation_preview_status.text("Stage 4 animation preview status: emphasized");
                animation_preview_status.semantic_label("Stage 4 animation preview status: emphasized");
                animation_preview_surface
                    .bg_color(current_theme().colors.accent_hovered)
                    .opacity(1.0);
                on_animation_preview_opacity_changed(100);
            });
        }
        {
            let animation_scroll_status = animation_scroll_status.clone();
            let animation_scroll_box = animation_scroll_box.clone();
            animation_top_button.on_click(move |_event| {
                animation_scroll_box.scroll_to_animated(
                    0.0,
                    0.0,
                    AnimationTiming::with_easing(420.0, Easings::cubic_out()),
                );
                animation_scroll_status.text("Stage 4 animated scroll target: top");
                animation_scroll_status.semantic_label("Stage 4 animated scroll target: top");
            });
        }
        {
            let animation_scroll_status = animation_scroll_status.clone();
            let animation_scroll_box = animation_scroll_box.clone();
            animation_middle_button.on_click(move |_event| {
                animation_scroll_box.scroll_to_animated(
                    0.0,
                    330.0,
                    AnimationTiming::with_easing(420.0, Easings::cubic_out()),
                );
                animation_scroll_status.text("Stage 4 animated scroll target: middle");
                animation_scroll_status.semantic_label("Stage 4 animated scroll target: middle");
            });
        }
        {
            let animation_scroll_status = animation_scroll_status.clone();
            let animation_scroll_box = animation_scroll_box.clone();
            animation_bottom_button.on_click(move |_event| {
                animation_scroll_box.scroll_to_animated(
                    0.0,
                    690.0,
                    AnimationTiming::with_easing(420.0, Easings::cubic_out()),
                );
                animation_scroll_status.text("Stage 4 animated scroll target: bottom");
                animation_scroll_status.semantic_label("Stage 4 animated scroll target: bottom");
            });
        }
        {
            let animation_scroll_status = animation_scroll_status.clone();
            let animation_scroll_box = animation_scroll_box.clone();
            animation_tail_button.on_click(move |_event| {
                animation_scroll_box.scroll_to_animated(
                    0.0,
                    960.0,
                    AnimationTiming::with_easing(420.0, Easings::cubic_out()),
                );
                animation_scroll_status.text("Stage 4 animated scroll target: tail");
                animation_scroll_status.semantic_label("Stage 4 animated scroll target: tail");
            });
        }
        let active_worker = Rc::new(RefCell::new(None::<Worker>));
        let set_worker_status: Rc<dyn Fn(String)> = {
            let worker_status = worker_status.clone();
            let worker_status_value = worker_status_value.clone();
            Rc::new(move |label: String| {
                *worker_status_value.borrow_mut() = label.clone();
                worker_status.text(&label);
                worker_status.semantic_label(label);
            })
        };
        let set_worker_detail: Rc<dyn Fn(String)> = {
            let worker_detail = worker_detail.clone();
            let worker_detail_value = worker_detail_value.clone();
            Rc::new(move |label: String| {
                *worker_detail_value.borrow_mut() = label.clone();
                worker_detail.text(&label);
                worker_detail.semantic_label(label);
            })
        };
        let start_prime_worker_action: Rc<dyn Fn()> = {
            let active_worker = active_worker.clone();
            let worker_progress = worker_progress.clone();
            let set_worker_status = set_worker_status.clone();
            let set_worker_detail = set_worker_detail.clone();
            Rc::new(move || {
                if active_worker.borrow().is_some() {
                    return;
                }
                worker_progress.value(0.0);
                set_worker_status(String::from("Stage 4 worker status: running"));
                set_worker_detail(String::from(
                    "Stage 4 worker detail: running • waiting for progress",
                ));
                let worker = Worker::new("./workers.wasm", "stage4PrimeWorker")
                    .on_progress({
                        let worker_progress = worker_progress.clone();
                        let set_worker_status = set_worker_status.clone();
                        let set_worker_detail = set_worker_detail.clone();
                        move |event| {
                            let percent = event.message.parse::<f32>().unwrap_or(0.0);
                            worker_progress.value(percent);
                            set_worker_status(format!(
                                "Stage 4 worker status: running • {:.0}%",
                                percent
                            ));
                            set_worker_detail(format!(
                                "Stage 4 worker detail: progress • {:.0}%",
                                percent
                            ));
                        }
                    })
                    .on_complete({
                        let active_worker = active_worker.clone();
                        let worker_progress = worker_progress.clone();
                        let set_worker_status = set_worker_status.clone();
                        let set_worker_detail = set_worker_detail.clone();
                        move |event| {
                            active_worker.borrow_mut().take();
                            worker_progress.value(100.0);
                            set_worker_status(String::from("Stage 4 worker status: complete"));
                            set_worker_detail(format!(
                                "Stage 4 worker detail: complete • {}",
                                event.result
                            ));
                        }
                    })
                    .on_error({
                        let active_worker = active_worker.clone();
                        let worker_progress = worker_progress.clone();
                        let set_worker_status = set_worker_status.clone();
                        let set_worker_detail = set_worker_detail.clone();
                        move |event| {
                            active_worker.borrow_mut().take();
                            if let Some(cancelled) = event.message.strip_prefix("cancelled:") {
                                let percent = cancelled.parse::<f32>().unwrap_or(0.0);
                                worker_progress.value(percent);
                                set_worker_status(String::from("Stage 4 worker status: cancelled"));
                                set_worker_detail(format!(
                                    "Stage 4 worker detail: cancelled • {:.0}%",
                                    percent
                                ));
                            } else {
                                worker_progress.value(0.0);
                                set_worker_status(String::from("Stage 4 worker status: error"));
                                set_worker_detail(format!(
                                    "Stage 4 worker detail: error • {}",
                                    event.message
                                ));
                            }
                        }
                    })
                    .start("stage4-workers");
                *active_worker.borrow_mut() = Some(worker);
            })
        };
        let start_fail_worker_action: Rc<dyn Fn()> = {
            let active_worker = active_worker.clone();
            let worker_progress = worker_progress.clone();
            let set_worker_status = set_worker_status.clone();
            let set_worker_detail = set_worker_detail.clone();
            Rc::new(move || {
                if active_worker.borrow().is_some() {
                    return;
                }
                worker_progress.value(0.0);
                set_worker_status(String::from("Stage 4 worker status: running"));
                set_worker_detail(String::from(
                    "Stage 4 worker detail: starting failing worker",
                ));
                let worker = Worker::new("./workers.wasm", "stage4FailWorker")
                    .on_complete({
                        let active_worker = active_worker.clone();
                        let set_worker_status = set_worker_status.clone();
                        let set_worker_detail = set_worker_detail.clone();
                        move |event| {
                            active_worker.borrow_mut().take();
                            set_worker_status(String::from("Stage 4 worker status: complete"));
                            set_worker_detail(format!(
                                "Stage 4 worker detail: complete • {}",
                                event.result
                            ));
                        }
                    })
                    .on_error({
                        let active_worker = active_worker.clone();
                        let set_worker_status = set_worker_status.clone();
                        let set_worker_detail = set_worker_detail.clone();
                        move |event| {
                            active_worker.borrow_mut().take();
                            set_worker_status(String::from("Stage 4 worker status: error"));
                            set_worker_detail(format!(
                                "Stage 4 worker detail: error • {}",
                                event.message
                            ));
                        }
                    })
                    .start("stage4-fail");
                *active_worker.borrow_mut() = Some(worker);
            })
        };
        {
            let start_prime_worker_action = start_prime_worker_action.clone();
            worker_start_button.on_click(move |_event| {
                start_prime_worker_action();
            });
        }
        {
            let active_worker = active_worker.clone();
            let set_worker_status = set_worker_status.clone();
            let set_worker_detail = set_worker_detail.clone();
            worker_cancel_button.on_click(move |_event| {
                let active = active_worker.borrow();
                let Some(worker) = active.as_ref() else {
                    return;
                };
                set_worker_status(String::from("Stage 4 worker status: cancelling"));
                set_worker_detail(String::from(
                    "Stage 4 worker detail: waiting for cooperative cancellation",
                ));
                worker.cancel();
            });
        }
        {
            let start_fail_worker_action = start_fail_worker_action.clone();
            worker_fail_button.on_click(move |_event| {
                start_fail_worker_action();
            });
        }
        let pending_file_guards = Rc::new(RefCell::new(Vec::<FileRequestGuard>::new()));
        let active_file_copy = Rc::new(RefCell::new(None::<FileWorkerProcessRequest>));
        let picked_file = Rc::new(RefCell::new(None::<BrowserFile>));
        let set_file_status: Rc<dyn Fn(String)> = {
            let file_status = file_status.clone();
            Rc::new(move |label: String| {
                file_status.text(&label);
                file_status.semantic_label(label);
            })
        };
        let set_file_detail: Rc<dyn Fn(String)> = {
            let file_detail = file_detail.clone();
            Rc::new(move |label: String| {
                file_detail.text(&label);
                file_detail.semantic_label(label);
            })
        };
        let set_picked_file_label: Rc<dyn Fn(String)> = {
            let file_picked = file_picked.clone();
            Rc::new(move |label: String| {
                file_picked.text(&label);
                file_picked.semantic_label(label);
            })
        };
        {
            let active_fetch_request = active_fetch_request.clone();
            let active_fetch_label = active_fetch_label.clone();
            let fetch_get_button_for_get = fetch_get_button.clone();
            let fetch_post_button_for_get = fetch_post_button.clone();
            let fetch_status = fetch_status.clone();
            let fetch_status_value = fetch_status_value.clone();
            let fetch_request = fetch_request.clone();
            let fetch_result = fetch_result.clone();
            fetch_get_button.on_click(move |_event| {
                if active_fetch_request.borrow().is_some() {
                    let label =
                        String::from("Fetch status: wait for the current request to finish");
                    fetch_status_value.replace(label.clone());
                    fetch_status.text(&label);
                    fetch_status.semantic_label(label);
                    return;
                }
                let request_label = format!("GET {}", JSON_PLACEHOLDER_GET_URL);
                active_fetch_label.replace(request_label.clone());
                fetch_status_value.replace(String::from("Fetch status: running"));
                fetch_status.text("Fetch status: running");
                fetch_status.semantic_label("Fetch status: running");
                fetch_request.text(format!("Latest request: {}", request_label));
                fetch_request.semantic_label(format!("Latest request: {}", request_label));
                fetch_result.text("Latest result: waiting for JSONPlaceholder to respond...");
                fetch_result
                    .semantic_label("Latest result: waiting for JSONPlaceholder to respond...");
                fetch_get_button_for_get.enabled(false);
                fetch_post_button_for_get.enabled(false);
                let request = Fetch::request(JSON_PLACEHOLDER_GET_URL)
                    .on_complete({
                        let active_fetch_request = active_fetch_request.clone();
                        let active_fetch_label = active_fetch_label.clone();
                        let fetch_get_button = fetch_get_button_for_get.clone();
                        let fetch_post_button = fetch_post_button_for_get.clone();
                        let fetch_status = fetch_status.clone();
                        let fetch_status_value = fetch_status_value.clone();
                        let fetch_result = fetch_result.clone();
                        move |response| {
                            active_fetch_request.borrow_mut().take();
                            let request_label = {
                                let value = active_fetch_label.borrow();
                                if value.is_empty() {
                                    String::from("Fetch request")
                                } else {
                                    value.clone()
                                }
                            };
                            active_fetch_label.replace(String::new());
                            fetch_status_value.replace(String::from("Fetch status: complete"));
                            fetch_status.text("Fetch status: complete");
                            fetch_status.semantic_label("Fetch status: complete");
                            let label = format!(
                                "Latest result: {} -> ok={} • status {} {} • resolved url {}",
                                request_label,
                                if response.ok { "true" } else { "false" },
                                response.status,
                                response.status_text,
                                response.url
                            );
                            fetch_result.text(&label);
                            fetch_result.semantic_label(label);
                            fetch_get_button.enabled(true);
                            fetch_post_button.enabled(true);
                        }
                    })
                    .on_error({
                        let active_fetch_request = active_fetch_request.clone();
                        let active_fetch_label = active_fetch_label.clone();
                        let fetch_get_button = fetch_get_button_for_get.clone();
                        let fetch_post_button = fetch_post_button_for_get.clone();
                        let fetch_status = fetch_status.clone();
                        let fetch_status_value = fetch_status_value.clone();
                        let fetch_result = fetch_result.clone();
                        move |event| {
                            active_fetch_request.borrow_mut().take();
                            let request_label = {
                                let value = active_fetch_label.borrow();
                                if value.is_empty() {
                                    String::from("Fetch request")
                                } else {
                                    value.clone()
                                }
                            };
                            active_fetch_label.replace(String::new());
                            fetch_status_value.replace(String::from("Fetch status: error"));
                            fetch_status.text("Fetch status: error");
                            fetch_status.semantic_label("Fetch status: error");
                            let label = format!(
                                "Latest result: {} -> error • {}",
                                request_label, event.message
                            );
                            fetch_result.text(&label);
                            fetch_result.semantic_label(label);
                            fetch_get_button.enabled(true);
                            fetch_post_button.enabled(true);
                        }
                    })
                    .start();
                active_fetch_request.borrow_mut().replace(request);
            });
        }
        {
            let active_fetch_request = active_fetch_request.clone();
            let active_fetch_label = active_fetch_label.clone();
            let fetch_get_button_for_post = fetch_get_button.clone();
            let fetch_post_button_for_post = fetch_post_button.clone();
            let fetch_status = fetch_status.clone();
            let fetch_status_value = fetch_status_value.clone();
            let fetch_request = fetch_request.clone();
            let fetch_result = fetch_result.clone();
            fetch_post_button.on_click(move |_event| {
                if active_fetch_request.borrow().is_some() {
                    let label =
                        String::from("Fetch status: wait for the current request to finish");
                    fetch_status_value.replace(label.clone());
                    fetch_status.text(&label);
                    fetch_status.semantic_label(label);
                    return;
                }
                let request_label = format!("POST {}", JSON_PLACEHOLDER_POST_URL);
                active_fetch_label.replace(request_label.clone());
                fetch_status_value.replace(String::from("Fetch status: running"));
                fetch_status.text("Fetch status: running");
                fetch_status.semantic_label("Fetch status: running");
                fetch_request.text(format!("Latest request: {}", request_label));
                fetch_request.semantic_label(format!("Latest request: {}", request_label));
                fetch_result.text("Latest result: waiting for JSONPlaceholder to respond...");
                fetch_result
                    .semantic_label("Latest result: waiting for JSONPlaceholder to respond...");
                fetch_get_button_for_post.enabled(false);
                fetch_post_button_for_post.enabled(false);
                let request = Fetch::request(JSON_PLACEHOLDER_POST_URL)
                    .method("POST")
                    .header("Content-Type", "application/json; charset=UTF-8")
                    .body_text(JSON_PLACEHOLDER_POST_BODY)
                    .on_complete({
                        let active_fetch_request = active_fetch_request.clone();
                        let active_fetch_label = active_fetch_label.clone();
                        let fetch_get_button = fetch_get_button_for_post.clone();
                        let fetch_post_button = fetch_post_button_for_post.clone();
                        let fetch_status = fetch_status.clone();
                        let fetch_status_value = fetch_status_value.clone();
                        let fetch_result = fetch_result.clone();
                        move |response| {
                            active_fetch_request.borrow_mut().take();
                            let request_label = {
                                let value = active_fetch_label.borrow();
                                if value.is_empty() {
                                    String::from("Fetch request")
                                } else {
                                    value.clone()
                                }
                            };
                            active_fetch_label.replace(String::new());
                            fetch_status_value.replace(String::from("Fetch status: complete"));
                            fetch_status.text("Fetch status: complete");
                            fetch_status.semantic_label("Fetch status: complete");
                            let label = format!(
                                "Latest result: {} -> ok={} • status {} {} • resolved url {}",
                                request_label,
                                if response.ok { "true" } else { "false" },
                                response.status,
                                response.status_text,
                                response.url
                            );
                            fetch_result.text(&label);
                            fetch_result.semantic_label(label);
                            fetch_get_button.enabled(true);
                            fetch_post_button.enabled(true);
                        }
                    })
                    .on_error({
                        let active_fetch_request = active_fetch_request.clone();
                        let active_fetch_label = active_fetch_label.clone();
                        let fetch_get_button = fetch_get_button_for_post.clone();
                        let fetch_post_button = fetch_post_button_for_post.clone();
                        let fetch_status = fetch_status.clone();
                        let fetch_status_value = fetch_status_value.clone();
                        let fetch_result = fetch_result.clone();
                        move |event| {
                            active_fetch_request.borrow_mut().take();
                            let request_label = {
                                let value = active_fetch_label.borrow();
                                if value.is_empty() {
                                    String::from("Fetch request")
                                } else {
                                    value.clone()
                                }
                            };
                            active_fetch_label.replace(String::new());
                            fetch_status_value.replace(String::from("Fetch status: error"));
                            fetch_status.text("Fetch status: error");
                            fetch_status.semantic_label("Fetch status: error");
                            let label = format!(
                                "Latest result: {} -> error • {}",
                                request_label, event.message
                            );
                            fetch_result.text(&label);
                            fetch_result.semantic_label(label);
                            fetch_get_button.enabled(true);
                            fetch_post_button.enabled(true);
                        }
                    })
                    .start();
                active_fetch_request.borrow_mut().replace(request);
            });
        }
        {
            let pending_file_guards = pending_file_guards.clone();
            let picked_file = picked_file.clone();
            let set_file_status = set_file_status.clone();
            let set_file_detail = set_file_detail.clone();
            let set_picked_file_label = set_picked_file_label.clone();
            pick_file_button.on_click(move |_event| {
                set_file_status(String::from("Stage 4 file status: opening picker"));
                set_file_detail(String::from(
                    "Stage 4 file detail: waiting for browser selection",
                ));
                let guard = File::open().multiple(false).pick_with_error(
                    {
                        let picked_file = picked_file.clone();
                        let set_file_status = set_file_status.clone();
                        let set_file_detail = set_file_detail.clone();
                        let set_picked_file_label = set_picked_file_label.clone();
                        move |event| {
                            let file = event.files.into_iter().next();
                            if let Some(file) = file {
                                let label = format!(
                                    "Picked file: {} • {} bytes",
                                    file.name(),
                                    file.size_bytes()
                                );
                                *picked_file.borrow_mut() = Some(file.clone());
                                set_picked_file_label(label);
                                set_file_status(String::from("Stage 4 file status: picked"));
                                set_file_detail(String::from(
                                    "Stage 4 file detail: BrowserFile handle ready for worker copy",
                                ));
                            } else {
                                set_file_status(String::from("Stage 4 file status: idle"));
                                set_file_detail(String::from(
                                    "Stage 4 file detail: picker returned no files",
                                ));
                            }
                        }
                    },
                    Some({
                        let set_file_status = set_file_status.clone();
                        let set_file_detail = set_file_detail.clone();
                        move |event: FileErrorEventArgs| {
                            set_file_status(String::from("Stage 4 file status: picker error"));
                            set_file_detail(format!("Stage 4 file detail: {}", event.message));
                        }
                    }),
                );
                pending_file_guards.borrow_mut().push(guard);
            });
        }
        {
            let pending_file_guards = pending_file_guards.clone();
            let set_file_status = set_file_status.clone();
            let set_file_detail = set_file_detail.clone();
            save_text_button.on_click(move |_event| {
                set_file_status(String::from("Stage 4 file status: saving text"));
                set_file_detail(String::from(
                    "Stage 4 file detail: sample text save request started",
                ));
                let guard = File::save()
                    .suggested_name("stage4-note")
                    .mime_type("text/plain")
                    .file_extension(".txt")
                    .save_text_with_error(
                        "Stage 4 FUI-RS file bridge sample text.",
                        {
                            let set_file_status = set_file_status.clone();
                            let set_file_detail = set_file_detail.clone();
                            move |result| {
                                set_file_status(String::from("Stage 4 file status: saved text"));
                                set_file_detail(format!(
                                    "Stage 4 file detail: {} • {} bytes",
                                    result.file_name, result.written_bytes
                                ));
                            }
                        },
                        Some({
                            let set_file_status = set_file_status.clone();
                            let set_file_detail = set_file_detail.clone();
                            move |event: FileErrorEventArgs| {
                                set_file_status(String::from(
                                    "Stage 4 file status: save text error",
                                ));
                                set_file_detail(format!("Stage 4 file detail: {}", event.message));
                            }
                        }),
                    );
                pending_file_guards.borrow_mut().push(guard);
            });
        }
        {
            let pending_file_guards = pending_file_guards.clone();
            let set_file_status = set_file_status.clone();
            let set_file_detail = set_file_detail.clone();
            save_bytes_button.on_click(move |_event| {
                set_file_status(String::from("Stage 4 file status: saving bytes"));
                set_file_detail(String::from(
                    "Stage 4 file detail: sample bytes save request started",
                ));
                let bytes = b"stage4-bytes".to_vec();
                let guard = File::save()
                    .suggested_name("stage4-bytes")
                    .mime_type("application/octet-stream")
                    .file_extension(".bin")
                    .save_bytes_with_error(
                        &bytes,
                        {
                            let set_file_status = set_file_status.clone();
                            let set_file_detail = set_file_detail.clone();
                            move |result| {
                                set_file_status(String::from("Stage 4 file status: saved bytes"));
                                set_file_detail(format!(
                                    "Stage 4 file detail: {} • {} bytes",
                                    result.file_name, result.written_bytes
                                ));
                            }
                        },
                        Some({
                            let set_file_status = set_file_status.clone();
                            let set_file_detail = set_file_detail.clone();
                            move |event: FileErrorEventArgs| {
                                set_file_status(String::from(
                                    "Stage 4 file status: save bytes error",
                                ));
                                set_file_detail(format!("Stage 4 file detail: {}", event.message));
                            }
                        }),
                    );
                pending_file_guards.borrow_mut().push(guard);
            });
        }
        {
            let active_file_copy = active_file_copy.clone();
            let picked_file = picked_file.clone();
            let set_file_status = set_file_status.clone();
            let set_file_detail = set_file_detail.clone();
            copy_file_button.on_click(move |_event| {
                if active_file_copy.borrow().is_some() {
                    set_file_status(String::from("Stage 4 file status: worker already running"));
                    set_file_detail(String::from(
                        "Stage 4 file detail: wait for the active copy request",
                    ));
                    return;
                }
                let Some(file) = picked_file.borrow().clone() else {
                    set_file_status(String::from("Stage 4 file status: no picked file"));
                    set_file_detail(String::from("Stage 4 file detail: pick a file first"));
                    return;
                };
                set_file_status(String::from("Stage 4 file status: worker copying"));
                set_file_detail(format!(
                    "Stage 4 file detail: starting worker copy for {}",
                    file.name()
                ));
                let request = File::process_file_in_worker(file.clone())
                    .worker("./workers.wasm", "stage4FileProcessorWorker")
                    .save_to_picked_file(format!("{}-copy", file.name()))
                    .on_progress({
                        let set_file_status = set_file_status.clone();
                        let set_file_detail = set_file_detail.clone();
                        move |progress| {
                            set_file_status(String::from("Stage 4 file status: worker copying"));
                            set_file_detail(format!(
                                "Stage 4 file detail: {} / {} bytes",
                                progress.processed_bytes, progress.total_bytes
                            ));
                        }
                    })
                    .on_complete({
                        let active_file_copy = active_file_copy.clone();
                        let set_file_status = set_file_status.clone();
                        let set_file_detail = set_file_detail.clone();
                        move |result| {
                            active_file_copy.borrow_mut().take();
                            set_file_status(String::from("Stage 4 file status: worker complete"));
                            set_file_detail(format!(
                                "Stage 4 file detail: {} • {}",
                                result
                                    .output_file_name
                                    .unwrap_or_else(|| String::from("(stream)")),
                                result
                                    .worker_result
                                    .unwrap_or_else(|| String::from("no worker result"))
                            ));
                        }
                    })
                    .on_error({
                        let active_file_copy = active_file_copy.clone();
                        let set_file_status = set_file_status.clone();
                        let set_file_detail = set_file_detail.clone();
                        move |event: FileErrorEventArgs| {
                            active_file_copy.borrow_mut().take();
                            set_file_status(String::from("Stage 4 file status: worker error"));
                            set_file_detail(format!("Stage 4 file detail: {}", event.message));
                        }
                    })
                    .start();
                *active_file_copy.borrow_mut() = Some(request);
            });
        }
        let worker_test_api = Stage4WorkerTestApi {
            start_prime: start_prime_worker_action.clone(),
            start_fail: start_fail_worker_action.clone(),
            status: Rc::new({
                let worker_status_value = worker_status_value.clone();
                move || worker_status_value.borrow().clone()
            }),
            detail: Rc::new({
                let worker_detail_value = worker_detail_value.clone();
                move || worker_detail_value.borrow().clone()
            }),
        };

        on_loaded({
            let loaded_status = loaded_status.clone();
            let drawing = drawing.clone();
            move |_| {
                let label = format!(
                    "Stage 4 loaded status: ready at {:.2} dpr",
                    device_pixel_ratio()
                );
                loaded_status.text(&label);
                loaded_status.semantic_label(label);
                schedule_immediate_draw_tick(drawing.clone());
            }
        });

        root.bind_theme({
            let vertical_scrollbar = root.vertical_scrollbar();
            let horizontal_scrollbar = root.horizontal_scrollbar();
            let page = page.clone();
            let accent_chip = accent_chip.clone();
            let theme_body = theme_body.clone();
            let theme_card = theme_card.clone();
            let responsive_card = responsive_card.clone();
            let probe = probe.clone();
            let drawing = drawing.clone();
            let scroll = scroll.clone();
            let typography_card = typography_card.clone();
            let custom_font_heading = custom_font_heading.clone();
            let custom_font_body = custom_font_body.clone();
            let custom_font_direct_stack = custom_font_direct_stack.clone();
            let custom_font_comparison = custom_font_comparison.clone();
            let on_theme_accent_changed = on_theme_accent_changed.clone();
            move |root, theme| {
                root.bg_color(theme.colors.background);
                page.bg_color(theme.colors.background);
                vertical_scrollbar
                    .track_color(theme.colors.scrollbar_track)
                    .thumb_color(theme.colors.scrollbar_thumb);
                horizontal_scrollbar
                    .track_color(theme.colors.scrollbar_track)
                    .thumb_color(theme.colors.scrollbar_thumb);
                accent_chip.bg_color(theme.colors.accent);
                theme_body.text(format!("Theme accent: {}", color_hex(theme.colors.accent)));
                theme_card.border(1.0, theme.colors.border);
                responsive_card.border_config(Border {
                    width: 1.0,
                    color: theme.colors.border,
                    style: BorderStyle::Dashed,
                    dash_on: 7.0,
                    dash_off: 5.0,
                });
                probe
                    .bg_color(demo_probe_surface(&theme))
                    .border(1.0, theme.colors.border);
                drawing
                    .bg_color(demo_subtle_surface(&theme))
                    .border(1.0, theme.colors.border);
                scroll
                    .bg_color(demo_subtle_surface(&theme))
                    .border(1.0, theme.colors.border);
                scroll
                    .vertical_scrollbar()
                    .track_color(theme.colors.scrollbar_track)
                    .thumb_color(theme.colors.scrollbar_thumb);
                scroll
                    .horizontal_scrollbar()
                    .track_color(theme.colors.scrollbar_track)
                    .thumb_color(theme.colors.scrollbar_thumb);
                typography_card.border(1.0, theme.colors.border);
                custom_font_heading.text_color(theme.colors.text_primary);
                custom_font_body.text_color(theme.colors.text_muted);
                custom_font_direct_stack.text_color(theme.colors.text_muted);
                custom_font_comparison.text_color(theme.colors.text_primary);
                on_theme_accent_changed(theme.colors.accent);
            }
        });
        let viewport_width_guard = viewport_width_signal().subscribe({
            let apply_breakpoint = apply_breakpoint.clone();
            move |width| {
                apply_breakpoint(width, viewport_height_signal().value());
            }
        });
        let viewport_height_guard = viewport_height_signal().subscribe({
            let apply_breakpoint = apply_breakpoint.clone();
            move |height| {
                apply_breakpoint(viewport_width_signal().value(), height);
            }
        });
        let selection_guard = selection.subscribe_selected_text({
            let selection_status = selection_status.clone();
            move |value| {
                if value.is_empty() {
                    selection_status.text("Selected text: none");
                } else {
                    selection_status.text(format!("Selected text: {}", value));
                }
            }
        });
        let route_relative_texture_guard =
            get_texture_asset_state(STAGE4_ROUTE_RELATIVE_TEXTURE_ID).subscribe(Rc::new({
                let route_relative_state = route_relative_state.clone();
                let drawing = drawing.clone();
                let drawing_texture_status = drawing_texture_status.clone();
                move || {
                    let state = get_texture_asset_state(STAGE4_ROUTE_RELATIVE_TEXTURE_ID).get();
                    let label = if state == AssetLoadState::Ready {
                        format!(
                            "Stage 4 route-relative image state: Ready {:.0}x{:.0}",
                            get_texture_asset_width(STAGE4_ROUTE_RELATIVE_TEXTURE_ID),
                            get_texture_asset_height(STAGE4_ROUTE_RELATIVE_TEXTURE_ID)
                        )
                    } else {
                        format!(
                            "Stage 4 route-relative image state: {}",
                            asset_state_name(state)
                        )
                    };
                    route_relative_state.text(&label);
                    route_relative_state.semantic_label(label);
                    let drawing_label = if state == AssetLoadState::Ready {
                        "Stage 4 direct texture draw status: Ready".to_string()
                    } else {
                        format!(
                            "Stage 4 direct texture draw status: {}",
                            asset_state_name(state)
                        )
                    };
                    drawing_texture_status.text(&drawing_label);
                    drawing_texture_status.semantic_label(drawing_label);
                    drawing.mark_dirty();
                }
            }));
        let explicit_svg_guard = get_svg_asset_state(STAGE4_SAMPLE_SVG_ID).subscribe(Rc::new({
            let explicit_svg_state = explicit_svg_state.clone();
            let drawing = drawing.clone();
            let drawing_svg_status = drawing_svg_status.clone();
            move || {
                let state = get_svg_asset_state(STAGE4_SAMPLE_SVG_ID).get();
                let label = if state == AssetLoadState::Ready {
                    format!(
                        "Stage 4 explicit SVG state: Ready {:.0}x{:.0}",
                        get_svg_asset_width(STAGE4_SAMPLE_SVG_ID),
                        get_svg_asset_height(STAGE4_SAMPLE_SVG_ID)
                    )
                } else {
                    format!("Stage 4 explicit SVG state: {}", asset_state_name(state))
                };
                explicit_svg_state.text(&label);
                explicit_svg_state.semantic_label(label);
                let drawing_label = if state == AssetLoadState::Ready {
                    "Stage 4 direct SVG draw status: Ready".to_string()
                } else {
                    format!(
                        "Stage 4 direct SVG draw status: {}",
                        asset_state_name(state)
                    )
                };
                drawing_svg_status.text(&drawing_label);
                drawing_svg_status.semantic_label(drawing_label);
                drawing.mark_dirty();
            }
        }));
        let missing_texture_guard =
            get_texture_asset_state(STAGE4_MISSING_TEXTURE_ID).subscribe(Rc::new({
                let missing_texture_state = missing_texture_state.clone();
                move || {
                    let state = get_texture_asset_state(STAGE4_MISSING_TEXTURE_ID).get();
                    let label = if state == AssetLoadState::Failed {
                        format!(
                            "Stage 4 missing texture state: Failed {}",
                            get_texture_asset_error(STAGE4_MISSING_TEXTURE_ID)
                        )
                    } else {
                        format!("Stage 4 missing texture state: {}", asset_state_name(state))
                    };
                    missing_texture_state.text(&label);
                    missing_texture_state.semantic_label(label);
                }
            }));
        let missing_svg_guard = get_svg_asset_state(STAGE4_MISSING_SVG_ID).subscribe(Rc::new({
            let missing_svg_state = missing_svg_state.clone();
            move || {
                let state = get_svg_asset_state(STAGE4_MISSING_SVG_ID).get();
                let label = if state == AssetLoadState::Failed {
                    format!(
                        "Stage 4 missing SVG state: Failed {}",
                        get_svg_asset_error(STAGE4_MISSING_SVG_ID)
                    )
                } else {
                    format!("Stage 4 missing SVG state: {}", asset_state_name(state))
                };
                missing_svg_state.text(&label);
                missing_svg_state.semantic_label(label);
            }
        }));

        Self {
            root,
            worker_test_api,
            _external_drop_panel: external_drop_panel,
            _reorder_panel: reorder_panel,
            _guards: vec![
                viewport_width_guard,
                viewport_height_guard,
                selection_guard,
                route_relative_texture_guard,
                explicit_svg_guard,
                missing_texture_guard,
                missing_svg_guard,
            ],
        }
    }
}

fn demo_route_nav_link(href: &str, label: &str, active: bool) -> NavLink {
    let theme = current_theme();
    let link = ui! {
    nav_link(href).text(label)
        .padding(12.0, 8.0, 12.0, 8.0)
        .corner_radius(999.0)
        .bg_color(if active {
            theme.colors.accent_hovered
        } else {
            theme.colors.surface
        })
        .border(
            1.0,
            if active {
                theme.colors.accent
            } else {
                theme.colors.border
            },
        )
    };
    link.bind_theme({
        move |link, theme| {
            link.bg_color(if active {
                theme.colors.accent_hovered
            } else {
                theme.colors.surface
            })
            .border(
                1.0,
                if active {
                    theme.colors.accent
                } else {
                    theme.colors.border
                },
            );
        }
    });
    link
}

pub fn demo_page_root(title: &str) -> FlexBox {
    let theme = current_theme();
    let root = ui! {
        column().width_len(fill())
        .height_len(fill())
        .padding(32.0, 32.0, 32.0, 32.0)
        .bg_color(theme.colors.background)
    };

    let nav = ui! {
        row().width_len(fill())
        .align_items(AlignItems::Center)
        .margin(0.0, 0.0, 0.0, 18.0)
    };
    let dashboard_link =
        demo_route_nav_link(demo_home_route(), "Dashboard", title.contains("dashboard"));
    let workbench_link = demo_route_nav_link(
        demo_workbench_route(),
        "Workbench",
        title.contains("workbench"),
    );
    workbench_link.margin(10.0, 0.0, 0.0, 0.0);
    let stage4_link = demo_route_nav_link(
        demo_stage4_route(),
        "Stage 4",
        title.contains("Stage 4") || title.contains("stage 4"),
    );
    stage4_link.margin(10.0, 0.0, 0.0, 0.0);
    let stage5_link = demo_route_nav_link(
        demo_stage5_route(),
        "Stage 5",
        title.contains("Stage 5") || title.contains("stage 5"),
    );
    stage5_link.margin(10.0, 0.0, 0.0, 0.0);
    let immediate_drawing_link = demo_route_nav_link(
        demo_immediate_drawing_route(),
        "Immediate Drawing",
        title.contains("Immediate Drawing"),
    );
    immediate_drawing_link.margin(10.0, 0.0, 0.0, 0.0);
    nav.child(&dashboard_link)
        .child(&workbench_link)
        .child(&stage4_link)
        .child(&stage5_link)
        .child(&immediate_drawing_link);
    root.child(&nav);

    let title_node = ui! {
    text(title)
        .font_family(theme.fonts.heading_family.clone())
        .font_size(28.0)
        .text_color(theme.colors.text_primary)
    };
    root.child(&title_node);
    root.bind_theme({
        let title_node = title_node.clone();
        move |root, theme| {
            root.bg_color(theme.colors.background);
            title_node
                .font_family(theme.fonts.heading_family.clone())
                .font_size(28.0)
                .text_color(theme.colors.text_primary);
        }
    });
    root
}

pub fn demo_card(title: &str, body: &str, color: u32) -> FlexBox {
    let theme = current_theme();
    let card = ui! {
        column().width_len(fill())
        .padding(18.0, 20.0, 18.0, 20.0)
        .bg_color(demo_card_color(&theme, color))
        .corner_radius(18.0)
        .border(1.0, theme.colors.border)
    };

    let title_node = ui! {
    text(title)
        .font_family(theme.fonts.body_family.clone())
        .font_size(18.0)
        .text_color(theme.colors.text_primary)
    };
    card.child(&title_node);

    let body_node = ui! {
    text(body)
        .font_family(theme.fonts.body_family.clone())
        .font_size(15.0)
        .text_color(theme.colors.text_muted)
    };
    card.child(&spacer(8.0)).child(&body_node);
    card.bind_theme({
        let title_node = title_node.clone();
        let body_node = body_node.clone();
        move |card, theme| {
            card.bg_color(demo_card_color(&theme, color))
                .border(1.0, theme.colors.border);
            title_node
                .font_family(theme.fonts.body_family.clone())
                .font_size(18.0)
                .text_color(theme.colors.text_primary);
            body_node
                .font_family(theme.fonts.body_family.clone())
                .font_size(15.0)
                .text_color(theme.colors.text_muted);
        }
    });

    card
}

pub(crate) fn stage4_panel(title: &str, color: u32) -> FlexBox {
    let theme = current_theme();
    let panel = ui! {
        column()
        .padding(18.0, 20.0, 18.0, 20.0)
        .bg_color(demo_card_color(&theme, color))
        .corner_radius(22.0)
        .border(1.0, theme.colors.border)
    };
    let title_node = demo_text(title, 12.0, 0x64748BFF);
    panel.child(&title_node).child(&spacer(10.0));
    panel.bind_theme({
        move |panel, theme| {
            panel
                .bg_color(demo_card_color(&theme, color))
                .border(1.0, theme.colors.border);
        }
    });
    panel
}

fn schedule_immediate_draw_tick(drawing: CustomDrawable) {
    set_timeout(STAGE4_DRAW_TICK_MS, move || {
        drawing.mark_dirty();
        schedule_immediate_draw_tick(drawing.clone());
    });
}

pub(crate) fn demo_text(content: &str, size: f32, color: u32) -> TextNode {
    let theme = current_theme();
    let node = ui! {
    text(content).font_family(theme.fonts.body_family.clone())
        .font_size(size)
        .text_color(demo_text_color(&theme, color))
    };
    node.bind_theme({
        move |node, theme| {
            node.font_family(theme.fonts.body_family.clone())
                .font_size(size)
                .text_color(demo_text_color(&theme, color));
        }
    });
    node
}

pub(crate) fn spacer(height: f32) -> FlexBox {
    let node = ui! {
        flex_box().width_len(fill()).height_len(px(height))
    };
    node
}

fn color_hex(color: u32) -> String {
    format!("#{:08X}", color)
}
