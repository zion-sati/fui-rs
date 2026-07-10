mod generated;

use fui_rs::prelude::*;
use fui_rs::TextInputColors;
use fui_rs_demo_shared::{
    clear_demo_shared_state, demo_card, demo_page_root, track_demo_theme_guard,
};
use std::cell::Cell;
use std::rc::Rc;
use AlignItems;

const COMBOBOX_FIELD_WIDTH: f32 = 220.0;

#[derive(Clone)]
struct Stage5DropdownTemplate;

impl DropdownOptionRowTemplate for Stage5DropdownTemplate {
    fn create(&self, sizing: Option<DropdownSizing>) -> Rc<dyn DropdownOptionRowPresenter> {
        struct Presenter {
            root: FlexBox,
            label: TextNode,
            metrics: DropdownOptionRowMetrics,
        }
        impl DropdownOptionRowPresenter for Presenter {
            fn root(&self) -> FlexBox {
                self.root.clone()
            }
            fn label_node(&self) -> TextNode {
                self.label.clone()
            }
            fn metrics(&self) -> DropdownOptionRowMetrics {
                self.metrics
            }
            fn apply(
                &self,
                theme: Theme,
                state: DropdownOptionRowVisualState,
                _colors: Option<DropdownColors>,
            ) {
                self.root
                    .padding(14.0, 0.0, 14.0, 0.0)
                    .corner_radius(10.0)
                    .bg_color(if state.highlighted {
                        if is_dark_mode() {
                            0x3F2B16FF
                        } else {
                            0xFEF3C7FF
                        }
                    } else {
                        0x00000000
                    });
                self.label
                    .font_family(theme.fonts.body_family.clone())
                    .font_size(self.metrics.font_size)
                    .text_color(if state.selected {
                        0xD97706FF
                    } else {
                        theme.colors.text_primary
                    });
            }
        }
        let metrics = sizing
            .filter(|value| value.has_option_height() || value.has_option_font_size())
            .map(|value| {
                DropdownOptionRowMetrics::new(
                    if value.has_option_height() {
                        value.option_height_px()
                    } else {
                        34.0
                    },
                    14.0,
                    14.0,
                    if value.has_option_font_size() {
                        value.option_font_size_px()
                    } else {
                        16.0
                    },
                )
            })
            .unwrap_or(DropdownOptionRowMetrics::new(34.0, 14.0, 14.0, 16.0));
        let label = ui! {
            text("")
                .selectable(false, crate::current_theme().colors.selection)
                .fill_size()
                .wrapping(false)
                .text_limits(0, 1)
        };
        let root = ui! {
            row()
                .fill_size()
                .align_items(AlignItems::Center) {
                    label,
                }
        };
        Rc::new(Presenter {
            root,
            label,
            metrics,
        })
    }
}

fn make_items(values: &[&str]) -> Vec<DropdownItem> {
    values
        .iter()
        .map(|value| DropdownItem::from_value(*value))
        .collect()
}

fn themed_text(content: &str, muted: bool) -> TextNode {
    let theme = current_theme();
    let node = ui! {
    text(content).text_color(if muted {
        theme.colors.text_muted
    } else {
        theme.colors.text_primary
    })
    };
    track_demo_theme_guard(bind_theme({
        let node = node.clone();
        move |theme| {
            node.text_color(if muted {
                theme.colors.text_muted
            } else {
                theme.colors.text_primary
            });
        }
    }));
    node
}

fn disabled_text_input_colors(theme: &Theme) -> TextInputColors {
    if is_dark_mode() {
        TextInputColors::new()
            .background(0x182238FF)
            .border(0x475569FF)
            .text_muted(theme.colors.text_muted)
            .placeholder(theme.colors.text_muted)
    } else {
        TextInputColors::new()
            .background(0xF8FAFCFF)
            .border(0xCBD5E1FF)
            .text_muted(theme.colors.text_muted)
            .placeholder(theme.colors.text_muted)
    }
}

fn build_page() -> FlexBox {
    let page = ui! {
    demo_page_root("FUI-RS Stage 5 controls").height_len(auto())
    };
    let scroll = ui! {
        scroll_box()
            .fill_size()
            .bg_color(if is_dark_mode() {
                0x0F172AFF
            } else {
                0xF8FAFCFF
            })
            .persist_scroll(false) {
                page,
            }
    };
    let root = ui! {
        column()
            .fill_size()
            .bg_color(if is_dark_mode() {
                0x0F172AFF
            } else {
                0xF8FAFCFF
            }) {
                scroll,
            }
    };

    page.child(    &ui! {
            demo_card(
                "Phase 5.1 + 5.2 + 5.3 controls",
                "This route verifies the popup list foundation, retained dropdown behavior, text input/text area editing, and the editable ComboBox slice including filtering, commit behavior, and popup/editor coordination.",
                0xD7EAFEFF,
            ).margin(0.0, 18.0, 0.0, 0.0)
    });

    let status_text = themed_text("No selection change yet.", true);
    let focus_text = themed_text("Normal focus: blurred", true);
    let value_text = themed_text("Normal selected index: 1", true);
    page.child(&ui! {
            demo_card("Last change", "No selection change yet.", 0xDCFCE7FF)
                .margin(0.0, 0.0, 0.0, 18.0) {
                    status_text,
                    focus_text,
                    value_text,
                }
    });

    let normal = ui! {
        dropdown()
        .node_id("stage5-dropdown-normal")
        .items(make_items(&["Calm", "Focused", "Energetic"]))
        .select_index(1)
        .on_changed({
            let status_text = status_text.clone();
            let value_text = value_text.clone();
            move |event| {
                status_text.text(format!(
                    "Normal changed: {} at index {}",
                    event.item.label, event.selected_index
                ));
                value_text.text(format!("Normal selected index: {}", event.selected_index));
            }
        })
        .on_focus_changed({
            let focus_text = focus_text.clone();
            move |event| {
                focus_text.text(if event.focused {
                    "Normal focus: focused"
                } else {
                    "Normal focus: blurred"
                });
            }
        })
    };
    page.child(    &ui! {
            demo_card(
                "Normal dropdown",
                "Pointer, Enter, Space, ArrowDown, ArrowUp, Home, End, Escape, and persisted selection should match FUI-AS behavior.",
                0xFFFFFFFF,
            ).margin(0.0, 0.0, 0.0, 18.0) {
                normal,
            }
    });

    let themed = ui! {
        dropdown()
        .items(make_items(&["Ocean", "Forest", "Amber"]))
        .colors(Some(
            DropdownColors::new()
                .background(0xFFF7EDFF)
                .border(0xF59E0BFF)
                .text_primary(0x92400EFF)
                .accent(0xD97706FF),
        ))
        .popup_panel_color(if is_dark_mode() {
            0x1F2937FF
        } else {
            0xFFFBEBFF
        })
        .popup_panel_background_blur(0.0)
        .popup_width(280.0)
    };
    page.child(    &ui! {
            demo_card(
                "Themed + popup width",
                "Popup panel color, popup width, and presenter color tokens should be public API, not demo hacks.",
                0xFDE68AFF,
            ).margin(0.0, 0.0, 0.0, 18.0) {
                themed,
            }
    });

    let disabled = ui! {
        dropdown()
        .items(make_items(&["Disabled", "Should", "Not", "Open"]))
        .enabled(false)
    };
    let disabled_card = ui! {
        demo_card(
            "Disabled dropdown",
            "Disabled should suppress pointer, keyboard, and popup state while still rendering the selected label through the field presenter.",
            0xFCE7F3FF,
        ).margin(0.0, 0.0, 0.0, 18.0) {
            disabled,
        }
    };
    page.child(&disabled_card);

    page.child(    &ui! {
            demo_card(
                "Long list + scrolling",
                "The popup list should clamp viewport height, open with the selected row highlighted, and auto-scroll the highlighted item into view during keyboard navigation.",
                0xE0F2FEFF,
            ).margin(0.0, 0.0, 0.0, 18.0) {
                ui! {
                        dropdown()
                        .items(
                            (0..24)
                                .map(|index| {
                                    DropdownItem::new(
                                        format!("value-{index}"),
                                        format!("Long option {}", index + 1),
                                    )
                                }),
                        )
                        .max_visible_items(6)
                        .popup_width(320.0)
                        .select_index(12)
                },
            }
    });

    let templated = ui! {
        dropdown()
        .items(make_items(&["Primary", "Secondary", "Ghost"]))
        .option_row_template(Some(Rc::new(Stage5DropdownTemplate)))
        .sizing(Some(
            DropdownSizing::new()
                .field_height(40.0)
                .field_font_size(17.0)
                .option_height(38.0)
                .option_font_size(17.0)
                .chevron_box_size(18.0)
                .chevron_icon_size(14.0),
        ))
    };
    page.child(    &ui! {
            demo_card(
                "Templated + sizing",
                "Stage 5 inherits the Stage 4 presenter contract rather than bypassing it. Custom row templates and sizing should flow through the real control.",
                0xF3E8FFFF,
            ).margin(0.0, 0.0, 0.0, 18.0) {
                templated,
            }
    });

    let combo_status = themed_text("No ComboBox event yet.", true);
    let combo_focus = themed_text("Combo focus: blurred", true);
    let combo_value = themed_text("Combo selected index: 1", true);
    let combo_text = themed_text("Combo text: Focused", true);
    let combo_popup_state = themed_text("Combo popup: closed", true);
    let combo_filter_state = themed_text("Combo filtered count: 3 • Highlighted index: 1", true);
    page.child(&ui! {
            demo_card("ComboBox status", "No ComboBox event yet.", 0xE0F2FEFF)
                .margin(0.0, 0.0, 0.0, 18.0) {
                    combo_status,
                    combo_focus,
                    combo_value,
                    combo_text,
                    combo_popup_state,
                    combo_filter_state,
            }
    });

    let combo_normal = ui! {
        combo_box()
        .node_id("stage5-combobox-normal")
        .width(COMBOBOX_FIELD_WIDTH, Unit::Pixel)
        .items(vec!["Calm", "Focused", "Energetic"])
        .select_index(1)
        .on_changed({
            let combo_status = combo_status.clone();
            let combo_value = combo_value.clone();
            let combo_text = combo_text.clone();
            let combo_popup_state = combo_popup_state.clone();
            let combo_filter_state = combo_filter_state.clone();
            move |event| {
                combo_status.text(format!(
                    "Combo changed: {} at index {}",
                    event.item.value, event.selected_index
                ));
                combo_value.text(format!("Combo selected index: {}", event.selected_index));
                combo_text.text(format!("Combo text: {}", event.item.value));
                combo_popup_state.text("Combo popup: closed");
                combo_filter_state.text(format!(
                    "Combo filtered count: 1 • Highlighted index: {}",
                    event.selected_index
                ));
            }
        })
        .on_text_changed({
            let combo_text = combo_text.clone();
            let combo_popup_state = combo_popup_state.clone();
            let combo_filter_state = combo_filter_state.clone();
            move |event| {
                combo_text.text(format!("Combo text: {}", event.text));
                let query = event.text.to_lowercase();
                let filtered_count = ["calm", "focused", "energetic"]
                    .iter()
                    .filter(|value| value.contains(&query))
                    .count();
                combo_popup_state.text(if query.is_empty() {
                    "Combo popup: open or ready"
                } else {
                    "Combo popup: filtering"
                });
                combo_filter_state.text(format!(
                    "Combo filtered count: {} • Highlighted index: {}",
                    filtered_count,
                    if filtered_count == 0 { -1 } else { 0 }
                ));
            }
        })
        .on_focus_changed({
            let combo_focus = combo_focus.clone();
            move |event| {
                combo_focus.text(if event.focused {
                    "Combo focus: focused"
                } else {
                    "Combo focus: blurred"
                });
            }
        })
    };
    combo_popup_state.text(if combo_normal.is_open() {
        "Combo popup: open"
    } else {
        "Combo popup: closed"
    });
    combo_filter_state.text(format!(
        "Combo filtered count: {} • Highlighted index: {}",
        combo_normal.filtered_count(),
        combo_normal.highlighted_index()
    ));
    page.child(    &ui! {
            demo_card(
                "Normal ComboBox",
                "Click to open, type to filter, and use Enter/Home/End/Arrow keys to commit without losing retained editor state.",
                0xFFFFFFFF,
            ).margin(0.0, 0.0, 0.0, 18.0) {
                combo_normal,
            }
    });

    page.child(    &ui! {
            demo_card(
                "Filtering + exact commit",
                "StartsWith filtering, exact-match commit, and non-custom entry mode should follow the FUI-AS state machine rather than demo-only logic.",
                0xDCFCE7FF,
            ).margin(0.0, 0.0, 0.0, 18.0) {
                ui! {
                        combo_box()
                        .node_id("stage5-combobox-filter")
                        .width(COMBOBOX_FIELD_WIDTH, Unit::Pixel)
                        .items(vec!["Alpha", "Beta", "Gamma", "Delta"])
                        .placeholder("Type to filter")
                        .allow_custom(false)
                        .filter_mode(ComboBoxFilterMode::StartsWith)
                        .commit_mode(ComboBoxCommitMode::SelectExactMatch)
                        .stays_open_on_edit(true)
                },
            }
    });

    page.child(    &ui! {
            demo_card(
                "Autocomplete ComboBox",
                "Autocomplete is opt-in. Typing a visible value prefix completes the field and selects the completion suffix.",
                0xE0F2FEFF,
            ).margin(0.0, 0.0, 0.0, 18.0) {
                ui! {
                        combo_box()
                        .node_id("stage5-combobox-autocomplete")
                        .width(COMBOBOX_FIELD_WIDTH, Unit::Pixel)
                        .items(vec!["Melbourne", "Sydney", "Singapore", "San Francisco"])
                        .placeholder("Type Mel")
                        .auto_complete(true)
                        .stays_open_on_edit(true)
                },
            }
    });

    let themed_combo_colors = if is_dark_mode() {
        DropdownColors::new()
            .background(0x3F2B16FF)
            .border(0xF59E0BFF)
            .text_primary(0xFFFBEBFF)
            .accent(0xFBBF24FF)
    } else {
        DropdownColors::new()
            .background(0xFFF7EDFF)
            .border(0xF59E0BFF)
            .text_primary(0x92400EFF)
            .accent(0xD97706FF)
    };
    page.child(    &ui! {
            demo_card(
                "Themed ComboBox",
                "ComboBox reuses Dropdown sizing/colors for popup chrome while driving the embedded text-input presenter through the public control tokens.",
                0xFDE68AFF,
            ).margin(0.0, 0.0, 0.0, 18.0) {
                ui! {
                        combo_box()
                        .node_id("stage5-combobox-themed")
                        .width(COMBOBOX_FIELD_WIDTH, Unit::Pixel)
                        .items(vec!["Ocean", "Forest", "Amber"])
                        .colors(Some(themed_combo_colors))
                        .popup_panel_color(if is_dark_mode() {
                            0x1F2937FF
                        } else {
                            0xFFFBEBFF
                        })
                        .popup_panel_background_blur(0.0)
                        .popup_width(280.0)
                },
            }
    });

    page.child(    &ui! {
            demo_card(
                "Disabled ComboBox",
                "Disabled should suppress pointer, keyboard, popup, and editor mutation while preserving the field visuals.",
                0xFCE7F3FF,
            ).margin(0.0, 0.0, 0.0, 18.0) {
                ui! {
                        combo_box()
                        .node_id("stage5-combobox-disabled")
                        .width(COMBOBOX_FIELD_WIDTH, Unit::Pixel)
                        .items(vec!["Disabled", "Should", "Not", "Open"])
                        .enabled(false)
                },
            }
    });

    page.child(    &ui! {
            demo_card(
                "Long ComboBox list",
                "The editable trigger should keep the popup anchored and auto-scroll the highlighted row into view while filtering and keyboard navigation mutate the visible subset.",
                0xEDE9FEFF,
            ).margin(0.0, 0.0, 0.0, 18.0) {
                ui! {
                        combo_box()
                        .node_id("stage5-combobox-long-list")
                        .width(COMBOBOX_FIELD_WIDTH, Unit::Pixel)
                        .items((1..=24).map(|index| format!("Long option {index}")))
                        .max_visible_items(6)
                        .popup_width(320.0)
                        .select_index(12)
                },
            }
    });

    page.child(    &ui! {
            demo_card(
                "Templated ComboBox",
                "Chevron templating, option row templating, and sizing reuse the same presenter contracts as Dropdown, with the editor retained inside the control.",
                0xF3E8FFFF,
            ).margin(0.0, 0.0, 0.0, 18.0) {
                ui! {
                        combo_box()
                        .node_id("stage5-combobox-templated")
                        .width(COMBOBOX_FIELD_WIDTH, Unit::Pixel)
                        .items(vec!["Primary", "Secondary", "Ghost"])
                        .option_row_template(Some(Rc::new(Stage5DropdownTemplate)))
                        .sizing(Some(
                            DropdownSizing::new()
                                .field_height(40.0)
                                .field_font_size(17.0)
                                .option_height(38.0)
                                .option_font_size(17.0)
                                .chevron_box_size(18.0)
                                .chevron_icon_size(14.0),
                        ))
                },
            }
    });

    let input_status = themed_text("No text event yet.", true);
    let password_status = themed_text("Password changed: 0 chars", true);
    let text_area_status = themed_text("TextArea value: 0 chars • 1 line", true);
    let text_area_focus_status = themed_text("TextArea focus: blurred", true);
    let text_area_selection_status = themed_text("TextArea selection: 0..0", true);
    let text_area_scroll_status = themed_text("TextArea scroll offset: 0, 0", true);
    let text_area_config_status = themed_text(
        "TextArea config: read-only off • wrapping on • tabs insert • visibility normal • V auto • H auto • line normal • font variable",
        true,
    );
    let selection_status = themed_text("Selection: 0..0", true);
    let byte_status = themed_text("Byte range: 0..0", true);
    let count_status = themed_text("Changed: 0 • Selection changed: 0", true);
    let focus_status = themed_text("Focus: blurred", true);
    let form_status = themed_text(
        "Form submit: none • projected fields: username/current-password",
        true,
    );
    page.child(&ui! {
            demo_card("Text input status", "No text event yet.", 0xEDE9FEFF)
                .margin(0.0, 0.0, 0.0, 18.0)
                .semantic_label("Stage 5 text input status card") {
                    input_status,
                    password_status,
                    text_area_status,
                    text_area_focus_status,
                    text_area_selection_status,
                    text_area_scroll_status,
                    text_area_config_status,
                    selection_status,
                    byte_status,
                    count_status,
                    focus_status,
                    form_status,
            }
    });

    let username = text_input();
    let changed_count = Rc::new(Cell::new(0u32));
    let selection_changed_count = Rc::new(Cell::new(0u32));
    username
        .fill_width()
        .node_id("stage5-username")
        .placeholder("Username or email")
        .host_autofill("username")
        .on_changed({
            let input_status = input_status.clone();
            let count_status = count_status.clone();
            let changed_count = changed_count.clone();
            let selection_changed_count = selection_changed_count.clone();
            move |event| {
                changed_count.set(changed_count.get() + 1);
                input_status.text(format!("Username changed: {}", event.text));
                count_status.text(format!(
                    "Changed: {} • Selection changed: {}",
                    changed_count.get(),
                    selection_changed_count.get()
                ));
            }
        })
        .on_selection_changed({
            let selection_status = selection_status.clone();
            let byte_status = byte_status.clone();
            let count_status = count_status.clone();
            let username = username.clone();
            let changed_count = changed_count.clone();
            let selection_changed_count = selection_changed_count.clone();
            move |event| {
                selection_changed_count.set(selection_changed_count.get() + 1);
                selection_status.text(format!("Selection: {}..{}", event.start, event.end));
                byte_status.text(format!(
                    "Byte range: {}..{}",
                    username.selection_start_byte_offset(),
                    username.selection_end_byte_offset()
                ));
                count_status.text(format!(
                    "Changed: {} • Selection changed: {}",
                    changed_count.get(),
                    selection_changed_count.get()
                ));
            }
        })
        .on_focus_changed({
            let focus_status = focus_status.clone();
            move |event| {
                focus_status.text(if event.focused {
                    "Focus: focused"
                } else {
                    "Focus: blurred"
                });
            }
        });
    let password = ui! {
        text_input()
        .fill_width()
        .node_id("stage5-password")
        .placeholder("Password")
        .password(true)
        .host_autofill("current-password")
        .on_changed({
            let password_status = password_status.clone();
            move |event| {
                password_status.text(format!(
                    "Password changed: {} chars",
                    event.text.chars().count()
                ));
            }
        })
    };
    let submit_button = ui! { button("OK").node_id("stage5-login-ok") };
    let cancel_button = ui! { button("Cancel").node_id("stage5-login-cancel") };
    submit_button.on_click({
        let form_status = form_status.clone();
        let username = username.clone();
        let password = password.clone();
        move |_event| {
            form_status.text(format!(
                "Form submit: OK • username '{}' • password {} chars",
                username.value(),
                password.value().chars().count()
            ));
        }
    });
    cancel_button.on_click({
        let form_status = form_status.clone();
        move |_event| {
            form_status.text("Form submit: Cancel • default/cancel key route works");
        }
    });
    page.child(&ui! {
            demo_card(
                "Form and host autofill",
                "The username and password fields live under a real semantic Form. Enter activates OK, Escape activates Cancel, and hostAutofill projects grouped browser/password-manager compatible inputs.",
                0xECFDF5FF,
            ).margin(0.0, 0.0, 0.0, 18.0) {
                ui! {
                        form()
                        .default_btn(&submit_button)
                        .cancel_btn(&cancel_button)
                        .children(children![
                            ui! {
                                demo_card(
                                    "Normal text input",
                                    "Typing, backspace, arrows, Home, End, click caret movement, drag selection, and placeholder visibility should match FUI-AS single-line text input behavior.",
                                    0xFFFFFFFF,
                                ).margin(0.0, 0.0, 0.0, 18.0) {
                                    username,
                                }
                            },
                            ui! {
                                demo_card(
                                    "Password text input",
                                    "Password mode should keep the same presenter and selection mechanics while projecting password metadata through the browser host.",
                                    0xFEF3C7FF,
                                ).margin(0.0, 0.0, 0.0, 18.0) {
                                    password,
                                }
                            },
                            ui! {
                                row().align_items(AlignItems::Center) {
                                    submit_button,
                                    cancel_button,
                                }
                            },
                        ])
                        .activate()
                },
                themed_text(
                    "Manual check: browser autofill/password-manager dropdowns should see one form containing username and current-password fields named from their node IDs.",
                    true,
                ),
            }
    });

    let text_area = text_area();
    let text_area_focused = Rc::new(Cell::new(false));
    text_area
        .fill_width()
        .height(220.0, Unit::Pixel)
        .node_id("stage5-text-area")
        .placeholder("Multiline notes")
        .text("Line one\nLine two\nLine three\nFallback sample: 你好，你好吗？\nLonger content so scrollbar policy is easy to spot.")
        .accepts_tab(true)
        .wrapping(true)
        .on_changed({
            let text_area_status = text_area_status.clone();
            move |event| {
                let line_count = event.text.lines().count().max(1);
                text_area_status.text(format!(
                    "TextArea value: {} chars • {} line{}",
                    event.text.chars().count(),
                    line_count,
                    if line_count == 1 { "" } else { "s" }
                ));
            }
        })
        .on_focus_changed({
            let text_area_focus_status = text_area_focus_status.clone();
            let text_area_focused = text_area_focused.clone();
            move |event| {
                text_area_focused.set(event.focused);
                text_area_focus_status.text(if event.focused {
                    "TextArea focus: focused"
                } else {
                    "TextArea focus: blurred"
                });
            }
        })
        .on_selection_changed({
            let text_area_selection_status = text_area_selection_status.clone();
            move |event| {
                text_area_selection_status.text(format!(
                    "TextArea selection: {}..{}",
                    event.start, event.end
                ));
            }
        });
    let read_only_toggle = ui! {
        checkbox("Read-only")
            .node_id("stage5-text-area-readonly-toggle")
    };
    let wrapping_toggle = ui! {
        checkbox("Wrapping")
        .node_id("stage5-text-area-wrapping-toggle")
        .check(true)
    };
    let accepts_tab_toggle = ui! {
        checkbox("Accepts Tab")
        .node_id("stage5-text-area-accepts-tab-toggle")
        .check(true)
    };
    let always_vertical_toggle = ui! {
        checkbox("Always show vertical scrollbar")
            .node_id("stage5-text-area-always-vertical-toggle")
    };
    let never_vertical_toggle = ui! {
        checkbox("Hide vertical scrollbar")
            .node_id("stage5-text-area-never-vertical-toggle")
    };
    let always_horizontal_toggle = ui! {
        checkbox("Always show horizontal scrollbar")
            .node_id("stage5-text-area-always-horizontal-toggle")
    };
    let never_horizontal_toggle = ui! {
        checkbox("Hide horizontal scrollbar")
            .node_id("stage5-text-area-never-horizontal-toggle")
    };

    let vertical_policy_group = ui! {
    radio_group().semantic_label("Stage 5 TextArea vertical scrollbar policy")
    };
    vertical_policy_group
        .add_option("auto", "Vertical scrollbar: Auto")
        .checked(true);
    vertical_policy_group.add_option("always", "Vertical scrollbar: Always");
    vertical_policy_group.add_option("never", "Vertical scrollbar: Never");
    vertical_policy_group.select_index(0);

    let horizontal_policy_group = ui! {
    radio_group().semantic_label("Stage 5 TextArea horizontal scrollbar policy")
    };
    horizontal_policy_group
        .add_option("auto", "Horizontal scrollbar: Auto")
        .checked(true);
    horizontal_policy_group.add_option("always", "Horizontal scrollbar: Always");
    horizontal_policy_group.add_option("never", "Horizontal scrollbar: Never");
    horizontal_policy_group.select_index(0);

    let line_height_group = ui! {
    radio_group().semantic_label("Stage 5 TextArea line height")
    };
    line_height_group
        .add_option("normal", "Line height: Normal")
        .checked(true);
    line_height_group.add_option("fixed-28", "Line height: Fixed 28 px");
    line_height_group.select_index(0);

    let font_mode_group = ui! {
    radio_group().semantic_label("Stage 5 TextArea font mode")
    };
    font_mode_group
        .add_option("variable", "Text font: Variable width")
        .checked(true);
    font_mode_group.add_option("mono", "Text font: Monospace");
    font_mode_group.select_index(0);

    let visibility_dropdown = ui! {
        dropdown()
        .node_id("stage5-text-area-visibility")
        .items(vec![
            DropdownItem::new(
                "normal",
                "Visibility: Normal - keep layout reserved and content rendered",
            ),
            DropdownItem::new(
                "hidden",
                "Visibility: Hidden - keep layout reserved but stop painting content",
            ),
            DropdownItem::new(
                "collapsed",
                "Visibility: Collapsed - remove layout space and hide the content",
            ),
        ])
        .select_index(0)
    };

    let sync_text_area_status: Rc<dyn Fn()> = Rc::new({
        let text_area = text_area.clone();
        let text_area_status = text_area_status.clone();
        let text_area_focus_status = text_area_focus_status.clone();
        let text_area_selection_status = text_area_selection_status.clone();
        let text_area_scroll_status = text_area_scroll_status.clone();
        let text_area_config_status = text_area_config_status.clone();
        let text_area_focused = text_area_focused.clone();
        let read_only_toggle = read_only_toggle.clone();
        let wrapping_toggle = wrapping_toggle.clone();
        let accepts_tab_toggle = accepts_tab_toggle.clone();
        let vertical_policy_group = vertical_policy_group.clone();
        let horizontal_policy_group = horizontal_policy_group.clone();
        let line_height_group = line_height_group.clone();
        let font_mode_group = font_mode_group.clone();
        let visibility_dropdown = visibility_dropdown.clone();
        move || {
            let value = text_area.value();
            let line_count = value.lines().count().max(1);
            text_area_status.text(format!(
                "TextArea value: {} chars • {} line{}",
                value.chars().count(),
                line_count,
                if line_count == 1 { "" } else { "s" }
            ));
            text_area_focus_status.text(if text_area_focused.get() {
                "TextArea focus: focused"
            } else {
                "TextArea focus: blurred"
            });
            text_area_selection_status.text(format!(
                "TextArea selection: {}..{}",
                text_area.selection_start(),
                text_area.selection_end()
            ));
            text_area_scroll_status.text(format!(
                "TextArea scroll offset: {:.0}, {:.0}",
                text_area.scroll_offset_x(),
                text_area.scroll_offset_y()
            ));
            let visibility = match visibility_dropdown.selected_index() {
                1 => "hidden",
                2 => "collapsed",
                _ => "normal",
            };
            text_area_config_status.text(format!(
                "TextArea config: read-only {} • wrapping {} • tabs {} • visibility {} • V {} • H {} • line {} • font {}",
                if read_only_toggle.is_checked() { "on" } else { "off" },
                if wrapping_toggle.is_checked() { "on" } else { "off" },
                if accepts_tab_toggle.is_checked() { "insert" } else { "traverse" },
                visibility,
                vertical_policy_group.selected_value(),
                horizontal_policy_group.selected_value(),
                if line_height_group.selected_value() == "fixed-28" { "fixed 28px" } else { "normal" },
                if font_mode_group.selected_value() == "mono" { "monospace" } else { "variable" },
            ));
        }
    });

    read_only_toggle.on_changed({
        let text_area = text_area.clone();
        let sync = sync_text_area_status.clone();
        move |event| {
            text_area.read_only(event.checked);
            sync();
        }
    });
    wrapping_toggle.on_changed({
        let text_area = text_area.clone();
        let horizontal_policy_group = horizontal_policy_group.clone();
        let always_horizontal_toggle = always_horizontal_toggle.clone();
        let never_horizontal_toggle = never_horizontal_toggle.clone();
        let sync = sync_text_area_status.clone();
        move |event| {
            text_area.wrapping(event.checked);
            if event.checked {
                horizontal_policy_group.select_index(0);
                always_horizontal_toggle.check(false);
                never_horizontal_toggle.check(false);
                text_area.horizontal_scrollbar_visibility(ScrollBarVisibility::Auto);
            }
            sync();
        }
    });
    accepts_tab_toggle.on_changed({
        let text_area = text_area.clone();
        let sync = sync_text_area_status.clone();
        move |event| {
            text_area.accepts_tab(event.checked);
            sync();
        }
    });

    let syncing_vertical_policy = Rc::new(Cell::new(false));
    let apply_vertical_policy: Rc<dyn Fn()> = Rc::new({
        let text_area = text_area.clone();
        let vertical_policy_group = vertical_policy_group.clone();
        let always_vertical_toggle = always_vertical_toggle.clone();
        let never_vertical_toggle = never_vertical_toggle.clone();
        let syncing_vertical_policy = syncing_vertical_policy.clone();
        let sync = sync_text_area_status.clone();
        move || {
            syncing_vertical_policy.set(true);
            match vertical_policy_group.selected_value().as_str() {
                "always" => {
                    text_area.vertical_scrollbar_visibility(ScrollBarVisibility::Always);
                    always_vertical_toggle.check(true);
                    never_vertical_toggle.check(false);
                }
                "never" => {
                    text_area.vertical_scrollbar_visibility(ScrollBarVisibility::Never);
                    always_vertical_toggle.check(false);
                    never_vertical_toggle.check(true);
                }
                _ => {
                    text_area.vertical_scrollbar_visibility(ScrollBarVisibility::Auto);
                    always_vertical_toggle.check(false);
                    never_vertical_toggle.check(false);
                }
            }
            syncing_vertical_policy.set(false);
            sync();
        }
    });
    vertical_policy_group.on_changed({
        let apply = apply_vertical_policy.clone();
        move |_event| apply()
    });
    always_vertical_toggle.on_changed({
        let vertical_policy_group = vertical_policy_group.clone();
        let never_vertical_toggle = never_vertical_toggle.clone();
        let apply = apply_vertical_policy.clone();
        let syncing = syncing_vertical_policy.clone();
        move |event| {
            if syncing.get() {
                return;
            }
            if event.checked {
                never_vertical_toggle.check(false);
                vertical_policy_group.select_index(1);
            } else {
                vertical_policy_group.select_index(0);
            }
            apply();
        }
    });
    never_vertical_toggle.on_changed({
        let vertical_policy_group = vertical_policy_group.clone();
        let always_vertical_toggle = always_vertical_toggle.clone();
        let apply = apply_vertical_policy.clone();
        let syncing = syncing_vertical_policy.clone();
        move |event| {
            if syncing.get() {
                return;
            }
            if event.checked {
                always_vertical_toggle.check(false);
                vertical_policy_group.select_index(2);
            } else {
                vertical_policy_group.select_index(0);
            }
            apply();
        }
    });

    let syncing_horizontal_policy = Rc::new(Cell::new(false));
    let apply_horizontal_policy: Rc<dyn Fn()> = Rc::new({
        let text_area = text_area.clone();
        let horizontal_policy_group = horizontal_policy_group.clone();
        let always_horizontal_toggle = always_horizontal_toggle.clone();
        let never_horizontal_toggle = never_horizontal_toggle.clone();
        let syncing_horizontal_policy = syncing_horizontal_policy.clone();
        let sync = sync_text_area_status.clone();
        move || {
            syncing_horizontal_policy.set(true);
            match horizontal_policy_group.selected_value().as_str() {
                "always" => {
                    text_area.horizontal_scrollbar_visibility(ScrollBarVisibility::Always);
                    always_horizontal_toggle.check(true);
                    never_horizontal_toggle.check(false);
                }
                "never" => {
                    text_area.horizontal_scrollbar_visibility(ScrollBarVisibility::Never);
                    always_horizontal_toggle.check(false);
                    never_horizontal_toggle.check(true);
                }
                _ => {
                    text_area.horizontal_scrollbar_visibility(ScrollBarVisibility::Auto);
                    always_horizontal_toggle.check(false);
                    never_horizontal_toggle.check(false);
                }
            }
            syncing_horizontal_policy.set(false);
            sync();
        }
    });
    horizontal_policy_group.on_changed({
        let apply = apply_horizontal_policy.clone();
        move |_event| apply()
    });
    always_horizontal_toggle.on_changed({
        let horizontal_policy_group = horizontal_policy_group.clone();
        let never_horizontal_toggle = never_horizontal_toggle.clone();
        let apply = apply_horizontal_policy.clone();
        let syncing = syncing_horizontal_policy.clone();
        move |event| {
            if syncing.get() {
                return;
            }
            if event.checked {
                never_horizontal_toggle.check(false);
                horizontal_policy_group.select_index(1);
            } else {
                horizontal_policy_group.select_index(0);
            }
            apply();
        }
    });
    never_horizontal_toggle.on_changed({
        let horizontal_policy_group = horizontal_policy_group.clone();
        let always_horizontal_toggle = always_horizontal_toggle.clone();
        let apply = apply_horizontal_policy.clone();
        let syncing = syncing_horizontal_policy.clone();
        move |event| {
            if syncing.get() {
                return;
            }
            if event.checked {
                always_horizontal_toggle.check(false);
                horizontal_policy_group.select_index(2);
            } else {
                horizontal_policy_group.select_index(0);
            }
            apply();
        }
    });

    line_height_group.on_changed({
        let text_area = text_area.clone();
        let line_height_group = line_height_group.clone();
        let sync = sync_text_area_status.clone();
        move |_event| {
            if line_height_group.selected_value() == "fixed-28" {
                text_area.line_height(28.0);
            } else {
                text_area.line_height(0.0);
            }
            sync();
        }
    });
    let sync_font_mode: Rc<dyn Fn()> = Rc::new({
        let text_area = text_area.clone();
        let font_mode_group = font_mode_group.clone();
        let sync = sync_text_area_status.clone();
        move || {
            let theme = current_theme();
            if font_mode_group.selected_value() == "mono" {
                text_area
                    .font_family(theme.fonts.mono_family)
                    .font_size(theme.fonts.size_mono);
            } else {
                text_area
                    .font_family(theme.fonts.body_family)
                    .font_size(theme.fonts.size_body);
            }
            sync();
        }
    });
    font_mode_group.on_changed({
        let sync_font_mode = sync_font_mode.clone();
        move |_event| sync_font_mode()
    });
    track_demo_theme_guard(bind_theme({
        let sync_font_mode = sync_font_mode.clone();
        move |_theme| sync_font_mode()
    }));
    visibility_dropdown.on_changed({
        let text_area = text_area.clone();
        let visibility_dropdown = visibility_dropdown.clone();
        let sync = sync_text_area_status.clone();
        move |_event| {
            match visibility_dropdown.selected_index() {
                1 => text_area.visibility(Visibility::Hidden),
                2 => text_area.visibility(Visibility::Collapsed),
                _ => text_area.visibility(Visibility::Normal),
            };
            sync();
        }
    });
    sync_text_area_status();
    let text_area_card = demo_card(
        "Advanced TextArea",
        "Configure a TextArea live to explore wrapping, read-only mode, scrollbar policy, line height, font family, visibility, and tofu-swap fallback text.",
        0xDBEAFEFF,
    );
    text_area_card
        .margin(0.0, 0.0, 0.0, 18.0)
        .child(&text_area)
        .child(&themed_text(
            "Use the quick toggles for common changes, or the radio groups when you want an exact scrollbar or line-height setting. The CJK sample exercises incremental tofu-swap while remaining editable.",
            true,
        ))
        .child(&ui! {
            row()
            .fill_width()
            .align_items(AlignItems::Stretch) {
                ui! {
                        column().width_len(auto()) {
                            read_only_toggle,
                            wrapping_toggle,
                            accepts_tab_toggle,
                            always_vertical_toggle,
                            never_vertical_toggle,
                            always_horizontal_toggle,
                            never_horizontal_toggle,
                        }
                },
                ui! {
                        flex_box()
                            .width(18.0, Unit::Pixel)
                            .height(1.0, Unit::Pixel)
                },
                ui! {
                        column().width_len(auto()) {
                            vertical_policy_group,
                            horizontal_policy_group,
                            line_height_group,
                            font_mode_group,
                            visibility_dropdown,
                        }
                },
            }
        });
    page.child(&text_area_card);

    let read_only = ui! {
        text_input()
        .fill_width()
        .text("Read-only sample")
        .read_only(true)
        .selection_range(0, 4)
    };
    let read_only_card = demo_card(
        "Read-only text input",
        "Read-only should still support selection and focus state, but reject editing.",
        0xDCFCE7FF,
    );
    read_only_card.margin(0.0, 0.0, 0.0, 18.0).child(&read_only);
    page.child(&read_only_card);

    let disabled = ui! {
        text_input()
        .fill_width()
        .text("Disabled themed value")
        .placeholder("Disabled field")
        .enabled(false)
        .colors(Some(disabled_text_input_colors(&current_theme())))
    };
    track_demo_theme_guard(bind_theme({
        let disabled = disabled.clone();
        move |theme| {
            disabled.colors(Some(disabled_text_input_colors(&theme)));
        }
    }));
    let disabled_card = demo_card(
        "Disabled + themed",
        "Disabled input must suppress editing while still going through the public colors API rather than demo-only styling hacks.",
        0xFCE7F3FF,
    );
    disabled_card
        .margin(0.0, 0.0, 0.0, 80.0)
        .semantic_label("Stage 5 disabled themed card")
        .child(&disabled);
    page.child(&disabled_card);

    let phase57_status = themed_text("Phase 5.7 status: idle", true);
    let phase57_card = demo_card(
        "Phase 5.7 modal, tab stop, and gestures",
        "Open the modal dialog, Tab through the focusable field while the skipped field is omitted, and use touch/trackpad gestures on the probe.",
        0xDBEAFEFF,
    );
    phase57_card.margin(0.0, 0.0, 0.0, 18.0);
    phase57_card.child(&phase57_status);

    let modal = dialog(
        "Stage 5 modal dialog",
        "This retained modal dialog should trap semantic scope, support Enter/Escape through Form, and dismiss from the backdrop.",
    );
    modal.title_text().node_id("stage5-modal-title");
    modal.content_host().child(&ui! {
        text_input()
        .fill_width()
        .placeholder("Focusable field inside modal")
    });
    modal.on_shown({
        let phase57_status = phase57_status.clone();
        move |_event| {
            phase57_status.text("Phase 5.7 status: modal shown");
        }
    });
    modal.on_accept({
        let phase57_status = phase57_status.clone();
        move || {
            phase57_status.text("Phase 5.7 status: modal accepted");
        }
    });
    modal.on_cancel({
        let phase57_status = phase57_status.clone();
        move || {
            phase57_status.text("Phase 5.7 status: modal cancelled");
        }
    });

    let gesture_probe = ui! {
        flex_box()
        .width(100.0, Unit::Percent)
        .height(72.0, Unit::Pixel)
        .corner_radius(12.0)
        .border(1.0, current_theme().colors.border)
        .bg_color(if is_dark_mode() {
            0x1E293BFF
        } else {
            0xEFF6FFFF
        })
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::Center)
        .child(&themed_text(
            "Gesture probe: pan, pinch, or long press",
            true,
        ))
        .on_pan_gesture({
            let phase57_status = phase57_status.clone();
            move |event| {
                phase57_status.text(format!(
                    "Phase 5.7 status: pan dx {:.1}, dy {:.1}",
                    event.delta_x, event.delta_y
                ));
                event.handled = true;
            }
        })
        .on_pinch_gesture({
            let phase57_status = phase57_status.clone();
            move |event| {
                phase57_status.text(format!("Phase 5.7 status: pinch scale {:.2}", event.scale));
                event.handled = true;
            }
        })
        .long_press_options(650, 18.0)
        .on_long_press({
            let phase57_status = phase57_status.clone();
            move |event| {
                phase57_status.text(format!(
                    "Phase 5.7 status: long press at {:.0},{:.0}",
                    event.x, event.y
                ));
                event.handled = true;
            }
        })
    };
    track_demo_theme_guard(bind_theme({
        let gesture_probe = gesture_probe.clone();
        move |theme| {
            gesture_probe
                .border(1.0, theme.colors.border)
                .bg_color(if is_dark_mode() {
                    0x1E293BFF
                } else {
                    0xEFF6FFFF
                });
        }
    }));

    phase57_card
        .child(&ui! {
            button("Open modal dialog")
                .on_click({
                    let modal = modal.clone();
                    move |_event| {
                        modal.show();
                    }
                })
        })
        .child(&ui! {
            text_input()
            .fill_width()
            .placeholder("Tab stop enabled")
            .node_id("stage5-tabstop-enabled")
        })
        .child(&ui! {
            text_input()
            .fill_width()
            .placeholder("Tab stop disabled")
            .node_id("stage5-tabstop-disabled")
            .focusable(false, 0)
        })
        .child(&gesture_probe);
    page.child(&phase57_card);
    root.child(&modal);

    root
}

fn dispose_stage5_page(_: &FlexBox) {
    clear_demo_shared_state();
}

fui_managed_app!(
    FlexBox,
    build_page,
    |page: &FlexBox| page.clone(),
    dispose: dispose_stage5_page
);
