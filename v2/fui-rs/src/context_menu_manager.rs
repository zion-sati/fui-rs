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
        return false;
    }
    let mut items = build_built_in_items(handle, x, y, true);

    let mut navigation_items = Vec::new();
    if navigation::can_navigate_back() {
        navigation_items.push(
            MenuItem::new("Back", ContextMenuAction::NavigateBack)
                .shortcut_label(resolve_history_shortcut_label(false)),
        );
    }
    if navigation::can_navigate_forward() {
        navigation_items.push(
            MenuItem::new("Forward", ContextMenuAction::NavigateForward)
                .shortcut_label(resolve_history_shortcut_label(true)),
        );
    }
    navigation_items.push(
        MenuItem::new("Reload Page", ContextMenuAction::ReloadPage)
            .shortcut_label(platform::format_primary_shortcut_label("r")),
    );
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
    let items = build_built_in_items(handle, x, y, false);
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
        append_menu_section(
            &mut items,
            vec![
                MenuItem::new("New Tab", ContextMenuAction::OpenLinkInNewTab).payload(href.clone()),
                MenuItem::new("Open", ContextMenuAction::OpenLink).payload(href),
            ],
        );
    }

    let selection_hit = ui::is_point_in_selection(point_x, point_y);
    if let Some(text_target) = resolve_text_target(target_node.clone()) {
        append_menu_section(&mut items, build_text_section(&text_target));
    }

    if let Some(image_url) = resolve_image_url(target_node.clone(), point_x, point_y) {
        append_menu_section(
            &mut items,
            vec![
                MenuItem::new("New Tab", ContextMenuAction::OpenImageInNewTab)
                    .payload(image_url.clone()),
                MenuItem::new("Open", ContextMenuAction::OpenImage).payload(image_url),
            ],
        );
    }

    if selection_hit && resolve_text_target(target_node.clone()).is_none() {
        append_menu_section(
            &mut items,
            vec![MenuItem::new(
                "Copy",
                ContextMenuAction::CopyCurrentSelection,
            )],
        );
    } else if clear_selection_on_background_miss && !selection_hit && items.is_empty() {
        ui::clear_current_selection();
    }

    items
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

fn build_text_section(target: &NodeRef) -> Vec<MenuItem> {
    let handle = target.handle().raw();
    if handle == HandleValue::Invalid as u64 || !target.is_selectable_text_for_routing() {
        return Vec::new();
    }

    let has_selection = unsafe { crate::ffi::ui_has_text_selection(handle) }
        || unsafe { crate::ffi::fui_has_text_selection_snapshot(handle) };
    let has_text = target
        .text_content_for_routing()
        .map(|content| !content.is_empty())
        .unwrap_or(false);

    vec![
        MenuItem::new("Copy", ContextMenuAction::CopyCurrentSelection)
            .shortcut_label(platform::format_primary_shortcut_label("c"))
            .disabled(!has_selection)
            .target_handle(handle),
        MenuItem::new("Select All", ContextMenuAction::SelectAllText)
            .shortcut_label(platform::format_primary_shortcut_label("a"))
            .disabled(!has_text)
            .target_handle(handle),
    ]
}
