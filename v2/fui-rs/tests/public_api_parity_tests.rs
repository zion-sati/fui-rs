#![allow(dead_code)]

use fui::ffi::{self, Call, NodeType};
use fui::prelude::*;
use fui::*;

fn assert_type<T: ?Sized>() {}

fn public_prelude_exports_compile() {
    assert_type::<AlignSelf>();
    assert_type::<AlignItems>();
    assert_type::<BorderStyle>();
    assert_type::<CursorStyle>();
    assert_type::<FlexDirection>();
    assert_type::<FlexWrap>();
    assert_type::<GridUnit>();
    assert_type::<GridTrack>();
    assert_type::<ImageSamplingMode>();
    assert_type::<JustifyContent>();
    assert_type::<KeyEventType>();
    assert_type::<KeyModifier>();
    assert_type::<ObjectFit>();
    assert_type::<Orientation>();
    assert_type::<PointerEventType>();
    assert_type::<PositionType>();
    assert_type::<SemanticCheckedState>();
    assert_type::<SemanticRole>();
    assert_type::<TextAlign>();
    assert_type::<TextOverflow>();
    assert_type::<TextVerticalAlign>();
    assert_type::<Unit>();
    assert_type::<Visibility>();

    assert_type::<Text>();
    assert_type::<TextNode>();
    assert_type::<Image>();
    assert_type::<ImageNode>();
    assert_type::<Svg>();
    assert_type::<SvgNode>();
    assert_type::<GradientStop>();

    assert_type::<Colors>();
    assert_type::<Spacing>();
    assert_type::<Fonts>();
    assert_type::<ContextMenuItemTheme>();
    assert_type::<ContextMenuTheme>();
    assert_type::<ToolTipTheme>();
    assert_type::<Theme>();

    assert_type::<PersistedBoolCodec>();
    assert_type::<PersistedFloat32Codec>();
    assert_type::<PersistedInt32Codec>();
    assert_type::<PersistedStringCodec>();
    assert_type::<PersistedScrollOffset>();
    assert_type::<PersistedTextState>();

    assert_type::<dyn ButtonPresenter>();
    assert_type::<dyn ButtonTemplate>();
    assert_type::<ButtonVisualState>();
    assert_type::<dyn CheckboxIndicatorPresenter>();
    assert_type::<dyn CheckboxIndicatorTemplate>();
    assert_type::<CheckboxIndicatorVisualState>();
    assert_type::<ControlTemplateSet>();
    assert_type::<dyn DropdownChevronPresenter>();
    assert_type::<dyn DropdownChevronTemplate>();
    assert_type::<DropdownChevronVisualState>();
    assert_type::<dyn DropdownFieldPresenter>();
    assert_type::<dyn DropdownFieldTemplate>();
    assert_type::<DropdownFieldVisualState>();
    assert_type::<dyn DropdownOptionRowPresenter>();
    assert_type::<dyn DropdownOptionRowTemplate>();
    assert_type::<DropdownOptionRowVisualState>();
    assert_type::<LabeledControlColors>();
    assert_type::<LabeledControlSizing>();
    assert_type::<PressableIndicatorMetrics>();
    assert_type::<dyn RadioIndicatorPresenter>();
    assert_type::<dyn RadioIndicatorTemplate>();
    assert_type::<RadioIndicatorVisualState>();
    assert_type::<dyn SliderPresenter>();
    assert_type::<SliderPresenterMetrics>();
    assert_type::<dyn SliderTemplate>();
    assert_type::<SliderVisualState>();
    assert_type::<dyn SwitchIndicatorPresenter>();
    assert_type::<dyn SwitchIndicatorTemplate>();
    assert_type::<SwitchIndicatorVisualState>();
    assert_type::<dyn TextInputPresenter>();
    assert_type::<dyn TextInputTemplate>();
    assert_type::<TextInputVisualState>();

    let _ = default_dark_theme as fn() -> Theme;
    let _ = default_light_theme as fn() -> Theme;
    let _ = generate_theme as fn(bool, u32) -> Theme;
    let _ = bind_theme(|_| {});
    let _ = platform_family as fn() -> PlatformFamily;
    let _ = primary_shortcut_modifier as fn() -> u32;
    let _ = word_navigation_modifier as fn() -> u32;
    let _ = line_boundary_modifier as fn() -> u32;
    let _ = document_boundary_modifier as fn() -> u32;
    let _ = has_primary_shortcut_modifier as fn(u32) -> bool;
    let _ = has_word_navigation_modifier as fn(u32) -> bool;
    let _ = has_line_boundary_modifier as fn(u32) -> bool;
    let _ = has_document_boundary_modifier as fn(u32) -> bool;
    let _ = format_shortcut_label as fn(&str, u32) -> String;
    let _ = format_primary_shortcut_label as fn(&str) -> String;
    let _ = format_undo_shortcut_label as fn() -> String;
    let _ = format_redo_shortcut_label as fn() -> String;
    let _ = is_undo_shortcut as fn(&str, u32) -> bool;
    let _ = is_redo_shortcut as fn(&str, u32) -> bool;
    let _ = show_keyboard_focus_for_key_event as fn(KeyEventType, &str, u32);

    let _ = clear_control_templates as fn();
    let _ = get_control_templates as fn() -> Option<ControlTemplateSet>;
    let _ = use_control_templates as fn(Option<ControlTemplateSet>) -> Option<ControlTemplateSet>;
}

fn gradient_stop_api_compiles() {
    let card = flex_box();
    card.linear_gradient_stops(
        0.0,
        0.0,
        1.0,
        1.0,
        vec![
            GradientStop::new(0.0, 0x000000ff),
            GradientStop::new(1.0, 0xffffffff),
        ],
    );
}

#[test]
fn rust_ui_macros_build_retained_mixed_child_trees() {
    ffi::test::reset();

    let retained_button = button("Retained");
    let root = ui! {
        column().semantic_label("Macro root") {
            text("Title").font_size(20.0),
            row().semantic_label("Actions") {
                retained_button.clone(),
                checkbox("Enabled").checked(true),
            },
        }
    };

    Application::mount(root.clone());

    let calls = ffi::test::take_calls();
    let flex_count = calls
        .iter()
        .filter(|call| matches!(call, Call::CreateNode { node_type, .. } if *node_type == NodeType::FlexBox as u32))
        .count();
    let text_count = calls
        .iter()
        .filter(|call| matches!(call, Call::CreateNode { node_type, .. } if *node_type == NodeType::Text as u32))
        .count();
    let child_add_count = calls
        .iter()
        .filter(|call| matches!(call, Call::NodeAddChild { .. }))
        .count();

    assert!(flex_count >= 2);
    assert!(text_count >= 3);
    assert!(child_add_count >= 4);
    assert!(calls.iter().any(|call| {
        matches!(call, Call::SetSemanticLabel { label, .. } if label == "Macro root")
    }));
    assert!(calls.iter().any(|call| {
        matches!(call, Call::SetSemanticLabel { label, .. } if label == "Actions")
    }));
}

#[test]
fn rust_children_macro_builds_mixed_child_vectors() {
    ffi::test::reset();

    let root = column();
    root.children(children![text("One"), button("Two"), checkbox("Three"),]);

    Application::mount(root.clone());

    let calls = ffi::test::take_calls();
    let child_add_count = calls
        .iter()
        .filter(|call| matches!(call, Call::NodeAddChild { .. }))
        .count();
    assert!(child_add_count >= 3);
}

#[test]
fn composed_controls_expose_inherited_root_event_surface() {
    ffi::test::reset();

    let combo = ui! {
        combo_box()
            .node_id("parity-combo")
            .width(220.0, Unit::Pixel)
            .visibility(Visibility::Normal)
            .on_focus_changed(|_| {})
            .on_click(|event| {
                event.handled = true;
            })
            .on_key_down(|event| {
                event.handled = true;
            })
    };

    Application::mount(combo);

    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| {
        matches!(call, Call::SetNodeId { node_id, .. } if node_id == "parity-combo")
    }));
}

fn build_lifecycle_macro_page() -> FlexBox {
    ui! {
        column().semantic_label("Lifecycle macro root") {
            text("Lifecycle macro page"),
        }
    }
}

fui_app!(FlexBox, build_lifecycle_macro_page);

#[test]
fn lifecycle_macro_exports_run_and_dispose_without_manual_entrypoint_code() {
    ffi::test::reset();

    __runApp();
    __disposeApp();

    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| {
        matches!(call, Call::SetSemanticLabel { label, .. } if label == "Lifecycle macro root")
    }));
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::SetRoot { .. })));
}
