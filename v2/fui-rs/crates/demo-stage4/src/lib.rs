#[cfg(feature = "web-route")]
mod generated;

use fui::controls::{
    ButtonColors, ButtonPresenter, ButtonTemplate, ButtonVisualState,
    CheckboxIndicatorPresenter, CheckboxIndicatorTemplate, CheckboxIndicatorVisualState,
    LabeledControlColors, LabeledControlSizing, PressableIndicatorMetrics,
    PressableIndicatorPresenter, RadioIndicatorPresenter, RadioIndicatorTemplate,
    RadioIndicatorVisualState, SliderColors, SliderSizing,
};
use fui::prelude::*;
use fui_rs_demo_shared::clear_demo_shared_state;
use fui_rs_demo_universal::design_system::{
    demo_palette, hint_text, page_header, page_surface, section_card, status_text, vertical_space,
    DemoTabSelector,
};
use fui_rs_demo_universal::{DemoEnvironment, DemoPageId, UniversalDemoPage};
use std::rc::Rc;

#[derive(Clone)]
struct HouseButtonPresenter {
    content_root: FlexBox,
    label_node: TextNode,
}

impl HouseButtonPresenter {
    fn new() -> Self {
        let label_node = TextNode::new("");
        label_node.selectable(false);
        let content_root = ui! {
            row()
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center) {
                    label_node,
                }
        };
        Self {
            content_root,
            label_node,
        }
    }
}

impl ButtonPresenter for HouseButtonPresenter {
    fn content_root(&self) -> FlexBox {
        self.content_root.clone()
    }

    fn label_node(&self) -> TextNode {
        self.label_node.clone()
    }

    fn present(
        &self,
        theme: &Theme,
        state: &ButtonVisualState,
        colors: Option<&ButtonColors>,
    ) -> PresenterHostStyle {
        let background = if !state.enabled {
            0xCBD5E1FF
        } else if state.pressed {
            0xBE123CFF
        } else if state.hovered {
            0xFB7185FF
        } else {
            colors
                .filter(|colors| colors.has_background())
                .map(|colors| colors.background_color())
                .unwrap_or(0xF43F5EFF)
        };
        let text_color = colors
            .filter(|colors| colors.has_text_primary())
            .map(|colors| colors.text_primary_color())
            .unwrap_or(0xFFFFFFFF);
        self.content_root
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center);
        self.label_node
            .font_family(theme.fonts.body_family.clone())
            .font_weight(FontWeight::Bold)
            .font_size(theme.fonts.size_body + 1.0)
            .text_color(text_color);
        PresenterHostStyle::new()
            .background(background)
            .border(Border::solid(2.0, 0x881337FF))
            .corners(Corners::all(20.0))
            .padding(EdgeInsets::new(18.0, 10.0, 18.0, 10.0))
            .shadow(Shadow::new(0x4C881337, 0.0, 8.0, 18.0, 0.0))
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct HouseButtonTemplate;

impl ButtonTemplate for HouseButtonTemplate {
    fn create(&self) -> Rc<dyn ButtonPresenter> {
        Rc::new(HouseButtonPresenter::new())
    }
}

fn nested_tab_panel(label: &'static str, color: Option<u32>) -> RetainedView {
    let label_node = text(label);
    if let Some(color) = color {
        label_node.text_color(color);
    }
    retained_view(&ui! {
        column().padding(14.0, 14.0, 14.0, 14.0) {
            label_node,
        }
    })
}

fn nested_tabs(
    items: impl IntoIterator<Item = TabItem>,
    semantic_label: &str,
) -> FlexBox {
    let tabs = tab_view();
    tabs.height(116.0, Unit::Pixel).semantic_label(semantic_label);
    for item in items {
        tabs.add_item(item);
    }
    let selector = DemoTabSelector::new(&tabs);
    tabs.on_selection_changed({
        let selector = selector.sync_handle();
        move |event| selector.sync_selected(event.selected_index)
    });
    ui! {
        column().fill_width().height_len(auto()) {
            selector,
            tabs,
        }
    }
}

fn default_nested_tabs() -> FlexBox {
    nested_tabs(
        tab_items![
            tab_item("Inner A").content(|| nested_tab_panel("Default nested content", None)),
            tab_item("Inner B").content(|| nested_tab_panel("State is retained across nested tabs", None)),
        ],
        "Default nested retained content",
    )
}

fn wrapped_nested_tabs() -> FlexBox {
    nested_tabs(
        tab_items![
            tab_item("Compact selector").content(|| nested_tab_panel("Selectors are ordinary demo-owned buttons", None)),
            tab_item("Long selector wraps").content(|| nested_tab_panel("The selector row grows vertically on narrow windows", None)),
        ],
        "Responsive nested retained content",
    )
}

fn composable_nested_tabs() -> FlexBox {
    nested_tabs(
        tab_items![
            tab_item("Pills").content(|| nested_tab_panel("This demo chose pill buttons", None)),
            tab_item("Composable").content(|| nested_tab_panel("Applications may use links, text, buttons, or custom drawing", None)),
        ],
        "Composable nested retained content",
    )
}

#[derive(Clone)]
struct HouseCheckboxIndicatorPresenter {
    root: FlexBox,
    mark: FlexBox,
    indicator_size: f32,
}

impl HouseCheckboxIndicatorPresenter {
    fn new(sizing: Option<LabeledControlSizing>) -> Self {
        let indicator_size = sizing
            .filter(|sizing| sizing.has_indicator_size())
            .map(|sizing| sizing.indicator_size_px())
            .unwrap_or(28.0);
        let mark_size = indicator_size * (12.0 / 28.0);
        let root = flex_box();
        let mark = flex_box();
        root.width(indicator_size, Unit::Pixel)
            .height(indicator_size, Unit::Pixel)
            .corner_radius(indicator_size * (9.0 / 28.0))
            .align_items(AlignItems::Center)
            .justify_content(JustifyContent::Center)
            .child(&mark);
        mark.width(mark_size, Unit::Pixel)
            .height(mark_size, Unit::Pixel);
        Self {
            root,
            mark,
            indicator_size,
        }
    }
}

impl PressableIndicatorPresenter for HouseCheckboxIndicatorPresenter {
    fn root(&self) -> FlexBox {
        self.root.clone()
    }

    fn metrics(&self) -> PressableIndicatorMetrics {
        PressableIndicatorMetrics::new(self.indicator_size, self.indicator_size)
    }
}

impl CheckboxIndicatorPresenter for HouseCheckboxIndicatorPresenter {
    fn apply(
        &self,
        theme: Theme,
        state: CheckboxIndicatorVisualState,
        colors: Option<LabeledControlColors>,
    ) {
        let accent = colors
            .filter(|colors| colors.has_accent())
            .map(|colors| colors.accent_color())
            .unwrap_or(0x0EA5E9FF);
        let checked = state.checked_state == SemanticCheckedState::True;
        let mixed = state.checked_state == SemanticCheckedState::Mixed;
        let background = if checked || mixed {
            accent
        } else if state.hovered {
            0xE0F2FEFF
        } else {
            theme.colors.surface
        };
        self.root
            .bg_color(background)
            .border(2.0, if checked || mixed { accent } else { 0x0369A1FF });
        self.mark
            .corner_radius(if mixed { 2.0 } else { 6.0 })
            .width(if mixed { 16.0 } else { 12.0 }, Unit::Pixel)
            .height(if mixed { 4.0 } else { 12.0 }, Unit::Pixel)
            .bg_color(0xFFFFFFFF)
            .opacity(if checked || mixed { 1.0 } else { 0.0 });
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct HouseCheckboxIndicatorTemplate;

impl CheckboxIndicatorTemplate for HouseCheckboxIndicatorTemplate {
    fn create(&self, sizing: Option<LabeledControlSizing>) -> Rc<dyn CheckboxIndicatorPresenter> {
        Rc::new(HouseCheckboxIndicatorPresenter::new(sizing))
    }
}

#[derive(Clone)]
struct LocalOverrideCheckboxIndicatorPresenter {
    root: FlexBox,
    stripe_node: FlexBox,
    indicator_size: f32,
}

impl LocalOverrideCheckboxIndicatorPresenter {
    fn new(sizing: Option<LabeledControlSizing>) -> Self {
        let indicator_size = sizing
            .filter(|sizing| sizing.has_indicator_size())
            .map(|sizing| sizing.indicator_size_px())
            .unwrap_or(24.0);
        let stripe_node = ui! {
            flex_box()
            .width(14.0, Unit::Pixel)
            .height(10.0, Unit::Pixel)
        };
        let root = ui! {
            flex_box()
                .width(indicator_size, Unit::Pixel)
                .height(indicator_size, Unit::Pixel)
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center) {
                    stripe_node,
                }
        };
        Self {
            root,
            stripe_node,
            indicator_size,
        }
    }
}

impl PressableIndicatorPresenter for LocalOverrideCheckboxIndicatorPresenter {
    fn root(&self) -> FlexBox {
        self.root.clone()
    }

    fn metrics(&self) -> PressableIndicatorMetrics {
        PressableIndicatorMetrics::new(self.indicator_size, self.indicator_size)
    }
}

impl CheckboxIndicatorPresenter for LocalOverrideCheckboxIndicatorPresenter {
    fn apply(
        &self,
        theme: Theme,
        state: CheckboxIndicatorVisualState,
        colors: Option<LabeledControlColors>,
    ) {
        let accent = colors
            .filter(|colors| colors.has_accent())
            .map(|colors| colors.accent_color())
            .unwrap_or_else(|| {
                if state.pressed {
                    theme.colors.accent_pressed
                } else if state.hovered {
                    theme.colors.accent_hovered
                } else {
                    theme.colors.accent
                }
            });
        let checked = state.checked_state != SemanticCheckedState::False;
        let mixed = state.checked_state == SemanticCheckedState::Mixed;
        self.root
            .corner_radius(4.0)
            .border(2.0, accent)
            .bg_color(if checked {
                0xFEF3C7FF
            } else {
                demo_palette().control_background
            });
        self.stripe_node
            .corner_radius(if mixed { 2.0 } else { 5.0 })
            .width(if mixed { 16.0 } else { 10.0 }, Unit::Pixel)
            .height(if mixed { 6.0 } else { 16.0 }, Unit::Pixel)
            .bg_color(if checked { accent } else { 0xD1D5DBFF })
            .opacity(1.0);
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct SquareOverrideCheckboxTemplate;

impl CheckboxIndicatorTemplate for SquareOverrideCheckboxTemplate {
    fn create(&self, sizing: Option<LabeledControlSizing>) -> Rc<dyn CheckboxIndicatorPresenter> {
        Rc::new(LocalOverrideCheckboxIndicatorPresenter::new(sizing))
    }
}

#[derive(Clone)]
struct HouseRadioIndicatorPresenter {
    root: FlexBox,
    dot_node: FlexBox,
    indicator_size: f32,
    dot_size: f32,
}

impl HouseRadioIndicatorPresenter {
    fn new(sizing: Option<LabeledControlSizing>) -> Self {
        let indicator_size = sizing
            .filter(|sizing| sizing.has_indicator_size())
            .map(|sizing| sizing.indicator_size_px())
            .unwrap_or(24.0);
        let dot_size = indicator_size * (10.0 / 24.0);
        let dot_node = ui! {
            flex_box()
            .width(dot_size, Unit::Pixel)
            .height(dot_size, Unit::Pixel)
        };
        let root = ui! {
            flex_box()
                .width(indicator_size, Unit::Pixel)
                .height(indicator_size, Unit::Pixel)
                .align_items(AlignItems::Center)
                .justify_content(JustifyContent::Center) {
                    dot_node,
                }
        };
        Self {
            root,
            dot_node,
            indicator_size,
            dot_size,
        }
    }
}

impl PressableIndicatorPresenter for HouseRadioIndicatorPresenter {
    fn root(&self) -> FlexBox {
        self.root.clone()
    }

    fn metrics(&self) -> PressableIndicatorMetrics {
        PressableIndicatorMetrics::new(self.indicator_size, self.indicator_size)
    }
}

impl RadioIndicatorPresenter for HouseRadioIndicatorPresenter {
    fn apply(
        &self,
        theme: Theme,
        state: RadioIndicatorVisualState,
        colors: Option<LabeledControlColors>,
    ) {
        let accent = colors
            .filter(|colors| colors.has_accent())
            .map(|colors| colors.accent_color())
            .unwrap_or_else(|| {
                if state.pressed {
                    theme.colors.accent_pressed
                } else if state.hovered {
                    theme.colors.accent_hovered
                } else {
                    theme.colors.accent
                }
            });
        let border_color = if state.checked {
            accent
        } else {
            colors
                .filter(|colors| colors.has_border())
                .map(|colors| colors.border_color())
                .unwrap_or(theme.colors.border)
        };
        self.root
            .corner_radius(self.indicator_size * 0.5)
            .border(2.0, border_color)
            .bg_color(
                colors
                    .filter(|colors| colors.has_background())
                    .map(|colors| colors.background_color())
                    .unwrap_or(theme.colors.surface),
            );
        self.dot_node
            .corner_radius(self.dot_size * 0.5)
            .bg_color(accent)
            .opacity(if state.checked { 1.0 } else { 0.0 });
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct HouseRadioIndicatorTemplate;

impl RadioIndicatorTemplate for HouseRadioIndicatorTemplate {
    fn create(&self, sizing: Option<LabeledControlSizing>) -> Rc<dyn RadioIndicatorPresenter> {
        Rc::new(HouseRadioIndicatorPresenter::new(sizing))
    }
}

struct Stage4PresentationShowcase {
    root: ScrollBox,
    _house_button: Button,
    _color_button: Button,
    _house_checkbox: Checkbox,
    _override_checkbox: Checkbox,
    _radio_alpha: RadioButton,
    _radio_beta: RadioButton,
    _switch: Switch,
    _slider: Slider,
    _large_slider: Slider,
    _tabs: TabView,
    _override_status: TextNode,
    _radio_status: TextNode,
    _switch_status: TextNode,
    _slider_status: TextNode,
    _dropdown_field_root: FlexBox,
    _dropdown_option_root: FlexBox,
}

impl Stage4PresentationShowcase {
    fn new() -> Self {
        let page = page_surface("FUI-RS Stage 4 presentation page");

        let root = ui! {
            scroll_box()
                .fill_size()
                .scrollbar_gutter(0.0)
                .persist_scroll(false) {
                    page,
                }
        };
        root.vertical_scrollbar()
            .track_width(12.0)
            .thumb_width(8.0)
            .thumb_min_height(36.0);
        root.vertical_scrollbar()
            .render()
            .semantic_label("Stage 4 presentation vertical scrollbar");

        page.children(children![title_block(), vertical_space(18.0),]);

        let house_button = ui! {
        button("House template button")
            .template(Rc::new(HouseButtonTemplate))
            .semantic_label("Stage 4 house template button")
            .node_id("stage4-template-house-button")
        };
        let house_checkbox = ui! {
        checkbox("House template checkbox")
            .template(Rc::new(HouseCheckboxIndicatorTemplate))
            .colors(stage4_labeled_colors(0x0EA5E9FF))
            .checked(true)
            .semantic_label("Stage 4 house template checkbox")
            .node_id("stage4-template-house-checkbox")
        };
        page.children(children![
            ui! {
                showcase_card(
                    "Design-system templates",
                    "Button and Checkbox receive their house templates explicitly from the demo design system.",
                    "Stage 4 design-system template card",
                ) {
                    house_button,
                    vertical_space(10.0),
                    house_checkbox,
                    hint_text("Expected: rose button chrome and blue rounded checkbox indicator come from explicit design-system templates."),
                }
            },
            vertical_space(18.0),
        ]);

        let override_checkbox = ui! {
        checkbox("Local override checkbox")
            .template(Rc::new(SquareOverrideCheckboxTemplate))
            .sizing(
                LabeledControlSizing::new()
                    .indicator_size(34.0)
                    .label_font_size(18.0),
            )
            .colors(stage4_labeled_colors(0xF59E0BFF).border(0x92400EFF))
            .checked(true)
            .semantic_label("Stage 4 local override checkbox")
            .node_id("stage4-template-local-checkbox")
        };
        let override_status = ui! {
        status_text("Local override checkbox: on").semantic_label("Local override checkbox: on")
        };
        override_checkbox.on_changed({
            let override_status = override_status.clone();
            move |event| {
                let value = if event.checked { "on" } else { "off" };
                let label = format!("Local override checkbox: {value}");
                override_status.text(&label).semantic_label(label);
            }
        });
        page.children(children![
            ui! {
                showcase_card(
                    "Alternative local template",
                    "This checkbox supplies a different local template and remains visually distinct from the house checkbox.",
                    "Stage 4 local template override card",
                ) {
                    override_checkbox,
                    vertical_space(8.0),
                    override_status,
                    hint_text(
                        "Click it: this one should toggle independently and keep the square amber template with an accent stripe, proving explicit local template ownership.",
                    ),
                }
            },
            vertical_space(18.0),
        ]);

        let sizing_card = showcase_card(
            "Control sizing tokens",
            "LabeledControlSizing and SliderSizing alter presenter metrics without changing interaction semantics.",
            "Stage 4 control sizing card",
        );
        let radio_group = ui! {
        radio_group().semantic_label("Stage 4 sizing radio group")
        };
        let radio_alpha = radio_button("Compact radio sizing");
        let radio_beta = radio_button("Large radio sizing");
        radio_alpha
            .template(Rc::new(HouseRadioIndicatorTemplate))
            .sizing(
                LabeledControlSizing::new()
                    .indicator_size(16.0)
                    .label_font_size(14.0),
            )
            .colors(stage4_labeled_colors(0x2563EBFF))
            .checked(true)
            .semantic_label("Stage 4 compact radio sizing");
        radio_beta
            .template(Rc::new(HouseRadioIndicatorTemplate))
            .sizing(
                LabeledControlSizing::new()
                    .indicator_size(30.0)
                    .label_font_size(20.0),
            )
            .colors(stage4_labeled_colors(0x2563EBFF))
            .semantic_label("Stage 4 large radio sizing");
        let radio_status = ui! {
        status_text("Radio sizing selected: compact").semantic_label("Radio sizing selected: compact")
        };
        radio_group
            .add_radio(radio_alpha.clone())
            .add_radio(radio_beta.clone())
            .select_index(0)
            .on_changed({
                let radio_status = radio_status.clone();
                move |event| {
                    let value = if event.value == "Large radio sizing" {
                        "large"
                    } else {
                        "compact"
                    };
                    let label = format!("Radio sizing selected: {value}");
                    radio_status.text(&label).semantic_label(label);
                }
            });
        let switch_control = ui! {
        switch("Large switch colors")
            .colors(
                stage4_labeled_colors(0x16A34AFF)
                    .background(demo_palette().control_background)
                    .border(0x15803DFF),
            )
            .checked(true)
            .semantic_label("Stage 4 switch color presenter")
        };
        let switch_status = ui! {
        status_text("Switch presenter state: on").semantic_label("Switch presenter state: on")
        };
        switch_control.on_changed({
            let switch_status = switch_status.clone();
            move |event| {
                let value = if event.checked { "on" } else { "off" };
                let label = format!("Switch presenter state: {value}");
                switch_status.text(&label).semantic_label(label);
            }
        });
        let slider_control = ui! {
            slider()
            .length(320.0)
            .value(42.0)
            .sizing(
                SliderSizing::new().thumb_size(30.0).track_thickness(10.0),
            )
            .colors(
                SliderColors::new()
                    .track(0xE2E8F0FF)
                    .fill(0x2563EBFF)
                    .thumb(0x1D4ED8FF),
            )
            .semantic_label("Stage 4 slider sizing presenter")
        };
        let slider_status = ui! {
        status_text("Slider sizing value: 42").semantic_label("Slider sizing value: 42")
        };
        slider_control.on_changed({
            let slider_status = slider_status.clone();
            move |event| {
                let label = format!("Slider sizing value: {:.0}", event.value);
                slider_status.text(&label).semantic_label(label);
            }
        });
        let sizing_card = ui! {
            sizing_card {
                hint_text("The circles below use the house RadioButton template with LabeledControlSizing: compact uses a 16px indicator, large uses a 30px indicator."),
                vertical_space(8.0),
                radio_alpha,
                vertical_space(8.0),
                radio_beta,
                vertical_space(8.0),
                radio_status,
                vertical_space(12.0),
                switch_control,
                vertical_space(8.0),
                switch_status,
                vertical_space(14.0),
                slider_control,
                vertical_space(8.0),
                slider_status,
                hint_text("Drag or click these controls: status text should update while the presenter-owned sizing and colors stay intact."),
            }
        };
        page.children(children![sizing_card, vertical_space(18.0)]);

        let colors_card = showcase_card(
            "Presenter color overrides",
            "Color value objects tint presenter-owned chrome while controls retain built-in semantics and callbacks.",
            "Stage 4 presenter color override card",
        );
        let color_button = ui! {
            button("Color override button")
            .colors(
                ButtonColors::new()
                    .background(0x0F766EFF)
                    .background_hover(0x14B8A6FF)
                    .background_pressed(0x115E59FF)
                    .text_primary(0xFFFFFFFF)
                    .border(0x134E4AFF),
            )
            .semantic_label("Stage 4 color override button")
            .node_id("stage4-template-color-button")
        };
        let large_slider = ui! {
            slider()
            .length(360.0)
            .min(0.0)
            .max(100.0)
            .value(72.0)
            .sizing(
                SliderSizing::new().thumb_size(24.0).track_thickness(8.0),
            )
            .colors(
                SliderColors::new()
                    .track(0xFDE68AFF)
                    .fill(0xF97316FF)
                    .thumb(0x9A3412FF),
            )
            .semantic_label("Stage 4 slider color override presenter")
        };
        let colors_card = ui! {
            colors_card {
                color_button,
                vertical_space(14.0),
                large_slider,
                hint_text("Expected: color overrides flow through the same presenters, not through custom event behavior."),
            }
        };
        page.children(children![colors_card, vertical_space(18.0)]);

        let tab_status = status_text("Selected tab: Overview");
        let tabs = tab_view();
        tabs.height(330.0, Unit::Pixel)
            .items(tab_items![
                tab_item("Overview").content(|| {
                    retained_view(&ui! {
                        column()
                            .fill_size()
                            .padding(16.0, 16.0, 16.0, 16.0) {
                                text("Default presentation")
                                    .font_size(18.0)
                                    .font_weight(FontWeight::Bold)
                                    .text_color(demo_palette().primary_text),
                                vertical_space(8.0),
                                text("The outer and inner controls both use the active application theme.")
                                    .font_size(15.0)
                                    .text_color(demo_palette().muted_text)
                                    .text_limits(-1, 3),
                                vertical_space(10.0),
                                default_nested_tabs(),
                            }
                    })
                }),
                tab_item("Settings").content(|| {
                    retained_view(&ui! {
                        column()
                            .fill_size()
                            .padding(16.0, 16.0, 16.0, 16.0) {
                                text("Responsive selector composition").font_weight(FontWeight::Bold),
                                vertical_space(10.0),
                                wrapped_nested_tabs(),
                            }
                    })
                }),
                tab_item("About").content(|| {
                    retained_view(&ui! {
                        column()
                            .fill_size()
                            .padding(16.0, 16.0, 16.0, 16.0) {
                                text("Application-owned selector chrome").font_weight(FontWeight::Bold),
                                vertical_space(10.0),
                                composable_nested_tabs(),
                            }
                    })
                }),
            ]);
        let tab_selector = DemoTabSelector::new(&tabs);
        tabs.on_selection_changed({
                let tab_selector = tab_selector.sync_handle();
                let tab_status = tab_status.clone();
                move |event| {
                    tab_selector.sync_selected(event.selected_index);
                    let label = event
                        .selected_item
                        .map(|item| item.label_text())
                        .unwrap_or_else(|| "None".to_string());
                    let value = format!("Selected tab: {label}");
                    tab_status.text(&value).semantic_label(value);
                }
            });
        page.children(children![
            ui! {
                showcase_card(
                    "Retained TabView",
                    "TabView retains and activates content while ordinary application controls own selector visuals, input, and semantics.",
                    "Stage 4 retained TabView card",
                ) {
                    tab_selector.clone(),
                    vertical_space(4.0),
                    tabs,
                    vertical_space(10.0),
                    tab_status,
                }
            },
            vertical_space(18.0),
        ]);

        let dropdown_card = showcase_card(
            "Dropdown presenter contracts",
            "Stage 4 exposes dropdown presenter contracts before the editable Dropdown/ComboBox controls move to Stage 5.",
            "Stage 4 dropdown presenter contract card",
        );
        let dropdown_sizing = DropdownSizing::new()
            .field_height(42.0)
            .field_font_size(17.0)
            .chevron_box_size(42.0)
            .chevron_icon_size(18.0)
            .option_height(38.0)
            .option_font_size(16.0);
        let dropdown_field = create_default_dropdown_field_presenter(Some(dropdown_sizing));
        dropdown_field.value_node().text("Presenter field preview");
        dropdown_field.apply(
            current_theme(),
            &DropdownFieldVisualState::new(false, false, true, false, "Presenter field preview"),
            Some(
                DropdownColors::new()
                    .background(demo_palette().control_background)
                    .border(0x0284C7FF)
                    .accent(0x0284C7FF)
                    .text_primary(demo_palette().primary_text),
            ),
        );
        let dropdown_option = create_default_dropdown_option_row_presenter(Some(dropdown_sizing));
        dropdown_option
            .label_node()
            .text("Option row presenter preview");
        dropdown_option.apply(
            current_theme(),
            DropdownOptionRowVisualState::new(true, true, true),
            Some(
                DropdownColors::new()
                    .background(demo_palette().selected_background)
                    .accent(0x0284C7FF)
                    .text_primary(demo_palette().primary_text),
            ),
        );
        let dropdown_field_root = dropdown_field.root();
        let dropdown_option_root = dropdown_option.root();
        dropdown_field_root.semantic_label("Stage 4 dropdown field presenter preview");
        dropdown_option_root.semantic_label("Stage 4 dropdown option row presenter preview");
        dropdown_card
            .child(&dropdown_field_root)
            .child(&vertical_space(10.0))
            .child(&dropdown_option_root)
            .child(&hint_text("Expected: presenter contracts can be created and styled independently of the future Dropdown control."));
        page.child(&dropdown_card);

        Self {
            root,
            _house_button: house_button,
            _color_button: color_button,
            _house_checkbox: house_checkbox,
            _override_checkbox: override_checkbox,
            _radio_alpha: radio_alpha,
            _radio_beta: radio_beta,
            _switch: switch_control,
            _slider: slider_control,
            _large_slider: large_slider,
            _tabs: tabs,
            _override_status: override_status,
            _radio_status: radio_status,
            _switch_status: switch_status,
            _slider_status: slider_status,
            _dropdown_field_root: dropdown_field_root,
            _dropdown_option_root: dropdown_option_root,
        }
    }
}

fn stage4_labeled_colors(accent: u32) -> LabeledControlColors {
    let palette = demo_palette();
    LabeledControlColors::new()
        .accent(accent)
        .background(palette.control_background)
        .border(palette.card_border)
        .text_primary(palette.primary_text)
        .text_muted(palette.muted_text)
}

fn title_block() -> FlexBox {
    page_header(
        "FUI-RS Stage 4 presentation verification",
        "Universal showcase for control sizing, explicit design-system templates, distinct per-control overrides, presenter color overrides, and dropdown presenter contracts.",
    )
}

fn showcase_card(title: &str, description: &str, semantic_label: &str) -> FlexBox {
    let card = section_card(title, description);
    card.semantic_label(semantic_label);
    card
}

#[cfg(feature = "web-route")]
fn dispose_stage4_page(page: &fui_rs_demo_shared::RoutedDemoShell<UniversalDemoPage>) {
    page.content().view().dispose();
}

fn build_stage4_page() -> Stage4PresentationShowcase {
    Stage4PresentationShowcase::new()
}

pub fn build_universal_page(_environment: &DemoEnvironment) -> UniversalDemoPage {
    let page = build_stage4_page();
    let root = page.root.clone();
    UniversalDemoPage::new(
        DemoPageId::BasicControls.metadata(),
        retained_view(&root)
            .keep_alive(page)
            .on_dispose(clear_demo_shared_state),
    )
}

#[cfg(feature = "web-route")]
fn web_environment() -> DemoEnvironment {
    DemoEnvironment::browser(
        fui::platform::platform_family(),
        fui_rs_demo_universal::DemoLinks::new(
            "https://github.com/zion-sati/fui-rs",
            "https://docs.rs/fui-rs/latest/fui",
        ),
    )
}

#[cfg(feature = "web-route")]
fn build_web_page() -> fui_rs_demo_shared::RoutedDemoShell<UniversalDemoPage> {
    Application::caption(DemoPageId::BasicControls.metadata().title);
    let page = build_universal_page(&web_environment());
    let root = page.view().root();
    fui_rs_demo_shared::routed_demo_shell(page, root, DemoPageId::BasicControls)
}

#[cfg(feature = "web-route")]
fui_managed_app!(
    fui_rs_demo_shared::RoutedDemoShell<UniversalDemoPage>,
    build_web_page,
    |page: &fui_rs_demo_shared::RoutedDemoShell<UniversalDemoPage>| page.root.clone(),
    dispose: dispose_stage4_page
);

#[cfg(test)]
mod tests {
    use super::*;
    use fui::ffi::{self, Call};

    #[test]
    fn house_button_typography_does_not_change_on_hover() {
        ffi::test::reset();
        let presenter = HouseButtonPresenter::new();
        presenter.present(
            &current_theme(),
            &ButtonVisualState {
                enabled: true,
                ..Default::default()
            },
            None,
        );
        Application::mount(presenter.content_root());
        let initial_calls = ffi::test::take_calls();
        assert!(initial_calls.iter().any(|call| matches!(
            call,
            Call::SetFont { font_id: 2, size, .. }
                if (*size - (current_theme().fonts.size_body + 1.0)).abs() < f32::EPSILON
        )));

        presenter.present(
            &current_theme(),
            &ButtonVisualState {
                hovered: true,
                enabled: true,
                ..Default::default()
            },
            None,
        );
        let hovered_calls = ffi::test::take_calls();
        assert!(hovered_calls.iter().any(|call| matches!(
            call,
            Call::SetFont { font_id: 2, size, .. }
                if (*size - (current_theme().fonts.size_body + 1.0)).abs() < f32::EPSILON
        )));
        Application::unmount();
    }
}
