#![allow(dead_code)]

use fui::ffi::{self, Call, NodeType};
use fui::prelude::*;
use fui::*;

#[derive(Clone)]
struct ParityComponent {
    root: FlexBox,
    value: std::rc::Rc<std::cell::Cell<i32>>,
}

fui_component!(ParityComponent => root, owner: value);

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
    assert_type::<Portal>();
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
    let _ = use_control_templates as fn(ControlTemplateSet);
}

#[test]
fn universal_node_surface_matrix_covers_every_public_retained_visual_type() {
    fn require<T: Node>() {}

    require::<FlexBox>();
    require::<TextNode>();
    require::<RichText>();
    require::<Grid>();
    require::<ImageNode>();
    require::<SvgNode>();
    require::<CustomDrawable>();
    require::<ScrollView>();
    require::<ScrollBox>();
    require::<VirtualList<FlexBox>>();
    require::<SelectionArea>();
    require::<AntiSelectionArea>();
    require::<Button>();
    require::<Checkbox>();
    require::<RadioButton>();
    require::<RadioGroup>();
    require::<Switch>();
    require::<Slider>();
    require::<ProgressBar>();
    require::<Dropdown>();
    require::<ComboBox>();
    require::<NavLink>();
    require::<Form>();
    require::<Popup>();
    require::<Dialog>();
    require::<ContextMenu>();
    require::<TextInput>();
    require::<TextArea>();

    // Public aliases are deliberately the same retained types, but keeping
    // them in the matrix prevents an export-only regression.
    require::<Text>();
    require::<Image>();
    require::<Svg>();
    require::<Portal>();
}

#[test]
fn layout_surface_matrix_matches_fui_as_flex_box_inheritance() {
    fn require<T: LayoutSurface>() {}

    require::<FlexBox>();
    require::<Grid>();
    require::<ImageNode>();
    require::<SvgNode>();
    require::<CustomDrawable>();
    require::<ScrollBox>();
    require::<VirtualList<FlexBox>>();
    require::<SelectionArea>();
    require::<AntiSelectionArea>();
    require::<Button>();
    require::<Checkbox>();
    require::<RadioButton>();
    require::<RadioGroup>();
    require::<Switch>();
    require::<Slider>();
    require::<ProgressBar>();
    require::<Dropdown>();
    require::<ComboBox>();
    require::<NavLink>();
    require::<Form>();
    require::<Popup>();
    require::<Dialog>();
    require::<ContextMenu>();
    require::<TextInput>();
    require::<TextArea>();
}

#[test]
fn box_style_surface_matrix_matches_fui_as_flex_box_inheritance() {
    fn require<T: BoxStyleSurface>() {}

    require::<FlexBox>();
    require::<Grid>();
    require::<ImageNode>();
    require::<SvgNode>();
    require::<CustomDrawable>();
    require::<ScrollBox>();
    require::<VirtualList<FlexBox>>();
    require::<SelectionArea>();
    require::<AntiSelectionArea>();
    require::<Button>();
    require::<Checkbox>();
    require::<RadioButton>();
    require::<RadioGroup>();
    require::<Switch>();
    require::<Slider>();
    require::<ProgressBar>();
    require::<Dropdown>();
    require::<ComboBox>();
    require::<NavLink>();
    require::<Form>();
    require::<Popup>();
    require::<Dialog>();
    require::<ContextMenu>();
    require::<TextInput>();
    require::<TextArea>();
}

#[test]
fn flex_layout_surface_matrix_matches_fui_as_flex_box_inheritance() {
    fn require<T: FlexLayoutSurface>() {}

    require::<FlexBox>();
    require::<Grid>();
    require::<ImageNode>();
    require::<SvgNode>();
    require::<CustomDrawable>();
    require::<ScrollBox>();
    require::<VirtualList<FlexBox>>();
    require::<SelectionArea>();
    require::<AntiSelectionArea>();
    require::<Button>();
    require::<Checkbox>();
    require::<RadioButton>();
    require::<RadioGroup>();
    require::<Switch>();
    require::<Slider>();
    require::<ProgressBar>();
    require::<Dropdown>();
    require::<ComboBox>();
    require::<NavLink>();
    require::<Form>();
    require::<Popup>();
    require::<Dialog>();
    require::<ContextMenu>();
    require::<TextInput>();
    require::<TextArea>();
}

#[test]
fn child_container_surface_matrix_matches_fui_as_flex_box_inheritance() {
    fn require<T: ChildContainerSurface>() {}

    require::<FlexBox>();
    require::<Grid>();
    require::<ImageNode>();
    require::<SvgNode>();
    require::<CustomDrawable>();
    require::<ScrollBox>();
    require::<VirtualList<FlexBox>>();
    require::<SelectionArea>();
    require::<AntiSelectionArea>();
    require::<Button>();
    require::<Checkbox>();
    require::<RadioButton>();
    require::<RadioGroup>();
    require::<Switch>();
    require::<Slider>();
    require::<ProgressBar>();
    require::<Dropdown>();
    require::<ComboBox>();
    require::<NavLink>();
    require::<Form>();
    require::<Popup>();
    require::<Dialog>();
    require::<ContextMenu>();
    require::<TextInput>();
    require::<TextArea>();
}

#[test]
fn text_surface_matrix_matches_fui_as_text_inheritance() {
    fn require<T: TextSurface>() {}

    require::<TextNode>();
    require::<RichText>();
    require::<Text>();
}

#[test]
fn editable_text_surface_matrix_matches_fui_as_text_input_core_inheritance() {
    fn require<T: TextEditorSurface>() {}

    require::<TextInput>();
    require::<TextArea>();
}

#[test]
fn specialized_activation_matrix_keeps_control_click_separate_from_pointer_click() {
    fn require_clickable<T: Clickable>() {}

    require_clickable::<Button>();
    require_clickable::<Checkbox>();
    require_clickable::<RadioButton>();
    require_clickable::<Switch>();

    let action = button("Action");
    action.on_click(|_| {});

    checkbox("Checkbox").on_click(|_| {});
    radio_button("Radio").on_click(|_| {});
    switch("Switch").on_click(|_| {});

    // Raw pointer clicks belong to every Node and carry routed pointer args.
    Node::on_pointer_click(&action, |event| event.handled = true);
    Node::on_pointer_double_click(&action, |event| event.handled = true);
    Node::on_pointer_triple_click(&action, |event| event.handled = true);

    // NavLink activation remains navigation-specific rather than pretending to
    // be Button activation.
    nav_link("/next").on_navigate(|_| {});
}

#[test]
fn universal_control_host_style_surface_compiles_cohesively() {
    fn accepts_node<T: Node>(control: &T) {
        let _ = control.handle();
    }
    let button = button("Button");
    button
        .width(180.0, Unit::Pixel)
        .padding(18.0, 10.0, 18.0, 10.0)
        .corner_radius(12.0)
        .border(1.0, 0xD1D5DBFF)
        .bg_color(0x2563EBFF)
        .drop_shadow(0x00000040, 0.0, 4.0, 12.0, 0.0);
    accepts_node(&button);
    accepts_node(&checkbox("Checkbox"));
    accepts_node(&radio_button("Radio"));
    accepts_node(&switch("Switch"));
    accepts_node(&slider());
    accepts_node(&dropdown());
    accepts_node(&combo_box());
    accepts_node(&progress_bar());
    accepts_node(&text_input());
    accepts_node(&text_area());
    accepts_node(&nav_link("/next"));

    let _ = SliderSizing::new().thumb_size(16.0).track_thickness(4.0);
    let _ = SliderColors::new()
        .track(0xCBD5E1FF)
        .fill(0x2563EBFF)
        .thumb(0xFFFFFFFF);
    let _ = LabeledControlSizing::new()
        .indicator_size(18.0)
        .label_font_size(14.0);
    let _ = LabeledControlColors::new()
        .accent(0x2563EBFF)
        .border(0x94A3B8FF);
}

#[test]
fn every_visual_control_exposes_the_universal_inherited_surface() {
    fn assert_surface<T: Node + FlexBoxSurface>(control: &T) {
        control
            .width(120.0, Unit::Pixel)
            .height(40.0, Unit::Pixel)
            .margin(1.0, 2.0, 3.0, 4.0)
            .padding(5.0, 6.0, 7.0, 8.0)
            .bg_color(0x102030FF)
            .corner_radius(6.0)
            .border(1.0, 0x405060FF)
            .cursor(CursorStyle::Pointer)
            .semantic_label("Universal surface")
            .on_pointer_down(|event| event.handled = true)
            .on_key_down(|event| event.handled = true);
    }

    assert_surface(&button("Button"));
    assert_surface(&checkbox("Checkbox"));
    assert_surface(&radio_button("Radio"));
    assert_surface(&switch("Switch"));
    assert_surface(&slider());
    assert_surface(&dropdown());
    assert_surface(&combo_box());
    assert_surface(&progress_bar());
    assert_surface(&text_input());
    assert_surface(&text_area());
    assert_surface(&nav_link("/next"));
    assert_surface(&selection_area());
    assert_surface(&anti_selection_area());
    assert_surface(&form());
    assert_surface(&radio_group());
    assert_surface(&popup());
    assert_surface(&dialog("Title", "Body"));
    assert_surface(&context_menu(Vec::<MenuItem>::new()));
    assert_surface(&CustomDrawable::new(|_| {}));
}

#[test]
fn every_composed_flex_box_control_exposes_the_complete_inherited_surface() {
    fn complete<T: Node + FlexBoxSurface>(control: &T) {
        LayoutSurface::width(control, 160.0, Unit::Pixel);
        LayoutSurface::height(control, 48.0, Unit::Pixel);
        BoxStyleSurface::padding(control, 1.0, 2.0, 3.0, 4.0);
        BoxStyleSurface::bg_color(control, 0x102030FF);
        FlexLayoutSurface::flex_direction(control, FlexDirection::Column);
        ChildContainerSurface::child(control, &text("user child"));
    }

    complete(&button("Button"));
    complete(&checkbox("Checkbox"));
    complete(&radio_button("Radio"));
    complete(&switch("Switch"));
    complete(&slider());
    complete(&progress_bar());
    complete(&dropdown());
    complete(&combo_box());
    complete(&nav_link("/next"));
    complete(&radio_group());
    complete(&form());
    complete(&popup());
    complete(&dialog("Title", "Body"));
    complete(&context_menu(Vec::<MenuItem>::new()));
    complete(&selection_area());
    complete(&anti_selection_area());
    complete(&text_input());
    complete(&text_area());
    complete(&CustomDrawable::new(|_| {}));
}

#[test]
fn composed_control_user_children_preserve_presenter_children_and_order() {
    ffi::test::reset();
    let control = button("Presenter-owned label");
    let presenter_child_count = control.child_count();
    assert!(presenter_child_count > 0);

    let user_child = text("User child");
    ChildContainerSurface::child(&control, &user_child);
    assert_eq!(control.child_count(), presenter_child_count + 1);

    Application::mount(control.clone());
    let control_handle = control.handle().raw();
    let user_child_handle = user_child.handle().raw();
    let calls = ffi::test::take_calls();
    let user_child_position = calls
        .iter()
        .position(|call| {
            matches!(
                call,
                Call::NodeAddChild { parent, child }
                    if *parent == control_handle && *child == user_child_handle
            )
        })
        .expect("user child should be attached to the composed control root");
    assert!(calls[..user_child_position].iter().any(|call| {
        matches!(
            call,
            Call::NodeAddChild { parent, child }
                if *parent == control_handle && *child != user_child_handle
        )
    }));

    assert!(Node::remove_child(&control, &user_child));
    assert_eq!(control.child_count(), presenter_child_count);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::NodeRemoveChild { parent, child }
            if *parent == control_handle && *child == user_child_handle
    )));
    assert!(!calls.iter().any(|call| matches!(
        call,
        Call::NodeRemoveChild { parent, child }
            if *parent == control_handle && *child != user_child_handle
    )));
}

#[test]
fn simple_composed_control_appends_and_removes_only_the_requested_user_child() {
    ffi::test::reset();
    let control = selection_area();
    let user_child = text("Selectable user content");
    ChildContainerSurface::child(&control, &user_child);
    assert_eq!(control.child_count(), 1);

    Application::mount(control.clone());
    let control_handle = control.handle().raw();
    let user_child_handle = user_child.handle().raw();
    ffi::test::take_calls();

    assert!(Node::remove_child(&control, &user_child));
    assert_eq!(control.child_count(), 0);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::NodeRemoveChild { parent, child }
            if *parent == control_handle && *child == user_child_handle
    )));
}

#[test]
fn popup_child_surface_routes_user_content_to_the_presented_panel() {
    let control = popup();
    let panel = control.surface();
    let root_child_count = control.child_count();
    let panel_child_count = panel.child_count();
    let user_child = text("Popup content");

    ChildContainerSurface::child(&control, &user_child);

    assert_eq!(control.child_count(), root_child_count);
    assert_eq!(panel.child_count(), panel_child_count + 1);
    assert!(Node::remove_child(&panel, &user_child));
    assert_eq!(panel.child_count(), panel_child_count);
}

#[test]
fn custom_drawable_exposes_focus_keyboard_pointer_layout_style_theme_and_children() {
    let drawable = CustomDrawable::new(|_| {});
    let child = text("Drawable child");
    drawable
        .focusable(true, 0)
        .on_focus_changed(|_| {})
        .on_key_down(|event| event.handled = true)
        .on_pointer_down(|event| event.handled = true);
    LayoutSurface::width(&drawable, 240.0, Unit::Pixel);
    BoxStyleSurface::bg_color(&drawable, 0x102030FF);
    FlexLayoutSurface::align_items(&drawable, AlignItems::Center);
    ChildContainerSurface::child(&drawable, &child);
    drawable.bind_theme(|control, theme| {
        BoxStyleSurface::border(control, 1.0, theme.colors.border);
    });

    assert_eq!(drawable.child_count(), 1);
}

#[test]
fn cohesive_flex_box_capability_traits_compile_independently_and_compositely() {
    fn layout<T: LayoutSurface>(value: &T) {
        value
            .width_len(px(120.0))
            .height_len(px(48.0))
            .margin(1.0, 2.0, 3.0, 4.0)
            .position_absolute()
            .position(5.0, 6.0);
    }
    fn style<T: BoxStyleSurface>(value: &T) {
        value
            .padding(1.0, 2.0, 3.0, 4.0)
            .bg_color(0x102030FF)
            .corner_radius(6.0)
            .border(1.0, 0x405060FF)
            .clip_to_bounds(true);
    }
    fn flex<T: FlexLayoutSurface>(value: &T) {
        value
            .flex_direction(FlexDirection::Column)
            .flex_wrap(FlexWrap::Wrap)
            .justify_content(JustifyContent::Center)
            .align_items(AlignItems::Center);
    }
    fn children<T: ChildContainerSurface>(value: &T) {
        value
            .child(&text("one"))
            .children(vec![Child::from(text("two"))]);
    }
    fn complete<T: FlexBoxSurface>(value: &T) {
        layout(value);
        style(value);
        flex(value);
        children(value);
    }

    complete(&flex_box());
    complete(&button("Button"));
}

#[test]
fn primitive_flex_box_surface_matrix_matches_fui_as_inheritance() {
    fn complete<T: FlexBoxSurface + ThemeBindable>(value: &T) {
        LayoutSurface::width(value, 180.0, Unit::Pixel);
        BoxStyleSurface::padding(value, 1.0, 2.0, 3.0, 4.0);
        FlexLayoutSurface::flex_direction(value, FlexDirection::Column);
        ChildContainerSurface::child(value, &text("surface child"));
        value.bind_theme(|_, _| {});
    }

    complete(&grid());
    complete(&image(1));
    complete(&svg(1));
    let virtual_list = VirtualList::<FlexBox>::new(8, 24.0);
    virtual_list.on_bind_item(|_, _| {});
    complete(&virtual_list);
    assert!(virtual_list.is_selection_barrier());
    virtual_list.render();
    complete(&scroll_box());
    let portal_node: Portal = portal();
    complete(&portal_node);

    let viewport = scroll_view();
    viewport
        .width(180.0, Unit::Pixel)
        .height(120.0, Unit::Pixel)
        .fill_width()
        .fill_height()
        .child(&text("viewport child"));
}

#[test]
fn scroll_view_retains_its_explicit_fui_as_sizing_surface_before_and_after_build() {
    ffi::test::reset();
    let viewport = scroll_view();
    viewport
        .fill_width_percent(72.0)
        .fill_height_percent(64.0)
        .min_width(80.0, Unit::Pixel)
        .max_width(480.0, Unit::Pixel)
        .min_height(60.0, Unit::Pixel)
        .max_height(360.0, Unit::Pixel)
        .flex_basis(140.0)
        .friction(0.92)
        .scroll_content_size(900.0, 1200.0);
    assert!(viewport.is_vertical_scroll_enabled());
    assert!(viewport.is_horizontal_scroll_enabled());
    let _ = viewport.scroll_state();

    Application::mount(viewport.clone());
    let handle = viewport.handle().raw();
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetFillWidthPercent { handle: target, percent: 72.0 } if *target == handle
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetFillHeightPercent { handle: target, percent: 64.0 } if *target == handle
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetMinWidth { handle: target, value: 80.0, unit_enum }
            if *target == handle && *unit_enum == Unit::Pixel as u32
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetMaxHeight { handle: target, value: 360.0, unit_enum }
            if *target == handle && *unit_enum == Unit::Pixel as u32
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetFlexBasis { handle: target, basis: 140.0 } if *target == handle
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetScrollFriction { handle: target, friction: 0.92 } if *target == handle
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetScrollContentSize {
            handle: target,
            content_width: 900.0,
            content_height: 1200.0,
        }
            if *target == handle
    )));

    viewport
        .width(320.0, Unit::Pixel)
        .height(180.0, Unit::Pixel)
        .flex_basis(160.0)
        .friction(0.88)
        .scroll_content_size(1000.0, 1400.0);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetWidth { handle: target, value: 320.0, unit_enum }
            if *target == handle && *unit_enum == Unit::Pixel as u32
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetHeight { handle: target, value: 180.0, unit_enum }
            if *target == handle && *unit_enum == Unit::Pixel as u32
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetFlexBasis { handle: target, basis: 160.0 } if *target == handle
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetScrollFriction { handle: target, friction: 0.88 } if *target == handle
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetScrollContentSize {
            handle: target,
            content_width: 1000.0,
            content_height: 1400.0,
        }
            if *target == handle
    )));
}

#[test]
fn primitive_flex_box_surfaces_retain_and_mutate_the_specialized_native_node() {
    ffi::test::reset();
    let root = flex_box();
    let grid_node = grid();
    let image_node = image(7);
    let svg_node = svg(9);
    let svg_child = text("svg child");

    LayoutSurface::width(&grid_node, 210.0, Unit::Pixel);
    LayoutSurface::margin(&grid_node, 1.0, 2.0, 3.0, 4.0);
    BoxStyleSurface::padding(&grid_node, 5.0, 6.0, 7.0, 8.0);
    BoxStyleSurface::corners(&grid_node, 9.0, 10.0, 11.0, 12.0);
    LayoutSurface::position(&grid_node, 13.0, 14.0);
    Grid::shared_size_scope(&grid_node, true);

    LayoutSurface::width(&image_node, 220.0, Unit::Pixel);
    BoxStyleSurface::bg_color(&image_node, 0x102030FF);

    LayoutSurface::height(&svg_node, 90.0, Unit::Pixel);
    FlexLayoutSurface::flex_direction(&svg_node, FlexDirection::Column);
    ChildContainerSurface::child(&svg_node, &svg_child);

    root.children(vec![
        Child::from(grid_node.clone()),
        Child::from(image_node.clone()),
        Child::from(svg_node.clone()),
    ]);
    Application::mount(root);

    let grid_handle = grid_node.handle().raw();
    let image_handle = image_node.handle().raw();
    let svg_handle = svg_node.handle().raw();
    let svg_child_handle = svg_child.handle().raw();
    let calls = ffi::test::take_calls();

    assert!(calls.iter().any(|call| matches!(
        call,
        Call::CreateNode { handle, node_type }
            if *handle == grid_handle && *node_type == NodeType::Grid as u32
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::CreateNode { handle, node_type }
            if *handle == image_handle && *node_type == NodeType::Image as u32
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::CreateNode { handle, node_type }
            if *handle == svg_handle && *node_type == NodeType::Svg as u32
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetPadding { handle, left, top, right, bottom }
            if *handle == grid_handle
                && *left == 5.0
                && *top == 6.0
                && *right == 7.0
                && *bottom == 8.0
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetMargin { handle, left, top, right, bottom }
            if *handle == grid_handle
                && *left == 1.0
                && *top == 2.0
                && *right == 3.0
                && *bottom == 4.0
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetBoxStyle {
            handle,
            radius_tl,
            radius_tr,
            radius_br,
            radius_bl,
            ..
        } if *handle == grid_handle
            && *radius_tl == 9.0
            && *radius_tr == 10.0
            && *radius_br == 11.0
            && *radius_bl == 12.0
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetPosition { handle, left, top, .. }
            if *handle == grid_handle && *left == 13.0 && *top == 14.0
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetIsSharedSizeScope { handle, is_scope: true } if *handle == grid_handle
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetBgColor { handle, color: 0x102030FF } if *handle == image_handle
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetFlexDirection { handle, dir_enum }
            if *handle == svg_handle && *dir_enum == FlexDirection::Column as u32
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::NodeAddChild { parent, child }
            if *parent == svg_handle && *child == svg_child_handle
    )));
    for handle in [grid_handle, image_handle, svg_handle] {
        assert_eq!(
            calls
                .iter()
                .filter(|call| matches!(call, Call::CreateNode { handle: created, .. } if *created == handle))
                .count(),
            1
        );
    }

    BoxStyleSurface::bg_color(&grid_node, 0x506070FF);
    Grid::shared_size_scope(&grid_node, false);
    LayoutSurface::margin(&image_node, 15.0, 16.0, 17.0, 18.0);
    LayoutSurface::position(&svg_node, 19.0, 20.0);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetBgColor { handle, color: 0x506070FF } if *handle == grid_handle
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetIsSharedSizeScope { handle, is_scope: false } if *handle == grid_handle
    )));
    assert!(
        calls.iter().any(|call| matches!(
            call,
            Call::SetMargin { handle, left, top, right, bottom }
                if *handle == image_handle
                    && *left == 15.0
                    && *top == 16.0
                    && *right == 17.0
                    && *bottom == 18.0
        )),
        "post-build primitive calls: {calls:#?}"
    );
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetPosition { handle, left, top, .. }
            if *handle == svg_handle && *left == 19.0 && *top == 20.0
    )));
}

#[test]
fn capability_traits_retain_before_build_and_mutate_the_same_node_after_build() {
    ffi::test::reset();
    let root = flex_box();
    let first = text("first");
    LayoutSurface::width(&root, 120.0, Unit::Pixel);
    BoxStyleSurface::bg_color(&root, 0x102030FF);
    FlexLayoutSurface::flex_direction(&root, FlexDirection::Column);
    ChildContainerSurface::child(&root, &first);

    Application::mount(root.clone());
    let handle = root.handle().raw();
    let first_handle = first.handle().raw();
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetWidth { handle: target, value: 120.0, unit_enum }
            if *target == handle && *unit_enum == Unit::Pixel as u32
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetBgColor { handle: target, color: 0x102030FF } if *target == handle
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetFlexDirection { handle: target, dir_enum }
            if *target == handle && *dir_enum == FlexDirection::Column as u32
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::NodeAddChild { parent, child }
            if *parent == handle && *child == first_handle
    )));
    assert_eq!(
        calls
            .iter()
            .filter(
                |call| matches!(call, Call::CreateNode { handle: target, .. } if *target == handle)
            )
            .count(),
        1
    );
    assert_eq!(
        calls
            .iter()
            .filter(|call| matches!(call, Call::CreateNode { handle: target, .. } if *target == first_handle))
            .count(),
        1
    );

    let second = text("second");
    LayoutSurface::width(&root, 240.0, Unit::Pixel);
    BoxStyleSurface::bg_color(&root, 0x506070FF);
    FlexLayoutSurface::flex_direction(&root, FlexDirection::Row);
    ChildContainerSurface::child(&root, &second);
    let second_handle = second.handle().raw();
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetWidth { handle: target, value: 240.0, unit_enum }
            if *target == handle && *unit_enum == Unit::Pixel as u32
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetBgColor { handle: target, color: 0x506070FF } if *target == handle
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetFlexDirection { handle: target, dir_enum }
            if *target == handle && *dir_enum == FlexDirection::Row as u32
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::NodeAddChild { parent, child }
            if *parent == handle && *child == second_handle
    )));
}

#[test]
fn every_node_exposes_the_universal_node_surface() {
    fn assert_surface<T: Node>(node: &T) {
        node.node_id("node-surface")
            .semantic_role(SemanticRole::List)
            .semantic_label("Node surface")
            .semantic_checked(SemanticCheckedState::True)
            .semantic_disabled(false)
            .clear_semantic_disabled()
            .semantic_selected(true)
            .clear_semantic_selected()
            .semantic_expanded(true)
            .clear_semantic_expanded()
            .semantic_value_range(1.0, 0.0, 2.0)
            .clear_semantic_value_range()
            .semantic_orientation(Orientation::Horizontal)
            .enabled(true)
            .visibility(Visibility::Normal)
            .cursor(CursorStyle::Pointer)
            .clear_cursor()
            .focusable(true, 0)
            .focus_now()
            .on_pointer_click(|event| event.handled = true);
        let _ = node.child_count();
        let _ = node.is_enabled();
        let _ = node.is_visible();
        let _ = node.cursor_style();
    }

    assert_surface(&flex_box());
    assert_surface(&text("Text"));
    assert_surface(&image(1));
    assert_surface(&svg(1));
    assert_surface(&grid());
}

#[test]
fn universal_node_semantics_retain_before_build_and_mutate_after_build() {
    ffi::test::reset();
    let node = text("Semantic node");
    node.semantic_checked(SemanticCheckedState::True)
        .semantic_disabled(true)
        .semantic_selected(true)
        .semantic_expanded(true)
        .semantic_value_range(4.0, 1.0, 9.0)
        .semantic_orientation(Orientation::Vertical)
        .request_semantic_announcement()
        .focus_now();

    assert!(ffi::test::take_calls().is_empty());

    Application::mount(node.clone());
    let handle = node.handle().raw();
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticChecked { handle: target, checked_state_enum }
            if *target == handle && *checked_state_enum == SemanticCheckedState::True as u32
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticDisabled { handle: target, has_disabled: true, disabled: true }
            if *target == handle
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticSelected { handle: target, has_selected: true, selected: true }
            if *target == handle
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticExpanded { handle: target, has_expanded: true, is_expanded: true }
            if *target == handle
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticValueRange {
            handle: target,
            has_value_range: true,
            value_now: 4.0,
            value_min: 1.0,
            value_max: 9.0,
        } if *target == handle
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticOrientation { handle: target, orientation_enum }
            if *target == handle && *orientation_enum == Orientation::Vertical as u32
    )));
    assert!(!calls.iter().any(|call| matches!(
        call,
        Call::RequestSemanticAnnouncement { .. } | Call::RequestFocus { .. }
    )));

    node.semantic_checked(SemanticCheckedState::None)
        .clear_semantic_disabled()
        .clear_semantic_selected()
        .clear_semantic_expanded()
        .clear_semantic_value_range()
        .request_semantic_announcement()
        .focus_now();
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticChecked { handle: target, checked_state_enum }
            if *target == handle && *checked_state_enum == SemanticCheckedState::None as u32
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticDisabled { handle: target, has_disabled: false, disabled: false }
            if *target == handle
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticSelected { handle: target, has_selected: false, selected: false }
            if *target == handle
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticExpanded { handle: target, has_expanded: false, is_expanded: false }
            if *target == handle
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSemanticValueRange { handle: target, has_value_range: false, .. }
            if *target == handle
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::RequestSemanticAnnouncement { handle: target } if *target == handle
    )));
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::RequestFocus { handle: target } if *target == handle
    )));
}

#[test]
fn control_configuration_uses_direct_values_and_explicit_clear_methods() {
    let button = button("Button");
    button
        .colors(ButtonColors::new())
        .clear_colors()
        .clear_template();

    let checkbox = checkbox("Checkbox");
    checkbox
        .sizing(LabeledControlSizing::new())
        .colors(LabeledControlColors::new())
        .clear_sizing()
        .clear_colors()
        .clear_template();

    let radio = radio_button("Radio");
    radio
        .sizing(LabeledControlSizing::new())
        .colors(LabeledControlColors::new())
        .clear_sizing()
        .clear_colors()
        .clear_template();

    let toggle = switch("Switch");
    toggle
        .sizing(LabeledControlSizing::new())
        .colors(LabeledControlColors::new())
        .clear_sizing()
        .clear_colors()
        .clear_template();

    let slider = slider();
    slider
        .sizing(SliderSizing::new())
        .colors(SliderColors::new())
        .clear_sizing()
        .clear_colors()
        .clear_template();

    let progress = progress_bar();
    progress
        .sizing(ProgressBarSizing::new().length(220.0).thickness(14.0))
        .colors(ProgressBarColors::new().track(0xCBD5E1FF).fill(0x2563EBFF))
        .clear_sizing()
        .clear_colors();

    let dropdown = dropdown();
    dropdown
        .sizing(DropdownSizing::new())
        .colors(DropdownColors::new())
        .clear_sizing()
        .clear_colors()
        .clear_field_template()
        .clear_chevron_template()
        .clear_option_row_template();

    let combo = combo_box();
    combo
        .sizing(DropdownSizing::new())
        .colors(DropdownColors::new())
        .clear_sizing()
        .clear_colors()
        .clear_chevron_template()
        .clear_option_row_template();

    let input = text_input();
    input
        .colors(TextInputColors::new())
        .clear_colors()
        .clear_template();
    let area = text_area();
    area.colors(TextInputColors::new())
        .clear_colors()
        .clear_template();

    use_control_templates(ControlTemplateSet::default());
    clear_control_templates();
}

#[test]
fn overlay_controls_use_cohesive_appearance_recipes() {
    let surface = SurfaceAppearance::new()
        .background(0xFFFFFFFF)
        .background_blur(8.0)
        .border(Border::solid(1.0, 0xD1D5DBFF))
        .corners(Corners::all(16.0))
        .shadow(Shadow::new(0x00000040, 0.0, 8.0, 20.0, 0.0));
    let backdrop = OverlayBackdropAppearance::new()
        .color(0x00000066)
        .blur(12.0);

    popup()
        .appearance(
            PopupAppearance::new()
                .panel(surface.clone())
                .backdrop(backdrop.clone()),
        )
        .clear_appearance();
    dialog("Title", "Body")
        .appearance(
            DialogAppearance::new()
                .card(surface.clone())
                .backdrop(backdrop.clone()),
        )
        .clear_appearance();
    context_menu(Vec::<MenuItem>::new())
        .appearance(
            ContextMenuAppearance::new()
                .width(240.0)
                .panel(surface)
                .backdrop(backdrop)
                .item(
                    ContextMenuItemAppearance::new()
                        .height(36.0)
                        .padding(EdgeInsets::new(12.0, 6.0, 12.0, 6.0))
                        .background(0x00000000)
                        .hover_background(0xE2E8F0FF)
                        .text_color(0x0F172AFF)
                        .corners(Corners::all(8.0))
                        .font_weight(FontWeight::Regular)
                        .font_style(FontStyle::Normal)
                        .font_size(14.0),
                )
                .separator_color(0xCBD5E1FF),
        )
        .clear_appearance();
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
fn rich_text_macro_builds_literal_dynamic_and_prebuilt_spans() {
    ffi::test::reset();

    let dynamic = String::from("dynamic");
    let punctuation = span("!").underline();
    let node = rich_text![
        "Literal ".italic(),
        { dynamic }.bold().text_color(rgb(0x12, 0x34, 0x56)),
        span => punctuation,
    ];
    node.build();

    let calls = ffi::test::take_calls();
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::SetText { text, .. } if text == "Literal dynamic!")));
    assert!(calls
        .iter()
        .any(|call| matches!(call, Call::SetTextStyleRuns { run_count: 3, .. })));
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
            TextNode::new("Core text")
                .selectable(false)
                .selection_color(0x778899FF),
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
fn text_and_rich_text_expose_the_complete_public_text_surface() {
    fn require_text_surface<T: TextSurface + ThemeBindable + Node>() {}

    require_text_surface::<TextNode>();
    require_text_surface::<RichText>();
}

#[test]
fn every_public_retained_visual_type_exposes_theme_binding() {
    fn require<T: ThemeBindable>() {}

    require::<FlexBox>();
    require::<TextNode>();
    require::<RichText>();
    require::<Grid>();
    require::<ImageNode>();
    require::<SvgNode>();
    require::<CustomDrawable>();
    require::<ScrollView>();
    require::<ScrollBox>();
    require::<VirtualList>();
    require::<SelectionArea>();
    require::<AntiSelectionArea>();
    require::<Button>();
    require::<Checkbox>();
    require::<RadioButton>();
    require::<RadioGroup>();
    require::<Switch>();
    require::<Slider>();
    require::<ProgressBar>();
    require::<Dropdown>();
    require::<ComboBox>();
    require::<NavLink>();
    require::<Form>();
    require::<Popup>();
    require::<Dialog>();
    require::<ContextMenu>();
    require::<TextInput>();
    require::<TextArea>();
}

#[test]
fn retained_scroll_view_theme_binding_survives_wrapper_drop_once() {
    ffi::test::reset();
    let previous_theme = current_theme();
    let invocations = std::rc::Rc::new(std::cell::Cell::new(0));
    let root = column();
    let view = ScrollView::new();
    view.bind_theme({
        let invocations = invocations.clone();
        move |control, _theme| {
            invocations.set(invocations.get() + 1);
            control.smooth_scrolling(false);
        }
    });
    root.child(&view);
    Application::mount(root);
    let view_handle = view.handle().raw();
    drop(view);
    ffi::test::take_calls();

    use_custom_theme(generate_theme(false, 0x271828FF));
    assert_eq!(invocations.get(), 2);
    let calls = ffi::test::take_calls();
    assert!(calls.iter().any(|call| matches!(
        call,
        Call::SetSmoothScrolling { handle, smooth_scrolling }
            if *handle == view_handle && !*smooth_scrolling
    )));

    use_custom_theme(previous_theme);
    Application::unmount();
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
fn component_macro_retains_declared_owner_for_inline_child_lifetime() {
    let parent = column();
    let owner = std::rc::Rc::new(std::cell::Cell::new(7));
    let weak_owner = std::rc::Rc::downgrade(&owner);
    parent.child(&ParityComponent {
        root: row(),
        value: owner.clone(),
    });
    drop(owner);

    assert_eq!(weak_owner.upgrade().map(|value| value.get()), Some(7));
    drop(parent);
    assert!(weak_owner.upgrade().is_none());
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
            .on_pointer_click(|event| {
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
