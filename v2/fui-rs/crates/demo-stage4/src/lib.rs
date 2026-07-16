mod generated;

use fui::controls::{
    clear_control_templates, use_control_templates, ButtonColors, ButtonPresenter, ButtonTemplate,
    ButtonVisualState, CheckboxIndicatorPresenter, CheckboxIndicatorTemplate,
    CheckboxIndicatorVisualState, ControlTemplateSet, LabeledControlColors, LabeledControlSizing,
    PressableIndicatorMetrics, PressableIndicatorPresenter, RadioIndicatorPresenter,
    RadioIndicatorTemplate, RadioIndicatorVisualState, SliderColors, SliderSizing,
    DEFAULT_SLIDER_TEMPLATE,
};
use fui::prelude::*;
use fui_rs_demo_shared::clear_demo_shared_state;
use std::rc::Rc;

const SOURCE_DEMO_BASE: &str = "/v2/fui-rs/demo";
const SOURCE_HOME_ROUTE: &str = "/v2/fui-rs/demo/index.html";
const SOURCE_WORKBENCH_ROUTE: &str = "/v2/fui-rs/demo/workbench/";
const SOURCE_STAGE4_ROUTE: &str = "/v2/fui-rs/demo/stage4/";
const PUBLISHED_HOME_ROUTE: &str = "/";
const PUBLISHED_WORKBENCH_ROUTE: &str = "/workbench/";
const PUBLISHED_STAGE4_ROUTE: &str = "/stage4/";

#[derive(Clone)]
struct HouseButtonPresenter {
    content_root: FlexBox,
    label_node: TextCore,
}

impl HouseButtonPresenter {
    fn new() -> Self {
        let label_node = TextCore::new("");
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

    fn label_node(&self) -> TextCore {
        self.label_node.clone()
    }

    fn present(
        &self,
        theme: Theme,
        state: ButtonVisualState,
        colors: Option<ButtonColors>,
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
                control_background_color()
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
    _override_status: TextNode,
    _radio_status: TextNode,
    _switch_status: TextNode,
    _slider_status: TextNode,
    _dropdown_field_root: FlexBox,
    _dropdown_option_root: FlexBox,
}

impl Stage4PresentationShowcase {
    fn new() -> Self {
        use_control_templates(ControlTemplateSet {
            button: Some(Rc::new(HouseButtonTemplate)),
            checkbox_indicator: Some(Rc::new(HouseCheckboxIndicatorTemplate)),
            radio_indicator: Some(Rc::new(HouseRadioIndicatorTemplate)),
            slider: Some(Rc::new(DEFAULT_SLIDER_TEMPLATE)),
            ..ControlTemplateSet::default()
        });

        let page = ui! {
            column()
                .fill_width()
                .height_len(auto())
                .padding(28.0, 28.0, 28.0, 48.0)
                .bg_color(page_background_color())
                .semantic_label("FUI-RS Stage 4 presentation page")
        };

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
            .thumb_min_height(36.0)
            .track_color(scrollbar_track_color())
            .thumb_color(scrollbar_thumb_color());
        root.vertical_scrollbar()
            .render()
            .semantic_label("Stage 4 presentation vertical scrollbar");

        page.children(children![
            nav_bar(),
            spacer(14.0),
            title_block(),
            spacer(18.0),
        ]);

        let house_button = ui! {
        button("House template button")
            .semantic_label("Stage 4 house template button")
            .node_id("stage4-template-house-button")
        };
        let house_checkbox = ui! {
        checkbox("House template checkbox")
            .colors(stage4_labeled_colors(0x0EA5E9FF))
            .checked(true)
            .semantic_label("Stage 4 house template checkbox")
            .node_id("stage4-template-house-checkbox")
        };
        page.children(children![
            ui! {
                showcase_card(
                    "App-level ControlTemplateSet",
                    "Button and Checkbox pick up route-wide templates before any per-instance calls.",
                    "Stage 4 app-level template card",
                ) {
                    house_button,
                    spacer(10.0),
                    house_checkbox,
                    hint("Expected: rose button chrome and blue rounded checkbox indicator come from app-level templates."),
                }
            },
            spacer(18.0),
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
                    "Per-instance template precedence",
                    "This checkbox supplies a local template and remains visually distinct from the app-level house checkbox.",
                    "Stage 4 local template override card",
                ) {
                    override_checkbox,
                    spacer(8.0),
                    override_status,
                    hint(
                        "Click it: this one should toggle independently and keep the square amber override with an accent stripe, proving local template precedence.",
                    ),
                }
            },
            spacer(18.0),
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
            .sizing(
                LabeledControlSizing::new()
                    .indicator_size(16.0)
                    .label_font_size(14.0),
            )
            .colors(stage4_labeled_colors(0x2563EBFF))
            .checked(true)
            .semantic_label("Stage 4 compact radio sizing");
        radio_beta
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
                    .background(control_background_color())
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
                hint("The circles below use the default RadioButton presenter so LabeledControlSizing is visible: compact uses a 16px indicator, large uses a 30px indicator."),
                spacer(8.0),
                radio_alpha,
                spacer(8.0),
                radio_beta,
                spacer(8.0),
                radio_status,
                spacer(12.0),
                switch_control,
                spacer(8.0),
                switch_status,
                spacer(14.0),
                slider_control,
                spacer(8.0),
                slider_status,
                hint("Drag or click these controls: status text should update while the presenter-owned sizing and colors stay intact."),
            }
        };
        page.children(children![sizing_card, spacer(18.0)]);

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
                spacer(14.0),
                large_slider,
                hint("Expected: color overrides flow through the same presenters, not through custom event behavior."),
            }
        };
        page.children(children![colors_card, spacer(18.0)]);

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
                    .background(control_background_color())
                    .border(0x0284C7FF)
                    .accent(0x0284C7FF)
                    .text_primary(primary_text_color()),
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
                    .background(selected_background_color())
                    .accent(0x0284C7FF)
                    .text_primary(primary_text_color()),
            ),
        );
        let dropdown_field_root = dropdown_field.root();
        let dropdown_option_root = dropdown_option.root();
        dropdown_field_root.semantic_label("Stage 4 dropdown field presenter preview");
        dropdown_option_root.semantic_label("Stage 4 dropdown option row presenter preview");
        dropdown_card
            .child(&dropdown_field_root)
            .child(&spacer(10.0))
            .child(&dropdown_option_root)
            .child(&hint("Expected: presenter contracts can be created and styled independently of the future Dropdown control."));
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
            _override_status: override_status,
            _radio_status: radio_status,
            _switch_status: switch_status,
            _slider_status: slider_status,
            _dropdown_field_root: dropdown_field_root,
            _dropdown_option_root: dropdown_option_root,
        }
    }
}

fn page_background_color() -> u32 {
    if is_dark_mode() {
        0x0B1120FF
    } else {
        0xF7F4ECFF
    }
}

fn card_background_color() -> u32 {
    if is_dark_mode() {
        0x111827FF
    } else {
        0xFFFFFFFF
    }
}

fn card_border_color() -> u32 {
    if is_dark_mode() {
        0x334155FF
    } else {
        0xE5E7EBFF
    }
}

fn control_background_color() -> u32 {
    if is_dark_mode() {
        0x1F2937FF
    } else {
        0xFFFFFFFF
    }
}

fn selected_background_color() -> u32 {
    if is_dark_mode() {
        0x0C4A6EFF
    } else {
        0xE0F2FEFF
    }
}

fn title_surface_color() -> u32 {
    if is_dark_mode() {
        0x1E293BFF
    } else {
        0x111827FF
    }
}

fn title_text_color() -> u32 {
    0xFFFFFFFF
}

fn title_muted_text_color() -> u32 {
    if is_dark_mode() {
        0xCBD5E1FF
    } else {
        0xD1D5DBFF
    }
}

fn primary_text_color() -> u32 {
    current_theme().colors.text_primary
}

fn muted_text_color() -> u32 {
    current_theme().colors.text_muted
}

fn hint_text_color() -> u32 {
    if is_dark_mode() {
        0x94A3B8FF
    } else {
        0x64748BFF
    }
}

fn scrollbar_track_color() -> u32 {
    if is_dark_mode() {
        0x1E293BFF
    } else {
        0xE2E8F0FF
    }
}

fn scrollbar_thumb_color() -> u32 {
    if is_dark_mode() {
        0x64748BFF
    } else {
        0x64748BFF
    }
}

fn stage4_labeled_colors(accent: u32) -> LabeledControlColors {
    LabeledControlColors::new()
        .accent(accent)
        .background(control_background_color())
        .border(card_border_color())
        .text_primary(primary_text_color())
        .text_muted(muted_text_color())
}

fn is_source_demo_route(route: &str) -> bool {
    route.is_empty() || route.starts_with(SOURCE_DEMO_BASE)
}

fn route_pair(source: &'static str, published: &'static str) -> &'static str {
    let route = current_route();
    if is_source_demo_route(&route) {
        source
    } else {
        published
    }
}

fn nav_bar() -> FlexBox {
    ui! {
        row()
            .fill_width()
            .height(44.0, Unit::Pixel)
            .align_items(AlignItems::Center)
            .padding(8.0, 0.0, 8.0, 0.0)
            .semantic_label("Stage 4 presentation route nav") {
                NavLink::with_label(
                    route_pair(SOURCE_HOME_ROUTE, PUBLISHED_HOME_ROUTE),
                    "Dashboard",
                ).semantic_label("Dashboard"),
                spacer_width(14.0),
                NavLink::with_label(
                    route_pair(SOURCE_WORKBENCH_ROUTE, PUBLISHED_WORKBENCH_ROUTE),
                    "Workbench",
                ).semantic_label("Workbench"),
                spacer_width(14.0),
                NavLink::with_label(
                    route_pair(SOURCE_STAGE4_ROUTE, PUBLISHED_STAGE4_ROUTE),
                    "Stage 4",
                ).semantic_label("Stage 4"),
        }
    }
}

fn title_block() -> FlexBox {
    ui! {
        column()
            .fill_width()
            .padding(24.0, 24.0, 24.0, 24.0)
            .corner_radius(24.0)
            .bg_color(title_surface_color())
            .semantic_label("FUI-RS Stage 4 presentation verification") {
                text("FUI-RS Stage 4 presentation verification")
                    .font_size(30.0)
                    .font_weight(FontWeight::Bold)
                    .text_color(title_text_color())
                    .semantic_label("FUI-RS Stage 4 presentation verification"),
                spacer(8.0),
                text("Dedicated routed WASM for control sizing, app-level templates, per-instance template precedence, presenter color overrides, and dropdown presenter contracts.")
                    .font_size(16.0)
                    .text_color(title_muted_text_color())
                    .text_limits(-1, 3),
        }
    }
}

fn showcase_card(title: &str, description: &str, semantic_label: &str) -> FlexBox {
    ui! {
        column()
            .fill_width()
            .padding(20.0, 20.0, 20.0, 20.0)
            .corner_radius(20.0)
            .bg_color(card_background_color())
            .border(1.0, card_border_color())
            .semantic_label(semantic_label) {
                text(title)
                    .font_size(20.0)
                    .font_weight(FontWeight::Bold)
                    .text_color(primary_text_color())
                    .semantic_label(title),
                text(description)
                    .font_size(15.0)
                    .text_color(muted_text_color())
                    .text_limits(-1, 3),
                spacer(6.0),
        }
    }
}

fn status_text(value: &str) -> TextNode {
    let node = ui! {
    text(value).font_size(14.0)
        .font_weight(FontWeight::Bold)
        .text_color(primary_text_color())
        .text_limits(-1, 2)
    };
    node
}

fn hint(value: &str) -> TextNode {
    let node = ui! {
    text(value).font_size(13.0)
        .text_color(hint_text_color())
        .text_limits(-1, 2)
    };
    node
}

fn spacer(height: f32) -> FlexBox {
    ui! { flex_box().height(height, Unit::Pixel) }
}

fn spacer_width(width: f32) -> FlexBox {
    ui! { flex_box().width(width, Unit::Pixel) }
}

fn dispose_stage4_page(_: &Stage4PresentationShowcase) {
    clear_control_templates();
    clear_demo_shared_state();
}

fui_managed_app!(
    Stage4PresentationShowcase,
    Stage4PresentationShowcase::new,
    |page: &Stage4PresentationShowcase| page.root.clone(),
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
            current_theme(),
            ButtonVisualState {
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
            current_theme(),
            ButtonVisualState {
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
