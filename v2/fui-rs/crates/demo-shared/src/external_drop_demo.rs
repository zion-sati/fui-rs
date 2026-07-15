use crate::{demo_text, spacer, stage4_panel};
use fui::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

fn format_item_count(count: usize) -> String {
    match count {
        1 => "1 file".to_string(),
        _ => format!("{} files", count),
    }
}

fn describe_item(item: &ExternalDropItemInfo) -> String {
    let kind = match item.kind {
        ExternalDropItemKind::File => "file",
        _ => "item",
    };
    let mime = item.mime_type.as_deref().unwrap_or("unknown");
    format!(
        "{} ({}, {}, {} bytes)",
        item.name, kind, mime, item.size_bytes as u64
    )
}

fn output_file_name_label(output_file_name: &Option<String>) -> String {
    output_file_name
        .clone()
        .unwrap_or_else(|| "(stream)".to_string())
}

fn worker_result_hash_label(worker_result: &Option<String>) -> String {
    match worker_result.as_deref() {
        Some(value) => format!(" — hash: {}", value),
        None => String::new(),
    }
}

fn resolve_copy_file_name(file: &BrowserFile) -> String {
    let name = file.name();
    if let Some((base, ext)) = name.rsplit_once('.') {
        if !base.is_empty() && !ext.is_empty() {
            return format!("{}-copy.{}", base, ext);
        }
    }
    format!("{}-copy", name)
}

struct ExternalDropDemoState {
    drop_target: FlexBox,
    drop_title_text: TextNode,
    drop_body_text: TextNode,
    status_text: TextNode,
    items_text: TextNode,
    capability_text: TextNode,
    hint_text: TextNode,
    copy_button: Button,
    last_items: RefCell<Vec<ExternalDropItemInfo>>,
    hovering_accepted: RefCell<bool>,
    ignore_next_leave: RefCell<bool>,
    dropped_file: RefCell<Option<BrowserFile>>,
    active_copy_request: RefCell<Option<FileWorkerProcessRequest>>,
}

impl ExternalDropDemoState {
    fn new(
        drop_target: FlexBox,
        drop_title_text: TextNode,
        drop_body_text: TextNode,
        status_text: TextNode,
        items_text: TextNode,
        capability_text: TextNode,
        hint_text: TextNode,
        copy_button: Button,
    ) -> Self {
        Self {
            drop_target,
            drop_title_text,
            drop_body_text,
            status_text,
            items_text,
            capability_text,
            hint_text,
            copy_button,
            last_items: RefCell::new(Vec::new()),
            hovering_accepted: RefCell::new(false),
            ignore_next_leave: RefCell::new(false),
            dropped_file: RefCell::new(None),
            active_copy_request: RefCell::new(None),
        }
    }

    fn can_copy_dropped_file(&self) -> bool {
        self.dropped_file.borrow().is_some()
            && self.active_copy_request.borrow().is_none()
            && File::capabilities().can_process_in_worker_to_picked_file
    }

    fn sync_status(&self, label: impl Into<String>) {
        let label = label.into();
        self.status_text.text(&label);
        self.status_text.semantic_label(label);
    }

    fn sync_items(&self) {
        let items = self.last_items.borrow();
        let label = if items.is_empty() {
            "External drop items: none".to_string()
        } else {
            format!(
                "External drop items: {}",
                items
                    .iter()
                    .map(describe_item)
                    .collect::<Vec<_>>()
                    .join(" | ")
            )
        };
        self.items_text.text(&label);
        self.items_text.semantic_label(label);
    }

    fn sync_capabilities(&self) {
        let capabilities = File::capabilities();
        let label = format!(
            "File bridge capabilities: open={} • chunk-read={} • save={} • native-save-picker={} • worker-process-save={}",
            if capabilities.can_pick_open { "yes" } else { "no" },
            if capabilities.can_read_chunks { "yes" } else { "no" },
            if capabilities.can_save { "yes" } else { "no" },
            if capabilities.can_use_native_save_picker { "yes" } else { "no" },
            if capabilities.can_process_in_worker_to_picked_file { "yes" } else { "no" },
        );
        self.capability_text.text(&label);
        self.capability_text.semantic_label(label);
    }

    fn apply_theme(&self, theme: &Theme) {
        let hovering = *self.hovering_accepted.borrow();
        self.drop_target
            .bg_color(if hovering {
                theme.colors.accent_hovered
            } else if is_dark_mode() {
                0x111C2CFF
            } else {
                0xF8FAFCFF
            })
            .border(
                1.0,
                if hovering {
                    theme.colors.accent
                } else {
                    theme.colors.border
                },
            );
        self.drop_title_text.text_color(if hovering {
            theme.colors.surface
        } else {
            theme.colors.text_primary
        });
        self.drop_body_text.text_color(if hovering {
            theme.colors.surface
        } else {
            theme.colors.text_muted
        });
        let can_copy = self.can_copy_dropped_file();
        self.copy_button
            .enabled(can_copy)
            .bg_color(if can_copy {
                if is_dark_mode() {
                    0x111C2CFF
                } else {
                    0xF8FAFCFF
                }
            } else {
                theme.colors.surface
            })
            .border(1.0, theme.colors.border)
            .text_color(if can_copy {
                theme.colors.text_primary
            } else {
                theme.colors.text_muted
            });
        self.status_text.text_color(theme.colors.text_primary);
        self.items_text.text_color(theme.colors.text_muted);
        self.capability_text.text_color(theme.colors.text_muted);
        self.hint_text.text_color(theme.colors.text_muted);
    }

    fn replace_items(&self, items: Vec<ExternalDropItemInfo>) {
        self.last_items.replace(items);
        self.sync_items();
    }

    fn handle_external_drag(&self, args: ExternalDropEventArgs) -> DropProposal {
        self.ignore_next_leave.replace(false);
        self.replace_items(args.items.clone());
        if args.items.is_empty() {
            self.hovering_accepted.replace(false);
            self.sync_status("External drop status: ignoring non-file drag");
            self.apply_theme(&current_theme());
            return DropProposal::none();
        }
        self.hovering_accepted.replace(true);
        self.sync_status(format!(
            "External drop status: hovering {} • effect Copy",
            format_item_count(args.items.len())
        ));
        self.apply_theme(&current_theme());
        DropProposal::new(DragDropEffects::Copy, false)
    }

    fn handle_external_leave(&self, _args: ExternalDropEventArgs) {
        if *self.ignore_next_leave.borrow() {
            self.ignore_next_leave.replace(false);
            return;
        }
        self.hovering_accepted.replace(false);
        if self.last_items.borrow().is_empty() {
            self.sync_status("External drop status: idle");
        } else {
            self.sync_status("External drop status: ready for another drop");
        }
        self.apply_theme(&current_theme());
    }

    fn handle_external_drop(&self, args: ExternalDropEventArgs) {
        self.hovering_accepted.replace(false);
        self.ignore_next_leave.replace(true);
        self.replace_items(args.items.clone());
        self.dropped_file
            .replace(args.items.iter().find_map(|item| item.file.clone()));
        self.sync_status(format!(
            "External drop status: dropped {} • effect Copy",
            format_item_count(args.items.len())
        ));
        self.apply_theme(&current_theme());
    }

    fn handle_copy_progress(&self, progress: FileWorkerProcessProgress) {
        self.sync_status(format!(
            "External drop status: worker copying {} / {} bytes to {}...",
            progress.processed_bytes,
            progress.total_bytes,
            output_file_name_label(&progress.output_file_name)
        ));
    }

    fn handle_copy_complete(&self, result: FileWorkerProcessResult) {
        self.active_copy_request.borrow_mut().take();
        self.sync_status(format!(
            "External drop status: worker copied {} bytes to {}.{}",
            result.processed_bytes,
            output_file_name_label(&result.output_file_name),
            worker_result_hash_label(&result.worker_result)
        ));
        self.apply_theme(&current_theme());
    }

    fn handle_copy_error(&self, message: String) {
        self.active_copy_request.borrow_mut().take();
        self.sync_status(format!(
            "External drop status: worker copy failed • {}",
            message
        ));
        self.apply_theme(&current_theme());
    }

    fn start_dropped_file_copy(self: &Rc<Self>) {
        let Some(file) = self.dropped_file.borrow().clone() else {
            self.sync_status("External drop status: drop a file first");
            return;
        };
        if self.active_copy_request.borrow().is_some() {
            self.sync_status("External drop status: worker copy already running");
            return;
        }
        if !File::capabilities().can_process_in_worker_to_picked_file {
            self.sync_status("External drop status: this browser needs worker plus native save-picker support for the worker copy demo");
            return;
        }
        let suggested_name = resolve_copy_file_name(&file);
        let weak_state = Rc::downgrade(self);
        let request = File::process_file_in_worker(file.clone())
            .worker("./workers.wasm", "stage4FileProcessorWorker")
            .save_to_picked_file(suggested_name.clone())
            .on_progress({
                let weak_state = weak_state.clone();
                move |progress| {
                    if let Some(state) = weak_state.upgrade() {
                        state.handle_copy_progress(progress);
                    }
                }
            })
            .on_complete({
                let weak_state = weak_state.clone();
                move |result| {
                    if let Some(state) = weak_state.upgrade() {
                        state.handle_copy_complete(result);
                    }
                }
            })
            .on_error({
                move |event| {
                    if let Some(state) = weak_state.upgrade() {
                        state.handle_copy_error(event.message);
                    }
                }
            })
            .start();
        self.active_copy_request.replace(Some(request));
        self.sync_status(format!(
            "External drop status: starting worker copy for {} with transfer-list chunk handoff to {}",
            file.name(),
            suggested_name
        ));
        self.apply_theme(&current_theme());
    }
}

#[derive(Clone)]
pub(crate) struct ExternalDropDemoPanel {
    root: FlexBox,
    _state: Rc<ExternalDropDemoState>,
}

fui_component!(ExternalDropDemoPanel => root);

impl ExternalDropDemoPanel {
    pub(crate) fn new() -> Self {
        let root = ui! {
        stage4_panel("External file drop", 0xFFFFFFFF).fill_width()
            .semantic_label("Stage 4 external file drop card")
        };

        let drop_title_text = demo_text("Drop files here", 18.0, 0x111827FF);
        let drop_body_text = demo_text(
            "The drop target receives a first-class BrowserFile handle, then the sample copies it through a Worker-read plus picker-write pipeline. Chunk payloads hop back with zero-copy transfer-list handoff.",
            15.0,
            0x334155FF,
        );
        drop_body_text.text_limits(-1, 3);

        let drop_target = ui! {
            flex_box()
            .fill_width()
            .height(156.0, Unit::Pixel)
            .padding(18.0, 18.0, 18.0, 18.0)
            .corner_radius(20.0)
            .allow_external_drop(true)
            .semantic_role(SemanticRole::Form)
            .semantic_label("External file drop target")
            .child(&ui! {
                column()
                .fill_width()
                .child(&drop_title_text)
                .child(&spacer(8.0))
                .child(&drop_body_text)
            })
        };

        let status_text = ui! {
        demo_text("External drop status: idle", 15.0, 0x111827FF).semantic_label("External drop status: idle")
        };
        let items_text = ui! {
        demo_text("External drop items: none", 15.0, 0x475569FF).text_limits(-1, 4)
        };
        items_text.semantic_label("External drop items: none");
        let capability_text = ui! {
        demo_text("", 14.0, 0x475569FF).text_limits(-1, 3)
        };
        let hint_text = demo_text(
            "Drop a file here, then choose Save dropped file copy. This demo keeps the save picker on the main thread, reads the dropped file in a dedicated Worker, and transfers each ArrayBuffer chunk back with a postMessage transfer list before writing it into the picked target file.",
            15.0,
            0x475569FF,
        );
        hint_text.text_limits(-1, 6);
        let copy_button = ui! {
        button("Save dropped file copy")
            .semantic_label("Save dropped file copy")
            .fill_width()
            .height(48.0, Unit::Pixel)
            .padding(14.0, 14.0, 14.0, 14.0)
            .corner_radius(16.0)
        };

        root.child(&drop_target)
            .child(&spacer(14.0))
            .child(&status_text)
            .child(&spacer(6.0))
            .child(&items_text)
            .child(&spacer(8.0))
            .child(&copy_button)
            .child(&spacer(8.0))
            .child(&capability_text)
            .child(&spacer(10.0))
            .child(&hint_text);

        let state = Rc::new(ExternalDropDemoState::new(
            drop_target.clone(),
            drop_title_text,
            drop_body_text,
            status_text,
            items_text,
            capability_text,
            hint_text,
            copy_button.clone(),
        ));
        state.sync_capabilities();
        state.sync_items();
        state.apply_theme(&current_theme());

        drop_target.on_external_drag_enter({
            let state = Rc::downgrade(&state);
            move |args| {
                state
                    .upgrade()
                    .map(|state| state.handle_external_drag(args))
                    .unwrap_or_else(DropProposal::none)
            }
        });
        drop_target.on_external_drag_over({
            let state = Rc::downgrade(&state);
            move |args| {
                state
                    .upgrade()
                    .map(|state| state.handle_external_drag(args))
                    .unwrap_or_else(DropProposal::none)
            }
        });
        drop_target.on_external_drag_leave({
            let state = Rc::downgrade(&state);
            move |args| {
                if let Some(state) = state.upgrade() {
                    state.handle_external_leave(args);
                }
            }
        });
        drop_target.on_external_drop({
            let state = Rc::downgrade(&state);
            move |args| {
                if let Some(state) = state.upgrade() {
                    state.handle_external_drop(args);
                }
            }
        });
        copy_button.on_click({
            let state = Rc::downgrade(&state);
            move |_event| {
                if let Some(state) = state.upgrade() {
                    state.start_dropped_file_copy();
                }
            }
        });

        root.bind_theme({
            let state = Rc::downgrade(&state);
            move |_root, theme| {
                if let Some(state) = state.upgrade() {
                    state.apply_theme(&theme);
                }
            }
        });

        Self { root, _state: state }
    }
}
