use super::*;
use crate::bindings::ui;
use crate::event::PointerType;
use crate::ffi::{CursorStyle, HandleValue, SemanticRole};
use crate::navigation;
use crate::node::{text, NodeHandle, NodeRef, WeakFlexBox};
use crate::platform;
use crate::theme::{current_theme, subscribe};
use crate::{focus_adorner, focus_visibility};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

thread_local! {
    static ACTIVE_PREVIEW_OWNER: Cell<u64> = const { Cell::new(HandleValue::Invalid as u64) };
}

fn is_primary_activation_pointer(event: &PointerEventArgs) -> bool {
    event.button == 0
        || event.pointer_type == PointerType::Touch
        || event.pointer_type == PointerType::Pen
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigateEventArgs {
    pub path: String,
}

type NavigateCallback = Rc<dyn Fn(NavigateEventArgs)>;

#[derive(Clone)]
pub struct NavLink {
    root: FlexBox,
    label: TextNode,
    href: Rc<RefCell<String>>,
    open_in_new_tab: Rc<Cell<bool>>,
    navigate: Rc<RefCell<Option<NavigateCallback>>>,
    hovered: Rc<Cell<bool>>,
    focused: Rc<Cell<bool>>,
    pointer_pressed: Rc<Cell<bool>>,
    pointer_pressed_open_in_new_tab: Rc<Cell<bool>>,
    enter_pressed: Rc<Cell<bool>>,
    enter_pressed_open_in_new_tab: Rc<Cell<bool>>,
    preview_pinned_for_context_menu: Rc<Cell<bool>>,
    text_color_override: Rc<Cell<Option<u32>>>,
}

impl NavLink {
    pub fn new(href: impl Into<String>) -> Self {
        let href = href.into();
        Self::with_label(href.clone(), href)
    }

    pub fn with_label(href: impl Into<String>, label: impl Into<String>) -> Self {
        let href = href.into();
        let label = label.into();
        let root = row();
        let label_node = text(&label);
        let theme = current_theme();
        label_node
            .font_family(theme.fonts.body_family.clone())
            .font_size(15.0)
            .text_color(theme.colors.accent)
            .cursor(CursorStyle::Pointer);
        root.justify_content(JustifyContent::Start)
            .align_items(AlignItems::Center)
            .interactive(true)
            .focusable(true, 0)
            .cursor(CursorStyle::Pointer)
            .semantic_role(SemanticRole::Link)
            .semantic_label(label)
            .reflect_semantic_disabled_from_enabled()
            .child(&label_node);
        root.retained_node_ref()
            .set_link_url_for_routing(Some(href.clone()));

        let link = Self {
            root,
            label: label_node,
            href: Rc::new(RefCell::new(href)),
            open_in_new_tab: Rc::new(Cell::new(false)),
            navigate: Rc::new(RefCell::new(None)),
            hovered: Rc::new(Cell::new(false)),
            focused: Rc::new(Cell::new(false)),
            pointer_pressed: Rc::new(Cell::new(false)),
            pointer_pressed_open_in_new_tab: Rc::new(Cell::new(false)),
            enter_pressed: Rc::new(Cell::new(false)),
            enter_pressed_open_in_new_tab: Rc::new(Cell::new(false)),
            preview_pinned_for_context_menu: Rc::new(Cell::new(false)),
            text_color_override: Rc::new(Cell::new(None)),
        };
        let node_ref = link.root.retained_node_ref();
        let pin_target = link.event_target();
        let release_target = link.event_target();
        node_ref.set_link_preview_handlers_for_routing(
            Some(Rc::new(move || {
                pin_target.pin_preview_for_context_menu();
            })),
            Some(Rc::new(move || {
                release_target.release_preview_for_context_menu();
            })),
        );
        link.install_subscriptions();
        link.bind_events();
        link.sync_visual_state();
        link.sync_focus_chrome();
        link
    }

    fn bind_events(&self) {
        let target = self.event_target();
        self.root.on_pointer_enter(move |_event| {
            target.hovered.set(true);
            target.sync_visual_state();
            target.show_preview();
        });
        let target = self.event_target();
        self.root.on_pointer_leave(move |_event| {
            target.hovered.set(false);
            target.pointer_pressed.set(false);
            target.pointer_pressed_open_in_new_tab.set(false);
            target.sync_visual_state();
            if !target.focused.get() && !target.preview_pinned_for_context_menu.get() {
                target.hide_preview();
            }
        });
        let target = self.event_target();
        self.root.on_pointer_down(move |event| {
            if !is_primary_activation_pointer(event) {
                target.pointer_pressed.set(false);
                target.pointer_pressed_open_in_new_tab.set(false);
                return;
            }
            target.pointer_pressed.set(true);
            target
                .pointer_pressed_open_in_new_tab
                .set(target.should_open_in_new_tab(event.modifiers));
            event.handled = true;
        });
        let target = self.event_target();
        self.root.on_pointer_up(move |event| {
            if target.pointer_pressed.replace(false) && is_primary_activation_pointer(event) {
                let open_in_new_tab = target.pointer_pressed_open_in_new_tab.get()
                    || target.should_open_in_new_tab(event.modifiers);
                target.activate(open_in_new_tab);
                event.handled = true;
            }
            target.pointer_pressed_open_in_new_tab.set(false);
        });
        let target = self.event_target();
        self.root.on_key_down(move |event| {
            if !target.should_handle_enter_key(event.modifiers) || event.key != "Enter" {
                return;
            }
            if target.enter_pressed.get() {
                event.handled = true;
                return;
            }
            target.enter_pressed.set(true);
            target
                .enter_pressed_open_in_new_tab
                .set(target.should_open_in_new_tab(event.modifiers));
            event.handled = true;
        });
        let target = self.event_target();
        self.root.on_key_up(move |event| {
            if !target.should_handle_enter_key(event.modifiers) || event.key != "Enter" {
                return;
            }
            if target.enter_pressed.replace(false) {
                let open_in_new_tab = target.enter_pressed_open_in_new_tab.get()
                    || target.should_open_in_new_tab(event.modifiers);
                target.activate(open_in_new_tab);
                event.handled = true;
            }
            target.enter_pressed_open_in_new_tab.set(false);
        });
        let target = self.event_target();
        self.root.on_focus_changed(move |event| {
            target.focused.set(event.focused);
            if !event.focused {
                target.enter_pressed.set(false);
                target.enter_pressed_open_in_new_tab.set(false);
            }
            target.sync_focus_chrome();
            if event.focused {
                target.show_preview();
            } else if !target.hovered.get() && !target.preview_pinned_for_context_menu.get() {
                target.hide_preview();
            }
        });
    }

    fn install_subscriptions(&self) {
        let target = self.event_target();
        let theme_guard = subscribe(move |_theme| {
            target.sync_visual_state();
            target.sync_focus_chrome();
        });
        self.root
            .retained_node_ref()
            .retain_attachment(Rc::new(theme_guard));
        let target = self.event_target();
        let focus_guard = focus_visibility::subscribe(move |_visible| {
            target.sync_focus_chrome();
        });
        self.root
            .retained_node_ref()
            .retain_attachment(Rc::new(focus_guard));
    }

    fn event_target(&self) -> NavLinkEventTarget {
        NavLinkEventTarget {
            weak_root: self.root.downgrade(),
            label: self.label.clone(),
            href: self.href.clone(),
            open_in_new_tab: self.open_in_new_tab.clone(),
            navigate: self.navigate.clone(),
            hovered: self.hovered.clone(),
            focused: self.focused.clone(),
            pointer_pressed: self.pointer_pressed.clone(),
            pointer_pressed_open_in_new_tab: self.pointer_pressed_open_in_new_tab.clone(),
            enter_pressed: self.enter_pressed.clone(),
            enter_pressed_open_in_new_tab: self.enter_pressed_open_in_new_tab.clone(),
            preview_pinned_for_context_menu: self.preview_pinned_for_context_menu.clone(),
            text_color_override: self.text_color_override.clone(),
        }
    }

    fn set_explicit_font_family(&self, family: crate::FontFamily) {
        self.label.font_family(family);
    }

    fn set_explicit_font_size(&self, size: f32) {
        self.label.font_size(size);
    }

    fn set_explicit_text_color(&self, color: u32) {
        self.text_color_override.set(Some(color));
        self.label.text_color(color);
        self.sync_visual_state();
    }

    pub fn href(&self) -> String {
        self.href.borrow().clone()
    }

    pub fn href_to(&self, href: impl Into<String>) -> &Self {
        let href = href.into();
        *self.href.borrow_mut() = href.clone();
        self.root
            .retained_node_ref()
            .set_link_url_for_routing(Some(href));
        if self.focused.get() || self.hovered.get() || self.preview_pinned_for_context_menu.get() {
            self.event_target().show_preview();
        }
        self
    }

    pub fn text(&self, value: impl Into<String>) -> &Self {
        let value = value.into();
        self.label.text(value.clone());
        self.root.semantic_label(value);
        self
    }

    pub fn label_node(&self) -> TextNode {
        self.label.clone()
    }

    pub fn open_in_new_tab(&self, open: bool) -> &Self {
        self.open_in_new_tab.set(open);
        self
    }

    pub fn on_navigate(&self, handler: impl Fn(NavigateEventArgs) + 'static) -> &Self {
        *self.navigate.borrow_mut() = Some(Rc::new(handler));
        self
    }

    fn sync_visual_state(&self) {
        self.event_target().sync_visual_state();
    }

    fn sync_focus_chrome(&self) {
        self.event_target().sync_focus_chrome();
    }
}

impl Node for NavLink {
    fn retained_node_ref(&self) -> NodeRef {
        self.root.retained_node_ref()
    }

    fn build_self(&self) {
        self.root.build_self();
    }
}

impl HasFlexBoxRoot for NavLink {
    fn flex_box_root(&self) -> &FlexBox {
        &self.root
    }
}

impl LabeledControlTextStyle for NavLink {
    fn set_label_font_family(&self, family: crate::FontFamily) {
        self.set_explicit_font_family(family);
    }

    fn set_label_font_size(&self, size: f32) {
        self.set_explicit_font_size(size);
    }

    fn set_label_text_color(&self, color: u32) {
        self.set_explicit_text_color(color);
    }
}

#[derive(Clone)]
struct NavLinkEventTarget {
    weak_root: WeakFlexBox,
    label: TextNode,
    href: Rc<RefCell<String>>,
    open_in_new_tab: Rc<Cell<bool>>,
    navigate: Rc<RefCell<Option<NavigateCallback>>>,
    hovered: Rc<Cell<bool>>,
    focused: Rc<Cell<bool>>,
    pointer_pressed: Rc<Cell<bool>>,
    pointer_pressed_open_in_new_tab: Rc<Cell<bool>>,
    enter_pressed: Rc<Cell<bool>>,
    enter_pressed_open_in_new_tab: Rc<Cell<bool>>,
    preview_pinned_for_context_menu: Rc<Cell<bool>>,
    text_color_override: Rc<Cell<Option<u32>>>,
}

impl NavLinkEventTarget {
    fn is_enabled(&self) -> bool {
        self.weak_root
            .upgrade()
            .map(|root| root.retained_node_ref().is_enabled_for_routing())
            .unwrap_or(false)
    }

    fn activate(&self, open_in_new_tab: bool) {
        let href = self.href.borrow().clone();
        navigation::navigate_to(&href, open_in_new_tab);
        if let Some(callback) = self.navigate.borrow().clone() {
            callback(NavigateEventArgs { path: href });
        }
    }

    fn should_open_in_new_tab(&self, modifiers: u32) -> bool {
        self.open_in_new_tab.get() || platform::has_primary_shortcut_modifier(modifiers)
    }

    fn should_handle_enter_key(&self, modifiers: u32) -> bool {
        self.is_enabled() && (modifiers == 0 || platform::has_primary_shortcut_modifier(modifiers))
    }

    fn show_preview(&self) {
        let Some(root) = self.weak_root.upgrade() else {
            return;
        };
        let handle = root.handle().raw();
        if handle == HandleValue::Invalid as u64 {
            return;
        }
        let href = self.href.borrow().clone();
        let bytes = href.as_bytes();
        unsafe {
            crate::ffi::fui_show_url_preview(
                if bytes.is_empty() {
                    0
                } else {
                    bytes.as_ptr() as usize
                },
                bytes.len() as u32,
            )
        };
        ACTIVE_PREVIEW_OWNER.with(|owner| owner.set(handle));
    }

    fn hide_preview(&self) {
        let Some(root) = self.weak_root.upgrade() else {
            return;
        };
        let handle = root.handle().raw();
        if !ACTIVE_PREVIEW_OWNER.with(|owner| owner.get() == handle) {
            return;
        }
        ACTIVE_PREVIEW_OWNER.with(|owner| owner.set(HandleValue::Invalid as u64));
        unsafe { crate::ffi::fui_hide_url_preview() };
    }

    fn pin_preview_for_context_menu(&self) {
        self.preview_pinned_for_context_menu.set(true);
        self.show_preview();
    }

    fn release_preview_for_context_menu(&self) {
        if !self.preview_pinned_for_context_menu.replace(false) {
            return;
        }
        if !self.hovered.get() && !self.focused.get() {
            self.hide_preview();
        }
    }

    fn sync_visual_state(&self) {
        let color = self.text_color_override.get().unwrap_or_else(|| {
            if self.hovered.get() {
                current_theme().colors.accent_hovered
            } else {
                current_theme().colors.accent
            }
        });
        if self.label.handle() != NodeHandle::INVALID {
            ui::set_text_color(self.label.handle().raw(), color);
        }
    }

    fn sync_focus_chrome(&self) {
        let Some(root) = self.weak_root.upgrade() else {
            return;
        };
        if self.focused.get() && self.is_enabled() && focus_visibility::keyboard_focus_visible() {
            focus_adorner::show_standard(&root, current_theme().spacing.sm);
            return;
        }
        focus_adorner::hide_owner(&root);
    }
}
