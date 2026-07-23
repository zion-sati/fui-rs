use fui::prelude::*;
use fui::AssetLoadState;
use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::{Rc, Weak};

#[derive(Default)]
struct NativeApplicationState {
    activations: Cell<u32>,
    ui_dispatches: Cell<u32>,
    file_dialog_result: RefCell<String>,
    drop_result: RefCell<String>,
    drop_completed: Cell<bool>,
    test_image: RefCell<Option<ImageNode>>,
    theme_guards: RefCell<Vec<Subscription>>,
}

struct NativeApplication {
    root: FlexBox,
    scroll_root: ScrollBox,
    action_button: Button,
    body_text: Text,
    selection_text: Text,
    click_text: Text,
    context_link: NavLink,
    context_image: ImageNode,
    context_svg: SvgNode,
    context_editor: TextInput,
    drop_zone: FlexBox,
    state: Rc<NativeApplicationState>,
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
    let node = ui! {
        column()
        .fill_width()
        .height_len(auto())
        .padding(20.0, 20.0, 20.0, 20.0)
        .corner_radius(16.0)
        .children(children![
            ui! { themed_text(state, title, 19.0, 0x172033FF) },
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

fn build_application() -> NativeApplication {
    use_system_theme();
    let state = Rc::new(NativeApplicationState::default());
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
    let action_menu = context_menu(vec![
        MenuItem::new("Increment", ContextMenuAction::OpenLink).on_invoke({
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
        }),
    ]);
    let action = button("Increment click count");
    action
        .node_id("native-action")
        .width(220.0, Unit::Pixel)
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
    controls_card.children(children![
        action.clone(),
        ui! { spacer(8.0) },
        click_text,
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
                .is_some_and(|path| platform::open_file(path));
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
                .is_some_and(|path| platform::reveal_file(path));
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
        ui! { themed_text(&state, "你好", 18.0, 0x172033FF) },
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
        selection_text,
        click_text,
        context_link,
        context_image,
        context_svg,
        context_editor,
        drop_zone: drop_card,
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
pub extern "C" fn __fui_native_activation_count() -> u32 {
    with_native_application(|application| application.state.activations.get()).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn __fui_native_context_menu_visible() -> bool {
    fui::bridge_callbacks::is_context_menu_visible()
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
