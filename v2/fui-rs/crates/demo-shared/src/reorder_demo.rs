use crate::{demo_text, spacer, stage4_panel};
use fui::bindings::ui;
use fui::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use {AlignItems, CursorStyle, JustifyContent, PositionType, Visibility};

const REORDER_DRAG_FORMAT: &str = "application/x-effindom-reorder-item-id";
const REORDER_MARKER_HEIGHT_PX: f32 = 8.0;
const REORDER_ROW_BODY_HEIGHT_PX: f32 = 72.0;
const REORDER_SLOT_HEIGHT_PX: f32 = REORDER_MARKER_HEIGHT_PX + REORDER_ROW_BODY_HEIGHT_PX;
const REORDER_END_SLOT_HEIGHT_PX: f32 = 44.0;
const REORDER_VIEWPORT_HEIGHT_PX: f32 = 248.0;
const REORDER_AUTOSCROLL_EDGE_ZONE_PX: f32 = 48.0;
const REORDER_AUTOSCROLL_MAX_OUTSIDE_PX: f32 = 120.0;
const REORDER_AUTOSCROLL_MIN_STEP_PX: f32 = 4.0;
const REORDER_AUTOSCROLL_MAX_STEP_PX: f32 = 34.0;
const AUTOSCROLL_DELAY_MS: i32 = 16;
const PREVIEW_WIDTH_PX: f32 = 272.0;
const PREVIEW_HEIGHT_PX: f32 = 116.0;
const PREVIEW_OFFSET_X_PX: f32 = 2.0;
const PREVIEW_OFFSET_Y_PX: f32 = 2.0;
const PREVIEW_MARGIN_PX: f32 = 12.0;

#[derive(Clone)]
struct ReorderDemoItem {
    id: &'static str,
    label: &'static str,
    detail: &'static str,
}

#[derive(Clone, Copy, Default)]
struct ReorderVisibleRange {
    first_visible_index: i32,
    last_visible_index: i32,
}

fn create_reorder_demo_items() -> Vec<ReorderDemoItem> {
    vec![
        ReorderDemoItem {
            id: "core-rename",
            label: "Document Core rename",
            detail: "Keep Tier 1 references consistently renamed to Core.",
        },
        ReorderDemoItem {
            id: "font-cache",
            label: "Audit font shard cache",
            detail: "Check cache bounds, eviction, and diagnostics before release.",
        },
        ReorderDemoItem {
            id: "drag-demo",
            label: "Add drag reorder demo",
            detail: "Prove the phase-4 drag/drop controller on a real routed sample.",
        },
        ReorderDemoItem {
            id: "nested-scroll",
            label: "Write nested scroll test",
            detail: "Lock drop-target routing when a scrollable list sits inside a scrolled route.",
        },
        ReorderDemoItem {
            id: "key-router",
            label: "Split key router",
            detail: "Keep keyboard routing focused instead of growing another god file.",
        },
        ReorderDemoItem {
            id: "semantics",
            label: "Tighten semantic ordering",
            detail: "Make sure reorder status reads in the same order the canvas shows.",
        },
        ReorderDemoItem {
            id: "find-mirror",
            label: "Probe find mirror fallback",
            detail: "Double-check the hidden DOM mirror when large text windows shift.",
        },
        ReorderDemoItem {
            id: "drop-cursor",
            label: "Validate drop cursor states",
            detail: "Keep drag affordances coherent while the source stays captured.",
        },
    ]
}

fn find_reorder_item_index(items: &[ReorderDemoItem], item_id: &str) -> i32 {
    items
        .iter()
        .position(|item| item.id == item_id)
        .map(|index| index as i32)
        .unwrap_or(-1)
}

fn compute_reorder_content_height(item_count: i32) -> f32 {
    let clamped = item_count.max(0);
    (clamped as f32 * REORDER_SLOT_HEIGHT_PX) + REORDER_END_SLOT_HEIGHT_PX
}

fn compute_reorder_visible_range(
    item_count: i32,
    offset_y: f32,
    viewport_height: f32,
) -> ReorderVisibleRange {
    if item_count <= 0 {
        return ReorderVisibleRange {
            first_visible_index: 0,
            last_visible_index: -1,
        };
    }
    let mut first_visible_index = (offset_y / REORDER_SLOT_HEIGHT_PX).floor() as i32;
    first_visible_index = first_visible_index.clamp(0, item_count - 1);
    let effective_viewport_height = if viewport_height > 0.0 {
        viewport_height
    } else {
        REORDER_VIEWPORT_HEIGHT_PX
    };
    let mut last_visible_index =
        ((offset_y + effective_viewport_height - 1.0) / REORDER_SLOT_HEIGHT_PX).floor() as i32;
    if last_visible_index < first_visible_index {
        last_visible_index = first_visible_index;
    }
    if last_visible_index > item_count - 1 {
        last_visible_index = item_count - 1;
    }
    ReorderVisibleRange {
        first_visible_index,
        last_visible_index,
    }
}

fn normalize_reorder_insertion_index(
    source_index: i32,
    raw_insertion_index: i32,
    item_count: i32,
) -> i32 {
    if item_count <= 0 {
        return 0;
    }
    let mut clamped = raw_insertion_index.clamp(0, item_count);
    if source_index >= 0 && source_index < item_count && source_index < clamped {
        clamped -= 1;
    }
    clamped.clamp(0, item_count - 1)
}

fn move_reorder_item(
    items: &mut Vec<ReorderDemoItem>,
    item_id: &str,
    raw_insertion_index: i32,
) -> bool {
    let source_index = find_reorder_item_index(items, item_id);
    if source_index < 0 || items.len() <= 1 {
        return false;
    }
    let target_index =
        normalize_reorder_insertion_index(source_index, raw_insertion_index, items.len() as i32);
    if target_index == source_index {
        return false;
    }
    let moved = items.remove(source_index as usize);
    items.insert(target_index as usize, moved);
    true
}

fn compute_reorder_max_scroll_offset(item_count: i32, viewport_height: f32) -> f32 {
    (compute_reorder_content_height(item_count) - viewport_height).max(0.0)
}

fn clamp01(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

fn compute_reorder_auto_scroll_step(activation_distance: f32) -> f32 {
    if activation_distance <= 0.0 {
        return 0.0;
    }
    let max_activation = REORDER_AUTOSCROLL_EDGE_ZONE_PX + REORDER_AUTOSCROLL_MAX_OUTSIDE_PX;
    let normalized = clamp01(activation_distance / max_activation);
    let eased = normalized * normalized;
    REORDER_AUTOSCROLL_MIN_STEP_PX
        + ((REORDER_AUTOSCROLL_MAX_STEP_PX - REORDER_AUTOSCROLL_MIN_STEP_PX) * eased)
}

fn compute_reorder_pointer_auto_scroll_delta(
    pointer_y: f32,
    viewport_top_y: f32,
    viewport_height: f32,
) -> f32 {
    if viewport_height <= 0.0 {
        return 0.0;
    }
    let viewport_bottom_y = viewport_top_y + viewport_height;
    let top_zone_bottom = viewport_top_y + REORDER_AUTOSCROLL_EDGE_ZONE_PX;
    if pointer_y <= top_zone_bottom {
        return -compute_reorder_auto_scroll_step(top_zone_bottom - pointer_y);
    }
    let bottom_zone_top = viewport_bottom_y - REORDER_AUTOSCROLL_EDGE_ZONE_PX;
    if pointer_y >= bottom_zone_top {
        return compute_reorder_auto_scroll_step(pointer_y - bottom_zone_top);
    }
    0.0
}

fn compute_next_reorder_auto_scroll_offset(
    current_offset_y: f32,
    delta_y: f32,
    item_count: i32,
    viewport_height: f32,
) -> f32 {
    if delta_y == 0.0 {
        return current_offset_y;
    }
    let max_offset = compute_reorder_max_scroll_offset(item_count, viewport_height);
    (current_offset_y + delta_y).clamp(0.0, max_offset)
}

fn compute_reorder_edge_insertion_index(
    direction: i32,
    item_count: i32,
    visible_range: ReorderVisibleRange,
) -> i32 {
    if direction < 0 {
        return visible_range.first_visible_index;
    }
    if direction > 0 {
        let edge = visible_range.last_visible_index + 1;
        return if edge < item_count { edge } else { item_count };
    }
    -1
}

fn surface_color() -> u32 {
    if is_dark_mode() {
        0x111C2CFF
    } else {
        0xFFFFFFFF
    }
}

fn alt_card_color(index: i32) -> u32 {
    if is_dark_mode() {
        if (index & 1) == 0 {
            0x0F1A28FF
        } else {
            0x132133FF
        }
    } else if (index & 1) == 0 {
        0xFFFFFFFF
    } else {
        0xF8FAFCFF
    }
}

fn vertical_spacer(height: f32) -> FlexBox {
    let spacer = ui! {
        flex_box().fill_width().height(height, Unit::Pixel)
    };
    spacer
}

struct ReorderRowView {
    raw_index: i32,
    marker: FlexBox,
    grip_label: TextNode,
    grip: FlexBox,
    title_text: TextNode,
    detail_text: TextNode,
    card: FlexBox,
    slot: FlexBox,
}

impl ReorderRowView {
    fn bind_item(&self, item: &ReorderDemoItem) {
        self.title_text.text(item.label);
        self.detail_text.text(item.detail);
        self.grip
            .semantic_label(format!("Drag grip for {}", item.label));
        self.card
            .semantic_role(SemanticRole::StaticText)
            .semantic_label(format!(
                "Reorder item {}: {}",
                self.raw_index + 1,
                item.label
            ));
    }

    fn apply_theme(
        &self,
        active_item_id: Option<&str>,
        raw_insertion_index: i32,
        item: &ReorderDemoItem,
        theme: &Theme,
    ) {
        let is_source = Some(item.id) == active_item_id;
        let marker_visible = raw_insertion_index == self.raw_index;
        self.marker
            .bg_color(theme.colors.accent)
            .opacity(if marker_visible { 1.0 } else { 0.0 });
        self.card
            .bg_color(if is_source {
                theme.colors.accent_hovered
            } else {
                alt_card_color(self.raw_index)
            })
            .border(
                1.0,
                if is_source {
                    theme.colors.accent
                } else {
                    theme.colors.border
                },
            );
        self.title_text.text_color(if is_source {
            theme.colors.surface
        } else {
            theme.colors.text_primary
        });
        self.detail_text.text_color(if is_source {
            theme.colors.surface
        } else {
            theme.colors.text_muted
        });
        self.grip
            .bg_color(if is_source {
                theme.colors.accent
            } else {
                surface_color()
            })
            .border(
                1.0,
                if is_source {
                    theme.colors.accent_pressed
                } else {
                    theme.colors.border
                },
            )
            .cursor(if is_source {
                CursorStyle::Grabbing
            } else {
                CursorStyle::Grab
            });
        self.grip_label
            .text_color(if is_source {
                theme.colors.surface
            } else {
                theme.colors.text_primary
            })
            .cursor(if is_source {
                CursorStyle::Grabbing
            } else {
                CursorStyle::Grab
            });
    }
}

struct ReorderDemoState {
    root: FlexBox,
    scroll_box: ScrollBox,
    order_status_text: TextNode,
    drag_status_text: TextNode,
    viewport_status_text: TextNode,
    preview_title_text: TextNode,
    preview_detail_text: TextNode,
    preview_effect_text: TextNode,
    preview_ghost: FlexBox,
    end_marker: FlexBox,
    end_drop_zone: FlexBox,
    rows: Vec<ReorderRowView>,
    items: RefCell<Vec<ReorderDemoItem>>,
    active_drag_item_id: RefCell<Option<String>>,
    raw_insertion_index: Cell<i32>,
    auto_scroll_delta_y: Cell<f32>,
    drag_status_message: RefCell<String>,
    preview_pointer_x: Cell<f32>,
    preview_pointer_y: Cell<f32>,
    preview_effect: Cell<DragDropEffects>,
    preview_insertion_slot: Cell<i32>,
    auto_scroll_timer: RefCell<Option<TimerHandle>>,
    self_weak: RefCell<Weak<RefCell<ReorderDemoState>>>,
}

impl ReorderDemoState {
    fn begin_drag(&self, raw_index: i32) -> Option<DragDataObject> {
        let item = self.items.borrow().get(raw_index as usize).cloned()?;
        ui::clear_current_selection();
        self.active_drag_item_id
            .borrow_mut()
            .replace(String::from(item.id));
        let source_index = find_reorder_item_index(&self.items.borrow(), item.id);
        self.raw_insertion_index
            .set(if source_index >= 0 { source_index } else { -1 });
        self.preview_pointer_x.set(f32::NAN);
        self.preview_pointer_y.set(f32::NAN);
        self.preview_effect.set(DragDropEffects::None);
        self.preview_insertion_slot.set(-1);
        self.drag_status_message
            .replace(format!("Reorder drag status: dragging {}", item.label));
        self.sync_all();
        Some(
            DragDataObject::new()
                .set_format(REORDER_DRAG_FORMAT, item.id)
                .set_text(item.label),
        )
    }

    fn complete_drag(&self, item_id: &str, effect: DragDropEffects) {
        let item = self.find_item(Some(item_id));
        let item_label = item.as_ref().map(|item| item.label).unwrap_or("item");
        self.active_drag_item_id.borrow_mut().take();
        self.raw_insertion_index.set(-1);
        self.preview_pointer_x.set(f32::NAN);
        self.preview_pointer_y.set(f32::NAN);
        self.preview_effect.set(DragDropEffects::None);
        self.preview_insertion_slot.set(-1);
        self.stop_auto_scroll();
        if effect == DragDropEffects::Move {
            let new_index = find_reorder_item_index(&self.items.borrow(), item_id);
            self.drag_status_message.replace(format!(
                "Reorder drag status: moved {} to slot {}",
                item_label,
                new_index + 1
            ));
        } else {
            self.drag_status_message
                .replace(format!("Reorder drag status: cancelled {}", item_label));
        }
        self.sync_all();
    }

    fn preview_insertion(&self, args: DragEventArgs, raw_insertion_index: i32) -> DropProposal {
        let Some(item_id) = args.session.data.get_format(REORDER_DRAG_FORMAT) else {
            self.stop_auto_scroll();
            return DropProposal::none();
        };
        if find_reorder_item_index(&self.items.borrow(), &item_id) < 0 {
            self.stop_auto_scroll();
            return DropProposal::none();
        }
        self.active_drag_item_id
            .borrow_mut()
            .replace(item_id.clone());
        self.raw_insertion_index.set(raw_insertion_index);
        let source_index = find_reorder_item_index(&self.items.borrow(), &item_id);
        let normalized_index = normalize_reorder_insertion_index(
            source_index,
            raw_insertion_index,
            self.items.borrow().len() as i32,
        );
        self.set_auto_scroll_delta(self.compute_pointer_auto_scroll_delta(args.y));
        self.preview_pointer_x.set(args.x);
        self.preview_pointer_y.set(args.y);
        self.preview_effect.set(DragDropEffects::Move);
        self.preview_insertion_slot.set(normalized_index);
        self.drag_status_message.replace(format!(
            "Reorder drag status: preview slot {}",
            normalized_index + 1
        ));
        self.sync_all();
        DropProposal::new(DragDropEffects::Move, true)
    }

    fn handle_target_leave(&self, args: DragEventArgs) {
        if self.active_drag_item_id.borrow().is_none() {
            return;
        }
        self.preview_pointer_x.set(args.x);
        self.preview_pointer_y.set(args.y);
        let visible_range = self.read_visible_range();
        self.set_auto_scroll_delta(self.compute_pointer_auto_scroll_delta(args.y));
        if self.auto_scroll_delta_y.get() == 0.0 {
            self.raw_insertion_index.set(-1);
            self.preview_effect.set(DragDropEffects::None);
            self.preview_insertion_slot.set(-1);
            self.drag_status_message.replace(format!(
                "Reorder drag status: dragging {}",
                self.active_drag_label()
            ));
        } else {
            let direction = if self.auto_scroll_delta_y.get() < 0.0 {
                -1
            } else {
                1
            };
            self.raw_insertion_index
                .set(compute_reorder_edge_insertion_index(
                    direction,
                    self.items.borrow().len() as i32,
                    visible_range,
                ));
            let source_index = self
                .active_drag_item_id
                .borrow()
                .as_deref()
                .map(|item_id| find_reorder_item_index(&self.items.borrow(), item_id))
                .unwrap_or(-1);
            let raw = self.raw_insertion_index.get();
            self.preview_effect.set(DragDropEffects::Move);
            self.preview_insertion_slot
                .set(if raw < 0 || source_index < 0 {
                    -1
                } else {
                    normalize_reorder_insertion_index(
                        source_index,
                        raw,
                        self.items.borrow().len() as i32,
                    )
                });
            self.drag_status_message.replace(format!(
                "Reorder drag status: auto-scrolling {}",
                if direction < 0 { "up" } else { "down" }
            ));
        }
        self.sync_all();
    }

    fn drop_at_preview(&self, args: DragEventArgs) {
        let Some(item_id) = args.session.data.get_format(REORDER_DRAG_FORMAT) else {
            return;
        };
        if self.raw_insertion_index.get() < 0 {
            return;
        }
        if move_reorder_item(
            &mut self.items.borrow_mut(),
            &item_id,
            self.raw_insertion_index.get(),
        ) {
            let items = self.items.borrow();
            for (index, row) in self.rows.iter().enumerate() {
                row.bind_item(&items[index]);
            }
        }
        self.sync_all();
    }

    fn handle_end_drag_over(&self, args: DragEventArgs) -> DropProposal {
        self.preview_insertion(args, self.items.borrow().len() as i32)
    }

    fn handle_auto_scroll_timer(owner: &Rc<RefCell<Self>>) {
        let (current_offset, delta_y, item_count, viewport_height) = {
            let state = owner.borrow();
            if state.active_drag_item_id.borrow().is_none()
                || state.auto_scroll_delta_y.get() == 0.0
            {
                drop(state);
                owner.borrow().stop_auto_scroll();
                return;
            }
            let item_count = state.items.borrow().len() as i32;
            let values = (
                state.scroll_box.scroll_state().offset_y(),
                state.auto_scroll_delta_y.get(),
                item_count,
                state.read_viewport_height(),
            );
            values
        };
        let next_offset = compute_next_reorder_auto_scroll_offset(
            current_offset,
            delta_y,
            item_count,
            viewport_height,
        );
        if next_offset == current_offset {
            owner.borrow().stop_auto_scroll();
            return;
        }
        let scroll_box = owner.borrow().scroll_box.clone();
        scroll_box.scroll_offset(0.0, next_offset);
        {
            let state = owner.borrow();
            let visible_range = state.read_visible_range();
            let direction = if delta_y < 0.0 { -1 } else { 1 };
            state
                .raw_insertion_index
                .set(compute_reorder_edge_insertion_index(
                    direction,
                    item_count,
                    visible_range,
                ));
            state.drag_status_message.replace(format!(
                "Reorder drag status: auto-scrolling {}",
                if direction < 0 { "up" } else { "down" }
            ));
            state.sync_all();
        }
        owner.borrow().arm_auto_scroll_timer();
    }

    fn active_drag_label(&self) -> String {
        self.find_item(self.active_drag_item_id.borrow().as_deref())
            .map(|item| String::from(item.label))
            .unwrap_or_else(|| String::from("item"))
    }

    fn sync_preview_ghost(&self) {
        let item = self.find_item(self.active_drag_item_id.borrow().as_deref());
        if item.is_none()
            || self.preview_pointer_x.get().is_nan()
            || self.preview_pointer_y.get().is_nan()
        {
            self.preview_ghost
                .visibility(Visibility::Hidden)
                .opacity(0.0);
            return;
        }
        let item = item.unwrap();
        self.preview_title_text.text(item.label);
        self.preview_detail_text.text(item.detail);
        if self.preview_effect.get() == DragDropEffects::Move
            && self.preview_insertion_slot.get() >= 0
        {
            self.preview_effect_text.text(format!(
                "Drop to move to slot {}",
                self.preview_insertion_slot.get() + 1
            ));
        } else {
            self.preview_effect_text
                .text("Release outside the list to cancel");
        }
        self.preview_ghost
            .semantic_label(format!("Reorder drag preview for {}", item.label));
        let section_bounds = self.root.get_bounds();
        let section_width = if section_bounds[2] > 0.0 {
            section_bounds[2]
        } else {
            viewport_width_signal()
                .value()
                .max(PREVIEW_WIDTH_PX + (PREVIEW_MARGIN_PX * 2.0))
        };
        let section_height = if section_bounds[3] > 0.0 {
            section_bounds[3]
        } else {
            viewport_height_signal()
                .value()
                .max(PREVIEW_HEIGHT_PX + (PREVIEW_MARGIN_PX * 2.0))
        };
        let pointer_local = self
            .root
            .absolute_to_local_position(self.preview_pointer_x.get(), self.preview_pointer_y.get());
        let max_x = PREVIEW_MARGIN_PX.max(section_width - PREVIEW_WIDTH_PX - PREVIEW_MARGIN_PX);
        let max_y = PREVIEW_MARGIN_PX.max(section_height - PREVIEW_HEIGHT_PX - PREVIEW_MARGIN_PX);
        let preview_x = (pointer_local[0] + PREVIEW_OFFSET_X_PX).clamp(PREVIEW_MARGIN_PX, max_x);
        let preview_y = (pointer_local[1] + PREVIEW_OFFSET_Y_PX).clamp(PREVIEW_MARGIN_PX, max_y);
        self.preview_ghost.position(preview_x, preview_y);
        self.preview_ghost
            .visibility(Visibility::Normal)
            .opacity(0.96);
    }

    fn sync_all(&self) {
        let mut summary = String::from("Reorder order: ");
        for (index, item) in self.items.borrow().iter().enumerate() {
            if index > 0 {
                summary.push_str(" | ");
            }
            summary.push_str(item.label);
        }
        self.order_status_text.text(summary);
        self.drag_status_text
            .text(self.drag_status_message.borrow().clone());
        self.sync_viewport_status();
        self.sync_preview_ghost();
        self.apply_theme(current_theme());
    }

    fn sync_viewport_status(&self) {
        let visible_range = self.read_visible_range();
        let first_visible = if visible_range.last_visible_index < 0 {
            0
        } else {
            visible_range.first_visible_index + 1
        };
        let last_visible = if visible_range.last_visible_index < 0 {
            0
        } else {
            visible_range.last_visible_index + 1
        };
        self.viewport_status_text.text(format!(
            "Reorder viewport status: offset {} | visible {}-{}",
            self.scroll_box.scroll_state().offset_y() as i32,
            first_visible,
            last_visible
        ));
    }

    fn read_visible_range(&self) -> ReorderVisibleRange {
        compute_reorder_visible_range(
            self.items.borrow().len() as i32,
            self.scroll_box.scroll_state().offset_y(),
            self.read_viewport_height(),
        )
    }

    fn read_viewport_height(&self) -> f32 {
        let current = self.scroll_box.scroll_state().viewport_height();
        if current > 0.0 {
            current
        } else {
            REORDER_VIEWPORT_HEIGHT_PX
        }
    }

    fn find_item(&self, item_id: Option<&str>) -> Option<ReorderDemoItem> {
        let item_id = item_id?;
        let index = find_reorder_item_index(&self.items.borrow(), item_id);
        if index < 0 {
            None
        } else {
            Some(self.items.borrow()[index as usize].clone())
        }
    }

    fn set_auto_scroll_delta(&self, next_delta_y: f32) {
        let delta_difference = (self.auto_scroll_delta_y.get() - next_delta_y).abs();
        if delta_difference <= 0.05 {
            if next_delta_y == 0.0 {
                self.stop_auto_scroll();
                return;
            }
            if next_delta_y != 0.0 {
                self.arm_auto_scroll_timer();
            }
            return;
        }
        self.auto_scroll_delta_y.set(next_delta_y);
        if next_delta_y == 0.0 {
            self.stop_auto_scroll();
            return;
        }
        self.arm_auto_scroll_timer();
    }

    fn compute_pointer_auto_scroll_delta(&self, pointer_y: f32) -> f32 {
        let bounds = self.scroll_box.viewport().get_bounds();
        compute_reorder_pointer_auto_scroll_delta(pointer_y, bounds[1], bounds[3])
    }

    fn arm_auto_scroll_timer(&self) {
        if let Some(existing) = self.auto_scroll_timer.borrow_mut().take() {
            cancel_timeout(existing);
        }
        let weak = self.self_weak.borrow().clone();
        let handle = set_timeout(AUTOSCROLL_DELAY_MS, move || {
            if let Some(state) = weak.upgrade() {
                ReorderDemoState::handle_auto_scroll_timer(&state);
            }
        });
        self.auto_scroll_timer.borrow_mut().replace(handle);
    }

    fn stop_auto_scroll(&self) {
        self.auto_scroll_delta_y.set(0.0);
        if let Some(handle) = self.auto_scroll_timer.borrow_mut().take() {
            cancel_timeout(handle);
        }
    }

    fn apply_theme(&self, theme: Theme) {
        for (index, row) in self.rows.iter().enumerate() {
            row.apply_theme(
                self.active_drag_item_id.borrow().as_deref(),
                self.raw_insertion_index.get(),
                &self.items.borrow()[index],
                &theme,
            );
        }
        self.end_marker.bg_color(theme.colors.accent).opacity(
            if self.raw_insertion_index.get() == self.items.borrow().len() as i32 {
                1.0
            } else {
                0.0
            },
        );
        self.end_drop_zone
            .bg_color(surface_color())
            .border(1.0, theme.colors.border);
        self.order_status_text.text_color(theme.colors.text_primary);
        self.drag_status_text.text_color(theme.colors.text_muted);
        self.viewport_status_text
            .text_color(theme.colors.text_muted);
        self.preview_ghost
            .bg_color(theme.colors.surface)
            .border(
                1.0,
                if self.preview_effect.get() == DragDropEffects::Move {
                    theme.colors.accent
                } else {
                    theme.colors.border
                },
            )
            .drop_shadow(theme.colors.panel_shadow, 0.0, 10.0, 24.0, 0.0);
        self.preview_title_text
            .text_color(theme.colors.text_primary);
        self.preview_detail_text.text_color(theme.colors.text_muted);
        self.preview_effect_text.text_color(
            if self.preview_effect.get() == DragDropEffects::Move {
                theme.colors.accent
            } else {
                theme.colors.text_primary
            },
        );
        self.scroll_box
            .border(1.0, theme.colors.border)
            .bg_color(surface_color());
        self.scroll_box
            .vertical_scrollbar()
            .track_color(theme.colors.scrollbar_track)
            .thumb_color(theme.colors.scrollbar_thumb);
    }
}

#[derive(Clone)]
pub(crate) struct ReorderDemoPanel {
    root: FlexBox,
    _state: Rc<RefCell<ReorderDemoState>>,
    _guards: Rc<Vec<Subscription>>,
}

fui_component!(ReorderDemoPanel => root, owners: [_state, _guards]);

impl ReorderDemoPanel {
    pub(crate) fn new() -> Self {
        let panel = ui! {
        stage4_panel("Drag-and-drop reorder", 0xFFFFFFFF)
            .fill_width()
            .semantic_label("Stage 4 drag-and-drop reorder card")
            .clip_to_bounds(false)
        };

        let order_status_text = demo_text("", 15.0, 0x111827FF);
        let drag_status_text = demo_text("", 15.0, 0x475569FF);
        let viewport_status_text = demo_text("", 15.0, 0x475569FF);
        let preview_title_text = demo_text("", 16.0, 0x111827FF);
        let preview_detail_text = ui! {
        demo_text("", 14.0, 0x475569FF).text_limits(-1, 2)
        };
        let preview_effect_text = demo_text("", 13.0, 0x111827FF);

        let preview_ghost = ui! {
            flex_box()
            .position_type(PositionType::Absolute)
            .width(PREVIEW_WIDTH_PX, Unit::Pixel)
            .padding(14.0, 14.0, 14.0, 14.0)
            .corner_radius(18.0)
            .child(&ui! {
                column()
                .fill_width()
                .child(&demo_text("Dragging", 13.0, 0x475569FF))
                .child(&vertical_spacer(6.0))
                .child(&preview_title_text)
                .child(&vertical_spacer(4.0))
                .child(&preview_detail_text)
                .child(&vertical_spacer(10.0))
                .child(&preview_effect_text)
            })
            .opacity(0.0)
            .visibility(Visibility::Hidden)
            .semantic_role(SemanticRole::StaticText)
            .semantic_label("Reorder drag preview")
        };

        let scroll_content = ui! {
            flex_box()
            .fill_width()
            .flex_direction(FlexDirection::Column)
        };

        let scroll_box = ui! {
            scroll_box()
            .fill_width()
            .height(REORDER_VIEWPORT_HEIGHT_PX, Unit::Pixel)
            .semantic_role(SemanticRole::StaticText)
            .semantic_label("Reorder demo viewport")
            .scroll_enabled_x(false)
            .scroll_enabled_y(true)
            .vertical_scrollbar_visibility(ScrollBarVisibility::Always)
            .horizontal_scrollbar_visibility(ScrollBarVisibility::Never)
            .persist_scroll(false)
            .child(&scroll_content)
        };
        scroll_box
            .vertical_scrollbar()
            .track_width(12.0)
            .thumb_width(8.0)
            .thumb_min_height(36.0)
            .track_corner_radius(6.0)
            .thumb_corner_radius(4.0);

        let end_marker = ui! {
            flex_box()
            .fill_width()
            .height(REORDER_MARKER_HEIGHT_PX, Unit::Pixel)
            .corner_radius(REORDER_MARKER_HEIGHT_PX * 0.5)
            .opacity(0.0)
        };
        let end_drop_zone = ui! {
            flex_box().fill_width().height(44.0, Unit::Pixel)
        };

        let state = Rc::new(RefCell::new(ReorderDemoState {
            root: panel.clone(),
            scroll_box: scroll_box.clone(),
            order_status_text: order_status_text.clone(),
            drag_status_text: drag_status_text.clone(),
            viewport_status_text: viewport_status_text.clone(),
            preview_title_text: preview_title_text.clone(),
            preview_detail_text: preview_detail_text.clone(),
            preview_effect_text: preview_effect_text.clone(),
            preview_ghost: preview_ghost.clone(),
            end_marker: end_marker.clone(),
            end_drop_zone: end_drop_zone.clone(),
            rows: Vec::new(),
            items: RefCell::new(create_reorder_demo_items()),
            active_drag_item_id: RefCell::new(None),
            raw_insertion_index: Cell::new(-1),
            auto_scroll_delta_y: Cell::new(0.0),
            drag_status_message: RefCell::new(String::from("Reorder drag status: idle")),
            preview_pointer_x: Cell::new(f32::NAN),
            preview_pointer_y: Cell::new(f32::NAN),
            preview_effect: Cell::new(DragDropEffects::None),
            preview_insertion_slot: Cell::new(-1),
            auto_scroll_timer: RefCell::new(None),
            self_weak: RefCell::new(Weak::new()),
        }));
        *state.borrow().self_weak.borrow_mut() = Rc::downgrade(&state);

        let weak = Rc::downgrade(&state);
        {
            let items = state.borrow().items.borrow().clone();
            let mut rows = Vec::new();
            for (index, item) in items.iter().enumerate() {
                let pending_drag_item_id = Rc::new(RefCell::new(None::<String>));
                let marker = ui! {
                    flex_box()
                    .fill_width()
                    .height(REORDER_MARKER_HEIGHT_PX, Unit::Pixel)
                    .corner_radius(REORDER_MARKER_HEIGHT_PX * 0.5)
                    .opacity(0.0)
                };

                let grip_label = ui! {
                demo_text("Drag", 14.0, 0x475569FF).cursor(CursorStyle::Grab)
                };
                grip_label
                    .drag_data({
                        let weak = weak.clone();
                        let pending_drag_item_id = pending_drag_item_id.clone();
                        move || {
                            let state = weak.upgrade()?;
                            let item_id = {
                                let state_ref = state.borrow();
                                let item_id = state_ref
                                    .items
                                    .borrow()
                                    .get(index)
                                    .map(|item| String::from(item.id))?;
                                item_id
                            };
                            pending_drag_item_id.borrow_mut().replace(item_id);
                            let drag_data = state.borrow().begin_drag(index as i32);
                            drag_data
                        }
                    })
                    .drag_allowed_effects(DragDropEffects::Move)
                    .on_drag_completed({
                        let weak = weak.clone();
                        let pending_drag_item_id = pending_drag_item_id.clone();
                        move |event| {
                            if let (Some(state), Some(item_id)) =
                                (weak.upgrade(), pending_drag_item_id.borrow_mut().take())
                            {
                                state.borrow().complete_drag(&item_id, event.effect);
                            }
                        }
                    });

                let grip = ui! {
                    flex_box().width(76.0, Unit::Pixel)
                    .height(40.0, Unit::Pixel)
                    .corner_radius(12.0)
                    .justify_content(JustifyContent::Center)
                    .align_items(AlignItems::Center)
                    .cursor(CursorStyle::Grab)
                    .semantic_role(SemanticRole::Button)
                    .semantic_label("Drag grip")
                    .drag_data({
                        let weak = weak.clone();
                        let pending_drag_item_id = pending_drag_item_id.clone();
                        move || {
                            let state = weak.upgrade()?;
                            let item_id = {
                                let state_ref = state.borrow();
                                let item_id = state_ref
                                    .items
                                    .borrow()
                                    .get(index)
                                    .map(|item| String::from(item.id))?;
                                item_id
                            };
                            pending_drag_item_id.borrow_mut().replace(item_id);
                            let drag_data = state.borrow().begin_drag(index as i32);
                            drag_data
                        }
                    })
                    .drag_allowed_effects(DragDropEffects::Move)
                    .on_drag_completed({
                        let weak = weak.clone();
                        let pending_drag_item_id = pending_drag_item_id.clone();
                        move |event| {
                            if let (Some(state), Some(item_id)) =
                                (weak.upgrade(), pending_drag_item_id.borrow_mut().take())
                            {
                                state.borrow().complete_drag(&item_id, event.effect);
                            }
                        }
                    })
                    .child(&grip_label)
                };

                let title_text = demo_text("", 16.0, 0x111827FF);
                let detail_text = ui! {
                demo_text("", 14.0, 0x475569FF).text_limits(-1, 2)
                };
                let card = ui! {
                    flex_box().fill_width()
                    .height(REORDER_ROW_BODY_HEIGHT_PX, Unit::Pixel)
                    .padding(16.0, 14.0, 16.0, 14.0)
                    .corner_radius(18.0)
                    .child(&ui! {
                        row()
                        .fill_width()
                        .child(&grip)
                        .child(&ui! { flex_box().width(14.0, Unit::Pixel).height(1.0, Unit::Pixel) })
                        .child(&ui! {
                            column()
                                .fill_width()
                                .child(&title_text)
                                .child(&vertical_spacer(4.0))
                                .child(&detail_text)
                        })
                    })
                };
                let slot = ui! {
                    flex_box().fill_width()
                    .flex_direction(FlexDirection::Column)
                    .allow_drop(true)
                    .on_drag_enter({
                        let weak = weak.clone();
                        move |args| {
                            weak.upgrade()
                                .map(|state| state.borrow().preview_insertion(args, index as i32))
                                .unwrap_or_else(DropProposal::none)
                        }
                    })
                    .on_drag_over({
                        let weak = weak.clone();
                        move |args| {
                            weak.upgrade()
                                .map(|state| state.borrow().preview_insertion(args, index as i32))
                                .unwrap_or_else(DropProposal::none)
                        }
                    })
                    .on_drag_leave({
                        let weak = weak.clone();
                        move |args| {
                            if let Some(state) = weak.upgrade() {
                                state.borrow().handle_target_leave(args);
                            }
                        }
                    })
                    .on_drop({
                        let weak = weak.clone();
                        move |args| {
                            if let Some(state) = weak.upgrade() {
                                state.borrow().drop_at_preview(args);
                            }
                        }
                    })
                    .child(&marker)
                    .child(&card)
                };
                let row = ReorderRowView {
                    raw_index: index as i32,
                    marker,
                    grip_label,
                    grip,
                    title_text,
                    detail_text,
                    card,
                    slot,
                };
                row.bind_item(item);
                scroll_content.child(&row.slot);
                rows.push(row);
            }
            state.borrow_mut().rows = rows;
        }

        end_drop_zone
            .allow_drop(true)
            .on_drag_enter({
                let weak = weak.clone();
                move |args| {
                    weak.upgrade()
                        .map(|state| state.borrow().handle_end_drag_over(args))
                        .unwrap_or_else(DropProposal::none)
                }
            })
            .on_drag_over({
                let weak = weak.clone();
                move |args| {
                    weak.upgrade()
                        .map(|state| state.borrow().handle_end_drag_over(args))
                        .unwrap_or_else(DropProposal::none)
                }
            })
            .on_drag_leave({
                let weak = weak.clone();
                move |args| {
                    if let Some(state) = weak.upgrade() {
                        state.borrow().handle_target_leave(args);
                    }
                }
            })
            .on_drop({
                let weak = weak.clone();
                move |args| {
                    if let Some(state) = weak.upgrade() {
                        state.borrow().drop_at_preview(args);
                    }
                }
            })
            .child(&end_marker)
            .child(&ui! {
                flex_box()
                .fill_width()
                .height(36.0, Unit::Pixel)
                .corner_radius(14.0)
                .justify_content(JustifyContent::Center)
                .align_items(AlignItems::Center)
                .child(&demo_text("Drop at end of reorder list", 14.0, 0x475569FF))
            });

        scroll_content.child(&end_drop_zone);
        scroll_box.scroll_content_size(
            -1.0,
            compute_reorder_content_height(state.borrow().items.borrow().len() as i32),
        );

        panel
            .child(&demo_text("Retained reorder drag/drop", 18.0, 0x111827FF))
            .child(&spacer(8.0))
            .child(&demo_text(
                "This ports the FUI-AS internal drag/drop session model: drag grips, insertion markers, preview ghost, edge auto-scroll, and retained list mutation inside an inner ScrollBox.",
                15.0,
                0x334155FF,
            ))
            .child(&spacer(12.0))
            .child(&scroll_box)
            .child(&spacer(12.0))
            .child(&order_status_text)
            .child(&spacer(6.0))
            .child(&drag_status_text)
            .child(&spacer(6.0))
            .child(&viewport_status_text)
            .child(&spacer(10.0))
            .child(&demo_text(
                "Drag a grip with a mouse, or touch and hold before moving. Release to drop. Hold near the viewport edge to auto-scroll the inner ScrollBox while the outer page remains scrollable.",
                15.0,
                0x475569FF,
            ))
            .child(&ui! {
            portal()
                .position_type(PositionType::Absolute)
                .position(0.0, 0.0)
                .fill_size()
                .child(&preview_ghost)
            });

        panel.bind_theme({
            let state = state.clone();
            move |_panel, theme| {
                state.borrow().apply_theme(theme);
            }
        });
        let guards = vec![
            scroll_box.scroll_state().subscribe_offset_y({
                let state = state.clone();
                move || {
                    state.borrow().sync_viewport_status();
                }
            }),
            scroll_box.scroll_state().subscribe_viewport_height({
                let state = state.clone();
                move || {
                    state.borrow().sync_viewport_status();
                }
            }),
        ];

        state.borrow().sync_all();

        Self {
            root: panel,
            _state: state,
            _guards: Rc::new(guards),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        compute_reorder_content_height, compute_reorder_visible_range, create_reorder_demo_items,
        move_reorder_item, normalize_reorder_insertion_index, REORDER_END_SLOT_HEIGHT_PX,
        REORDER_SLOT_HEIGHT_PX,
    };

    #[test]
    fn reorder_content_height_matches_fui_as_formula() {
        assert_eq!(
            compute_reorder_content_height(3),
            (3.0 * REORDER_SLOT_HEIGHT_PX) + REORDER_END_SLOT_HEIGHT_PX
        );
    }

    #[test]
    fn normalize_reorder_insertion_index_matches_fui_as_adjustment() {
        assert_eq!(normalize_reorder_insertion_index(1, 4, 5), 3);
        assert_eq!(normalize_reorder_insertion_index(3, 1, 5), 1);
    }

    #[test]
    fn move_reorder_item_reorders_retained_list() {
        let mut items = create_reorder_demo_items();
        assert!(move_reorder_item(&mut items, "drag-demo", 0));
        assert_eq!(items[0].id, "drag-demo");
    }

    #[test]
    fn visible_range_clamps_to_item_bounds() {
        let range = compute_reorder_visible_range(8, 9999.0, 100.0);
        assert_eq!(range.first_visible_index, 7);
        assert_eq!(range.last_visible_index, 7);
    }
}
