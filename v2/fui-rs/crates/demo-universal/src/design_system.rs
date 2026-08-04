use fui::prelude::*;
use fui::node::NodeRef;
use std::any::Any;
use std::cell::Cell;
use std::rc::{Rc, Weak};

pub const PAGE_HORIZONTAL_PADDING: f32 = 28.0;
pub const PAGE_TOP_PADDING: f32 = 28.0;
pub const PAGE_BOTTOM_PADDING: f32 = 48.0;
pub const SECTION_GAP: f32 = 18.0;
pub const CONTENT_GAP: f32 = 10.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CardTone {
    #[default]
    Neutral,
    Accent,
    Success,
    Warning,
    Highlight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DemoPalette {
    pub page_background: u32,
    pub card_background: u32,
    pub card_border: u32,
    pub control_background: u32,
    pub selected_background: u32,
    pub title_surface: u32,
    pub title_text: u32,
    pub title_muted_text: u32,
    pub primary_text: u32,
    pub muted_text: u32,
    pub hint_text: u32,
    pub scrollbar_track: u32,
    pub scrollbar_thumb: u32,
}

pub fn demo_palette() -> DemoPalette {
    let theme = current_theme();
    DemoPalette {
        page_background: theme.colors.background,
        card_background: theme.colors.surface,
        card_border: theme.colors.border,
        control_background: theme.colors.surface,
        selected_background: fui::color::mix_color(theme.colors.surface, theme.colors.accent, 0.16),
        title_surface: fui::color::mix_color(theme.colors.surface, theme.colors.accent, 0.12),
        title_text: theme.colors.text_primary,
        title_muted_text: theme.colors.text_muted,
        primary_text: theme.colors.text_primary,
        muted_text: theme.colors.text_muted,
        hint_text: theme.colors.text_muted,
        scrollbar_track: theme.colors.scrollbar_track,
        scrollbar_thumb: theme.colors.scrollbar_thumb,
    }
}

fn card_background(theme: &Theme, tone: CardTone) -> u32 {
    match tone {
        CardTone::Neutral => theme.colors.surface,
        CardTone::Accent => fui::color::mix_color(theme.colors.surface, theme.colors.accent, 0.16),
        CardTone::Success => {
            fui::color::mix_color(theme.colors.surface, fui::color::rgb(34, 197, 94), 0.14)
        }
        CardTone::Warning => {
            fui::color::mix_color(theme.colors.surface, fui::color::rgb(245, 158, 11), 0.16)
        }
        CardTone::Highlight => {
            fui::color::mix_color(theme.colors.surface, theme.colors.accent, 0.24)
        }
    }
}

pub fn page_surface(summary: &str) -> FlexBox {
    ui! {
        column()
            .fill_width()
            .height_len(auto())
            .padding(
                PAGE_HORIZONTAL_PADDING,
                PAGE_TOP_PADDING,
                PAGE_HORIZONTAL_PADDING,
                PAGE_BOTTOM_PADDING,
            )
            .semantic_label(summary)
            .bind_theme(|node, theme| {
                node.bg_color(theme.colors.background);
            })
    }
}

pub fn page_header(title: &str, summary: &str) -> FlexBox {
    ui! {
        column()
            .fill_width()
            .height_len(auto())
            .semantic_label(title) {
                heading_text(title, 28.0),
                vertical_space(8.0),
                body_text(summary),
                vertical_space(SECTION_GAP),
            }
    }
}

pub fn section_card(title: &str, description: &str) -> FlexBox {
    section_card_with_tone(title, description, CardTone::Neutral)
}

pub fn section_card_with_tone(title: &str, description: &str, tone: CardTone) -> FlexBox {
    ui! {
        column()
            .fill_width()
            .height_len(auto())
            .padding(20.0, 20.0, 20.0, 20.0)
            .corner_radius(20.0)
            .semantic_label(title)
            .bind_theme(move |node, theme| {
                node.bg_color(card_background(&theme, tone))
                    .border(1.0, theme.colors.border);
            }) {
                heading_text(title, 20.0),
                muted_text(description).font_size(15.0).text_limits(-1, 4),
                vertical_space(CONTENT_GAP),
            }
    }
}

pub fn heading_text(value: &str, size: f32) -> TextNode {
    ui! {
        text(value)
            .font_size(size)
            .font_weight(FontWeight::Bold)
            .semantic_role(SemanticRole::Heading)
            .bind_theme(|node, theme| {
                node.font_family(theme.fonts.heading_family.clone())
                    .text_color(theme.colors.text_primary);
            })
    }
}

pub fn body_text(value: &str) -> TextNode {
    ui! {
        text(value)
            .font_size(15.0)
            .bind_theme(|node, theme| {
                node.font_family(theme.fonts.body_family.clone())
                    .text_color(theme.colors.text_primary);
            })
    }
}

pub fn muted_text(value: &str) -> TextNode {
    ui! {
        text(value)
            .font_size(14.0)
            .bind_theme(|node, theme| {
                node.font_family(theme.fonts.body_family.clone())
                    .text_color(theme.colors.text_muted);
            })
    }
}

pub fn status_text(value: &str) -> TextNode {
    let node = muted_text(value);
    node.font_weight(FontWeight::Bold)
        .text_limits(-1, 2)
        .semantic_label(value);
    node
}

pub fn hint_text(value: &str) -> TextNode {
    let node = muted_text(value);
    node.font_size(13.0).text_limits(-1, 2);
    node
}

pub fn responsive_section() -> FlexBox {
    ui! {
        row()
            .fill_width()
            .height_len(auto())
            .flex_wrap(FlexWrap::Wrap)
            .align_items(AlignItems::Start)
    }
}

pub fn vertical_space(height: f32) -> FlexBox {
    ui! { flex_box().height(height, Unit::Pixel) }
}

pub fn horizontal_space(width: f32) -> FlexBox {
    ui! { flex_box().width(width, Unit::Pixel) }
}

#[derive(Clone)]
pub struct DemoTabSelector {
    inner: Rc<DemoTabSelectorInner>,
}

struct DemoTabSelectorInner {
    root: FlexBox,
    buttons: Vec<Button>,
    selected_index: Cell<i32>,
}

#[derive(Clone)]
pub struct DemoTabSelectorSync {
    inner: Weak<DemoTabSelectorInner>,
}

impl DemoTabSelectorSync {
    pub fn sync_selected(&self, selected_index: i32) {
        let Some(inner) = self.inner.upgrade() else {
            return;
        };
        inner.sync_selected(selected_index, &current_theme());
    }
}

impl DemoTabSelectorInner {
    fn sync_selected(&self, selected_index: i32, theme: &Theme) {
        self.selected_index.set(selected_index);
        apply_tab_selector_theme(&self.buttons, selected_index, theme);
    }
}

impl DemoTabSelector {
    pub fn new(tabs: &TabView) -> Self {
        let root = ui! {
            row()
                .fill_width()
                .height_len(auto())
                .flex_wrap(FlexWrap::Wrap)
                .align_items(AlignItems::Center)
                .semantic_label("Demo page selector")
        };
        let buttons = (0..tabs.item_count())
            .filter_map(|index| tabs.item_at(index as i32).map(|item| item.label_text()))
            .map(|label| {
                let selector = ui! {
                    button(label.clone())
                        .height(40.0, Unit::Pixel)
                        .padding(16.0, 8.0, 16.0, 8.0)
                        .margin(0.0, 0.0, 8.0, 8.0)
                        .corner_radius(20.0)
                        .semantic_label(label)
                };
                selector
            })
            .collect::<Vec<_>>();
        let selector = Self {
            inner: Rc::new(DemoTabSelectorInner {
                root,
                buttons,
                selected_index: Cell::new(tabs.selected_index()),
            }),
        };
        for (index, button) in selector.inner.buttons.iter().enumerate() {
            button.on_click({
                let tabs = tabs.clone();
                let inner = Rc::downgrade(&selector.inner);
                move |_| {
                    tabs.select_index(index as i32);
                    if let Some(inner) = inner.upgrade() {
                        inner.sync_selected(tabs.selected_index(), &current_theme());
                    }
                }
            });
        }
        for button in &selector.inner.buttons {
            selector.inner.root.child(button);
        }
        selector.inner.root.bind_theme({
            let inner = Rc::downgrade(&selector.inner);
            move |_root, theme| {
                if let Some(inner) = inner.upgrade() {
                    inner.sync_selected(inner.selected_index.get(), &theme);
                }
            }
        });
        selector.sync_selected(tabs.selected_index());
        selector
    }

    pub fn root(&self) -> FlexBox {
        self.inner.root.clone()
    }

    pub fn sync_selected(&self, selected_index: i32) {
        self.inner.sync_selected(selected_index, &current_theme());
    }

    pub fn sync_handle(&self) -> DemoTabSelectorSync {
        DemoTabSelectorSync {
            inner: Rc::downgrade(&self.inner),
        }
    }
}

impl Node for DemoTabSelector {
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

fn apply_tab_selector_theme(buttons: &[Button], selected_index: i32, theme: &Theme) {
    for (index, button) in buttons.iter().enumerate() {
        let selected = index as i32 == selected_index;
        let background = if selected {
            fui::color::mix_color(theme.colors.surface, theme.colors.accent, 0.22)
        } else {
            theme.colors.surface
        };
        button
            .colors(
                ButtonColors::new()
                    .background(background)
                    .background_hover(fui::color::mix_color(
                        background,
                        theme.colors.accent,
                        if selected { 0.14 } else { 0.10 },
                    ))
                    .background_pressed(fui::color::mix_color(
                        background,
                        theme.colors.accent,
                        0.22,
                    ))
                    .text_primary(theme.colors.text_primary)
                    .border(if selected {
                        theme.colors.accent
                    } else {
                        theme.colors.border
                    }),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fui::ffi::{self, Call};

    #[test]
    fn canonical_primitives_follow_live_theme_and_lazy_creation() {
        ffi::test::reset();
        let previous = current_theme();
        let root = page_surface("Design-system test page");
        let card = section_card("Retained card", "Retained body");
        let retained_text = muted_text("Retained text");
        root.child(&card).child(&retained_text);
        Application::mount(root.clone());
        let root_handle = root.handle().raw();
        let card_handle = card.handle().raw();
        let text_handle = retained_text.handle().raw();
        let _ = ffi::test::take_calls();

        let mut changed = current_theme();
        changed.colors.background = 0x102030FF;
        changed.colors.surface = 0x203040FF;
        changed.colors.border = 0x304050FF;
        changed.colors.text_muted = 0x405060FF;
        use_custom_theme(changed.clone());
        let calls = ffi::test::take_calls();

        assert!(calls.iter().any(|call| matches!(
            call,
            Call::SetBgColor { handle, color }
                if *handle == root_handle && *color == changed.colors.background
        )));
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::SetBgColor { handle, color }
                if *handle == card_handle && *color == changed.colors.surface
        )));
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::SetBoxStyle { handle, border_color, .. }
                if *handle == card_handle && *border_color == changed.colors.border
        )));
        assert!(calls.iter().any(|call| matches!(
            call,
            Call::SetTextColor { handle, color }
                if *handle == text_handle && *color == changed.colors.text_muted
        )));

        let lazy_card = section_card("Lazy card", "Created after the theme changed");
        Application::unmount();
        let _ = ffi::test::take_calls();
        Application::mount(lazy_card.clone());
        let lazy_handle = lazy_card.handle().raw();
        let lazy_calls = ffi::test::take_calls();
        assert!(lazy_calls.iter().any(|call| matches!(
            call,
            Call::SetBoxStyle { handle, bg_color, border_color, .. }
                if *handle == lazy_handle
                    && *bg_color == changed.colors.surface
                    && *border_color == changed.colors.border
        )));

        use_custom_theme(previous);
        Application::unmount();
    }
}
