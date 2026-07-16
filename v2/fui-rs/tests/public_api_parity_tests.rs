#![allow(dead_code)]

use fui::ffi::{self, Call, NodeType};
use fui::prelude::*;
use fui::*;

#[derive(Clone)]
struct ParityComponent {
    root: FlexBox,
    value: std::rc::Rc<std::cell::Cell<i32>>,
}

fui_component!(ParityComponent => root);

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

#[test]
fn text_helper_accepts_owned_rust_strings() {
    ffi::test::reset();
    let value = text(format!("Dynamic value: {}", 42));
    Application::mount(value);
    assert!(ffi::test::take_calls()
        .iter()
        .any(|call| { matches!(call, Call::SetText { text, .. } if text == "Dynamic value: 42") }));
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
fn rust_ui_macro_accepts_borrowed_fluent_nodes_without_changing_identity() {
    ffi::test::reset();

    let retained_button = button("Borrowed");
    let root = ui! {
        column() {
            retained_button
                .node_id("borrowed-button")
                .margin(1.0, 2.0, 3.0, 4.0),
        }
    };

    Application::mount(root);

    let calls = ffi::test::take_calls();
    let borrowed_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::SetNodeId { handle, node_id } if node_id == "borrowed-button" => Some(*handle),
            _ => None,
        })
        .expect("borrowed button should retain its configured identity");
    assert_eq!(
        calls
            .iter()
            .filter(
                |call| matches!(call, Call::CreateNode { handle, .. } if *handle == borrowed_handle)
            )
            .count(),
        1
    );
    assert!(calls.iter().any(|call| {
        matches!(call, Call::NodeAddChild { child, .. } if *child == borrowed_handle)
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
fn labeled_controls_share_fluent_text_styling_and_selection_properties_are_independent() {
    ffi::test::reset();

    let family = current_theme().fonts.body_family;
    let root = ui! {
        column() {
            button("Button").font_family(family.clone()).font_size(18.0).text_color(0x112233FF),
            NavLink::with_label("/docs", "Docs").font_family(family.clone()).font_size(19.0).text_color(0x223344FF),
            checkbox("Check").font_family(family.clone()).font_size(20.0).text_color(0x334455FF),
            radio_button("Radio").font_family(family.clone()).font_size(21.0).text_color(0x445566FF),
            switch("Switch").font_family(family).font_size(22.0).text_color(0x556677FF),
            text("Selectable").selection_color(0x667788FF).selectable(false),
            TextCore::new("Core text").selection_color(0x778899FF),
        }
    };

    Application::mount(root);
    let calls = ffi::test::take_calls();
    for color in [0x112233FF, 0x223344FF, 0x334455FF, 0x445566FF, 0x556677FF] {
        assert!(
            calls.iter().any(
                |call| matches!(call, Call::SetTextColor { color: actual, .. } if *actual == color)
            ),
            "missing configured labeled-control color {color:#010x}"
        );
    }
    assert!(calls.iter().any(|call| {
        matches!(call, Call::SetSelectable { selectable: false, selection_color, .. }
            if *selection_color == 0x667788FF)
    }));
    assert!(calls.iter().any(|call| {
        matches!(call, Call::SetSelectable { selectable: false, selection_color, .. }
            if *selection_color == 0x778899FF)
    }));
}

#[test]
fn node_owned_theme_binding_survives_wrapper_drop_without_external_guard() {
    ffi::test::reset();
    let previous_theme = current_theme();
    let parent = column();
    {
        let themed = column();
        themed.bind_theme(|root, theme| {
            root.bg_color(theme.colors.accent);
        });
        parent.child(&themed);
    }
    Application::mount(parent);
    ffi::test::take_calls();

    let changed = generate_theme(false, 0x2468ACFF);
    use_custom_theme(changed.clone());
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| {
        matches!(call, Call::SetBgColor { color, .. } if *color == changed.colors.accent)
    }));

    use_custom_theme(previous_theme);
}

#[test]
fn node_owned_theme_binding_unsubscribes_when_the_retained_node_drops() {
    let previous_theme = current_theme();
    let calls = std::rc::Rc::new(std::cell::Cell::new(0));
    {
        let root = column();
        root.bind_theme({
            let calls = calls.clone();
            move |_root, _theme| calls.set(calls.get() + 1)
        });
        assert_eq!(calls.get(), 1);
    }

    use_custom_theme(generate_theme(false, 0x13579BFF));
    assert_eq!(calls.get(), 1);
    use_custom_theme(previous_theme);
}

#[test]
fn component_macro_delegates_to_the_designated_retained_root_without_wrapper_nodes() {
    ffi::test::reset();
    let component = ParityComponent {
        root: row().node_id("component-root").clone(),
        value: std::rc::Rc::new(std::cell::Cell::new(7)),
    };
    let state = component.value.clone();
    let root = ui! { column() { component.margin(1.0, 1.0, 1.0, 1.0) } };
    Application::mount(root);

    assert_eq!(state.get(), 7);
    let calls = ffi::test::take_calls();
    let component_handle = calls
        .iter()
        .find_map(|call| match call {
            Call::SetNodeId { handle, node_id } if node_id == "component-root" => Some(*handle),
            _ => None,
        })
        .expect("component root should be built directly");
    assert_eq!(
        calls
            .iter()
            .filter(|call| matches!(call, Call::CreateNode { handle, .. } if *handle == component_handle))
            .count(),
        1
    );
    assert!(calls.iter().any(
        |call| matches!(call, Call::NodeAddChild { child, .. } if *child == component_handle)
    ));
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
