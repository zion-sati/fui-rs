use super::dropdown_option_row_presenter::{
    create_default_dropdown_option_row_presenter, DropdownOptionRowPresenter,
    DropdownOptionRowTemplate, DropdownOptionRowVisualState,
};
use crate::controls::control_template_set::get_control_templates;
use crate::controls::{DropdownColors, DropdownSizing};
use crate::ffi::{CursorStyle, FlexDirection, HandleValue, PositionType, SemanticRole, Unit};
use crate::logger;
use crate::node::{
    flex_box, portal, scroll_box, FlexBox, FlexBoxSurface, Node, NodeHandle, ScrollBarVisibility,
    ScrollBox,
};
use crate::popup_presenter::PopupPresenter;
use crate::theme::current_theme;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

const PANEL_EDGE_PADDING: f32 = 8.0;
const OPTION_HEIGHT: f32 = 34.0;
pub(crate) const SELECTABLE_POPUP_LIST_PANEL_PADDING: f32 = 4.0;
const UNLIMITED_VISIBLE_ITEMS: i32 = 0;

#[derive(Clone)]
pub(crate) struct SelectablePopupListOwner {
    pub(crate) item_count: Rc<dyn Fn() -> i32>,
    pub(crate) item_label: Rc<dyn Fn(i32) -> String>,
    pub(crate) item_selected: Rc<dyn Fn(i32) -> bool>,
    pub(crate) enabled: Rc<dyn Fn() -> bool>,
    pub(crate) highlight_index: Rc<dyn Fn(i32)>,
    pub(crate) activate_index: Rc<dyn Fn(i32)>,
    pub(crate) pointer_down: Rc<dyn Fn(i32)>,
    pub(crate) pointer_up: Rc<dyn Fn(i32)>,
}

fn create_option_row_presenter(
    template: Option<Rc<dyn DropdownOptionRowTemplate>>,
    sizing: Option<DropdownSizing>,
) -> Rc<dyn DropdownOptionRowPresenter> {
    if let Some(template) = template {
        return template.create(sizing);
    }
    let template_set = get_control_templates();
    let app_template = template_set.and_then(|set| set.dropdown_option_row);
    if let Some(app_template) = app_template {
        app_template.create(sizing)
    } else {
        create_default_dropdown_option_row_presenter(sizing)
    }
}

#[derive(Clone)]
pub(crate) struct SelectablePopupListOptionNode {
    root: FlexBox,
    presenter: Rc<RefCell<Rc<dyn DropdownOptionRowPresenter>>>,
    owner: SelectablePopupListOwner,
    slot_index: Rc<Cell<i32>>,
    current_label: Rc<RefCell<String>>,
}

impl SelectablePopupListOptionNode {
    pub(crate) fn new(
        owner: SelectablePopupListOwner,
        slot_index: i32,
        template: Option<Rc<dyn DropdownOptionRowTemplate>>,
        sizing: Option<DropdownSizing>,
    ) -> Self {
        let presenter = create_option_row_presenter(template, sizing);
        let root = flex_box();
        root.semantic_role(SemanticRole::ListItem)
            .width(100.0, Unit::Percent)
            .cursor(CursorStyle::Pointer)
            .focusable(false, 0)
            .interactive(true)
            .child(&presenter.root());
        let control = Self {
            root,
            presenter: Rc::new(RefCell::new(presenter.clone())),
            owner,
            slot_index: Rc::new(Cell::new(slot_index)),
            current_label: Rc::new(RefCell::new(String::new())),
        };
        control.sync_presenter_layout();
        control.bind_events();
        control.bind_presenter_pointer_events(&presenter);
        control
    }

    fn bind_events(&self) {
        let owner = self.owner.clone();
        let slot_index = self.slot_index.clone();
        self.root.on_pointer_enter(move |event| {
            (owner.highlight_index)(slot_index.get());
            event.handled = true;
        });
        let owner = self.owner.clone();
        let slot_index = self.slot_index.clone();
        self.root.on_pointer_down(move |event| {
            event.capture_pointer();
            (owner.pointer_down)(slot_index.get());
            event.handled = true;
        });
        let owner = self.owner.clone();
        let slot_index = self.slot_index.clone();
        self.root.on_pointer_up(move |event| {
            event.release_pointer_capture();
            (owner.pointer_up)(slot_index.get());
            (owner.activate_index)(slot_index.get());
            event.handled = true;
        });
        let owner = self.owner.clone();
        let slot_index = self.slot_index.clone();
        self.root.on_pointer_cancel(move |event| {
            event.release_pointer_capture();
            (owner.pointer_up)(slot_index.get());
            event.handled = true;
        });
    }

    fn bind_presenter_pointer_events(&self, presenter: &Rc<dyn DropdownOptionRowPresenter>) {
        presenter.root().cursor(CursorStyle::Pointer);
        presenter.label_node().cursor(CursorStyle::Pointer);
        self.bind_nested_pointer_events(&presenter.root());
        self.bind_nested_pointer_events(&presenter.label_node());
    }

    fn bind_nested_pointer_events<T>(&self, node: &T)
    where
        T: Node,
    {
        let owner = self.owner.clone();
        let slot_index = self.slot_index.clone();
        node.on_pointer_enter(move |event| {
            (owner.highlight_index)(slot_index.get());
            event.handled = true;
        });
        let owner = self.owner.clone();
        let slot_index = self.slot_index.clone();
        node.on_pointer_down(move |event| {
            event.capture_pointer();
            (owner.pointer_down)(slot_index.get());
            event.handled = true;
        });
        let owner = self.owner.clone();
        let slot_index = self.slot_index.clone();
        node.on_pointer_up(move |event| {
            event.release_pointer_capture();
            (owner.pointer_up)(slot_index.get());
            (owner.activate_index)(slot_index.get());
            event.handled = true;
        });
        let owner = self.owner.clone();
        let slot_index = self.slot_index.clone();
        node.on_pointer_cancel(move |event| {
            event.release_pointer_capture();
            (owner.pointer_up)(slot_index.get());
            event.handled = true;
        });
    }

    pub(crate) fn root(&self) -> FlexBox {
        self.root.clone()
    }

    pub(crate) fn row_height(&self) -> f32 {
        self.presenter.borrow().metrics().height
    }

    pub(crate) fn label(&self, label: impl Into<String>) {
        let label = label.into();
        *self.current_label.borrow_mut() = label.clone();
        self.root.semantic_label(label.clone());
        self.presenter.borrow().label_node().text(label);
    }

    pub(crate) fn template(
        &self,
        template: Option<Rc<dyn DropdownOptionRowTemplate>>,
        sizing: Option<DropdownSizing>,
    ) {
        let previous_presenter = self.presenter.borrow().clone();
        let next_presenter = create_option_row_presenter(template, sizing);
        *self.presenter.borrow_mut() = next_presenter.clone();
        self.root.remove_child(&previous_presenter.root());
        self.root.child(&next_presenter.root());
        previous_presenter.root().dispose();
        next_presenter
            .label_node()
            .text(self.current_label.borrow().clone());
        self.bind_presenter_pointer_events(&next_presenter);
        self.sync_presenter_layout();
    }

    pub(crate) fn apply_theme(
        &self,
        highlighted: bool,
        selected: bool,
        enabled: bool,
        colors: Option<DropdownColors>,
    ) {
        self.root.semantic_selected(selected);
        self.root.semantic_disabled(!enabled);
        self.presenter.borrow().apply(
            current_theme(),
            DropdownOptionRowVisualState::new(highlighted, selected, enabled),
            colors,
        );
    }

    fn sync_presenter_layout(&self) {
        let presenter = self.presenter.borrow();
        self.root.height(presenter.metrics().height, Unit::Pixel);
        presenter.root().fill_size();
    }
}

#[derive(Clone)]
pub(crate) struct SelectablePopupList {
    pub(crate) root: FlexBox,
    pub(crate) panel_node: FlexBox,
    pub(crate) popup_presenter: PopupPresenter,
    pub(crate) popup_scroll_box: ScrollBox,
    pub(crate) options_host: FlexBox,
    owner: SelectablePopupListOwner,
    option_nodes: Rc<RefCell<Vec<SelectablePopupListOptionNode>>>,
    option_row_template_value: Rc<RefCell<Option<Rc<dyn DropdownOptionRowTemplate>>>>,
    sizing_value: Rc<Cell<Option<DropdownSizing>>>,
    colors_value: Rc<Cell<Option<DropdownColors>>>,
    highlighted_index_value: Rc<Cell<i32>>,
    max_visible_items_value: Rc<Cell<i32>>,
    popup_width_value: Rc<Cell<f32>>,
}

impl SelectablePopupList {
    pub(crate) fn new(owner: SelectablePopupListOwner) -> Self {
        let root = portal();
        root.position_type(PositionType::Absolute)
            .position(0.0, 0.0)
            .width(100.0, Unit::Percent)
            .height(100.0, Unit::Percent);
        let popup_scroll_box = scroll_box();
        popup_scroll_box
            .scroll_enabled_x(false)
            .scroll_enabled_y(true)
            .horizontal_scrollbar_visibility(ScrollBarVisibility::Never)
            .vertical_scrollbar_visibility(ScrollBarVisibility::Auto);
        let options_host = flex_box();
        options_host
            .flex_direction(FlexDirection::Column)
            .semantic_role(SemanticRole::List)
            .semantic_label("Dropdown options");
        let panel_node = flex_box();
        panel_node
            .position_type(PositionType::Absolute)
            .flex_direction(FlexDirection::Column);
        let popup_presenter =
            PopupPresenter::new_with_semantic_scope(root.clone(), panel_node.clone(), None);
        popup_scroll_box.child(&options_host);
        panel_node.child(&popup_scroll_box);
        Self {
            root,
            panel_node,
            popup_presenter,
            popup_scroll_box,
            options_host,
            owner,
            option_nodes: Rc::new(RefCell::new(Vec::new())),
            option_row_template_value: Rc::new(RefCell::new(None)),
            sizing_value: Rc::new(Cell::new(None)),
            colors_value: Rc::new(Cell::new(None)),
            highlighted_index_value: Rc::new(Cell::new(-1)),
            max_visible_items_value: Rc::new(Cell::new(UNLIMITED_VISIBLE_ITEMS)),
            popup_width_value: Rc::new(Cell::new(0.0)),
        }
    }

    pub(crate) fn is_open(&self) -> bool {
        self.popup_presenter.is_open()
    }

    pub(crate) fn highlighted_index(&self) -> i32 {
        self.highlighted_index_value.get()
    }

    pub(crate) fn max_visible_items(&self, count: i32) {
        if count <= 0 {
            logger::warn(
                "Layout",
                &format!(
                    "Dropdown.maxVisibleItems() received {count}; using unlimited visible items."
                ),
            );
        }
        self.max_visible_items_value.set(if count > 0 {
            count
        } else {
            UNLIMITED_VISIBLE_ITEMS
        });
        self.refresh_panel_layout();
    }

    pub(crate) fn popup_width(&self, value: f32) {
        if value <= 0.0 {
            logger::warn(
                "Layout",
                &format!("Dropdown.popupWidth() received {value}; clamping to 0.0."),
            );
        }
        self.popup_width_value.set(value.max(0.0));
        self.refresh_panel_layout();
    }

    pub(crate) fn sizing(&self, sizing: Option<DropdownSizing>) {
        self.sizing_value.set(sizing);
        let template = self.option_row_template_value.borrow().clone();
        let nodes = self.option_nodes.borrow();
        for node in nodes.iter() {
            node.template(template.clone(), self.sizing_value.get());
        }
        drop(nodes);
        self.refresh_panel_layout();
    }

    pub(crate) fn colors(&self, colors: Option<DropdownColors>) {
        self.colors_value.set(colors);
        self.sync_option_visuals();
    }

    pub(crate) fn option_row_template(&self, template: Option<Rc<dyn DropdownOptionRowTemplate>>) {
        *self.option_row_template_value.borrow_mut() = template.clone();
        let nodes = self.option_nodes.borrow();
        for node in nodes.iter() {
            node.template(template.clone(), self.sizing_value.get());
        }
        drop(nodes);
        self.refresh_panel_layout();
        self.sync_option_visuals();
    }

    pub(crate) fn open(
        &self,
        trigger_x: f32,
        trigger_y: f32,
        trigger_width: f32,
        trigger_height: f32,
        initial_highlight_index: i32,
    ) -> bool {
        if self.is_open()
            || (self.owner.item_count)() == 0
            || self.root.handle() == NodeHandle::INVALID
            || self.root.handle().raw() == HandleValue::Invalid as u64
        {
            return false;
        }
        self.ensure_option_nodes();
        self.rebuild_panel();
        self.set_highlighted_index(initial_highlight_index);
        self.position_panel(trigger_x, trigger_y, trigger_width, trigger_height);
        true
    }

    pub(crate) fn refresh_open(
        &self,
        trigger_x: f32,
        trigger_y: f32,
        trigger_width: f32,
        trigger_height: f32,
        highlighted_index: i32,
    ) {
        if !self.is_open() {
            return;
        }
        self.ensure_option_nodes();
        self.rebuild_panel();
        let count = (self.owner.item_count)();
        let mut next_highlight = highlighted_index;
        if next_highlight >= count {
            next_highlight = if count > 0 { count - 1 } else { -1 };
        }
        if next_highlight < 0 && count > 0 {
            next_highlight = 0;
        }
        self.set_highlighted_index(next_highlight);
        self.position_panel(trigger_x, trigger_y, trigger_width, trigger_height);
    }

    pub(crate) fn close(&self) {
        self.popup_presenter.hide();
    }

    pub(crate) fn dispose(&self) {
        self.popup_presenter.dispose();
    }

    pub(crate) fn clear(&self) {
        self.close();
        self.highlighted_index_value.set(-1);
    }

    pub(crate) fn set_highlighted_index(&self, index: i32) {
        self.highlighted_index_value.set(index);
        self.sync_option_visuals();
        self.ensure_highlighted_visible();
    }

    pub(crate) fn highlight_index(&self, index: i32) {
        let count = (self.owner.item_count)();
        if index < 0 || index >= count || self.highlighted_index_value.get() == index {
            if index < 0 || index >= count {
                logger::warn(
                    "Layout",
                    &format!(
                        "Dropdown.highlightIndex() received {index} outside the available item range."
                    ),
                );
            }
            return;
        }
        self.highlighted_index_value.set(index);
        self.sync_option_visuals();
        self.ensure_highlighted_visible();
    }

    pub(crate) fn move_highlight(&self, delta: i32) {
        let count = (self.owner.item_count)();
        if count == 0 {
            return;
        }
        let mut next_index = self.highlighted_index_value.get();
        if next_index < 0 {
            next_index = 0;
        }
        next_index += delta;
        if next_index < 0 {
            next_index = count - 1;
        } else if next_index >= count {
            next_index = 0;
        }
        self.highlight_index(next_index);
    }

    pub(crate) fn refresh_panel_layout(&self) {
        let count = (self.owner.item_count)();
        self.options_host
            .width(100.0, Unit::Percent)
            .height(count as f32 * self.resolve_option_row_height(), Unit::Pixel);
        self.popup_scroll_box.width(100.0, Unit::Percent).height(
            (self.resolve_viewport_clamped_panel_outer_height()
                - (SELECTABLE_POPUP_LIST_PANEL_PADDING * 2.0))
                .max(0.0),
            Unit::Pixel,
        );
        if self.is_open() {
            self.ensure_highlighted_visible();
        }
    }

    pub(crate) fn position_panel(
        &self,
        trigger_x: f32,
        trigger_y: f32,
        trigger_width: f32,
        trigger_height: f32,
    ) {
        let popup_width = self.resolve_popup_width(trigger_width);
        let panel_height = self.resolve_viewport_clamped_panel_outer_height();
        self.panel_node
            .width(popup_width, Unit::Pixel)
            .height(panel_height, Unit::Pixel);
        self.popup_scroll_box.width(100.0, Unit::Percent).height(
            (panel_height - (SELECTABLE_POPUP_LIST_PANEL_PADDING * 2.0)).max(0.0),
            Unit::Pixel,
        );
        self.popup_presenter.show_anchored(
            trigger_x,
            trigger_y,
            trigger_width,
            trigger_height,
            popup_width,
            panel_height,
        );
    }

    pub(crate) fn sync_option_visuals(&self) {
        self.ensure_option_nodes();
        let count = (self.owner.item_count)();
        let nodes = self.option_nodes.borrow();
        for (index, node) in nodes.iter().take(count as usize).enumerate() {
            let index = index as i32;
            node.apply_theme(
                index == self.highlighted_index_value.get(),
                (self.owner.item_selected)(index),
                (self.owner.enabled)(),
                self.colors_value.get(),
            );
        }
    }

    fn rebuild_panel(&self) {
        {
            let nodes = self.option_nodes.borrow();
            for node in nodes.iter() {
                self.options_host.remove_child(&node.root());
            }
        }
        let count = (self.owner.item_count)();
        let nodes = self.option_nodes.borrow();
        for index in 0..count {
            let node = &nodes[index as usize];
            node.label((self.owner.item_label)(index));
            self.options_host.child(&node.root());
        }
        drop(nodes);
        self.refresh_panel_layout();
    }

    fn ensure_option_nodes(&self) {
        let count = (self.owner.item_count)();
        while self.option_nodes.borrow().len() < count as usize {
            let slot_index = self.option_nodes.borrow().len() as i32;
            let node = SelectablePopupListOptionNode::new(
                self.owner.clone(),
                slot_index,
                self.option_row_template_value.borrow().clone(),
                self.sizing_value.get(),
            );
            self.option_nodes.borrow_mut().push(node);
        }
    }

    fn resolve_option_row_height(&self) -> f32 {
        let nodes = self.option_nodes.borrow();
        if nodes.is_empty() {
            if let Some(sizing) = self.sizing_value.get() {
                if sizing.has_option_height() && self.option_row_template_value.borrow().is_none() {
                    return sizing.option_height_px();
                }
            }
            return OPTION_HEIGHT;
        }
        nodes[0].row_height()
    }

    fn resolve_visible_item_count(&self) -> i32 {
        let count = (self.owner.item_count)();
        let max_visible = self.max_visible_items_value.get();
        if max_visible <= 0 || count <= max_visible {
            count
        } else {
            max_visible
        }
    }

    fn resolve_panel_outer_height(&self) -> f32 {
        self.resolve_visible_item_count() as f32 * self.resolve_option_row_height()
            + (SELECTABLE_POPUP_LIST_PANEL_PADDING * 2.0)
    }

    fn resolve_viewport_clamped_panel_outer_height(&self) -> f32 {
        let max_height = (crate::bindings::ui::get_viewport_height() - (PANEL_EDGE_PADDING * 2.0))
            .max(PANEL_EDGE_PADDING);
        self.resolve_panel_outer_height().min(max_height)
    }

    fn resolve_popup_width(&self, trigger_width: f32) -> f32 {
        let width = self.popup_width_value.get();
        if width > 0.0 {
            width
        } else {
            trigger_width
        }
    }

    fn ensure_highlighted_visible(&self) {
        let highlighted_index = self.highlighted_index_value.get();
        if !self.is_open() || highlighted_index < 0 {
            return;
        }
        let visible_height = (self.resolve_viewport_clamped_panel_outer_height()
            - (SELECTABLE_POPUP_LIST_PANEL_PADDING * 2.0))
            .max(0.0);
        if visible_height <= 0.0 {
            return;
        }
        let row_height = self.resolve_option_row_height();
        let item_top = highlighted_index as f32 * row_height;
        let item_bottom = item_top + row_height;
        let state = self.popup_scroll_box.scroll_state();
        let mut next_offset = state.offset_y();
        if item_top < next_offset {
            next_offset = item_top;
        } else if item_bottom > next_offset + visible_height {
            next_offset = item_bottom - visible_height;
        }
        self.popup_scroll_box
            .set_runtime_scroll_offset(0.0, next_offset);
    }
}
