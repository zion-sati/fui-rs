use super::*;
use crate::ffi::SemanticRole;
use crate::node::{column, NodeRef};
use crate::retained_view::{retained_view, RetainedView};
use std::any::Any;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

pub type TabContentFactory = Box<dyn FnOnce() -> RetainedView>;
type TabSelectionChangedCallback = Rc<dyn Fn(TabSelectionChangedEventArgs)>;

#[derive(Clone)]
pub struct TabSelectionChangedEventArgs {
    pub previous_index: i32,
    pub selected_index: i32,
    pub previous_item: Option<TabItem>,
    pub selected_item: Option<TabItem>,
}

struct TabItemInner {
    label: RefCell<String>,
    description: RefCell<String>,
    enabled: Cell<bool>,
    factory: RefCell<Option<TabContentFactory>>,
    prepared_view: RefCell<Option<RetainedView>>,
    retained_view: RefCell<Option<RetainedView>>,
    owner: RefCell<Weak<TabViewInner>>,
}

#[derive(Clone)]
pub struct TabItem {
    inner: Rc<TabItemInner>,
}

impl From<&TabItem> for TabItem {
    fn from(item: &TabItem) -> Self {
        item.clone()
    }
}

impl TabItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            inner: Rc::new(TabItemInner {
                label: RefCell::new(label.into()),
                description: RefCell::new(String::new()),
                enabled: Cell::new(true),
                factory: RefCell::new(None),
                prepared_view: RefCell::new(None),
                retained_view: RefCell::new(None),
                owner: RefCell::new(Weak::new()),
            }),
        }
    }

    pub fn with_content(
        label: impl Into<String>,
        factory: impl FnOnce() -> RetainedView + 'static,
    ) -> Self {
        let item = Self::new(label);
        item.content(factory);
        item
    }

    pub fn with_content_view(label: impl Into<String>, view: RetainedView) -> Self {
        let item = Self::new(label);
        item.content_view(view);
        item
    }

    pub fn label_text(&self) -> String {
        self.inner.label.borrow().clone()
    }

    pub fn description_text(&self) -> String {
        self.inner.description.borrow().clone()
    }

    pub fn is_enabled(&self) -> bool {
        self.inner.enabled.get()
    }

    pub fn is_materialized(&self) -> bool {
        self.inner.retained_view.borrow().is_some()
    }

    pub fn label(&self, value: impl Into<String>) -> &Self {
        *self.inner.label.borrow_mut() = value.into();
        self.notify_metadata_changed();
        self
    }

    pub fn description(&self, value: impl Into<String>) -> &Self {
        *self.inner.description.borrow_mut() = value.into();
        self.notify_metadata_changed();
        self
    }

    pub fn enabled(&self, enabled: bool) -> &Self {
        if self.inner.enabled.replace(enabled) != enabled {
            if let Some(owner) = self.inner.owner.borrow().upgrade() {
                TabView { inner: owner }.item_enabled_changed(self);
            }
        }
        self
    }

    pub fn content(&self, factory: impl FnOnce() -> RetainedView + 'static) -> &Self {
        if self.is_materialized() {
            crate::logger::warn(
                "Lifecycle",
                "TabItem.content() cannot replace content after the tab has been materialized.",
            );
            return self;
        }
        self.inner.prepared_view.borrow_mut().take();
        *self.inner.factory.borrow_mut() = Some(Box::new(factory));
        self
    }

    pub fn content_view(&self, view: RetainedView) -> &Self {
        if self.is_materialized() {
            crate::logger::warn(
                "Lifecycle",
                "TabItem.content_view() cannot replace content after the tab has been materialized.",
            );
            return self;
        }
        *self.inner.prepared_view.borrow_mut() = Some(view);
        self.inner.factory.borrow_mut().take();
        self
    }

    fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }

    fn bind_owner(&self, owner: Weak<TabViewInner>) {
        *self.inner.owner.borrow_mut() = owner;
    }

    fn notify_metadata_changed(&self) {
        if let Some(owner) = self.inner.owner.borrow().upgrade() {
            TabView { inner: owner }.item_metadata_changed(self);
        }
    }

    fn resolve_view(&self) -> RetainedView {
        if let Some(view) = self.inner.retained_view.borrow().clone() {
            return view;
        }
        let resolved = if let Some(view) = self.inner.prepared_view.borrow_mut().take() {
            view
        } else if let Some(factory) = self.inner.factory.borrow_mut().take() {
            factory()
        } else {
            let root = column();
            root.fill_size();
            retained_view(&root)
        };
        *self.inner.retained_view.borrow_mut() = Some(resolved.clone());
        resolved
    }

    fn existing_view(&self) -> Option<RetainedView> {
        self.inner.retained_view.borrow().clone()
    }

    fn dispose_content(&self) {
        if let Some(view) = self.inner.retained_view.borrow_mut().take() {
            view.dispose();
        }
        if let Some(view) = self.inner.prepared_view.borrow_mut().take() {
            view.dispose();
        }
        self.inner.factory.borrow_mut().take();
    }
}

#[derive(Clone, Copy)]
struct SelectionRequest {
    index: i32,
    emit: bool,
}

struct TabViewInner {
    root: FlexBox,
    content_presenter: FlexBox,
    items: RefCell<Vec<TabItem>>,
    selected_index: Cell<i32>,
    selecting: Cell<bool>,
    pending: Cell<Option<SelectionRequest>>,
    changed: RefCell<Option<TabSelectionChangedCallback>>,
}

impl Drop for TabViewInner {
    fn drop(&mut self) {
        for item in self.items.get_mut().drain(..) {
            item.bind_owner(Weak::new());
            item.dispose_content();
        }
    }
}

#[derive(Clone)]
pub struct TabView {
    inner: Rc<TabViewInner>,
}

impl TabView {
    pub fn new() -> Self {
        let root = column();
        let content_presenter = column();
        content_presenter
            .fill_size()
            .semantic_role(SemanticRole::TabPanel)
            .semantic_label("Tab panel");
        root.child(&content_presenter);
        Self {
            inner: Rc::new(TabViewInner {
                root,
                content_presenter,
                items: RefCell::new(Vec::new()),
                selected_index: Cell::new(-1),
                selecting: Cell::new(false),
                pending: Cell::new(None),
                changed: RefCell::new(None),
            }),
        }
    }

    pub fn with_items<I>(items: I) -> Self
    where
        I: IntoIterator<Item = TabItem>,
    {
        let tabs = Self::new();
        tabs.items(items);
        tabs
    }

    pub fn selected_index(&self) -> i32 {
        self.inner.selected_index.get()
    }

    pub fn selected_item(&self) -> Option<TabItem> {
        self.item_at(self.selected_index())
    }

    pub fn item_count(&self) -> usize {
        self.inner.items.borrow().len()
    }

    pub fn content_presenter(&self) -> FlexBox {
        self.inner.content_presenter.clone()
    }

    pub fn item_at(&self, index: i32) -> Option<TabItem> {
        if index < 0 {
            return None;
        }
        self.inner.items.borrow().get(index as usize).cloned()
    }

    pub fn items<I>(&self, items: I) -> &Self
    where
        I: IntoIterator<Item = TabItem>,
    {
        self.clear_items();
        for item in items {
            self.add_item(item);
        }
        self
    }

    pub fn add_item(&self, item: TabItem) -> &Self {
        if self.index_of_item(&item) >= 0 {
            return self;
        }
        item.bind_owner(Rc::downgrade(&self.inner));
        self.inner.items.borrow_mut().push(item.clone());
        if self.selected_index() < 0 && item.is_enabled() {
            self.select_index_internal(self.item_count() as i32 - 1, false, None, -1);
        }
        self
    }

    pub fn remove_item(&self, item: &TabItem) -> &Self {
        let index = self.index_of_item(item);
        if index >= 0 {
            self.remove_item_at(index);
        }
        self
    }

    pub fn remove_item_at(&self, index: i32) -> &Self {
        if index < 0 || index as usize >= self.item_count() {
            return self;
        }
        let removed = self.item_at(index).expect("validated tab index");
        let was_selected = index == self.selected_index();
        if was_selected {
            self.detach_active_view();
        }
        removed.bind_owner(Weak::new());
        self.inner.items.borrow_mut().remove(index as usize);
        if was_selected {
            let previous_index = self.selected_index();
            self.inner.selected_index.set(-1);
            let replacement = self.find_replacement_index(index);
            if replacement >= 0 {
                self.select_index_internal(
                    replacement,
                    true,
                    Some(removed.clone()),
                    previous_index,
                );
            } else {
                self.emit_changed(previous_index, -1, Some(removed.clone()), None);
            }
        } else if self.selected_index() > index {
            self.inner.selected_index.set(self.selected_index() - 1);
        }
        removed.dispose_content();
        self
    }

    pub fn clear_items(&self) -> &Self {
        let previous_index = self.selected_index();
        let previous_item = self.selected_item();
        self.detach_active_view();
        for item in self.inner.items.replace(Vec::new()) {
            item.bind_owner(Weak::new());
            item.dispose_content();
        }
        self.inner.selected_index.set(-1);
        if previous_item.is_some() {
            self.emit_changed(previous_index, -1, previous_item, None);
        }
        self
    }

    pub fn select_index(&self, index: i32) -> &Self {
        self.select_index_internal(index, true, None, -1);
        self
    }

    pub fn on_selection_changed(
        &self,
        handler: impl Fn(TabSelectionChangedEventArgs) + 'static,
    ) -> &Self {
        *self.inner.changed.borrow_mut() = Some(Rc::new(handler));
        self
    }

    pub fn clear_selection_changed(&self) -> &Self {
        self.inner.changed.borrow_mut().take();
        self
    }

    fn item_enabled_changed(&self, item: &TabItem) {
        let index = self.index_of_item(item);
        if index < 0 {
            return;
        }
        if !item.is_enabled() && index == self.selected_index() {
            let previous_index = self.selected_index();
            self.detach_active_view();
            self.inner.selected_index.set(-1);
            let replacement = self.find_replacement_index(index);
            if replacement >= 0 {
                self.select_index_internal(replacement, true, Some(item.clone()), previous_index);
            } else {
                self.emit_changed(previous_index, -1, Some(item.clone()), None);
            }
        } else if item.is_enabled() && self.selected_index() < 0 {
            self.select_index_internal(index, true, None, -1);
        }
    }

    fn item_metadata_changed(&self, item: &TabItem) {
        if self.index_of_item(item) == self.selected_index() {
            self.inner
                .content_presenter
                .semantic_label(format!("{} tab panel", item.label_text()));
        }
    }

    fn select_index_internal(
        &self,
        index: i32,
        emit: bool,
        previous_item_override: Option<TabItem>,
        previous_index_override: i32,
    ) {
        if !self.is_enabled_index(index) {
            return;
        }
        if self.inner.selecting.replace(true) {
            self.inner
                .pending
                .set(Some(SelectionRequest { index, emit }));
            return;
        }
        let mut request = SelectionRequest { index, emit };
        let mut override_item = previous_item_override;
        let mut override_index = previous_index_override;
        let mut iterations = 0usize;
        loop {
            if !self.is_enabled_index(request.index) {
                break;
            }
            let previous_index = if override_item.is_some() {
                override_index
            } else {
                self.selected_index()
            };
            let previous_item = override_item.take().or_else(|| self.selected_item());
            override_index = -1;
            if self.selected_index() != request.index {
                self.detach_active_view();
                self.inner.selected_index.set(request.index);
                let item = self.item_at(request.index).expect("enabled tab exists");
                let view = item.resolve_view();
                self.inner
                    .content_presenter
                    .retained_node_ref()
                    .append_child_ref(&view.root());
                self.inner
                    .content_presenter
                    .semantic_label(format!("{} tab panel", item.label_text()));
                view.activate();
                if request.emit {
                    self.emit_changed(previous_index, request.index, previous_item, Some(item));
                }
            }
            let Some(next) = self.inner.pending.take() else {
                break;
            };
            request = next;
            iterations += 1;
            if iterations > self.item_count() + 8 {
                crate::logger::warn("Lifecycle", "TabView stopped a re-entrant selection loop.");
                break;
            }
        }
        self.inner.pending.set(None);
        self.inner.selecting.set(false);
    }

    fn detach_active_view(&self) {
        let Some(item) = self.selected_item() else {
            return;
        };
        let Some(view) = item.existing_view() else {
            return;
        };
        view.deactivate();
        view.root().detach_from_parent();
    }

    fn emit_changed(
        &self,
        previous_index: i32,
        selected_index: i32,
        previous_item: Option<TabItem>,
        selected_item: Option<TabItem>,
    ) {
        let callback = self.inner.changed.borrow().clone();
        if let Some(callback) = callback {
            callback(TabSelectionChangedEventArgs {
                previous_index,
                selected_index,
                previous_item,
                selected_item,
            });
        }
    }

    fn index_of_item(&self, item: &TabItem) -> i32 {
        self.inner
            .items
            .borrow()
            .iter()
            .position(|candidate| candidate.ptr_eq(item))
            .map_or(-1, |index| index as i32)
    }

    fn is_enabled_index(&self, index: i32) -> bool {
        self.item_at(index).is_some_and(|item| item.is_enabled())
    }

    fn find_replacement_index(&self, index: i32) -> i32 {
        for cursor in index..self.item_count() as i32 {
            if self.is_enabled_index(cursor) {
                return cursor;
            }
        }
        for cursor in (0..index.min(self.item_count() as i32)).rev() {
            if self.is_enabled_index(cursor) {
                return cursor;
            }
        }
        -1
    }
}

impl Default for TabView {
    fn default() -> Self {
        Self::new()
    }
}

impl Node for TabView {
    fn retained_node_ref(&self) -> NodeRef {
        self.inner.root.retained_node_ref()
    }

    fn retained_owner_attachment(&self) -> Option<Rc<dyn Any>> {
        Some(self.inner.clone())
    }

    fn build_self(&self) {
        self.inner.root.build_self();
    }
}

impl HasFlexBoxRoot for TabView {
    fn flex_box_root(&self) -> &FlexBox {
        &self.inner.root
    }
}

impl ThemeBindable for TabView {
    fn theme_binding_node(&self) -> NodeRef {
        self.inner.root.retained_node_ref()
    }

    fn weak_theme_target(&self) -> Box<dyn Fn() -> Option<Self>> {
        let weak = Rc::downgrade(&self.inner);
        Box::new(move || {
            Some(Self {
                inner: weak.upgrade()?,
            })
        })
    }
}

pub fn tab_view() -> TabView {
    TabView::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::{self, Call};
    use crate::Application;

    fn view(label: &str, events: Rc<RefCell<Vec<String>>>) -> RetainedView {
        let root = column();
        root.child(&crate::node::text(label));
        retained_view(&root)
            .on_activate({
                let events = events.clone();
                let label = label.to_string();
                move |_| events.borrow_mut().push(format!("activate:{label}"))
            })
            .on_deactivate({
                let events = events.clone();
                let label = label.to_string();
                move |_| events.borrow_mut().push(format!("deactivate:{label}"))
            })
    }

    #[test]
    fn lazy_content_materializes_once_and_lifecycle_follows_selection() {
        ffi::test::reset();
        let events = Rc::new(RefCell::new(Vec::new()));
        let builds = Rc::new(Cell::new(0));
        let first = TabItem::with_content("First", {
            let events = events.clone();
            let builds = builds.clone();
            move || {
                builds.set(builds.get() + 1);
                view("first", events)
            }
        });
        let second = TabItem::with_content_view("Second", view("second", events.clone()));
        let tabs = TabView::with_items([first.clone(), second]);
        assert_eq!(tabs.selected_index(), 0);
        assert_eq!(builds.get(), 1);
        tabs.select_index(1).select_index(0);
        assert_eq!(builds.get(), 1);
        assert_eq!(
            &*events.borrow(),
            &[
                "activate:first",
                "deactivate:first",
                "activate:second",
                "deactivate:second",
                "activate:first"
            ]
        );
        assert_eq!(
            tabs.content_presenter()
                .retained_node_ref()
                .children()
                .len(),
            1
        );
    }

    #[test]
    fn retained_tree_contains_only_the_selected_content_host() {
        ffi::test::reset();
        let tabs = TabView::with_items([TabItem::new("Alpha"), TabItem::new("Beta")]);
        Application::mount(tabs.clone());
        assert_eq!(tabs.retained_node_ref().children().len(), 1);
        let calls = ffi::test::take_calls();
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::SetSemanticRole { role_enum, .. } if *role_enum == SemanticRole::TabPanel as u32
        )));
        assert!(!calls.iter().any(|call| matches!(
            call,
            Call::SetSemanticRole { role_enum, .. }
                if *role_enum == SemanticRole::TabList as u32 || *role_enum == SemanticRole::Tab as u32
        )));
        Application::unmount();
    }

    #[test]
    fn disabled_removal_and_reentrant_selection_are_deterministic() {
        let tabs = TabView::with_items([
            TabItem::new("Alpha"),
            TabItem::new("Beta"),
            TabItem::new("Gamma"),
        ]);
        let callback_tabs = tabs.clone();
        tabs.on_selection_changed(move |event| {
            if event.selected_index == 1 {
                callback_tabs.select_index(2);
            }
        });
        tabs.select_index(1);
        assert_eq!(tabs.selected_index(), 2);
        tabs.remove_item_at(2);
        assert_eq!(tabs.selected_index(), 1);
        tabs.item_at(1).unwrap().enabled(false);
        assert_eq!(tabs.selected_index(), 0);
    }

    #[test]
    fn inline_composition_retains_owner_and_items_accept_fluent_values() {
        let changes = Rc::new(Cell::new(0));
        let first = TabItem::new("First");
        let tabs = tab_view();
        tabs.items(crate::tab_items![
            &first,
            TabItem::new("Second").content(|| retained_view(&crate::node::text("Second"))),
        ])
        .on_selection_changed({
            let changes = changes.clone();
            move |_| changes.set(changes.get() + 1)
        });
        let root = crate::ui! { column() { tabs } };
        drop(tabs);
        first.enabled(false);
        assert_eq!(changes.get(), 1);
        Application::mount(root);
        Application::unmount();
    }
}
