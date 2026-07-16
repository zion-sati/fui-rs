use crate::controls::LabeledControlColors;
use crate::event::{FocusChangedEventArgs, KeyEventArgs, PointerEventArgs};
use crate::ffi::{
    AlignItems, CursorStyle, FlexDirection, KeyEventType, PointerEventType, SemanticRole, Unit,
};
use crate::node::{flex_box, FlexBox, Node, TextCore, WeakFlexBox};
use crate::theme::{current_theme, subscribe};
use crate::{focus_adorner, focus_visibility};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

const TRANSPARENT: u32 = 0x00000000;

type ActivationCallback = Rc<dyn Fn(PressableLabeledControlState)>;
type StateCallback = Rc<dyn Fn(PressableLabeledControlState)>;
type KeyCallback = Rc<dyn Fn(&mut KeyEventArgs) -> bool>;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct PressableLabeledControlState {
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
    pub enabled: bool,
}

#[derive(Clone)]
pub(crate) struct PressableLabeledControl {
    root: FlexBox,
    indicator_root: Rc<RefCell<FlexBox>>,
    label_node: TextCore,
    gap_node: FlexBox,
    label_host: FlexBox,
    hovered_state: Rc<Cell<bool>>,
    pressed_state: Rc<Cell<bool>>,
    focused_state: Rc<Cell<bool>>,
    key_pressed_state: Rc<Cell<bool>>,
    pointer_pressed_state: Rc<Cell<bool>>,
    enabled_state: Rc<Cell<bool>>,
    label_font_family_override: Rc<RefCell<Option<crate::FontFamily>>>,
    label_font_size_override: Rc<Cell<f32>>,
    label_text_color_override: Rc<Cell<Option<u32>>>,
    colors_value: Rc<Cell<Option<LabeledControlColors>>>,
    activation: Rc<RefCell<Option<ActivationCallback>>>,
    state_changed: Rc<RefCell<Option<StateCallback>>>,
    key_handler: Rc<RefCell<Option<KeyCallback>>>,
}

impl PressableLabeledControl {
    pub fn new(role: SemanticRole, label: impl Into<String>, indicator_root: FlexBox) -> Self {
        let label = label.into();
        let root = flex_box();
        let label_node = TextCore::new(&label);
        let gap_node = flex_box();
        let label_host = flex_box();
        label_host.child(&label_node);
        root.flex_direction(FlexDirection::Row)
            .align_items(AlignItems::Center)
            .interactive(true)
            .focusable(true, 0)
            .reflect_semantic_disabled_from_enabled()
            .cursor(CursorStyle::Pointer)
            .semantic_role(role)
            .semantic_label(label)
            .child(&indicator_root)
            .child(&gap_node)
            .child(&label_host);

        let control = Self {
            root,
            indicator_root: Rc::new(RefCell::new(indicator_root)),
            label_node,
            gap_node,
            label_host,
            hovered_state: Rc::new(Cell::new(false)),
            pressed_state: Rc::new(Cell::new(false)),
            focused_state: Rc::new(Cell::new(false)),
            key_pressed_state: Rc::new(Cell::new(false)),
            pointer_pressed_state: Rc::new(Cell::new(false)),
            enabled_state: Rc::new(Cell::new(true)),
            label_font_family_override: Rc::new(RefCell::new(None)),
            label_font_size_override: Rc::new(Cell::new(0.0)),
            label_text_color_override: Rc::new(Cell::new(None)),
            colors_value: Rc::new(Cell::new(None)),
            activation: Rc::new(RefCell::new(None)),
            state_changed: Rc::new(RefCell::new(None)),
            key_handler: Rc::new(RefCell::new(None)),
        };
        control.install_handlers();
        control.install_theme_subscription();
        control.sync_base_theme();
        control.sync_focus_chrome();
        control
    }

    pub(crate) fn root(&self) -> FlexBox {
        self.root.clone()
    }

    #[cfg(test)]
    pub(crate) fn test_parts(&self) -> (FlexBox, FlexBox, FlexBox, FlexBox) {
        (
            self.root.clone(),
            self.indicator_root.borrow().clone(),
            self.gap_node.clone(),
            self.label_host.clone(),
        )
    }

    pub(crate) fn set_activation(&self, callback: impl Fn(PressableLabeledControlState) + 'static) {
        *self.activation.borrow_mut() = Some(Rc::new(callback));
    }

    pub(crate) fn set_state_changed(
        &self,
        callback: impl Fn(PressableLabeledControlState) + 'static,
    ) {
        *self.state_changed.borrow_mut() = Some(Rc::new(callback));
    }

    pub(crate) fn set_key_handler(&self, callback: impl Fn(&mut KeyEventArgs) -> bool + 'static) {
        *self.key_handler.borrow_mut() = Some(Rc::new(callback));
    }

    pub(crate) fn colors(&self, colors: Option<LabeledControlColors>) {
        self.colors_value.set(colors);
        self.sync_base_theme();
        self.event_target().notify_state_changed();
    }

    pub(crate) fn set_label_font_size_override(&self, font_size: f32) {
        self.label_font_size_override
            .set(if font_size > 0.0 { font_size } else { 0.0 });
        self.sync_base_theme();
    }

    pub(crate) fn font_family(&self, family: crate::FontFamily) {
        self.label_font_family_override.replace(Some(family));
        self.sync_base_theme();
    }

    pub(crate) fn font_size(&self, size: f32) {
        self.label_font_size_override.set(size);
        self.sync_base_theme();
    }

    pub(crate) fn text_color(&self, color: u32) {
        self.label_text_color_override.set(Some(color));
        self.sync_base_theme();
    }

    pub(crate) fn replace_indicator_root(&self, next_root: FlexBox) {
        if next_root
            .retained_node_ref()
            .ptr_eq(&self.indicator_root.borrow().retained_node_ref())
        {
            return;
        }
        let previous_root = self.indicator_root.borrow().clone();
        self.root.remove_child(&previous_root);
        self.root.remove_child(&self.gap_node);
        self.root.remove_child(&self.label_host);
        self.root
            .child(&next_root)
            .child(&self.gap_node)
            .child(&self.label_host);
        previous_root.dispose();
        self.indicator_root.replace(next_root);
    }

    pub(crate) fn enabled(&self, enabled: bool) {
        let previous = self.enabled_state.get();
        self.root.enabled(enabled);
        let effective = self.root.retained_node_ref().is_enabled_for_routing();
        if previous == effective {
            return;
        }
        self.enabled_state.set(effective);
        if !effective {
            self.hovered_state.set(false);
            self.pointer_pressed_state.set(false);
            self.key_pressed_state.set(false);
            self.pressed_state.set(false);
        }
        self.sync_base_theme();
        self.sync_focus_chrome();
        self.event_target().notify_state_changed();
    }

    pub(crate) fn snapshot_state(&self) -> PressableLabeledControlState {
        PressableLabeledControlState {
            hovered: self.hovered_state.get(),
            pressed: self.pressed_state.get(),
            focused: self.focused_state.get(),
            enabled: self.enabled_state.get(),
        }
    }

    pub(crate) fn state_snapshotter(&self) -> Rc<dyn Fn() -> PressableLabeledControlState> {
        let hovered_state = self.hovered_state.clone();
        let pressed_state = self.pressed_state.clone();
        let focused_state = self.focused_state.clone();
        let enabled_state = self.enabled_state.clone();
        Rc::new(move || PressableLabeledControlState {
            hovered: hovered_state.get(),
            pressed: pressed_state.get(),
            focused: focused_state.get(),
            enabled: enabled_state.get(),
        })
    }

    fn install_handlers(&self) {
        let target = self.event_target();
        self.root.on_pointer_enter(move |event| {
            target.handle_pointer_event(event);
        });
        let target = self.event_target();
        self.root.on_pointer_leave(move |event| {
            target.handle_pointer_event(event);
        });
        let target = self.event_target();
        self.root.on_pointer_down(move |event| {
            target.handle_pointer_event(event);
        });
        let target = self.event_target();
        self.root.on_pointer_up(move |event| {
            target.handle_pointer_event(event);
        });
        let target = self.event_target();
        self.root.on_key_down(move |event| {
            target.handle_key_event(event);
        });
        let target = self.event_target();
        self.root.on_key_up(move |event| {
            target.handle_key_event(event);
        });
        let target = self.event_target();
        self.root.on_focus_changed(move |event| {
            target.handle_focus_changed(event);
        });
    }

    fn install_theme_subscription(&self) {
        let target = self.event_target();
        let theme_guard = subscribe(move |_theme| {
            target.sync_base_theme();
            target.notify_state_changed();
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

    fn event_target(&self) -> PressableLabeledControlEventTarget {
        PressableLabeledControlEventTarget {
            weak_root: self.root.downgrade(),
            label_node: self.label_node.clone(),
            gap_node: self.gap_node.clone(),
            hovered_state: self.hovered_state.clone(),
            pressed_state: self.pressed_state.clone(),
            focused_state: self.focused_state.clone(),
            key_pressed_state: self.key_pressed_state.clone(),
            pointer_pressed_state: self.pointer_pressed_state.clone(),
            enabled_state: self.enabled_state.clone(),
            label_font_family_override: self.label_font_family_override.clone(),
            label_font_size_override: self.label_font_size_override.clone(),
            label_text_color_override: self.label_text_color_override.clone(),
            colors_value: self.colors_value.clone(),
            activation: self.activation.clone(),
            state_changed: self.state_changed.clone(),
            key_handler: self.key_handler.clone(),
        }
    }

    fn sync_base_theme(&self) {
        sync_base_theme_parts(
            &self.root,
            &self.label_node,
            &self.gap_node,
            self.label_font_size_override.get(),
            self.label_font_family_override.borrow().clone(),
            self.label_text_color_override.get(),
            self.colors_value.get(),
            self.enabled_state.get(),
        );
    }

    fn sync_focus_chrome(&self) {
        sync_focus_chrome_parts(
            &self.root,
            self.focused_state.get(),
            self.enabled_state.get(),
        );
    }
}

fn sync_base_theme_parts(
    root: &FlexBox,
    label_node: &TextCore,
    gap_node: &FlexBox,
    label_font_size_override: f32,
    label_font_family_override: Option<crate::FontFamily>,
    label_text_color_override: Option<u32>,
    colors: Option<LabeledControlColors>,
    enabled: bool,
) {
    let theme = current_theme();
    root.cursor(if enabled {
        CursorStyle::Pointer
    } else {
        CursorStyle::Default
    });
    root.corner_radius(theme.spacing.sm);
    root.border(2.0, TRANSPARENT);
    root.padding(
        theme.spacing.xs,
        theme.spacing.xs,
        theme.spacing.xs,
        theme.spacing.xs,
    );
    root.opacity(if enabled { 1.0 } else { 0.6 });
    gap_node.width(theme.spacing.sm, Unit::Pixel);
    label_node
        .font_family(label_font_family_override.unwrap_or_else(|| theme.fonts.body_family.clone()))
        .font_size(if label_font_size_override > 0.0 {
            label_font_size_override
        } else {
            theme.fonts.size_body
        });
    let label_color = if enabled {
        colors
            .filter(|colors| colors.has_text_primary())
            .map(|colors| colors.text_primary_color())
            .unwrap_or(theme.colors.text_primary)
    } else {
        colors
            .filter(|colors| colors.has_text_muted())
            .map(|colors| colors.text_muted_color())
            .unwrap_or(theme.colors.text_muted)
    };
    label_node.text_color(label_text_color_override.unwrap_or(label_color));
}

fn sync_focus_chrome_parts(root: &FlexBox, focused: bool, enabled: bool) {
    if focused && enabled && focus_visibility::keyboard_focus_visible() {
        focus_adorner::show_standard(root, current_theme().spacing.sm);
        return;
    }
    focus_adorner::hide_owner(root);
}

fn is_space_key(event: &KeyEventArgs) -> bool {
    event.key == " " || event.key == "Space" || event.key == "Spacebar"
}

#[derive(Clone)]
struct PressableLabeledControlEventTarget {
    weak_root: WeakFlexBox,
    label_node: TextCore,
    gap_node: FlexBox,
    hovered_state: Rc<Cell<bool>>,
    pressed_state: Rc<Cell<bool>>,
    focused_state: Rc<Cell<bool>>,
    key_pressed_state: Rc<Cell<bool>>,
    pointer_pressed_state: Rc<Cell<bool>>,
    enabled_state: Rc<Cell<bool>>,
    label_font_family_override: Rc<RefCell<Option<crate::FontFamily>>>,
    label_font_size_override: Rc<Cell<f32>>,
    label_text_color_override: Rc<Cell<Option<u32>>>,
    colors_value: Rc<Cell<Option<LabeledControlColors>>>,
    activation: Rc<RefCell<Option<ActivationCallback>>>,
    state_changed: Rc<RefCell<Option<StateCallback>>>,
    key_handler: Rc<RefCell<Option<KeyCallback>>>,
}

impl PressableLabeledControlEventTarget {
    fn snapshot_state(&self) -> PressableLabeledControlState {
        PressableLabeledControlState {
            hovered: self.hovered_state.get(),
            pressed: self.pressed_state.get(),
            focused: self.focused_state.get(),
            enabled: self.enabled_state.get(),
        }
    }

    fn sync_base_theme(&self) {
        let Some(root) = self.weak_root.upgrade() else {
            return;
        };
        sync_base_theme_parts(
            &root,
            &self.label_node,
            &self.gap_node,
            self.label_font_size_override.get(),
            self.label_font_family_override.borrow().clone(),
            self.label_text_color_override.get(),
            self.colors_value.get(),
            self.enabled_state.get(),
        );
    }

    fn sync_focus_chrome(&self) {
        let Some(root) = self.weak_root.upgrade() else {
            return;
        };
        sync_focus_chrome_parts(&root, self.focused_state.get(), self.enabled_state.get());
    }

    fn activate(&self) {
        if let Some(callback) = self.activation.borrow().clone() {
            callback(self.snapshot_state());
        }
    }

    fn clear_pointer_state(&self) {
        self.pointer_pressed_state.set(false);
        self.pressed_state.set(false);
        self.notify_state_changed();
        self.sync_focus_chrome();
    }

    fn notify_state_changed(&self) {
        if let Some(callback) = self.state_changed.borrow().clone() {
            callback(self.snapshot_state());
        }
    }

    fn handle_pointer_event(&self, event: &mut PointerEventArgs) {
        if !self.enabled_state.get() {
            self.clear_pointer_state();
            return;
        }
        match event.event_type {
            PointerEventType::Enter => {
                self.hovered_state.set(true);
                self.notify_state_changed();
                self.sync_focus_chrome();
            }
            PointerEventType::Leave => {
                self.hovered_state.set(false);
                self.clear_pointer_state();
            }
            PointerEventType::Down => {
                self.pointer_pressed_state.set(true);
                self.pressed_state.set(true);
                self.notify_state_changed();
                self.sync_focus_chrome();
            }
            PointerEventType::Up => {
                if self.pointer_pressed_state.get() {
                    self.pointer_pressed_state.set(false);
                    self.pressed_state.set(false);
                    self.notify_state_changed();
                    self.sync_focus_chrome();
                    self.activate();
                }
            }
            PointerEventType::Move | PointerEventType::Cancel => {}
        }
    }

    fn handle_key_event(&self, event: &mut KeyEventArgs) -> bool {
        if event.event_type == KeyEventType::Down {
            self.sync_focus_chrome();
        }
        if !self.enabled_state.get() {
            return false;
        }
        if is_space_key(event) && event.modifiers == 0 {
            return match event.event_type {
                KeyEventType::Down => {
                    self.key_pressed_state.set(true);
                    self.pressed_state.set(true);
                    self.notify_state_changed();
                    event.handled = true;
                    true
                }
                KeyEventType::Up => {
                    if self.key_pressed_state.get() {
                        self.key_pressed_state.set(false);
                        self.pressed_state.set(false);
                        self.notify_state_changed();
                        self.activate();
                        event.handled = true;
                        true
                    } else {
                        false
                    }
                }
            };
        }
        if let Some(callback) = self.key_handler.borrow().clone() {
            if callback(event) {
                event.handled = true;
                return true;
            }
        }
        false
    }

    fn handle_focus_changed(&self, event: FocusChangedEventArgs) {
        if self.focused_state.get() == event.focused {
            return;
        }
        self.focused_state.set(event.focused);
        if !event.focused {
            self.key_pressed_state.set(false);
            self.pressed_state.set(false);
        }
        self.sync_focus_chrome();
        self.notify_state_changed();
    }
}
