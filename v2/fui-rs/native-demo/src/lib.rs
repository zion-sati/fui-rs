use fui::prelude::*;
use fui::AssetLoadState;
use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct NativeApplicationState {
    activations: Cell<u32>,
    ui_dispatches: Cell<u32>,
    file_dialog_result: RefCell<String>,
    drop_result: RefCell<String>,
    drop_completed: Cell<bool>,
    test_image: RefCell<Option<ImageNode>>,
    custom_draw_calls: Cell<u32>,
    custom_draw_features: Cell<u32>,
    custom_draw_phase: Cell<u32>,
    timer_fires: Cell<u32>,
    custom_draw_timer: RefCell<Option<TimerHandle>>,
    initial_raster_timer: RefCell<Option<TimerHandle>>,
    animation_running: Cell<bool>,
    animation_steps: Cell<u32>,
    bitmap_full_uploads: Cell<u32>,
    bitmap_dirty_uploads: Cell<u32>,
    bitmap_clears: Cell<u32>,
    offscreen_compositions: Cell<u32>,
    offscreen_sample_rgba: Cell<u32>,
    retained_rasters: Cell<u32>,
    dark_mode: Cell<bool>,
    active_worker: RefCell<Option<Worker>>,
    worker_progress: Cell<f32>,
    worker_status: Cell<u32>,
    worker_detail: RefCell<String>,
    worker_execution_thread: Arc<Mutex<Option<std::thread::ThreadId>>>,
    worker_callback_thread: RefCell<Option<std::thread::ThreadId>>,
    worker_ui_thread: RefCell<Option<std::thread::ThreadId>>,
    theme_guards: RefCell<Vec<Subscription>>,
}

struct NativeApplication {
    root: FlexBox,
    scroll_root: ScrollBox,
    action_button: Button,
    body_text: Text,
    system_fallback_text: Text,
    selection_text: Text,
    click_text: Text,
    context_link: NavLink,
    context_image: ImageNode,
    context_svg: SvgNode,
    context_editor: TextInput,
    custom_drawable: CustomDrawable,
    bitmap_drawable: CustomDrawable,
    offscreen_drawable: CustomDrawable,
    retained_drawable: CustomDrawable,
    custom_draw_text: Text,
    animation_status: Text,
    bitmap_status: Text,
    offscreen_status: Text,
    retained_status: Text,
    waveform_bitmap: Bitmap,
    offscreen_bitmap: Bitmap,
    retained_bitmap: Bitmap,
    retained_source: FlexBox,
    animation_start_button: Button,
    animation_pause_button: Button,
    animation_step_button: Button,
    animation_reset_button: Button,
    bitmap_full_button: Button,
    bitmap_dirty_button: Button,
    bitmap_clear_button: Button,
    offscreen_compose_button: Button,
    retained_raster_button: Button,
    drop_zone: FlexBox,
    worker_card: FlexBox,
    worker_start: Rc<dyn Fn()>,
    worker_cancel: Rc<dyn Fn()>,
    worker_fail: Rc<dyn Fn()>,
    _worker_host_service: fui::worker_host_services::NativeWorkerHostServiceRegistration,
    state: Rc<NativeApplicationState>,
}

struct NativeWorkerShowcase {
    card: FlexBox,
    start: Rc<dyn Fn()>,
    cancel: Rc<dyn Fn()>,
    fail: Rc<dyn Fn()>,
}

#[derive(Clone)]
struct DrawingAnimation {
    state: Weak<NativeApplicationState>,
    immediate_invalidator: DrawableInvalidator,
    bitmap_invalidator: DrawableInvalidator,
    waveform_bitmap: Bitmap,
    animation_status: Text,
    bitmap_status: Text,
}

impl DrawingAnimation {
    fn update_status(&self) {
        let Some(state) = self.state.upgrade() else {
            return;
        };
        self.animation_status.text(format!(
            "Animation: {} | frame {} | timer fires {}",
            if state.animation_running.get() {
                "running"
            } else {
                "paused"
            },
            state.custom_draw_phase.get(),
            state.timer_fires.get()
        ));
        self.bitmap_status.text(format!(
            "Uploads: {} full | {} dirty | {} clear",
            state.bitmap_full_uploads.get(),
            state.bitmap_dirty_uploads.get(),
            state.bitmap_clears.get()
        ));
    }

    fn full_upload(&self) {
        let Some(state) = self.state.upgrade() else {
            return;
        };
        paint_waveform_full(
            &self.waveform_bitmap,
            state.custom_draw_phase.get(),
            state.dark_mode.get(),
        );
        state
            .bitmap_full_uploads
            .set(state.bitmap_full_uploads.get() + 1);
        self.immediate_invalidator.mark_dirty();
        self.bitmap_invalidator.mark_dirty();
        self.update_status();
    }

    fn dirty_step(&self, timer_fired: bool) {
        let Some(state) = self.state.upgrade() else {
            return;
        };
        let phase = state.custom_draw_phase.get().wrapping_add(1);
        state.custom_draw_phase.set(phase);
        state.animation_steps.set(state.animation_steps.get() + 1);
        if timer_fired {
            state.timer_fires.set(state.timer_fires.get() + 1);
        }
        paint_waveform_dirty(&self.waveform_bitmap, phase, state.dark_mode.get());
        state
            .bitmap_dirty_uploads
            .set(state.bitmap_dirty_uploads.get() + 1);
        self.immediate_invalidator.mark_dirty();
        self.bitmap_invalidator.mark_dirty();
        self.update_status();
    }

    fn clear(&self) {
        let Some(state) = self.state.upgrade() else {
            return;
        };
        self.waveform_bitmap.pixels().fill(0);
        self.waveform_bitmap.clear_dirty_rects().commit();
        state.bitmap_clears.set(state.bitmap_clears.get() + 1);
        self.immediate_invalidator.mark_dirty();
        self.bitmap_invalidator.mark_dirty();
        self.update_status();
    }

    fn start(&self) {
        let Some(state) = self.state.upgrade() else {
            return;
        };
        if state.animation_running.replace(true) {
            return;
        }
        self.update_status();
        self.arm_timer();
    }

    fn pause(&self) {
        let Some(state) = self.state.upgrade() else {
            return;
        };
        state.animation_running.set(false);
        if let Some(timer) = state.custom_draw_timer.borrow_mut().take() {
            cancel_timeout(timer);
        }
        self.update_status();
    }

    fn reset(&self) {
        self.pause();
        let Some(state) = self.state.upgrade() else {
            return;
        };
        state.custom_draw_phase.set(0);
        state.animation_steps.set(0);
        state.timer_fires.set(0);
        state.bitmap_full_uploads.set(0);
        state.bitmap_dirty_uploads.set(0);
        state.bitmap_clears.set(0);
        self.full_upload();
    }

    fn arm_timer(&self) {
        let Some(state) = self.state.upgrade() else {
            return;
        };
        if !state.animation_running.get() || state.custom_draw_timer.borrow().is_some() {
            return;
        }
        let animation = self.clone();
        let timer = set_timeout(80, move || {
            let Some(state) = animation.state.upgrade() else {
                return;
            };
            state.custom_draw_timer.borrow_mut().take();
            if !state.animation_running.get() {
                return;
            }
            animation.dirty_step(true);
            animation.arm_timer();
        });
        state.custom_draw_timer.borrow_mut().replace(timer);
    }
}

fn paint_waveform_full(bitmap: &Bitmap, phase: u32, dark_mode: bool) {
    let width = bitmap.width() as usize;
    let height = bitmap.height() as usize;
    let background = if dark_mode {
        [12, 23, 43, 255]
    } else {
        [232, 242, 255, 255]
    };
    let foreground = if dark_mode {
        [67, 214, 255, 255]
    } else {
        [14, 116, 190, 255]
    };
    let mut pixels = bitmap.pixels();
    for pixel in pixels.chunks_exact_mut(4) {
        pixel.copy_from_slice(&background);
    }
    for x in 0..width {
        let wave = ((x as f32 * 0.24 + phase as f32 * 0.18).sin() * 13.0).round() as i32;
        let y = (height as i32 / 2 + wave).clamp(0, height as i32 - 1) as usize;
        let offset = (y * width + x) * 4;
        pixels[offset..offset + 4].copy_from_slice(&foreground);
    }
    drop(pixels);
    bitmap.clear_dirty_rects().commit();
}

fn paint_waveform_dirty(bitmap: &Bitmap, phase: u32, dark_mode: bool) {
    let width = bitmap.width() as usize;
    let height = bitmap.height() as usize;
    let stripe_width = 16usize.min(width);
    let start = (phase as usize * 11) % (width - stripe_width + 1);
    let background = if dark_mode {
        [20, 36, 62, 255]
    } else {
        [215, 232, 252, 255]
    };
    let foreground = if dark_mode {
        [255, 153, 92, 255]
    } else {
        [210, 67, 32, 255]
    };
    let mut pixels = bitmap.pixels();
    for y in 0..height {
        for x in start..start + stripe_width {
            let offset = (y * width + x) * 4;
            pixels[offset..offset + 4].copy_from_slice(&background);
        }
    }
    for x in start..start + stripe_width {
        let wave = ((x as f32 * 0.24 + phase as f32 * 0.18).sin() * 13.0).round() as i32;
        let y = (height as i32 / 2 + wave).clamp(0, height as i32 - 1) as usize;
        let offset = (y * width + x) * 4;
        pixels[offset..offset + 4].copy_from_slice(&foreground);
    }
    drop(pixels);
    bitmap
        .clear_dirty_rects()
        .dirty_rect(start as u32, 0, stripe_width as u32, height as u32)
        .commit();
}

fn compose_offscreen(bitmap: &Bitmap, phase: u32, dark_mode: bool) -> u32 {
    let canvas = bitmap.canvas();
    canvas.draw_rect(
        0.0,
        0.0,
        bitmap.width() as f32,
        bitmap.height() as f32,
        Paint::fill(if dark_mode {
            rgba(14, 24, 48, 255)
        } else {
            rgba(229, 240, 255, 255)
        }),
    );
    let offset = (phase % 9) as f32 - 4.0;
    canvas.draw_circle(
        bitmap.width() as f32 * 0.5 + offset,
        bitmap.height() as f32 * 0.5,
        13.0,
        Paint::fill(rgba(255, 184, 76, 255)),
    );
    canvas.draw_line(
        4.0,
        bitmap.height() as f32 - 5.0,
        bitmap.width() as f32 - 4.0,
        5.0,
        rgba(34, 197, 151, 255),
        2.0,
    );
    bitmap.clear_dirty_rects().commit();
    let sample_offset =
        (((bitmap.height() / 2 * bitmap.width()) + bitmap.width() / 2) * 4) as usize;
    let pixels = bitmap.pixels();
    u32::from_be_bytes([
        pixels[sample_offset],
        pixels[sample_offset + 1],
        pixels[sample_offset + 2],
        pixels[sample_offset + 3],
    ])
}

fn rasterize_retained(
    bitmap: &Bitmap,
    source: &FlexBox,
    drawable: &DrawableInvalidator,
    status: &Text,
    state: &NativeApplicationState,
) {
    let bounds = source.get_bounds();
    if bounds[2] <= 0.0 || bounds[3] <= 0.0 {
        status.text("Retained raster: source is not laid out yet");
        return;
    }
    bitmap.render(source, bounds[0], bounds[1], 1.0);
    bitmap.clear_dirty_rects().commit();
    state.retained_rasters.set(state.retained_rasters.get() + 1);
    status.text(format!(
        "Retained raster: {} capture(s) | {:.0} x {:.0} source",
        state.retained_rasters.get(),
        bounds[2],
        bounds[3]
    ));
    drawable.mark_dirty();
}

fn themed_text(state: &Rc<NativeApplicationState>, value: &str, size: f32, color: u32) -> Text {
    let node = ui! { text(value).fill_width().font_size(size) };
    let guard = bind_theme({
        let node = node.clone();
        move |theme| {
            let color = match color {
                0x172033FF | 0x24324AFF => theme.colors.text_primary,
                _ => theme.colors.text_muted,
            };
            node.text_color(color);
        }
    });
    state.theme_guards.borrow_mut().push(guard);
    node
}

fn spacer(height: f32) -> FlexBox {
    let node = flex_box();
    node.height(height, Unit::Pixel);
    node
}

fn horizontal_spacer(width: f32) -> FlexBox {
    let node = flex_box();
    node.width(width, Unit::Pixel);
    node
}

fn themed_card(state: &Rc<NativeApplicationState>, title: &str, description: &str) -> FlexBox {
    let title_node = themed_text(state, title, 19.0, 0x172033FF);
    title_node.semantic_role(SemanticRole::Heading);
    let node = ui! {
        column()
        .fill_width()
        .height_len(auto())
        .padding(20.0, 20.0, 20.0, 20.0)
        .corner_radius(16.0)
        .children(children![
            title_node,
            ui! { spacer(6.0) },
            ui! { themed_text(state, description, 14.0, 0x58677DFF) },
            ui! { spacer(16.0) },
        ])
    };
    let guard = bind_theme({
        let node = node.clone();
        move |theme| {
            node.bg_color(theme.colors.surface)
                .border(1.0, theme.colors.border);
        }
    });
    state.theme_guards.borrow_mut().push(guard);
    node
}

fn build_worker_showcase(state: &Rc<NativeApplicationState>) -> NativeWorkerShowcase {
    const IDLE: u32 = 0;
    const RUNNING: u32 = 1;
    const CANCELLING: u32 = 2;
    const COMPLETE: u32 = 3;
    const CANCELLED: u32 = 4;
    const ERROR: u32 = 5;

    state.worker_status.set(IDLE);
    state.worker_progress.set(0.0);
    *state.worker_detail.borrow_mut() = String::from("Native worker detail: waiting");

    let progress = ui! {
        progress_bar()
            .sizing(ProgressBarSizing::new().length(360.0).thickness(18.0))
            .value(0.0)
            .semantic_label("Native prime worker progress")
    };
    let status = themed_text(state, "Native worker status: idle", 15.0, 0x475569FF);
    let detail = themed_text(state, "Native worker detail: waiting", 14.0, 0x65748BFF);

    let set_status: Rc<dyn Fn(u32, String)> = {
        let state = Rc::downgrade(state);
        let status = status.clone();
        Rc::new(move |code, value| {
            let Some(state) = state.upgrade() else { return };
            state.worker_status.set(code);
            status.text(&value).semantic_label(value);
        })
    };
    let set_detail: Rc<dyn Fn(String)> = {
        let state = Rc::downgrade(state);
        let detail = detail.clone();
        Rc::new(move |value| {
            let Some(state) = state.upgrade() else { return };
            *state.worker_detail.borrow_mut() = value.clone();
            detail.text(&value).semantic_label(value);
        })
    };
    let record_callback: Rc<dyn Fn()> = {
        let state = Rc::downgrade(state);
        Rc::new(move || {
            if let Some(state) = state.upgrade() {
                state
                    .worker_callback_thread
                    .borrow_mut()
                    .replace(std::thread::current().id());
            }
        })
    };

    let start: Rc<dyn Fn()> = {
        let state = Rc::downgrade(state);
        let progress = progress.clone();
        let set_status = set_status.clone();
        let set_detail = set_detail.clone();
        let record_callback = record_callback.clone();
        Rc::new(move || {
            let Some(state) = state.upgrade() else { return };
            if state.active_worker.borrow().is_some() {
                return;
            }
            state.worker_progress.set(0.0);
            progress.value(0.0);
            set_status(RUNNING, String::from("Native worker status: running"));
            set_detail(String::from(
                "Native worker detail: running - waiting for progress",
            ));
            let weak_state = Rc::downgrade(&state);
            let progress_handler = progress.clone();
            let progress_status = set_status.clone();
            let progress_detail = set_detail.clone();
            let complete_status = set_status.clone();
            let complete_detail = set_detail.clone();
            let error_status = set_status.clone();
            let error_detail = set_detail.clone();
            let progress_callback = record_callback.clone();
            let complete_callback = record_callback.clone();
            let error_callback = record_callback.clone();
            let complete_progress = progress.clone();
            let error_progress = progress.clone();
            let worker = Worker::new("./workers.wasm", "stage4PrimeWorker")
                .on_progress(move |event| {
                    progress_callback();
                    let percent = event.message.parse::<f32>().unwrap_or(0.0);
                    if let Some(state) = weak_state.upgrade() {
                        state.worker_progress.set(percent);
                    }
                    progress_handler.value(percent);
                    progress_status(
                        RUNNING,
                        format!("Native worker status: running - {percent:.0}%"),
                    );
                    progress_detail(format!("Native worker detail: progress - {percent:.0}%"));
                })
                .on_complete({
                    let state = Rc::downgrade(&state);
                    move |event| {
                        complete_callback();
                        if let Some(state) = state.upgrade() {
                            state.active_worker.borrow_mut().take();
                            state.worker_progress.set(100.0);
                        }
                        complete_progress.value(100.0);
                        complete_status(COMPLETE, String::from("Native worker status: complete"));
                        complete_detail(format!(
                            "Native worker detail: complete - {}",
                            event.result
                        ));
                    }
                })
                .on_error({
                    let state = Rc::downgrade(&state);
                    move |event| {
                        error_callback();
                        if let Some(state) = state.upgrade() {
                            state.active_worker.borrow_mut().take();
                        }
                        if let Some(cancelled) = event.message.strip_prefix("cancelled:") {
                            let percent = cancelled.parse::<f32>().unwrap_or(0.0);
                            if let Some(state) = state.upgrade() {
                                state.worker_progress.set(percent);
                            }
                            error_progress.value(percent);
                            error_status(
                                CANCELLED,
                                String::from("Native worker status: cancelled"),
                            );
                            error_detail(format!(
                                "Native worker detail: cancelled - {percent:.0}%"
                            ));
                        } else {
                            error_status(ERROR, String::from("Native worker status: error"));
                            error_detail(format!(
                                "Native worker detail: error - {}",
                                event.message
                            ));
                        }
                    }
                })
                .start("stage4-workers");
            state.active_worker.borrow_mut().replace(worker);
        })
    };

    let cancel: Rc<dyn Fn()> = {
        let state = Rc::downgrade(state);
        let set_status = set_status.clone();
        let set_detail = set_detail.clone();
        Rc::new(move || {
            let Some(state) = state.upgrade() else { return };
            let active = state.active_worker.borrow();
            let Some(worker) = active.as_ref() else {
                return;
            };
            set_status(CANCELLING, String::from("Native worker status: cancelling"));
            set_detail(String::from(
                "Native worker detail: waiting for cooperative cancellation",
            ));
            worker.cancel();
        })
    };

    let fail: Rc<dyn Fn()> = {
        let state = Rc::downgrade(state);
        let set_status = set_status.clone();
        let set_detail = set_detail.clone();
        Rc::new(move || {
            let Some(state) = state.upgrade() else { return };
            if state.active_worker.borrow().is_some() {
                return;
            }
            set_status(RUNNING, String::from("Native worker status: running"));
            set_detail(String::from(
                "Native worker detail: starting failing worker",
            ));
            let complete_status = set_status.clone();
            let complete_detail = set_detail.clone();
            let error_status = set_status.clone();
            let error_detail = set_detail.clone();
            let complete_callback = record_callback.clone();
            let error_callback = record_callback.clone();
            let worker = Worker::new("./workers.wasm", "stage4FailWorker")
                .on_complete({
                    let state = Rc::downgrade(&state);
                    move |event| {
                        complete_callback();
                        if let Some(state) = state.upgrade() {
                            state.active_worker.borrow_mut().take();
                        }
                        complete_status(COMPLETE, String::from("Native worker status: complete"));
                        complete_detail(format!(
                            "Native worker detail: complete - {}",
                            event.result
                        ));
                    }
                })
                .on_error({
                    let state = Rc::downgrade(&state);
                    move |event| {
                        error_callback();
                        if let Some(state) = state.upgrade() {
                            state.active_worker.borrow_mut().take();
                        }
                        error_status(ERROR, String::from("Native worker status: error"));
                        error_detail(format!("Native worker detail: error - {}", event.message));
                    }
                })
                .start("stage4-fail");
            state.active_worker.borrow_mut().replace(worker);
        })
    };

    let start_button = button("Start prime worker");
    start_button.on_click({
        let start = start.clone();
        move |_| start()
    });
    let cancel_button = button("Cancel worker");
    cancel_button.on_click({
        let cancel = cancel.clone();
        move |_| cancel()
    });
    let fail_button = button("Run failing worker");
    fail_button.on_click({
        let fail = fail.clone();
        move |_| fail()
    });
    let card = themed_card(
        state,
        "Worker parity",
        "The same Worker controller and prime-search entry run in browser Worker WASM or on a dedicated native thread.",
    );
    card.children(children![
        ui! { row().fill_width().height_len(auto()).children(children![
            start_button,
            ui! { horizontal_spacer(10.0) },
            cancel_button,
            ui! { horizontal_spacer(10.0) },
            fail_button,
        ]) },
        ui! { spacer(12.0) },
        progress,
        ui! { spacer(8.0) },
        status,
        ui! { spacer(4.0) },
        detail,
    ]);
    NativeWorkerShowcase {
        card,
        start,
        cancel,
        fail,
    }
}

fn build_application() -> NativeApplication {
    use_system_theme();
    let state = Rc::new(NativeApplicationState::default());
    state
        .worker_ui_thread
        .borrow_mut()
        .replace(std::thread::current().id());
    let worker_execution_thread = state.worker_execution_thread.clone();
    let worker_host_service =
        fui_rs_demo_worker::register_native_worker_host_services_with_clock(move || {
            worker_execution_thread
                .lock()
                .map_err(|_| String::from("Native worker thread diagnostic lock was poisoned."))?
                .replace(std::thread::current().id());
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_secs_f64() * 1000.0)
                .map_err(|error| format!("Native wall clock is unavailable: {error}"))
        })
        .expect("native demo Worker clock host service must register");
    state.dark_mode.set(is_dark_mode());
    fui::load_svg(
        9001,
        "data:image/svg+xml;utf8,%3Csvg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24'%3E%3Ccircle cx='12' cy='12' r='9' fill='%230A84FF'/%3E%3C/svg%3E",
    );

    let click_text = themed_text(
        &state,
        &format!("Button clicks: {}", state.activations.get()),
        15.0,
        0x334155FF,
    );
    let action_menu = context_menu(vec![MenuItem::new(
        "Increment",
        ContextMenuAction::OpenLink,
    )
    .on_invoke({
        let click_text = click_text.clone();
        let state = Rc::downgrade(&state);
        move || {
            let Some(state) = state.upgrade() else {
                return;
            };
            let next = state.activations.get() + 1;
            state.activations.set(next);
            click_text.text(format!("Button clicks: {next}"));
        }
    })]);
    let action = button("Increment click count");
    action
        .node_id("native-action")
        .width(220.0, Unit::Pixel)
        .tool_tip(
            ToolTip::text("Native timers open this retained tooltip.")
                .initial_show_delay(10)
                .show_duration(0),
        )
        .on_click({
            let click_text = click_text.clone();
            let state = Rc::downgrade(&state);
            move |_| {
                let Some(state) = state.upgrade() else {
                    return;
                };
                let next = state.activations.get() + 1;
                state.activations.set(next);
                click_text.text(format!("Button clicks: {next}"));
            }
        })
        .on_context_menu({
            let menu = action_menu.clone();
            move |event| menu.show(event.x, event.y)
        });

    let body_text = themed_text(&state,
        "A retained Rust application running directly on SDL3, Skia Metal, and native EffinDOM Tier 1/2 without a WebView.",
        16.0,
        0x45536CFF,
    );

    let custom_draw_text = ui! {
        text("Retained text inside an immediate canvas")
            .fill_width()
            .font_size(14.0)
            .text_color(rgba(238, 246, 255, 255))
    };
    let waveform_bitmap = Bitmap::new(96, 40);
    paint_waveform_full(&waveform_bitmap, 0, state.dark_mode.get());
    state.bitmap_full_uploads.set(1);

    let offscreen_bitmap = Bitmap::new(40, 40);
    let sample = compose_offscreen(&offscreen_bitmap, 0, state.dark_mode.get());
    state.offscreen_compositions.set(1);
    state.offscreen_sample_rgba.set(sample);

    let retained_bitmap = Bitmap::new(320, 96);

    let mut retained_path = Path::new();
    retained_path
        .move_to(126.0, 22.0)
        .line_to(154.0, 58.0)
        .line_to(98.0, 58.0)
        .close();
    let custom_drawable = custom_drawable({
        let state = Rc::downgrade(&state);
        let waveform_bitmap = waveform_bitmap.clone();
        let offscreen_bitmap = offscreen_bitmap.clone();
        let retained_path = retained_path.clone();
        let custom_draw_text = custom_draw_text.clone();
        move |context| {
            let Some(state) = state.upgrade() else {
                return;
            };
            let phase = state.custom_draw_phase.get();
            let background = if phase % 2 == 0 {
                rgba(18, 35, 62, 255)
            } else {
                rgba(36, 54, 84, 255)
            };
            context.draw_rect(-100.0, -100.0, 2000.0, 2000.0, Paint::fill(background));
            context.draw_round_rect(
                12.0,
                12.0,
                72.0,
                52.0,
                12.0,
                12.0,
                Paint::filled_stroke(rgba(39, 190, 126, 255), rgba(217, 255, 238, 255), 2.0),
            );
            context.draw_circle(50.0, 38.0, 12.0, Paint::fill(rgba(255, 255, 255, 210)));
            context.draw_line(18.0, 74.0, 178.0, 74.0, rgba(138, 180, 255, 255), 2.0);
            context.draw_path(
                &retained_path,
                Paint::filled_stroke(rgba(255, 126, 95, 255), rgba(255, 231, 224, 255), 2.0),
            );
            context.draw_image(waveform_bitmap.texture_id(), 190.0, 18.0, 192.0, 80.0);
            context.draw_image(offscreen_bitmap.texture_id(), 398.0, 18.0, 80.0, 80.0);
            context.draw_svg(9001, 494.0, 30.0, 56.0, 56.0);
            context.draw_text_node(&custom_draw_text, 18.0, 118.0);
            state
                .custom_draw_calls
                .set(state.custom_draw_calls.get() + 1);
            state.custom_draw_features.set(0x3Fu32);
        }
    });
    custom_drawable
        .node_id("native-custom-drawing")
        .fill_width()
        .height(164.0, Unit::Pixel)
        .corner_radius(14.0)
        .clip_to_bounds(true);
    let custom_drawing_card = themed_card(
        &state,
        "Native custom drawing",
        "Real FUI-RS immediate primitives, paths, dynamic pixels, offscreen composition, retained text, and SVG drawing share the same Skia command path as the web runtime.",
    );
    custom_drawing_card.children(children![
        custom_drawable.clone(),
        ui! { spacer(10.0) },
        custom_draw_text.clone(),
    ]);

    let animation_status = themed_text(
        &state,
        "Animation: paused | frame 0 | timer fires 0",
        14.0,
        0x475569FF,
    );
    let bitmap_status = themed_text(
        &state,
        "Uploads: 1 full | 0 dirty | 0 clear",
        14.0,
        0x475569FF,
    );
    let bitmap_drawable = fui::custom_drawable({
        let bitmap = waveform_bitmap.clone();
        let state = Rc::downgrade(&state);
        move |context| {
            let dark_mode = state.upgrade().is_none_or(|state| state.dark_mode.get());
            context.draw_round_rect(
                0.0,
                0.0,
                560.0,
                112.0,
                14.0,
                14.0,
                Paint::fill(if dark_mode {
                    rgba(8, 17, 34, 255)
                } else {
                    rgba(245, 249, 255, 255)
                }),
            );
            context.draw_image(bitmap.texture_id(), 18.0, 16.0, 384.0, 80.0);
            context.draw_line(420.0, 16.0, 420.0, 96.0, rgba(56, 189, 248, 255), 2.0);
            context.draw_circle(484.0, 56.0, 22.0, Paint::fill(rgba(251, 146, 60, 255)));
        }
    });
    bitmap_drawable
        .node_id("native-dynamic-bitmap")
        .fill_width()
        .height(112.0, Unit::Pixel)
        .corner_radius(14.0)
        .clip_to_bounds(true)
        .semantic_label("Dynamic bitmap preview");

    let animation = DrawingAnimation {
        state: Rc::downgrade(&state),
        immediate_invalidator: custom_drawable.invalidator(),
        bitmap_invalidator: bitmap_drawable.invalidator(),
        waveform_bitmap: waveform_bitmap.clone(),
        animation_status: animation_status.clone(),
        bitmap_status: bitmap_status.clone(),
    };
    let animation_start_button = button("Start animation");
    animation_start_button.on_click({
        let animation = animation.clone();
        move |_| animation.start()
    });
    let animation_pause_button = button("Pause animation");
    animation_pause_button.on_click({
        let animation = animation.clone();
        move |_| animation.pause()
    });
    let animation_step_button = button("Single step");
    animation_step_button.on_click({
        let animation = animation.clone();
        move |_| animation.dirty_step(false)
    });
    let animation_reset_button = button("Reset animation");
    animation_reset_button.on_click({
        let animation = animation.clone();
        move |_| animation.reset()
    });
    let bitmap_full_button = button("Full bitmap upload");
    bitmap_full_button.on_click({
        let animation = animation.clone();
        move |_| animation.full_upload()
    });
    let bitmap_dirty_button = button("Dirty rectangle update");
    bitmap_dirty_button.on_click({
        let animation = animation.clone();
        move |_| animation.dirty_step(false)
    });
    let bitmap_clear_button = button("Clear bitmap");
    bitmap_clear_button.on_click({
        let animation = animation.clone();
        move |_| animation.clear()
    });
    let animation_controls = ui! {
        column().fill_width().height_len(auto()).children(children![
            ui! { row().fill_width().height_len(auto()).children(children![
                animation_start_button.clone(),
                ui! { horizontal_spacer(10.0) },
                animation_pause_button.clone(),
            ]) },
            ui! { spacer(10.0) },
            ui! { row().fill_width().height_len(auto()).children(children![
                animation_step_button.clone(),
                ui! { horizontal_spacer(10.0) },
                animation_reset_button.clone(),
            ]) },
        ])
    };
    let bitmap_controls = ui! {
        column().fill_width().height_len(auto()).children(children![
            bitmap_full_button.clone(),
            ui! { spacer(8.0) },
            bitmap_dirty_button.clone(),
            ui! { spacer(8.0) },
            bitmap_clear_button.clone(),
        ])
    };
    let dynamic_bitmap_card = themed_card(
        &state,
        "Dynamic bitmap",
        "Write RGBA pixels directly, upload the complete texture, update only a dirty rectangle, or clear it. Animation uses the shared native timer coordinator.",
    );
    dynamic_bitmap_card.children(children![
        bitmap_drawable.clone(),
        ui! { spacer(12.0) },
        animation_controls,
        ui! { spacer(10.0) },
        animation_status.clone(),
        ui! { spacer(16.0) },
        bitmap_controls,
        ui! { spacer(10.0) },
        bitmap_status.clone(),
    ]);

    let offscreen_status = themed_text(
        &state,
        &format!("Offscreen: 1 composition | center RGBA #{sample:08X}"),
        14.0,
        0x475569FF,
    );
    let offscreen_drawable = fui::custom_drawable({
        let bitmap = offscreen_bitmap.clone();
        let state = Rc::downgrade(&state);
        move |context| {
            let dark_mode = state.upgrade().is_none_or(|state| state.dark_mode.get());
            context.draw_round_rect(
                0.0,
                0.0,
                560.0,
                116.0,
                14.0,
                14.0,
                Paint::fill(if dark_mode {
                    rgba(11, 22, 42, 255)
                } else {
                    rgba(242, 247, 255, 255)
                }),
            );
            context.draw_image(bitmap.texture_id(), 24.0, 18.0, 80.0, 80.0);
            context.draw_image(bitmap.texture_id(), 132.0, 10.0, 192.0, 96.0);
            context.draw_image(bitmap.texture_id(), 356.0, 26.0, 144.0, 72.0);
        }
    });
    offscreen_drawable
        .node_id("native-offscreen-composition")
        .fill_width()
        .height(116.0, Unit::Pixel)
        .corner_radius(14.0)
        .clip_to_bounds(true)
        .semantic_label("Offscreen composition preview");
    let offscreen_compose_button = button("Recompose offscreen surface");
    offscreen_compose_button.on_click({
        let bitmap = offscreen_bitmap.clone();
        let drawable = offscreen_drawable.invalidator();
        let status = offscreen_status.clone();
        let state = Rc::downgrade(&state);
        move |_| {
            let Some(state) = state.upgrade() else {
                return;
            };
            let sample = compose_offscreen(
                &bitmap,
                state.custom_draw_phase.get().wrapping_add(1),
                state.dark_mode.get(),
            );
            state
                .offscreen_compositions
                .set(state.offscreen_compositions.get() + 1);
            state.offscreen_sample_rgba.set(sample);
            status.text(format!(
                "Offscreen: {} composition(s) | center RGBA #{sample:08X}",
                state.offscreen_compositions.get()
            ));
            drawable.mark_dirty();
        }
    });
    let offscreen_card = themed_card(
        &state,
        "Offscreen composition",
        "Draw into an offscreen Skia surface, read its premultiplied RGBA pixels, upload the result, and sample the same texture at multiple sizes.",
    );
    offscreen_card.children(children![
        offscreen_drawable.clone(),
        ui! { spacer(12.0) },
        offscreen_compose_button.clone(),
        ui! { spacer(10.0) },
        offscreen_status.clone(),
    ]);

    let retained_source_label = themed_text(&state, "Retained source node", 18.0, 0x172033FF);
    let retained_source = ui! {
        column()
            .node_id("native-retained-raster-source")
            .width(280.0, Unit::Pixel)
            .height(72.0, Unit::Pixel)
            .padding(12.0, 14.0, 12.0, 14.0)
            .corner_radius(12.0)
            .children(children![
                retained_source_label,
                ui! { spacer(4.0) },
                ui! { themed_text(&state, "Rendered from the retained tree", 13.0, 0x475569FF) },
            ])
    };
    let source_theme_guard = bind_theme({
        let source = retained_source.clone();
        move |theme| {
            source
                .bg_color(theme.colors.surface)
                .border(1.0, theme.colors.accent);
        }
    });
    state.theme_guards.borrow_mut().push(source_theme_guard);
    let retained_status = themed_text(
        &state,
        "Retained raster: waiting for first layout",
        14.0,
        0x475569FF,
    );
    let retained_drawable = fui::custom_drawable({
        let bitmap = retained_bitmap.clone();
        let state = Rc::downgrade(&state);
        move |context| {
            let dark_mode = state.upgrade().is_none_or(|state| state.dark_mode.get());
            context.draw_round_rect(
                0.0,
                0.0,
                560.0,
                116.0,
                14.0,
                14.0,
                Paint::fill(if dark_mode {
                    rgba(10, 20, 38, 255)
                } else {
                    rgba(239, 246, 255, 255)
                }),
            );
            context.draw_image(bitmap.texture_id(), 18.0, 10.0, 320.0, 96.0);
            context.draw_round_rect(
                366.0,
                22.0,
                148.0,
                72.0,
                12.0,
                12.0,
                Paint::filled_stroke(rgba(59, 130, 246, 180), rgba(147, 197, 253, 255), 2.0),
            );
        }
    });
    retained_drawable
        .node_id("native-retained-raster")
        .fill_width()
        .height(116.0, Unit::Pixel)
        .corner_radius(14.0)
        .clip_to_bounds(true)
        .semantic_label("Retained node raster preview");
    let retained_raster_button = button("Rasterize retained node");
    retained_raster_button.on_click({
        let bitmap = retained_bitmap.clone();
        let source = retained_source.clone();
        let drawable = retained_drawable.invalidator();
        let status = retained_status.clone();
        let state = Rc::downgrade(&state);
        move |_| {
            if let Some(state) = state.upgrade() {
                rasterize_retained(&bitmap, &source, &drawable, &status, &state);
            }
        }
    });
    let retained_card = themed_card(
        &state,
        "Retained rasterization",
        "Render a normal retained FUI-RS subtree into RGBA pixels, upload those pixels as a bitmap, then compose that bitmap through immediate drawing.",
    );
    retained_card.children(children![
        retained_source.clone(),
        ui! { spacer(12.0) },
        retained_drawable.clone(),
        ui! { spacer(12.0) },
        retained_raster_button.clone(),
        ui! { spacer(10.0) },
        retained_status.clone(),
    ]);
    on_loaded({
        let bitmap = retained_bitmap.clone();
        let source = retained_source.clone();
        let drawable = retained_drawable.invalidator();
        let status = retained_status.clone();
        let state = Rc::downgrade(&state);
        move |_| {
            let Some(owner) = state.upgrade() else {
                return;
            };
            let state = state.clone();
            let timer = set_timeout(0, move || {
                let Some(state) = state.upgrade() else {
                    return;
                };
                state.initial_raster_timer.borrow_mut().take();
                rasterize_retained(&bitmap, &source, &drawable, &status, &state);
            });
            owner.initial_raster_timer.borrow_mut().replace(timer);
        }
    });

    let drawing_theme_guard = bind_theme({
        let state = Rc::downgrade(&state);
        let animation = animation.clone();
        let offscreen_bitmap = offscreen_bitmap.clone();
        let offscreen_invalidator = offscreen_drawable.invalidator();
        let offscreen_status = offscreen_status.clone();
        let retained_invalidator = retained_drawable.invalidator();
        move |_| {
            let Some(state) = state.upgrade() else {
                return;
            };
            state.dark_mode.set(is_dark_mode());
            animation.full_upload();
            let sample = compose_offscreen(
                &offscreen_bitmap,
                state.custom_draw_phase.get(),
                state.dark_mode.get(),
            );
            state
                .offscreen_compositions
                .set(state.offscreen_compositions.get() + 1);
            state.offscreen_sample_rgba.set(sample);
            offscreen_status.text(format!(
                "Offscreen: {} composition(s) | center RGBA #{sample:08X}",
                state.offscreen_compositions.get()
            ));
            offscreen_invalidator.mark_dirty();
            retained_invalidator.mark_dirty();
        }
    });
    state.theme_guards.borrow_mut().push(drawing_theme_guard);

    let checkbox_status = themed_text(&state, "Checkbox: off", 14.0, 0x475569FF);
    let checkbox_control = checkbox("Enable retained option");
    checkbox_control.on_changed({
        let status = checkbox_status.clone();
        move |event| {
            status.text(if event.checked {
                "Checkbox: on"
            } else {
                "Checkbox: off"
            });
        }
    });

    let switch_status = themed_text(&state, "Switch: off", 14.0, 0x475569FF);
    let switch_control = switch("Native feature switch");
    switch_control.on_changed({
        let status = switch_status.clone();
        move |event| {
            status.text(if event.checked {
                "Switch: on"
            } else {
                "Switch: off"
            });
        }
    });

    let slider_status = themed_text(&state, "Slider value: 35", 14.0, 0x475569FF);
    let progress = progress_bar();
    progress.min(0.0).max(100.0).value(35.0).length(420.0);
    let slider_control = slider();
    slider_control
        .min(0.0)
        .max(100.0)
        .step(1.0)
        .value(35.0)
        .length(420.0)
        .on_changed({
            let status = slider_status.clone();
            let progress = progress.clone();
            move |event| {
                status.text(format!("Slider value: {:.0}", event.value));
                progress.value(event.value);
            }
        });

    let controls_card = themed_card(&state,
        "Retained controls",
        "Pointer, keyboard, focus, state, and rendering all use the same FUI-RS control implementations as the browser runtime.",
    );
    let accessibility_editor = ui! {
        text_input()
            .width(420.0, Unit::Pixel)
            .text("AT-SPI Unicode text")
    };
    controls_card.children(children![
        action.clone(),
        ui! { spacer(8.0) },
        click_text,
        ui! { spacer(12.0) },
        accessibility_editor,
        ui! { spacer(18.0) },
        checkbox_control,
        ui! { spacer(6.0) },
        checkbox_status,
        ui! { spacer(14.0) },
        switch_control,
        ui! { spacer(6.0) },
        switch_status,
        ui! { spacer(18.0) },
        slider_control,
        ui! { spacer(10.0) },
        progress,
        ui! { spacer(8.0) },
        slider_status,
    ]);

    let dispatch_status = themed_text(&state, "UI dispatch: idle", 14.0, 0x475569FF);
    let dispatch_button = button("Dispatch from worker thread");
    dispatch_button.width(240.0, Unit::Pixel).on_click({
        let status = dispatch_status.clone();
        move |_| {
            status.text("UI dispatch: queued");
            let completion_status = status.clone();
            let dispatch = platform::UiDispatcher::prepare(move || {
                completion_status.text("UI dispatch: completed on UI thread");
            });
            std::thread::spawn(move || {
                let _ = dispatch.dispatch();
            });
        }
    });

    let clipboard_status = themed_text(&state, "Clipboard: ready", 14.0, 0x475569FF);
    let copy_button = button("Copy native text");
    copy_button.width(190.0, Unit::Pixel).on_click({
        let status = clipboard_status.clone();
        move |_| {
            status.text(
                if platform::write_clipboard_text("Copied from native FUI-RS") {
                    "Clipboard: native text copied"
                } else {
                    "Clipboard: write failed"
                },
            );
        }
    });
    let read_button = button("Read clipboard");
    read_button.width(190.0, Unit::Pixel).on_click({
        let status = clipboard_status.clone();
        move |_| {
            let value =
                platform::read_clipboard_text().unwrap_or_else(|| "<unavailable>".to_string());
            status.text(format!("Clipboard: {value}"));
        }
    });
    let clipboard_row = row();
    clipboard_row
        .fill_width()
        .height_len(auto())
        .children(children![
            copy_button,
            ui! { horizontal_spacer(12.0) },
            read_button
        ]);

    let system_card = themed_card(&state,
        "Native dispatch and clipboard",
        "Work may originate off-thread, but retained mutations return to the SDL UI thread. Clipboard access uses macOS services directly.",
    );
    system_card.children(children![
        dispatch_button,
        ui! { spacer(8.0) },
        dispatch_status,
        ui! { spacer(18.0) },
        clipboard_row,
        ui! { spacer(8.0) },
        clipboard_status,
    ]);

    let selected_path = Rc::new(RefCell::new(None::<PathBuf>));
    let file_status = themed_text(&state, "File dialogs: no selection", 14.0, 0x475569FF);
    let open_dialog_button = button("Open files...");
    open_dialog_button.width(190.0, Unit::Pixel).on_click({
        let status = file_status.clone();
        let selected_path = selected_path.clone();
        move |_| {
            status.text("File dialogs: opening file picker...");
            let completion_status = status.clone();
            let completion_path = selected_path.clone();
            if platform::show_open_file_dialog(
                platform::NativeFileDialogOptions {
                    filters: vec![platform::NativeFileFilter::new(
                        "Text and Markdown",
                        ["txt", "md"],
                    )],
                    default_location: None,
                    allow_multiple: true,
                },
                move |result| match result {
                    platform::NativeFileDialogResult::Selected { paths, .. } => {
                        *completion_path.borrow_mut() = paths.first().cloned();
                        completion_status
                            .text(format!("File dialogs: selected {} file(s)", paths.len()));
                    }
                    platform::NativeFileDialogResult::Cancelled => {
                        completion_status.text("File dialogs: cancelled");
                    }
                    platform::NativeFileDialogResult::Error(error) => {
                        completion_status.text(format!("File dialogs: {error}"));
                    }
                },
            )
            .is_none()
            {
                status.text("File dialogs: could not open picker");
            }
        }
    });

    let save_dialog_button = button("Choose save path...");
    save_dialog_button.width(190.0, Unit::Pixel).on_click({
        let status = file_status.clone();
        let selected_path = selected_path.clone();
        move |_| {
            status.text("File dialogs: choosing save path...");
            let completion_status = status.clone();
            let completion_path = selected_path.clone();
            if platform::show_save_file_dialog(
                platform::NativeFileDialogOptions {
                    filters: vec![platform::NativeFileFilter::new("Text", ["txt"])],
                    default_location: None,
                    allow_multiple: false,
                },
                move |result| match result {
                    platform::NativeFileDialogResult::Selected { paths, .. } => {
                        *completion_path.borrow_mut() = paths.first().cloned();
                        completion_status.text("File dialogs: save path selected");
                    }
                    platform::NativeFileDialogResult::Cancelled => {
                        completion_status.text("File dialogs: save cancelled");
                    }
                    platform::NativeFileDialogResult::Error(error) => {
                        completion_status.text(format!("File dialogs: {error}"));
                    }
                },
            )
            .is_none()
            {
                status.text("File dialogs: could not open save picker");
            }
        }
    });

    let folder_dialog_button = button("Choose folder...");
    folder_dialog_button.width(190.0, Unit::Pixel).on_click({
        let status = file_status.clone();
        let selected_path = selected_path.clone();
        move |_| {
            status.text("File dialogs: choosing folder...");
            let completion_status = status.clone();
            let completion_path = selected_path.clone();
            if platform::show_open_folder_dialog(
                platform::NativeFileDialogOptions::default(),
                move |result| match result {
                    platform::NativeFileDialogResult::Selected { paths, .. } => {
                        *completion_path.borrow_mut() = paths.first().cloned();
                        completion_status.text("File dialogs: folder selected");
                    }
                    platform::NativeFileDialogResult::Cancelled => {
                        completion_status.text("File dialogs: folder cancelled");
                    }
                    platform::NativeFileDialogResult::Error(error) => {
                        completion_status.text(format!("File dialogs: {error}"));
                    }
                },
            )
            .is_none()
            {
                status.text("File dialogs: could not open folder picker");
            }
        }
    });

    let open_selected_button = button("Open selected path");
    open_selected_button.width(190.0, Unit::Pixel).on_click({
        let status = file_status.clone();
        let selected_path = selected_path.clone();
        move |_| {
            let opened = selected_path
                .borrow()
                .as_ref()
                .is_some_and(platform::open_file);
            status.text(if opened {
                "Selected path opened"
            } else {
                "Select an existing file first"
            });
        }
    });
    let reveal_selected_button = button("Reveal selected path");
    reveal_selected_button.width(190.0, Unit::Pixel).on_click({
        let status = file_status.clone();
        let selected_path = selected_path.clone();
        move |_| {
            let revealed = selected_path
                .borrow()
                .as_ref()
                .is_some_and(platform::reveal_file);
            status.text(if revealed {
                "Selected path revealed"
            } else {
                "Select an existing path first"
            });
        }
    });
    let open_web_button = button("Open effindom.dev");
    open_web_button.width(190.0, Unit::Pixel).on_click({
        let status = file_status.clone();
        move |_| {
            status.text(if platform::open_external_url("https://effindom.dev") {
                "Opened effindom.dev in the system browser"
            } else {
                "Could not open external URL"
            });
        }
    });
    let file_dialog_row = row();
    file_dialog_row
        .fill_width()
        .height_len(auto())
        .children(children![
            open_dialog_button,
            ui! { horizontal_spacer(12.0) },
            save_dialog_button,
            ui! { horizontal_spacer(12.0) },
            folder_dialog_button,
        ]);
    let file_action_row = row();
    file_action_row
        .fill_width()
        .height_len(auto())
        .children(children![
            open_selected_button,
            ui! { horizontal_spacer(12.0) },
            reveal_selected_button,
            ui! { horizontal_spacer(12.0) },
            open_web_button,
        ]);
    let files_card = themed_card(&state,
        "Native files and external targets",
        "Open, save, and folder dialogs return filesystem paths. Applications keep ordinary Rust ownership of file I/O.",
    );
    files_card.children(children![
        file_dialog_row,
        ui! { spacer(12.0) },
        file_action_row,
        ui! { spacer(8.0) },
        file_status,
    ]);

    let drop_status = themed_text(
        &state,
        "Drop status: drag files, text, or URLs over this card",
        14.0,
        0x475569FF,
    );
    let drop_card = themed_card(&state,
        "Native drag and drop",
        "SDL drop events preserve enter, over, drop, and leave routing with native paths and multi-item payloads.",
    );
    drop_card
        .min_height(150.0, Unit::Pixel)
        .bg_color(0xF8FBFFFF)
        .border(2.0, 0x8DB6EEFF)
        .child(&drop_status)
        .on_external_drag_enter({
            let status = drop_status.clone();
            let state = Rc::downgrade(&state);
            move |_| {
                status.text("Drop status: native drag entered");
                if let Some(state) = state.upgrade() {
                    state.drop_completed.set(false);
                    state.drop_result.borrow_mut().push_str("enter,");
                }
                DropProposal::new(DragDropEffects::Copy, false)
            }
        })
        .on_external_drag_over({
            let status = drop_status.clone();
            let state = Rc::downgrade(&state);
            move |_| {
                status.text("Drop status: release to copy payload metadata");
                if let Some(state) = state.upgrade() {
                    state.drop_result.borrow_mut().push_str("over,");
                }
                DropProposal::new(DragDropEffects::Copy, false)
            }
        })
        .on_external_drag_leave({
            let status = drop_status.clone();
            let state = Rc::downgrade(&state);
            move |_| {
                if let Some(state) = state.upgrade() {
                    if !state.drop_completed.replace(false) {
                        status.text("Drop status: drag left the drop zone");
                    }
                    state.drop_result.borrow_mut().push_str("leave");
                }
            }
        })
        .on_external_drop({
            let status = drop_status.clone();
            let state = Rc::downgrade(&state);
            move |event| {
                let item_count = event.items.len();
                status.text(format!("Drop status: received {item_count} item(s)"));
                if let Some(state) = state.upgrade() {
                    state.drop_completed.set(true);
                    let mut value = state.drop_result.borrow_mut();
                    value.push_str("drop:");
                    value.push_str(&item_count.to_string());
                    for item in event.items {
                        value.push(':');
                        match item.kind {
                            ExternalDropItemKind::File => {
                                value.push_str("file=");
                                if let Some(path) = item.native_path() {
                                    value.push_str(&path.to_string_lossy());
                                }
                            }
                            ExternalDropItemKind::Uri => {
                                value.push_str("uri=");
                                value.push_str(&item.id);
                            }
                            ExternalDropItemKind::Text => {
                                value.push_str("text=");
                                value.push_str(&item.id);
                            }
                            ExternalDropItemKind::Unknown(kind) => {
                                value.push_str("unknown=");
                                value.push_str(&kind.to_string());
                            }
                        }
                    }
                    value.push(',');
                }
            }
        });

    let assets_card = themed_card(&state,
        "Offline assets and font fallback",
        "Packaged application fonts take priority; system fallback is used only for coverage the application did not supply.",
    );
    let packaged_fallback_stack = FontStack::load("fonts/NotoSans-Regular.ttf")
        .fallback_loaded("fonts/NotoSansThai-Regular.ttf")
        .fallback_loaded("fonts/NotoNaskhArabic-Variable.ttf")
        .fallback_loaded("fonts/NotoColorEmoji.ttf");
    let packaged_fallback_sample = themed_text(&state, "ไทย · مرحبا · 😀", 18.0, 0x172033FF);
    packaged_fallback_sample.font_stack(packaged_fallback_stack, 18.0);
    let system_fallback_text = themed_text(&state, "你好", 18.0, 0x172033FF);
    assets_card.children(children![
        ui! {
            row().fill_width().height_len(auto()).children(children![
                ui! { svg(9001).width(48.0, Unit::Pixel).height(48.0, Unit::Pixel) },
                ui! { horizontal_spacer(18.0) },
                ui! {
                    image(0)
                        .source("app/demo-texture.png")
                        .width(96.0, Unit::Pixel)
                        .height(64.0, Unit::Pixel)
                },
            ])
        },
        ui! { spacer(14.0) },
        ui! { themed_text(&state, "Packaged Noto fallback", 14.0, 0x475569FF) },
        ui! { spacer(4.0) },
        packaged_fallback_sample,
        ui! { spacer(12.0) },
        ui! { themed_text(&state, "macOS system fallback", 14.0, 0x475569FF) },
        ui! { spacer(4.0) },
        system_fallback_text.clone(),
        ui! { spacer(6.0) },
        ui! { themed_text(&state, "No HTTP font request is made; the CJK face is resolved from installed macOS fonts.", 14.0, 0x475569FF) },
    ]);

    let selection_card = themed_card(&state,
        "Selection and native input",
        "Drag across the text below. Selection, pointer capture, keyboard focus, wheel input, and resize all route through the native SDL host.",
    );
    let selection_content = selection_area();
    let selection_text = themed_text(
        &state,
        "Native retained text remains selectable while the surrounding application scrolls.",
        16.0,
        0x24324AFF,
    );
    selection_content.child(&ui! {
        column().fill_width().height_len(auto()).children(children![
            selection_text.clone(),
            ui! { themed_text(&state, "The selection repaint is demand-driven and updates during pointer movement, before pointer-up.", 16.0, 0x24324AFF) },
        ])
    });
    selection_card.child(&selection_content);

    let text_area_card = themed_card(
        &state,
        "Multiline native editor",
        "Test letter case, Caps Lock, keypad input, cursor keys, selection, wrapping, and internal scrolling.",
    );
    let native_text_area = ui! {
        text_area()
            .fill_width()
            .height(180.0, Unit::Pixel)
            .placeholder("Type several lines of native text")
            .text("Native TextArea\n\nTry lowercase and uppercase letters.\nTry the numeric keypad.\nUse cursor keys and Shift selection.\nAdd enough lines to test scrolling.")
            .wrapping(true)
            .accepts_tab(true)
            .vertical_scrollbar_visibility(ScrollBarVisibility::Auto)
            .horizontal_scrollbar_visibility(ScrollBarVisibility::Auto)
    };
    text_area_card.child(&native_text_area);

    let context_menu_card = themed_card(
        &state,
        "Retained context menus",
        "Right-click the link, image, SVG, or editor for capability-aware desktop actions. Right-click blank card space for no menu; the increment button demonstrates an application-defined menu.",
    );
    let context_link = NavLink::with_label("https://effindom.dev/", "Open EffinDOM website");
    let context_image = ui! {
        image(0)
            .source("app/demo-texture.png")
            .width(96.0, Unit::Pixel)
            .height(64.0, Unit::Pixel)
    };
    let context_svg = ui! {
        svg(0)
            .source("data:image/svg+xml;utf8,%3Csvg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24'%3E%3Ccircle cx='12' cy='12' r='9' fill='%230A84FF'/%3E%3C/svg%3E")
            .width(48.0, Unit::Pixel)
            .height(48.0, Unit::Pixel)
    };
    let context_editor = text_input();
    context_editor
        .width(420.0, Unit::Pixel)
        .placeholder("Editable text context menu")
        .text("Select or edit this text");
    context_menu_card.children(children![
        context_link.clone(),
        ui! { spacer(12.0) },
        ui! {
            row().fill_width().height_len(auto()).children(children![
                context_image.clone(),
                ui! { horizontal_spacer(18.0) },
                context_svg.clone(),
            ])
        },
        ui! { spacer(12.0) },
        context_editor.clone(),
    ]);

    let worker_showcase = build_worker_showcase(&state);
    let content = column();
    content
        .fill_width()
        .height_len(auto())
        .padding(32.0, 32.0, 32.0, 32.0)
        .children(children![
            ui! { themed_text(&state, "EffinDOM native FUI-RS", 30.0, 0x172033FF) },
            ui! { spacer(8.0) },
            body_text,
            ui! { spacer(10.0) },
            ui! { themed_text(&state, "SDL3 input · Skia Metal rendering · demand-driven frames · retained Rust UI", 14.0, 0x65748BFF) },
            ui! { spacer(22.0) },
            controls_card,
            ui! { spacer(16.0) },
            worker_showcase.card.clone(),
            ui! { spacer(16.0) },
            custom_drawing_card,
            ui! { spacer(16.0) },
            system_card,
            ui! { spacer(16.0) },
            files_card,
            ui! { spacer(16.0) },
            drop_card,
            ui! { spacer(16.0) },
            assets_card,
            ui! { spacer(16.0) },
            selection_card,
            ui! { spacer(16.0) },
            text_area_card,
            ui! { spacer(16.0) },
            dynamic_bitmap_card,
            ui! { spacer(16.0) },
            offscreen_card,
            ui! { spacer(16.0) },
            retained_card,
            ui! { spacer(16.0) },
            context_menu_card,
            ui! { spacer(32.0) },
        ]);

    let scroll = ui! {
        scroll_box()
        .node_id("native-scroll-root")
        .fill_size()
        .persist_scroll(false)
        .scrollbar_gutter(0.0)
        .child(&content)
    };
    scroll
        .vertical_scrollbar()
        .track_width(12.0)
        .thumb_width(8.0)
        .thumb_min_height(36.0)
        .track_corner_radius(6.0)
        .thumb_corner_radius(4.0)
        .track_color(current_theme().colors.scrollbar_track)
        .thumb_color(current_theme().colors.scrollbar_thumb);
    let root = ui! {
        column()
        .node_id("native-root")
        .fill_size()
        .children(children![scroll.clone(), action_menu])
    };
    let theme_guard = bind_theme({
        let root = root.clone();
        let scroll = scroll.clone();
        move |theme| {
            root.bg_color(theme.colors.background);
            scroll.bg_color(theme.colors.background);
            scroll
                .vertical_scrollbar()
                .track_color(theme.colors.scrollbar_track)
                .thumb_color(theme.colors.scrollbar_thumb);
            scroll
                .horizontal_scrollbar()
                .track_color(theme.colors.scrollbar_track)
                .thumb_color(theme.colors.scrollbar_thumb);
        }
    });
    state.theme_guards.borrow_mut().push(theme_guard);
    NativeApplication {
        root,
        scroll_root: scroll,
        action_button: action,
        body_text,
        system_fallback_text,
        selection_text,
        click_text,
        context_link,
        context_image,
        context_svg,
        context_editor,
        custom_drawable,
        bitmap_drawable,
        offscreen_drawable,
        retained_drawable,
        custom_draw_text,
        animation_status,
        bitmap_status,
        offscreen_status,
        retained_status,
        waveform_bitmap,
        offscreen_bitmap,
        retained_bitmap,
        retained_source,
        animation_start_button,
        animation_pause_button,
        animation_step_button,
        animation_reset_button,
        bitmap_full_button,
        bitmap_dirty_button,
        bitmap_clear_button,
        offscreen_compose_button,
        retained_raster_button,
        drop_zone: drop_card,
        worker_card: worker_showcase.card,
        worker_start: worker_showcase.start,
        worker_cancel: worker_showcase.cancel,
        worker_fail: worker_showcase.fail,
        _worker_host_service: worker_host_service,
        state,
    }
}

fui_managed_app!(
    NativeApplication,
    build_application,
    |application: &NativeApplication| application.root.clone()
);

fn with_native_application<T>(callback: impl FnOnce(&NativeApplication) -> T) -> Option<T> {
    __fui_rs_with_app(|application| application.get_active_page().as_deref().map(callback))
}

#[no_mangle]
pub extern "C" fn __fui_native_worker_panel_handle() -> u64 {
    with_native_application(|application| application.worker_card.handle().raw()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_worker_start_prime() {
    let _ = with_native_application(|application| (application.worker_start)());
}

#[no_mangle]
pub extern "C" fn __fui_native_worker_cancel() {
    let _ = with_native_application(|application| (application.worker_cancel)());
}

#[no_mangle]
pub extern "C" fn __fui_native_worker_start_fail() {
    let _ = with_native_application(|application| (application.worker_fail)());
}

#[no_mangle]
pub extern "C" fn __fui_native_worker_status() -> u32 {
    with_native_application(|application| application.state.worker_status.get()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_worker_progress() -> f32 {
    with_native_application(|application| application.state.worker_progress.get()).unwrap_or(0.0)
}

#[no_mangle]
pub extern "C" fn __fui_native_worker_detail_has_prime_and_clock() -> bool {
    with_native_application(|application| {
        let detail = application.state.worker_detail.borrow();
        detail.contains("prime=") && detail.contains("clock=")
    })
    .unwrap_or(false)
}

#[no_mangle]
pub extern "C" fn __fui_native_worker_detail_has_failure_and_clock() -> bool {
    with_native_application(|application| {
        let detail = application.state.worker_detail.borrow();
        detail.contains("worker failure") && detail.contains("clock=")
    })
    .unwrap_or(false)
}

#[no_mangle]
pub extern "C" fn __fui_native_worker_threads_are_split() -> bool {
    with_native_application(|application| {
        let execution = application
            .state
            .worker_execution_thread
            .lock()
            .ok()
            .and_then(|thread| *thread);
        let callback = *application.state.worker_callback_thread.borrow();
        let ui = *application.state.worker_ui_thread.borrow();
        matches!((execution, callback, ui), (Some(execution), Some(callback), Some(ui)) if execution != ui && callback == ui)
    })
    .unwrap_or(false)
}

#[no_mangle]
pub extern "C" fn __fui_native_action_handle() -> u64 {
    with_native_application(|application| application.action_button.handle().raw()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_application_root_handle() -> u64 {
    with_native_application(|application| application.root.handle().raw()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_scroll_handle() -> u64 {
    with_native_application(|application| application.scroll_root.handle().raw()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_scroll_view_handle() -> u64 {
    with_native_application(|application| application.scroll_root.viewport().handle().raw())
        .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_drop_zone_handle() -> u64 {
    with_native_application(|application| application.drop_zone.handle().raw()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_body_text_handle() -> u64 {
    with_native_application(|application| application.body_text.handle().raw()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_system_fallback_text_handle() -> u64 {
    with_native_application(|application| application.system_fallback_text.handle().raw()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_selection_text_handle() -> u64 {
    with_native_application(|application| application.selection_text.handle().raw()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_click_text_handle() -> u64 {
    with_native_application(|application| application.click_text.handle().raw()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_context_link_handle() -> u64 {
    with_native_application(|application| application.context_link.handle().raw()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_context_image_handle() -> u64 {
    with_native_application(|application| application.context_image.handle().raw()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_context_svg_handle() -> u64 {
    with_native_application(|application| application.context_svg.handle().raw()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_context_editor_handle() -> u64 {
    with_native_application(|application| application.context_editor.handle().raw()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_custom_draw_handle() -> u64 {
    with_native_application(|application| application.custom_drawable.handle().raw()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_custom_draw_text_handle() -> u64 {
    with_native_application(|application| application.custom_draw_text.handle().raw()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_custom_draw_calls() -> u32 {
    with_native_application(|application| application.state.custom_draw_calls.get()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_custom_draw_features() -> u32 {
    with_native_application(|application| application.state.custom_draw_features.get()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_waveform_texture_id() -> u32 {
    with_native_application(|application| application.waveform_bitmap.texture_id()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_offscreen_texture_id() -> u32 {
    with_native_application(|application| application.offscreen_bitmap.texture_id()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_retained_texture_id() -> u32 {
    with_native_application(|application| application.retained_bitmap.texture_id()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_bitmap_draw_handle() -> u64 {
    with_native_application(|application| application.bitmap_drawable.handle().raw()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_offscreen_draw_handle() -> u64 {
    with_native_application(|application| application.offscreen_drawable.handle().raw())
        .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_retained_draw_handle() -> u64 {
    with_native_application(|application| application.retained_drawable.handle().raw()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_retained_source_handle() -> u64 {
    with_native_application(|application| application.retained_source.handle().raw()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_animation_running() -> bool {
    with_native_application(|application| application.state.animation_running.get())
        .unwrap_or(false)
}

#[no_mangle]
pub extern "C" fn __fui_native_animation_step_count() -> u32 {
    with_native_application(|application| application.state.animation_steps.get()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_bitmap_full_upload_count() -> u32 {
    with_native_application(|application| application.state.bitmap_full_uploads.get()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_bitmap_dirty_upload_count() -> u32 {
    with_native_application(|application| application.state.bitmap_dirty_uploads.get()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_bitmap_clear_count() -> u32 {
    with_native_application(|application| application.state.bitmap_clears.get()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_offscreen_composition_count() -> u32 {
    with_native_application(|application| application.state.offscreen_compositions.get())
        .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_offscreen_sample_rgba() -> u32 {
    with_native_application(|application| application.state.offscreen_sample_rgba.get())
        .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_retained_raster_count() -> u32 {
    with_native_application(|application| application.state.retained_rasters.get()).unwrap_or(0)
}

macro_rules! export_handle {
    ($name:ident, $field:ident) => {
        #[no_mangle]
        pub extern "C" fn $name() -> u64 {
            with_native_application(|application| application.$field.handle().raw()).unwrap_or(0)
        }
    };
}

export_handle!(__fui_native_animation_start_handle, animation_start_button);
export_handle!(__fui_native_animation_pause_handle, animation_pause_button);
export_handle!(__fui_native_animation_step_handle, animation_step_button);
export_handle!(__fui_native_animation_reset_handle, animation_reset_button);
export_handle!(__fui_native_bitmap_full_handle, bitmap_full_button);
export_handle!(__fui_native_bitmap_dirty_handle, bitmap_dirty_button);
export_handle!(__fui_native_bitmap_clear_handle, bitmap_clear_button);
export_handle!(
    __fui_native_offscreen_compose_handle,
    offscreen_compose_button
);
export_handle!(__fui_native_retained_raster_handle, retained_raster_button);

#[no_mangle]
pub extern "C" fn __fui_native_start_drawing_animation() {
    let _ = with_native_application(|application| {
        let animation = DrawingAnimation {
            state: Rc::downgrade(&application.state),
            immediate_invalidator: application.custom_drawable.invalidator(),
            bitmap_invalidator: application.bitmap_drawable.invalidator(),
            waveform_bitmap: application.waveform_bitmap.clone(),
            animation_status: application.animation_status.clone(),
            bitmap_status: application.bitmap_status.clone(),
        };
        animation.start();
    });
}

#[no_mangle]
pub extern "C" fn __fui_native_pause_drawing_animation() {
    let _ = with_native_application(|application| {
        let animation = DrawingAnimation {
            state: Rc::downgrade(&application.state),
            immediate_invalidator: application.custom_drawable.invalidator(),
            bitmap_invalidator: application.bitmap_drawable.invalidator(),
            waveform_bitmap: application.waveform_bitmap.clone(),
            animation_status: application.animation_status.clone(),
            bitmap_status: application.bitmap_status.clone(),
        };
        animation.pause();
    });
}

#[no_mangle]
pub extern "C" fn __fui_native_step_drawing_animation() {
    let _ = with_native_application(|application| {
        let animation = DrawingAnimation {
            state: Rc::downgrade(&application.state),
            immediate_invalidator: application.custom_drawable.invalidator(),
            bitmap_invalidator: application.bitmap_drawable.invalidator(),
            waveform_bitmap: application.waveform_bitmap.clone(),
            animation_status: application.animation_status.clone(),
            bitmap_status: application.bitmap_status.clone(),
        };
        animation.dirty_step(false);
    });
}

#[no_mangle]
pub extern "C" fn __fui_native_reset_drawing_animation() {
    let _ = with_native_application(|application| {
        let animation = DrawingAnimation {
            state: Rc::downgrade(&application.state),
            immediate_invalidator: application.custom_drawable.invalidator(),
            bitmap_invalidator: application.bitmap_drawable.invalidator(),
            waveform_bitmap: application.waveform_bitmap.clone(),
            animation_status: application.animation_status.clone(),
            bitmap_status: application.bitmap_status.clone(),
        };
        animation.reset();
    });
}

#[no_mangle]
pub extern "C" fn __fui_native_update_waveform() {
    __fui_native_step_drawing_animation();
}

#[no_mangle]
pub extern "C" fn __fui_native_recompose_offscreen() {
    let _ = with_native_application(|application| {
        let sample = compose_offscreen(
            &application.offscreen_bitmap,
            application.state.custom_draw_phase.get().wrapping_add(1),
            application.state.dark_mode.get(),
        );
        application
            .state
            .offscreen_compositions
            .set(application.state.offscreen_compositions.get() + 1);
        application.state.offscreen_sample_rgba.set(sample);
        application.offscreen_status.text(format!(
            "Offscreen: {} composition(s) | center RGBA #{sample:08X}",
            application.state.offscreen_compositions.get()
        ));
        application.offscreen_drawable.mark_dirty();
    });
}

#[no_mangle]
pub extern "C" fn __fui_native_rasterize_retained() {
    let _ = with_native_application(|application| {
        rasterize_retained(
            &application.retained_bitmap,
            &application.retained_source,
            &application.retained_drawable.invalidator(),
            &application.retained_status,
            &application.state,
        );
    });
}

#[no_mangle]
pub extern "C" fn __fui_native_set_custom_draw_visible(visible: bool) {
    let _ = with_native_application(|application| {
        application.custom_drawable.visibility(if visible {
            Visibility::Normal
        } else {
            Visibility::Collapsed
        });
    });
}

#[no_mangle]
pub extern "C" fn __fui_native_timer_fire_count() -> u32 {
    with_native_application(|application| application.state.timer_fires.get()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_schedule_custom_draw_timer(delay_ms: i32) {
    let _ = with_native_application(|application| {
        if let Some(timer) = application.state.custom_draw_timer.borrow_mut().take() {
            cancel_timeout(timer);
        }
        let state = Rc::downgrade(&application.state);
        let drawable = application.custom_drawable.clone();
        let timer = set_timeout(delay_ms, move || {
            let Some(state) = state.upgrade() else {
                return;
            };
            state.timer_fires.set(state.timer_fires.get() + 1);
            state
                .custom_draw_phase
                .set(state.custom_draw_phase.get() + 1);
            drawable.mark_dirty();
        });
        application
            .state
            .custom_draw_timer
            .borrow_mut()
            .replace(timer);
    });
}

#[no_mangle]
pub extern "C" fn __fui_native_cancel_custom_draw_timer() {
    let _ = with_native_application(|application| {
        if let Some(timer) = application.state.custom_draw_timer.borrow_mut().take() {
            cancel_timeout(timer);
        }
    });
}

#[no_mangle]
pub extern "C" fn __fui_native_activation_count() -> u32 {
    with_native_application(|application| application.state.activations.get()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_context_menu_visible() -> bool {
    fui::bridge_callbacks::is_context_menu_visible()
}

#[no_mangle]
pub extern "C" fn __fui_native_tool_tip_visible() -> bool {
    fui::bridge_callbacks::is_tool_tip_visible()
}

#[no_mangle]
pub extern "C" fn __fui_native_schedule_ui_dispatch() {
    let state = with_native_application(|application| Rc::downgrade(&application.state));
    let dispatch = platform::UiDispatcher::prepare(move || {
        if let Some(state) = state.as_ref().and_then(Weak::upgrade) {
            state.ui_dispatches.set(state.ui_dispatches.get() + 1);
        }
    });
    std::thread::spawn(move || {
        dispatch.dispatch();
    })
    .join()
    .expect("native UI dispatch worker panicked");
}

#[no_mangle]
pub extern "C" fn __fui_native_schedule_cancelled_ui_dispatch() {
    let state = with_native_application(|application| Rc::downgrade(&application.state));
    let dispatch = platform::UiDispatcher::prepare(move || {
        if let Some(state) = state.as_ref().and_then(Weak::upgrade) {
            state.ui_dispatches.set(state.ui_dispatches.get() + 1);
        }
    });
    std::thread::spawn(move || drop(dispatch))
        .join()
        .expect("native UI dispatch cancellation worker panicked");
}

#[no_mangle]
pub extern "C" fn __fui_native_ui_dispatch_count() -> u32 {
    with_native_application(|application| application.state.ui_dispatches.get()).unwrap_or(0)
}

#[no_mangle]
/// # Safety
/// `text` must reference `length` readable bytes when `length` is non-zero.
pub unsafe extern "C" fn __fui_native_clipboard_roundtrip(text: *const u8, length: u32) -> bool {
    let bytes = unsafe { std::slice::from_raw_parts(text, length as usize) };
    let Ok(expected) = std::str::from_utf8(bytes) else {
        return false;
    };
    platform::write_clipboard_text(expected)
        && platform::read_clipboard_text().as_deref() == Some(expected)
}

#[no_mangle]
pub extern "C" fn __fui_native_start_test_file_dialog() -> u64 {
    let state = with_native_application(|application| Rc::downgrade(&application.state))
        .expect("native application must be mounted");
    if let Some(state) = state.upgrade() {
        state.file_dialog_result.borrow_mut().clear();
    }
    let request = platform::show_open_file_dialog(
        platform::NativeFileDialogOptions {
            filters: vec![platform::NativeFileFilter::new("Text", ["txt", "md"])],
            default_location: None,
            allow_multiple: true,
        },
        move |result| {
            let text = match result {
                platform::NativeFileDialogResult::Selected {
                    paths,
                    selected_filter,
                } => {
                    format!("selected:{}:{selected_filter:?}", paths.len())
                }
                platform::NativeFileDialogResult::Cancelled => "cancelled".to_string(),
                platform::NativeFileDialogResult::Error(error) => format!("error:{error}"),
            };
            if let Some(state) = state.upgrade() {
                *state.file_dialog_result.borrow_mut() = text;
            }
        },
    )
    .expect("test native file dialog should start");
    request.id()
}

#[no_mangle]
pub extern "C" fn __fui_native_file_dialog_result_length() -> u32 {
    with_native_application(|application| {
        application.state.file_dialog_result.borrow().len() as u32
    })
    .unwrap_or(0)
}

#[no_mangle]
/// # Safety
/// `destination` must reference `capacity` writable bytes when `capacity` is non-zero.
pub unsafe extern "C" fn __fui_native_copy_file_dialog_result(
    destination: *mut u8,
    capacity: u32,
) -> u32 {
    with_native_application(|application| {
        let value = application.state.file_dialog_result.borrow();
        let copied = value.len().min(capacity as usize);
        if !destination.is_null() && copied > 0 {
            unsafe { std::ptr::copy_nonoverlapping(value.as_ptr(), destination, copied) };
        }
        copied as u32
    })
    .unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_clear_drop_result() {
    let _ =
        with_native_application(|application| application.state.drop_result.borrow_mut().clear());
}

#[no_mangle]
pub extern "C" fn __fui_native_drop_result_length() -> u32 {
    with_native_application(|application| application.state.drop_result.borrow().len() as u32)
        .unwrap_or(0)
}

#[no_mangle]
/// # Safety
/// `destination` must reference `capacity` writable bytes when `capacity` is non-zero.
pub unsafe extern "C" fn __fui_native_copy_drop_result(destination: *mut u8, capacity: u32) -> u32 {
    with_native_application(|application| {
        let value = application.state.drop_result.borrow();
        let copied = value.len().min(capacity as usize);
        if !destination.is_null() && copied > 0 {
            unsafe { std::ptr::copy_nonoverlapping(value.as_ptr(), destination, copied) };
        }
        copied as u32
    })
    .unwrap_or(0)
}

#[no_mangle]
/// # Safety
/// `source` must reference `length` readable bytes when `length` is non-zero.
pub unsafe extern "C" fn __fui_native_set_test_image_source(source: *const u8, length: u32) {
    let bytes = unsafe { std::slice::from_raw_parts(source, length as usize) };
    let source = String::from_utf8_lossy(bytes);
    let image = image(9100);
    image.source(source.into_owned());
    let _ = with_native_application(|application| {
        application.state.test_image.borrow_mut().replace(image);
    });
}

#[no_mangle]
pub extern "C" fn __fui_native_test_image_state() -> u32 {
    with_native_application(|application| {
        application
            .state
            .test_image
            .borrow()
            .as_ref()
            .map_or(AssetLoadState::Idle as u32, |image| {
                image.asset_state() as u32
            })
    })
    .unwrap_or(AssetLoadState::Idle as u32)
}

#[no_mangle]
pub extern "C" fn __fui_native_test_image_width() -> f32 {
    with_native_application(|application| {
        application
            .state
            .test_image
            .borrow()
            .as_ref()
            .map_or(0.0, ImageNode::asset_width)
    })
    .unwrap_or(0.0)
}

#[no_mangle]
pub extern "C" fn __fui_native_test_image_height() -> f32 {
    with_native_application(|application| {
        application
            .state
            .test_image
            .borrow()
            .as_ref()
            .map_or(0.0, ImageNode::asset_height)
    })
    .unwrap_or(0.0)
}

#[no_mangle]
pub extern "C" fn __fui_native_clear_test_image() {
    let _ = with_native_application(|application| {
        application.state.test_image.borrow_mut().take();
    });
}
