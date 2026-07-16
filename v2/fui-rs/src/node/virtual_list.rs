use super::*;
use crate::bindings::ui;
use crate::controls::selection_area;
use crate::controls::SelectionArea;
use crate::frame_scheduler::mark_needs_commit;
use crate::signal::Subscription;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

const FULL_SIZE: f32 = 100.0;
const DEFAULT_MAX_VISIBLE_ITEMS: i32 = 20;
const POOL_OVERSCAN_ITEMS: i32 = 2;
const MISSING_BIND_ITEM_MESSAGE: &str =
    "VirtualList: item renderer not configured. Call .onBindItem() after construction.";

type VirtualListBinder = Rc<dyn Fn(&FlexBox, i32)>;

struct VirtualListInner {
    root: FlexBox,
    total_items: Cell<i32>,
    item_height: f32,
    bind_item: RefCell<VirtualListBinder>,
    scroll_state: ScrollState,
    scroll_box: ScrollBox,
    top_spacer: FlexBox,
    bottom_spacer: FlexBox,
    pool_size: i32,
    pool_rows: Vec<SelectionArea>,
    pool_containers: Vec<FlexBox>,
    pool_item_index_by_row: RefCell<Vec<i32>>,
    subscriptions: RefCell<Vec<Subscription>>,
    current_first_visible_index: Cell<i32>,
    current_last_visible_index: Cell<i32>,
}

#[derive(Clone)]
pub struct VirtualList {
    inner: Rc<VirtualListInner>,
}

impl VirtualList {
    pub fn new(total_items: i32, item_height: f32) -> Self {
        Self::with_max_visible(total_items, item_height, DEFAULT_MAX_VISIBLE_ITEMS)
    }

    pub fn with_max_visible(total_items: i32, item_height: f32, max_visible: i32) -> Self {
        let total_items = total_items.max(0);
        let item_height = if item_height > 0.0 { item_height } else { 1.0 };
        let pool_size = if max_visible > 0 {
            max_visible + POOL_OVERSCAN_ITEMS
        } else {
            POOL_OVERSCAN_ITEMS
        };
        let scroll_state = ScrollState::new();
        let top_spacer = flex_box();
        top_spacer
            .width(FULL_SIZE, Unit::Percent)
            .height(0.0, Unit::Pixel);
        let bottom_spacer = flex_box();
        bottom_spacer
            .width(FULL_SIZE, Unit::Percent)
            .height(0.0, Unit::Pixel);
        let content = column();
        content.width(FULL_SIZE, Unit::Percent).child(&top_spacer);

        let mut pool_rows = Vec::new();
        let mut pool_containers = Vec::new();
        let mut pool_item_index_by_row = Vec::new();
        for _ in 0..pool_size {
            let container = flex_box();
            container.fill_size();
            let row_area = selection_area();
            row_area
                .width(FULL_SIZE, Unit::Percent)
                .height(0.0, Unit::Pixel)
                .child(&container);
            content.child(&row_area);
            pool_rows.push(row_area);
            pool_containers.push(container);
            pool_item_index_by_row.push(-1);
        }
        content.child(&bottom_spacer);

        let scroll_box = ScrollBox::with_parts(scroll_state.clone(), ScrollView::new());
        scroll_box
            .scroll_enabled_x(false)
            .scroll_enabled_y(true)
            .scroll_offset(scroll_state.offset_x(), scroll_state.offset_y())
            .scroll_content_size(-1.0, total_items as f32 * item_height)
            .fill_size()
            .child(&content);
        scroll_state.set_content_height(total_items as f32 * item_height);

        let root = column();
        root.fill_size().child(&scroll_box);

        let inner = Rc::new(VirtualListInner {
            root,
            total_items: Cell::new(total_items),
            item_height,
            bind_item: RefCell::new(Rc::new(|_, _| panic!("{MISSING_BIND_ITEM_MESSAGE}"))),
            scroll_state,
            scroll_box,
            top_spacer,
            bottom_spacer,
            pool_size,
            pool_rows,
            pool_containers,
            pool_item_index_by_row: RefCell::new(pool_item_index_by_row),
            subscriptions: RefCell::new(Vec::new()),
            current_first_visible_index: Cell::new(-1),
            current_last_visible_index: Cell::new(-1),
        });
        let list = Self { inner };
        list.attach_listeners();
        list
    }

    pub fn scroll_state(&self) -> ScrollState {
        self.inner.scroll_state.clone()
    }

    pub fn total_items(&self) -> i32 {
        self.inner.total_items.get()
    }

    pub fn scroll_box(&self) -> ScrollBox {
        self.inner.scroll_box.clone()
    }

    pub fn item_height(&self) -> f32 {
        self.inner.item_height
    }

    pub fn total_content_height(&self) -> f32 {
        self.total_items() as f32 * self.item_height()
    }

    pub fn first_visible_index(&self) -> i32 {
        let current = self.inner.current_first_visible_index.get();
        if current >= 0 {
            current
        } else {
            0
        }
    }

    pub fn rendered_item_count(&self) -> i32 {
        let first = self.inner.current_first_visible_index.get();
        let last = self.inner.current_last_visible_index.get();
        if first < 0 || last < first {
            0
        } else {
            last - first
        }
    }

    pub fn node_id(&self, id: impl Into<String>) -> &Self {
        self.inner.scroll_box.node_id(id);
        self
    }

    pub fn persist_scroll(&self, flag: bool) -> &Self {
        self.inner.scroll_box.persist_scroll(flag);
        self
    }

    pub fn width(&self, value: f32, unit: Unit) -> &Self {
        self.inner.root.width(value, unit);
        if unit == Unit::Pixel {
            self.inner.scroll_state.set_viewport_width(value);
        }
        self
    }

    pub fn height(&self, value: f32, unit: Unit) -> &Self {
        self.inner.root.height(value, unit);
        if unit == Unit::Pixel {
            self.inner.scroll_state.set_viewport_height(value);
        }
        self
    }

    pub fn on_bind_item(&self, renderer: impl Fn(&FlexBox, i32) + 'static) -> &Self {
        *self.inner.bind_item.borrow_mut() = Rc::new(renderer);
        self.inner.current_first_visible_index.set(-1);
        self.inner.current_last_visible_index.set(-1);
        self.rebuild_visible_range(true);
        self
    }

    #[allow(non_snake_case)]
    pub fn onBindItem(&self, renderer: impl Fn(&FlexBox, i32) + 'static) -> &Self {
        self.on_bind_item(renderer)
    }

    pub fn update_item_count(&self, next: i32) {
        self.inner.total_items.set(next.max(0));
        self.inner
            .scroll_state
            .set_content_height(self.total_content_height());
        self.inner
            .scroll_box
            .scroll_content_size(-1.0, self.total_content_height());
        self.inner.current_first_visible_index.set(-1);
        self.inner.current_last_visible_index.set(-1);

        let clamped_offset = self.max_offset_for_current_viewport();
        if self.inner.scroll_state.offset_y() > clamped_offset {
            self.inner
                .scroll_box
                .scroll_offset(self.inner.scroll_state.offset_x(), clamped_offset);
            return;
        }
        self.rebuild_visible_range(true);
    }

    fn attach_listeners(&self) {
        let weak = Rc::downgrade(&self.inner);
        self.inner
            .subscriptions
            .borrow_mut()
            .push(self.inner.scroll_state.subscribe_offset_y(move || {
                if let Some(inner) = weak.upgrade() {
                    VirtualList { inner }.handle_scroll_offset_changed();
                }
            }));
        let weak = Rc::downgrade(&self.inner);
        self.inner.subscriptions.borrow_mut().push(
            self.inner.scroll_state.subscribe_viewport_height(move || {
                if let Some(inner) = weak.upgrade() {
                    VirtualList { inner }.handle_metrics_changed();
                }
            }),
        );
        let weak = Rc::downgrade(&self.inner);
        self.inner.subscriptions.borrow_mut().push(
            self.inner.scroll_state.subscribe_content_height(move || {
                if let Some(inner) = weak.upgrade() {
                    VirtualList { inner }.handle_metrics_changed();
                }
            }),
        );
    }

    fn handle_metrics_changed(&self) {
        self.rebuild_visible_range(true);
    }

    fn handle_scroll_offset_changed(&self) {
        self.rebuild_visible_range(true);
    }

    fn rebuild_visible_range(&self, commit: bool) {
        let total_items = self.total_items();
        if total_items <= 0 {
            if self.inner.current_first_visible_index.get() == 0
                && self.inner.current_last_visible_index.get() == 0
            {
                return;
            }
            self.inner.current_first_visible_index.set(0);
            self.inner.current_last_visible_index.set(0);
            self.inner.top_spacer.height(0.0, Unit::Pixel);
            self.inner.bottom_spacer.height(0.0, Unit::Pixel);
            for pool_index in 0..self.inner.pool_size as usize {
                self.hide_pool_item(pool_index);
            }
            self.commit_if_built(commit);
            return;
        }

        let mut first_visible_index =
            (self.inner.scroll_state.offset_y() / self.inner.item_height).floor() as i32;
        if first_visible_index < 0 {
            first_visible_index = 0;
        }
        if first_visible_index > total_items {
            first_visible_index = total_items;
        }

        let viewport_height = if self.inner.scroll_state.viewport_height() > 0.0 {
            self.inner.scroll_state.viewport_height()
        } else {
            self.inner.item_height
        };
        let mut visible_count = (viewport_height / self.inner.item_height).ceil() as i32 + 1;
        if visible_count < 1 {
            visible_count = 1;
        }
        if visible_count > self.inner.pool_size {
            visible_count = self.inner.pool_size;
        }

        let mut last_visible_index = first_visible_index + visible_count;
        if last_visible_index > total_items {
            last_visible_index = total_items;
        }

        if first_visible_index == self.inner.current_first_visible_index.get()
            && last_visible_index == self.inner.current_last_visible_index.get()
        {
            return;
        }

        self.inner
            .current_first_visible_index
            .set(first_visible_index);
        self.inner
            .current_last_visible_index
            .set(last_visible_index);

        let top_spacer_height = first_visible_index as f32 * self.inner.item_height;
        let mut bottom_spacer_height =
            self.total_content_height() - (last_visible_index as f32 * self.inner.item_height);
        if bottom_spacer_height < 0.0 {
            bottom_spacer_height = 0.0;
        }

        self.inner.top_spacer.height(top_spacer_height, Unit::Pixel);
        self.inner
            .bottom_spacer
            .height(bottom_spacer_height, Unit::Pixel);

        let previous_item_index_by_row = self.inner.pool_item_index_by_row.borrow().clone();
        let visible_items = (last_visible_index - first_visible_index) as usize;

        for pool_index in 0..self.inner.pool_size as usize {
            let previous_item_index = previous_item_index_by_row[pool_index];
            if previous_item_index != -1
                && (previous_item_index < first_visible_index
                    || previous_item_index >= last_visible_index)
            {
                self.clear_row_selection(pool_index);
            }
        }

        for pool_index in 0..self.inner.pool_size as usize {
            let row_area = &self.inner.pool_rows[pool_index];
            if pool_index < visible_items {
                let next_item_index = first_visible_index + pool_index as i32;
                row_area.height(self.inner.item_height, Unit::Pixel);
                let container = &self.inner.pool_containers[pool_index];
                self.render_item(container, next_item_index);
                self.inner.pool_item_index_by_row.borrow_mut()[pool_index] = next_item_index;
            } else {
                self.hide_pool_item(pool_index);
            }
        }

        for pool_index in 0..visible_items {
            let next_item_index = first_visible_index + pool_index as i32;
            let previous_pool_index =
                Self::find_pool_index_for_item(&previous_item_index_by_row, next_item_index);
            if previous_pool_index != -1 && previous_pool_index as usize != pool_index {
                self.retarget_row_selection(previous_pool_index as usize, pool_index);
            }
        }

        self.commit_if_built(commit);
    }

    fn hide_pool_item(&self, pool_index: usize) {
        if self.inner.pool_item_index_by_row.borrow()[pool_index] != -1 {
            self.clear_row_selection(pool_index);
            self.inner.pool_item_index_by_row.borrow_mut()[pool_index] = -1;
        }
        self.inner.pool_rows[pool_index].height(0.0, Unit::Pixel);
        self.clear_item_node(&self.inner.pool_containers[pool_index].retained_node_ref());
    }

    fn clear_row_selection(&self, pool_index: usize) {
        if !self.has_built_handle() {
            return;
        }
        self.clear_selection_node(&self.inner.pool_rows[pool_index].retained_node_ref());
    }

    fn retarget_row_selection(&self, from_pool_index: usize, to_pool_index: usize) {
        if !self.has_built_handle() {
            return;
        }
        let mut from_texts = Vec::new();
        let mut to_texts = Vec::new();
        self.collect_text_nodes(
            &self.inner.pool_rows[from_pool_index].retained_node_ref(),
            &mut from_texts,
        );
        self.collect_text_nodes(
            &self.inner.pool_rows[to_pool_index].retained_node_ref(),
            &mut to_texts,
        );
        let pair_count = from_texts.len().min(to_texts.len());
        for index in 0..pair_count {
            let from_text = from_texts[index];
            let to_text = to_texts[index];
            if from_text != NodeHandle::INVALID
                && to_text != NodeHandle::INVALID
                && from_text != to_text
            {
                ui::retarget_selection(from_text.raw(), to_text.raw());
            }
        }
    }

    fn collect_text_nodes(&self, node: &NodeRef, out: &mut Vec<NodeHandle>) {
        if node.text_content_for_routing().is_some() {
            out.push(node.handle());
            return;
        }
        for child in node.children() {
            self.collect_text_nodes(&child, out);
        }
    }

    fn clear_selection_node(&self, node: &NodeRef) {
        if node.text_content_for_routing().is_some() {
            let handle = node.handle();
            if handle != NodeHandle::INVALID {
                ui::clear_selection(handle.raw());
            }
            return;
        }
        for child in node.children() {
            self.clear_selection_node(&child);
        }
    }

    fn clear_item_node(&self, node: &NodeRef) {
        node.set_semantic_label_for_routing(Some(String::new()));
        let handle = node.handle();
        if handle != NodeHandle::INVALID {
            ui::set_semantic_label(handle.raw(), "");
        }
        if node.text_content_for_routing().is_some() {
            node.set_text_content_for_routing(Some(String::new()));
            if handle != NodeHandle::INVALID {
                ui::set_text(handle.raw(), "");
            }
        }
        for child in node.children() {
            self.clear_item_node(&child);
        }
    }

    fn commit_if_built(&self, commit: bool) {
        if !commit || !self.has_built_handle() {
            return;
        }
        mark_needs_commit();
    }

    fn render_item(&self, container: &FlexBox, index: i32) {
        let binder = self.inner.bind_item.borrow().clone();
        binder(container, index);
    }

    fn max_offset_for_current_viewport(&self) -> f32 {
        let viewport_height = if self.inner.scroll_state.viewport_height() > 0.0 {
            self.inner.scroll_state.viewport_height()
        } else {
            0.0
        };
        let max_offset = self.total_content_height() - viewport_height;
        if max_offset > 0.0 {
            max_offset
        } else {
            0.0
        }
    }

    fn find_pool_index_for_item(previous_item_index_by_row: &[i32], item_index: i32) -> i32 {
        previous_item_index_by_row
            .iter()
            .position(|candidate| *candidate == item_index)
            .map(|index| index as i32)
            .unwrap_or(-1)
    }
}

impl Node for VirtualList {
    fn retained_node_ref(&self) -> NodeRef {
        let list = self.clone();
        self.inner
            .root
            .retained_node_ref()
            .with_build_callback(move || list.build())
    }

    fn build(&self) {
        self.ensure_handle();
        self.inner.root.build_self();
        self.build_children();
        if self.inner.current_first_visible_index.get() < 0 && self.total_items() > 0 {
            self.rebuild_visible_range(false);
        }
        ui::set_selection_area_barrier(self.handle().raw(), true);
    }

    fn build_self(&self) {
        self.inner.root.build_self();
    }

    fn dispose(&self) {
        self.inner.subscriptions.borrow_mut().clear();
        self.inner.root.dispose();
    }
}

impl HasFlexBoxRoot for VirtualList {
    fn flex_box_root(&self) -> &FlexBox {
        &self.inner.root
    }
}

impl ThemeBindable for VirtualList {
    fn theme_binding_node(&self) -> NodeRef {
        self.inner.root.retained_node_ref()
    }

    fn weak_theme_target(&self) -> Box<dyn Fn() -> Option<Self>> {
        let weak = Rc::downgrade(&self.inner);
        Box::new(move || {
            Some(VirtualList {
                inner: weak.upgrade()?,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::{self, Call};
    use crate::prelude::*;
    use std::collections::HashMap;

    fn tracked_bind_virtual_list_item(
        rendered_indices: &Rc<RefCell<Vec<i32>>>,
        labels: &Rc<RefCell<HashMap<usize, TextNode>>>,
        container: &FlexBox,
        index: i32,
    ) {
        rendered_indices.borrow_mut().push(index);
        let key = std::ptr::from_ref(container) as usize;
        let existing = { labels.borrow().get(&key).cloned() };
        let label = if let Some(existing) = existing {
            existing
        } else {
            let label = text("");
            container.child(&label);
            labels.borrow_mut().insert(key, label.clone());
            label
        };
        let text_value = format!("Item {}", index);
        label.text(&text_value);
        label.semantic_label(&text_value);
    }

    fn static_bind_virtual_list_item(
        labels: &Rc<RefCell<HashMap<usize, TextNode>>>,
        container: &FlexBox,
        index: i32,
    ) {
        let key = std::ptr::from_ref(container) as usize;
        let existing = { labels.borrow().get(&key).cloned() };
        let label = if let Some(existing) = existing {
            existing
        } else {
            let label = text("");
            container.child(&label);
            labels.borrow_mut().insert(key, label.clone());
            label
        };
        label.text(format!("row {}", index));
    }

    #[test]
    fn binds_only_the_visible_window_for_a_fixed_height_list() {
        ffi::test::reset();
        let rendered_indices = Rc::new(RefCell::new(Vec::new()));
        let labels = Rc::new(RefCell::new(HashMap::new()));
        let list = VirtualList::new(10_000, 20.0);
        let captured_indices = rendered_indices.clone();
        let captured_labels = labels.clone();
        list.onBindItem(move |container, index| {
            tracked_bind_virtual_list_item(&captured_indices, &captured_labels, container, index);
        });
        list.width(180.0, Unit::Pixel);
        list.height(100.0, Unit::Pixel);

        list.build();

        let rendered = rendered_indices.borrow();
        let last_window_start = rendered.len() - 6;
        assert_eq!(rendered[last_window_start], 0);
        assert_eq!(rendered[last_window_start + 5], 5);
        assert_eq!(list.first_visible_index(), 0);
        assert_eq!(list.rendered_item_count(), 6);
        list.dispose();
    }

    #[test]
    fn rebinds_pooled_rows_without_recreating_nodes_when_scroll_offset_changes() {
        ffi::test::reset();
        let rendered_indices = Rc::new(RefCell::new(Vec::new()));
        let labels = Rc::new(RefCell::new(HashMap::new()));
        let list = VirtualList::new(10_000, 20.0);
        let captured_indices = rendered_indices.clone();
        let captured_labels = labels.clone();
        list.onBindItem(move |container, index| {
            tracked_bind_virtual_list_item(&captured_indices, &captured_labels, container, index);
        });
        list.width(180.0, Unit::Pixel);
        list.height(100.0, Unit::Pixel);
        Application::mount(list.clone());

        rendered_indices.borrow_mut().clear();
        ffi::test::reset();
        list.scroll_state().set_offset_y(60.0);

        let rendered = rendered_indices.borrow();
        assert_eq!(rendered.len(), 6);
        assert_eq!(rendered[0], 3);
        assert_eq!(rendered[5], 8);
        assert_eq!(list.first_visible_index(), 3);
        let calls = ffi::test::take_calls();
        assert!(!calls
            .iter()
            .any(|call| matches!(call, Call::CreateNode { .. })));
        assert!(!calls
            .iter()
            .any(|call| matches!(call, Call::DeleteNode { .. })));
        assert!(calls
            .iter()
            .any(|call| matches!(call, Call::SetText { text, .. } if text == "Item 3")));
        assert!(calls.iter().any(|call| {
            matches!(
                call,
                Call::SetHeight {
                    value,
                    unit_enum,
                    ..
                } if *value == 20.0 && *unit_enum == Unit::Pixel as u32
            )
        }));
        Application::unmount();
    }

    #[test]
    fn tracks_content_height_and_clamps_the_visible_window_when_item_count_changes() {
        ffi::test::reset();
        let labels = Rc::new(RefCell::new(HashMap::new()));
        let list = VirtualList::new(10_000, 24.0);
        let captured_labels = labels.clone();
        list.onBindItem(move |container, index| {
            static_bind_virtual_list_item(&captured_labels, container, index);
        });
        list.height(120.0, Unit::Pixel);

        assert_eq!(list.scroll_state().content_height(), 240_000.0);
        assert_eq!(list.rendered_item_count(), 6);

        list.update_item_count(5);

        assert_eq!(list.scroll_state().content_height(), 120.0);
        assert_eq!(list.rendered_item_count(), 5);
        list.dispose();
    }

    #[test]
    fn supports_owner_captured_renderers_with_on_bind_item() {
        ffi::test::reset();
        let rendered_indices = Rc::new(RefCell::new(Vec::new()));
        let labels = Rc::new(RefCell::new(HashMap::new()));
        let list = VirtualList::new(10_000, 20.0);
        let captured_indices = rendered_indices.clone();
        let captured_labels = labels.clone();
        list.onBindItem(move |container, index| {
            tracked_bind_virtual_list_item(&captured_indices, &captured_labels, container, index);
        });
        list.width(180.0, Unit::Pixel);
        list.height(100.0, Unit::Pixel);

        list.build();

        let rendered = rendered_indices.borrow();
        assert!(rendered.len() >= 6);
        assert_eq!(rendered[0], 0);
        assert_eq!(rendered[rendered.len() - 1], 5);
        list.dispose();
    }
}
