use crate::bindings::ui;
use crate::controls::{ContextMenu, ContextMenuAction, MenuItem};
use crate::event;
use crate::ffi::{HandleValue, PointerEventType};
use crate::navigation;
use crate::node::NodeRef;
use crate::platform;
use std::cell::RefCell;

thread_local! {
    static ACTIVE_POINTER_SELECTION_HANDLES: RefCell<Vec<u64>> = const { RefCell::new(Vec::new()) };
    static DEFAULT_MENU: RefCell<Option<ContextMenu>> = const { RefCell::new(None) };
    static ACTIVE_MENU_LINK: RefCell<Option<NodeRef>> = const { RefCell::new(None) };
}

fn append_menu_section(items: &mut Vec<MenuItem>, section: Vec<MenuItem>) {
    if section.is_empty() {
        return;
    }
    if !items.is_empty() {
        items.push(MenuItem::separator());
    }
    items.extend(section);
}

fn default_menu() -> ContextMenu {
    DEFAULT_MENU.with(|slot| {
        let mut slot = slot.borrow_mut();
        if let Some(menu) = slot.as_ref() {
            return menu.clone();
        }
        let menu = create_default_menu();
        *slot = Some(menu.clone());
        menu
    })
}

fn resolve_history_shortcut_label(forward: bool) -> String {
    if platform::platform_family() == platform::PlatformFamily::Apple {
        return platform::format_shortcut_label(
            if forward { "]" } else { "[" },
            crate::ffi::KeyModifier::Meta as u32,
        );
    }
    platform::format_shortcut_label(
        if forward { "ArrowRight" } else { "ArrowLeft" },
        crate::ffi::KeyModifier::Alt as u32,
    )
}

pub(crate) fn create_default_menu() -> ContextMenu {
    let menu = ContextMenu::new();
    menu.on_visibility_changed(|event| {
        if !event.visible {
            hide_active_menu();
        }
    });
    DEFAULT_MENU.with(|slot| slot.replace(Some(menu.clone())));
    menu
}

pub(crate) fn track_pointer_event(event_type: PointerEventType, handle: u64) {
    if event_type != PointerEventType::Down {
        return;
    }

    ACTIVE_POINTER_SELECTION_HANDLES.with(|handles| {
        let mut handles = handles.borrow_mut();
        handles.clear();
        if handle != HandleValue::Invalid as u64 {
            handles.push(handle);
        }
    });
}

pub(crate) fn can_show_for_handle(handle: u64) -> bool {
    let Some(target) = event::registered_node(handle) else {
        return handle == HandleValue::Invalid as u64;
    };
    !has_disabled_context_menu_ancestor(Some(target))
}

pub(crate) fn show_for_current_selection(handle: u64, x: f32, y: f32) -> bool {
    let target_node = event::registered_node(handle);
    if has_disabled_context_menu_ancestor(target_node.clone()) {
        return false;
    }
    if invoke_custom_context_menu_handler(target_node.clone(), handle, x, y) {
        return true;
    }
    let host = platform::host_context();
    let mut items = build_built_in_items(handle, x, y, true, host);

    let mut navigation_items = Vec::new();
    if host.supports(platform::HostCapability::BrowserHistory) && navigation::can_navigate_back() {
        navigation_items.push(
            MenuItem::new("Back", ContextMenuAction::NavigateBack)
                .shortcut_label(resolve_history_shortcut_label(false)),
        );
    }
    if host.supports(platform::HostCapability::BrowserHistory) && navigation::can_navigate_forward()
    {
        navigation_items.push(
            MenuItem::new("Forward", ContextMenuAction::NavigateForward)
                .shortcut_label(resolve_history_shortcut_label(true)),
        );
    }
    if host.supports(platform::HostCapability::Reload) {
        navigation_items.push(
            MenuItem::new("Reload Page", ContextMenuAction::ReloadPage)
                .shortcut_label(platform::format_primary_shortcut_label("r")),
        );
    }
    append_menu_section(&mut items, navigation_items);

    if items.is_empty() {
        return false;
    }

    let menu = default_menu();
    menu.items(items);
    menu.show_from_context_pointer(x, y);
    true
}

pub(crate) fn show_for_long_press(handle: u64, x: f32, y: f32) -> bool {
    let target_node = event::registered_node(handle);
    if has_disabled_context_menu_ancestor(target_node.clone()) {
        return false;
    }
    if invoke_custom_context_menu_handler(target_node, handle, x, y) {
        return true;
    }
    let items = build_built_in_items(handle, x, y, false, platform::host_context());
    if items.is_empty() {
        return false;
    }

    let menu = default_menu();
    menu.items(items);
    menu.show_from_context_pointer(x, y);
    true
}

pub(crate) fn hide_active_menu() {
    release_active_menu_link_preview();
    ContextMenu::hide_active_menu();
}

fn build_built_in_items(
    handle: u64,
    point_x: f32,
    point_y: f32,
    clear_selection_on_background_miss: bool,
    host: platform::HostContext,
) -> Vec<MenuItem> {
    release_active_menu_link_preview();
    let target_node = event::registered_node(handle);
    if has_disabled_context_menu_ancestor(target_node.clone()) {
        return Vec::new();
    }
    let mut items = Vec::new();

    if let Some((link, href)) = resolve_nav_link(target_node.clone()) {
        link.pin_link_preview_for_routing();
        ACTIVE_MENU_LINK.with(|slot| slot.replace(Some(link)));
        let mut link_items = Vec::new();
        if host.supports(platform::HostCapability::NewBrowsingContext) {
            link_items.push(
                MenuItem::new("New Tab", ContextMenuAction::OpenLinkInNewTab).payload(href.clone()),
            );
        }
        if host.supports(platform::HostCapability::OpenExternalUri) {
            link_items.push(MenuItem::new("Open", ContextMenuAction::OpenLink).payload(href));
        }
        append_menu_section(&mut items, link_items);
    }

    let current_selection_text = current_selection_text();
    let selection_hit =
        ui::is_point_in_selection(point_x, point_y) || selection_hint_contains_handle(handle);
    if let Some(text_target) = resolve_text_target(target_node.clone()) {
        append_menu_section(
            &mut items,
            build_text_section(
                &text_target,
                if selection_hit {
                    current_selection_text.as_str()
                } else {
                    ""
                },
                host,
            ),
        );
    }

    if let Some(image_url) = resolve_image_url(target_node.clone(), point_x, point_y) {
        let mut image_items = Vec::new();
        if host.supports(platform::HostCapability::NewBrowsingContext) {
            image_items.push(
                MenuItem::new("New Tab", ContextMenuAction::OpenImageInNewTab)
                    .payload(image_url.clone()),
            );
        }
        if host.supports(platform::HostCapability::OpenExternalUri) {
            image_items
                .push(MenuItem::new("Open", ContextMenuAction::OpenImage).payload(image_url));
        }
        append_menu_section(&mut items, image_items);
    }

    if selection_hit
        && resolve_text_target(target_node.clone()).is_none()
        && !current_selection_text.is_empty()
        && host.supports(platform::HostCapability::ClipboardWrite)
    {
        append_menu_section(
            &mut items,
            vec![
                MenuItem::new("Copy", ContextMenuAction::CopyCurrentSelection)
                    .payload(current_selection_text),
            ],
        );
    } else if clear_selection_on_background_miss && !selection_hit && items.is_empty() {
        clear_selection_context();
        ui::clear_current_selection();
    }

    items
}

#[derive(Default)]
struct SelectionContextState {
    active_pointer_selection_handles: Vec<u64>,
    current_selection_handle_hints: Vec<u64>,
    current_selection_text: String,
}

thread_local! {
    static SELECTION_CONTEXT: std::cell::RefCell<SelectionContextState> =
        std::cell::RefCell::new(SelectionContextState::default());
}

pub(crate) fn handle_pointer_selection_event(is_pointer_down: bool, handle: u64) {
    SELECTION_CONTEXT.with(|state| {
        let mut state = state.borrow_mut();
        if is_pointer_down {
            state.active_pointer_selection_handles.clear();
        }
        if handle != HandleValue::Invalid as u64
            && !state.active_pointer_selection_handles.contains(&handle)
        {
            state.active_pointer_selection_handles.push(handle);
        }
    });
}

pub(crate) fn handle_selection_changed(text: &str) {
    SELECTION_CONTEXT.with(|state| {
        let mut state = state.borrow_mut();
        state.current_selection_text.clear();
        state.current_selection_text.push_str(text);
        state.current_selection_handle_hints.clear();
        if !text.is_empty() {
            state.current_selection_handle_hints = state.active_pointer_selection_handles.clone();
        }
    });
}

fn current_selection_text() -> String {
    SELECTION_CONTEXT.with(|state| state.borrow().current_selection_text.clone())
}

fn selection_hint_contains_handle(handle: u64) -> bool {
    handle != HandleValue::Invalid as u64
        && SELECTION_CONTEXT.with(|state| {
            state
                .borrow()
                .current_selection_handle_hints
                .contains(&handle)
        })
}

fn clear_selection_context() {
    SELECTION_CONTEXT.with(|state| {
        let mut state = state.borrow_mut();
        state.current_selection_text.clear();
        state.current_selection_handle_hints.clear();
    });
}

fn has_disabled_context_menu_ancestor(node: Option<NodeRef>) -> bool {
    let mut current = node;
    while let Some(node) = current {
        if node.is_context_menu_disabled_for_routing() {
            return true;
        }
        current = node.parent();
    }
    false
}

fn invoke_custom_context_menu_handler(
    node: Option<NodeRef>,
    target_handle: u64,
    x: f32,
    y: f32,
) -> bool {
    let mut current = node;
    while let Some(node) = current {
        if node.is_context_menu_disabled_for_routing() {
            return true;
        }
        if let Some(handler) = node.context_menu_handler_for_routing() {
            handler(crate::node::ContextMenuEventArgs {
                target: crate::node::NodeHandle::from_raw(target_handle),
                x,
                y,
                host: platform::host_context(),
            });
            return true;
        }
        current = node.parent();
    }
    false
}

fn release_active_menu_link_preview() {
    ACTIVE_MENU_LINK.with(|slot| {
        let link = slot.borrow_mut().take();
        if let Some(link) = link {
            link.release_link_preview_for_routing();
        }
    });
}

fn resolve_nav_link(node: Option<NodeRef>) -> Option<(NodeRef, String)> {
    let mut current = node;
    while let Some(node) = current {
        if let Some(href) = node.link_url_for_routing() {
            if !href.is_empty() {
                return Some((node, href));
            }
        }
        current = node.parent();
    }
    None
}

fn resolve_text_target(node: Option<NodeRef>) -> Option<NodeRef> {
    let mut current = node;
    while let Some(node) = current {
        if let Some(editor) = node.text_input_editor_for_routing() {
            return Some(editor);
        }
        if node.is_selectable_text_for_routing() || node.is_editable_text_for_routing() {
            return Some(node);
        }
        current = node.parent();
    }
    None
}

fn resolve_image_url(node: Option<NodeRef>, point_x: f32, point_y: f32) -> Option<String> {
    let mut current = node.clone();
    while let Some(node) = current {
        if let Some(url) = node.image_url_for_routing() {
            if !url.is_empty() {
                return Some(url);
            }
        }
        current = node.parent();
    }
    node.and_then(|node| resolve_descendant_image_url(&node, point_x, point_y))
}

fn resolve_descendant_image_url(node: &NodeRef, point_x: f32, point_y: f32) -> Option<String> {
    for child in node.children() {
        let bounds = if child.handle() == crate::node::NodeHandle::INVALID {
            [0.0; 4]
        } else {
            ui::get_bounds(child.handle().raw()).unwrap_or([0.0; 4])
        };
        let contains = point_x >= bounds[0]
            && point_y >= bounds[1]
            && point_x <= bounds[0] + bounds[2]
            && point_y <= bounds[1] + bounds[3];
        if !contains {
            continue;
        }
        if let Some(url) = child.image_url_for_routing() {
            if !url.is_empty() {
                return Some(url);
            }
        }
        if let Some(url) = resolve_descendant_image_url(&child, point_x, point_y) {
            return Some(url);
        }
    }
    None
}

fn build_text_section(
    target: &NodeRef,
    current_selection_text: &str,
    host: platform::HostContext,
) -> Vec<MenuItem> {
    let handle = target.handle().raw();
    if handle == HandleValue::Invalid as u64 || !target.is_selectable_text_for_routing() {
        return Vec::new();
    }

    let content = target.text_content_for_routing().unwrap_or_default();
    let (selection_start, selection_end) = target
        .text_selection_range_bytes_for_routing()
        .unwrap_or((0, 0));
    let range_start = selection_start.min(selection_end).min(content.len() as u32);
    let range_end = selection_start.max(selection_end).min(content.len() as u32);
    let selected_payload = if current_selection_text.is_empty() {
        content
            .get(range_start as usize..range_end as usize)
            .filter(|text| !text.is_empty())
            .map(str::to_owned)
    } else {
        Some(current_selection_text.to_owned())
    };
    let has_selection =
        unsafe { crate::ffi::ui_has_text_selection(handle) } || selected_payload.is_some();
    let has_text = !content.is_empty();

    let mut items = Vec::new();
    if target.is_editable_text_for_routing() {
        items.push(
            MenuItem::new("Undo", ContextMenuAction::UndoTextEdit)
                .shortcut_label(platform::format_undo_shortcut_label())
                .disabled(!unsafe { crate::ffi::ui_can_undo_text_edit(handle) })
                .target_handle(handle)
                .focus_target_after_action(true),
        );
        items.push(
            MenuItem::new("Redo", ContextMenuAction::RedoTextEdit)
                .shortcut_label(platform::format_redo_shortcut_label())
                .disabled(!unsafe { crate::ffi::ui_can_redo_text_edit(handle) })
                .target_handle(handle)
                .focus_target_after_action(true),
        );
        items.push(MenuItem::separator());
        if host.supports(platform::HostCapability::ClipboardWrite) {
            items.push(
                MenuItem::new("Cut", ContextMenuAction::CutTextSelection)
                    .payload(selected_payload.clone().unwrap_or_default())
                    .shortcut_label(platform::format_primary_shortcut_label("x"))
                    .disabled(!has_selection)
                    .target_handle(handle)
                    .with_selection_range(range_start, range_end)
                    .focus_target_after_action(true),
            );
            items.push(
                MenuItem::new("Copy", ContextMenuAction::CopyCurrentSelection)
                    .payload(selected_payload.clone().unwrap_or_default())
                    .shortcut_label(platform::format_primary_shortcut_label("c"))
                    .disabled(!has_selection)
                    .target_handle(handle)
                    .focus_target_after_action(true),
            );
        }
        if host.supports(platform::HostCapability::ClipboardRead) {
            items.push(
                MenuItem::new("Paste", ContextMenuAction::PasteText)
                    .shortcut_label(platform::format_primary_shortcut_label("v"))
                    .target_handle(handle)
                    .focus_target_after_action(true),
            );
        }
        items.push(
            MenuItem::new("Select All", ContextMenuAction::SelectAllText)
                .shortcut_label(platform::format_primary_shortcut_label("a"))
                .disabled(!has_text)
                .target_handle(handle)
                .focus_target_after_action(true),
        );
        return items;
    }

    if host.supports(platform::HostCapability::ClipboardWrite) {
        items.push(
            MenuItem::new("Copy", ContextMenuAction::CopyCurrentSelection)
                .payload(selected_payload.unwrap_or_default())
                .shortcut_label(platform::format_primary_shortcut_label("c"))
                .disabled(!has_selection)
                .target_handle(handle),
        );
    }
    items.push(
        MenuItem::new("Select All", ContextMenuAction::SelectAllText)
            .shortcut_label(platform::format_primary_shortcut_label("a"))
            .disabled(!has_text)
            .target_handle(handle),
    );
    items
}

#[cfg(test)]
mod tests {
    use super::{build_built_in_items, invoke_custom_context_menu_handler};
    use crate::app::Application;
    use crate::controls::ContextMenuAction;
    use crate::ffi;
    use crate::node::Node;
    use crate::platform::{HostCapability, HostContext, HostEnvironment, PlatformFamily};
    use crate::prelude::*;
    use std::cell::Cell;
    use std::rc::Rc;

    fn has_action(items: &[crate::controls::MenuItem], action: ContextMenuAction) -> bool {
        items.iter().any(|item| item.action == action)
    }

    #[test]
    fn policy_matches_fui_as_for_browser_desktop_and_headless_targets() {
        ffi::test::reset();
        let root = column();
        let blank = column();
        let link = nav_link("https://example.test");
        let image_node = image(0);
        image_node.source("https://example.test/image.png");
        let svg_node = svg(0);
        svg_node.source("https://example.test/image.svg");
        let static_text = text("Selectable");
        let editable = text_input();
        editable.text("Editable");
        root.child(&blank)
            .child(&link)
            .child(&image_node)
            .child(&svg_node)
            .child(&static_text)
            .child(&editable);
        Application::mount(root.clone());

        let browser = HostContext::new(
            PlatformFamily::Apple,
            HostEnvironment::Browser,
            HostCapability::BrowserHistory as u32
                | HostCapability::Reload as u32
                | HostCapability::NewBrowsingContext as u32
                | HostCapability::OpenExternalUri as u32
                | HostCapability::ClipboardRead as u32
                | HostCapability::ClipboardWrite as u32,
        );
        let desktop = HostContext::new(
            PlatformFamily::Apple,
            HostEnvironment::Desktop,
            HostCapability::OpenExternalUri as u32
                | HostCapability::ClipboardRead as u32
                | HostCapability::ClipboardWrite as u32,
        );
        let headless = HostContext::new(PlatformFamily::Linux, HostEnvironment::Headless, 0);

        assert!(build_built_in_items(blank.handle().raw(), 10.0, 10.0, false, desktop).is_empty());
        assert!(build_built_in_items(blank.handle().raw(), 10.0, 10.0, false, headless).is_empty());

        for handle in [
            link.handle().raw(),
            image_node.handle().raw(),
            svg_node.handle().raw(),
        ] {
            let browser_items = build_built_in_items(handle, 10.0, 10.0, false, browser);
            assert!(
                has_action(&browser_items, ContextMenuAction::OpenLinkInNewTab)
                    || has_action(&browser_items, ContextMenuAction::OpenImageInNewTab)
            );
            let desktop_items = build_built_in_items(handle, 10.0, 10.0, false, desktop);
            assert!(!has_action(
                &desktop_items,
                ContextMenuAction::OpenLinkInNewTab
            ));
            assert!(!has_action(
                &desktop_items,
                ContextMenuAction::OpenImageInNewTab
            ));
            assert!(
                has_action(&desktop_items, ContextMenuAction::OpenLink)
                    || has_action(&desktop_items, ContextMenuAction::OpenImage)
            );
            assert!(build_built_in_items(handle, 10.0, 10.0, false, headless).is_empty());
        }

        let browser_text =
            build_built_in_items(static_text.handle().raw(), 10.0, 10.0, false, browser);
        assert!(has_action(
            &browser_text,
            ContextMenuAction::CopyCurrentSelection
        ));
        assert!(has_action(&browser_text, ContextMenuAction::SelectAllText));
        crate::event::__fui_on_pointer_event_with_metadata(
            1,
            static_text.handle().raw(),
            10.0,
            10.0,
            0,
            1,
            crate::event::PointerType::Mouse as u32,
            0,
            1,
            0.5,
            1.0,
            1.0,
            1,
        );
        let cross_selection = "cross-selected text";
        unsafe {
            crate::event::__fui_on_cross_selection_changed(
                static_text.handle().raw(),
                cross_selection.as_ptr(),
                cross_selection.len() as u32,
            );
        }
        let browser_cross_selection =
            build_built_in_items(static_text.handle().raw(), 10.0, 10.0, false, browser);
        let cross_copy = browser_cross_selection
            .iter()
            .find(|item| item.action == ContextMenuAction::CopyCurrentSelection)
            .expect("cross-selection copy action");
        assert_eq!(cross_copy.payload.as_deref(), Some("cross-selected text"));
        assert!(!cross_copy.disabled);
        unsafe {
            crate::event::__fui_on_cross_selection_changed(
                static_text.handle().raw(),
                std::ptr::null(),
                0,
            );
        }
        let headless_text =
            build_built_in_items(static_text.handle().raw(), 10.0, 10.0, false, headless);
        assert!(!has_action(
            &headless_text,
            ContextMenuAction::CopyCurrentSelection
        ));
        assert!(has_action(&headless_text, ContextMenuAction::SelectAllText));

        let editor_handle = editable.editor_node().handle().raw();
        editable.selection_range(1, 5);
        let browser_editable = build_built_in_items(editor_handle, 10.0, 10.0, false, browser);
        for action in [
            ContextMenuAction::UndoTextEdit,
            ContextMenuAction::RedoTextEdit,
            ContextMenuAction::CutTextSelection,
            ContextMenuAction::CopyCurrentSelection,
            ContextMenuAction::PasteText,
            ContextMenuAction::SelectAllText,
        ] {
            assert!(has_action(&browser_editable, action));
        }
        for action in [
            ContextMenuAction::CutTextSelection,
            ContextMenuAction::CopyCurrentSelection,
        ] {
            let item = browser_editable
                .iter()
                .find(|item| item.action == action)
                .expect("selection action");
            assert_eq!(item.payload.as_deref(), Some("dita"));
            assert!(!item.disabled);
        }
        let headless_editable = build_built_in_items(editor_handle, 10.0, 10.0, false, headless);
        assert!(has_action(
            &headless_editable,
            ContextMenuAction::UndoTextEdit
        ));
        assert!(has_action(
            &headless_editable,
            ContextMenuAction::RedoTextEdit
        ));
        assert!(!has_action(
            &headless_editable,
            ContextMenuAction::CutTextSelection
        ));
        assert!(!has_action(
            &headless_editable,
            ContextMenuAction::CopyCurrentSelection
        ));
        assert!(!has_action(
            &headless_editable,
            ContextMenuAction::PasteText
        ));
        assert!(has_action(
            &headless_editable,
            ContextMenuAction::SelectAllText
        ));

        assert!(build_built_in_items(blank.handle().raw(), 10.0, 10.0, false, desktop).is_empty());

        let custom_count = Rc::new(Cell::new(0));
        let custom = column();
        custom.on_context_menu({
            let custom_count = custom_count.clone();
            move |_| custom_count.set(custom_count.get() + 1)
        });
        root.child(&custom);
        for environment in [
            HostEnvironment::Browser,
            HostEnvironment::Desktop,
            HostEnvironment::Headless,
        ] {
            ffi::test::set_host_environment(environment as u32);
            assert!(invoke_custom_context_menu_handler(
                Some(custom.retained_node_ref()),
                custom.handle().raw(),
                10.0,
                10.0,
            ));
        }
        assert_eq!(custom_count.get(), 3);
    }
}
