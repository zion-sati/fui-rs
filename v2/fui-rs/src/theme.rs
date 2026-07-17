use crate::color::{
    color_alpha, color_blue, color_green, color_red, mix_color, rgb, rgba, with_alpha,
};
use crate::generated::framework_host_services;
use crate::signal::{Callback, Signal, SubscriptionGuard};
use crate::typography::{FontFamily, FontStack};
use std::cell::RefCell;
use std::rc::Rc;

const DEFAULT_ACCENT_COLOR: u32 = rgb(0x25, 0x63, 0xeb);
const WHITE: u32 = rgb(0xff, 0xff, 0xff);
const BLACK: u32 = rgb(0x00, 0x00, 0x00);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Colors {
    pub background: u32,
    pub surface: u32,
    pub text_primary: u32,
    pub text_muted: u32,
    pub text_on_accent: u32,
    pub accent: u32,
    pub accent_pressed: u32,
    pub accent_hovered: u32,
    pub border: u32,
    pub selection: u32,
    pub scrollbar_track: u32,
    pub scrollbar_thumb: u32,
    pub dialog_backdrop: u32,
    pub dialog_shadow: u32,
    pub panel_shadow: u32,
    pub focus_ring: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spacing {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Fonts {
    pub body_stack: FontStack,
    pub heading_stack: FontStack,
    pub mono_stack: FontStack,
    pub mono_bold_stack: FontStack,
    pub body_family: FontFamily,
    pub heading_family: FontFamily,
    pub mono_family: FontFamily,
    pub size_body: f32,
    pub size_heading: f32,
    pub size_mono: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ContextMenuItemTheme {
    pub background: u32,
    pub hover_background: u32,
    pub text_color: u32,
    pub corner_radius: f32,
    pub font_family: FontFamily,
    pub font_size: f32,
    pub height: f32,
    pub padding_left: f32,
    pub padding_top: f32,
    pub padding_right: f32,
    pub padding_bottom: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ContextMenuTheme {
    pub panel_background: u32,
    pub panel_border_color: u32,
    pub panel_shadow_color: u32,
    pub panel_corner_radius: f32,
    pub separator_color: u32,
    pub shadow_offset_y: f32,
    pub shadow_blur: f32,
    pub shadow_spread: f32,
    pub item: ContextMenuItemTheme,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ToolTipTheme {
    pub panel_background: u32,
    pub panel_border_color: u32,
    pub panel_shadow_color: u32,
    pub panel_corner_radius: f32,
    pub text_color: u32,
    pub font_family: FontFamily,
    pub font_size: f32,
    pub max_width: f32,
    pub padding_left: f32,
    pub padding_top: f32,
    pub padding_right: f32,
    pub padding_bottom: f32,
    pub shadow_offset_y: f32,
    pub shadow_blur: f32,
    pub shadow_spread: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub colors: Colors,
    pub spacing: Spacing,
    pub fonts: Fonts,
    pub context_menu: ContextMenuTheme,
    pub tool_tip: ToolTipTheme,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ThemeSource {
    System,
    Custom,
}

struct ThemeState {
    signal: Signal<Theme>,
    theme_source: ThemeSource,
    system_dark_mode: bool,
    system_accent_color: u32,
    current_dark_mode: bool,
}

thread_local! {
    static THEME_STATE: RefCell<ThemeState> = RefCell::new(ThemeState {
        signal: Signal::new(default_dark_theme()),
        theme_source: ThemeSource::System,
        system_dark_mode: true,
        system_accent_color: DEFAULT_ACCENT_COLOR,
        current_dark_mode: true,
    });
}

const DEFAULT_SPACING: Spacing = Spacing {
    xs: 4.0,
    sm: 8.0,
    md: 16.0,
    lg: 24.0,
    xl: 32.0,
};
fn default_fonts() -> Fonts {
    let body_stack = FontStack::from_id(1).fallback_face(crate::typography::FontFace::new(3));
    let heading_stack = FontStack::from_id(2).fallback_face(crate::typography::FontFace::new(3));
    let mono_stack = FontStack::from_id(7)
        .fallback_face(crate::typography::FontFace::new(4))
        .fallback_face(crate::typography::FontFace::new(3));
    let mono_bold_stack = FontStack::from_id(8)
        .fallback_face(crate::typography::FontFace::new(4))
        .fallback_face(crate::typography::FontFace::new(3));
    Fonts {
        body_stack: body_stack.clone(),
        heading_stack: heading_stack.clone(),
        mono_stack: mono_stack.clone(),
        mono_bold_stack: mono_bold_stack.clone(),
        body_family: FontFamily::regular_bold_stacks(body_stack.clone(), heading_stack.clone())
            .italic_stack(FontStack::from_id(5))
            .bold_italic_stack(FontStack::from_id(6)),
        heading_family: FontFamily::regular_bold_stacks(
            heading_stack.clone(),
            heading_stack.clone(),
        ),
        mono_family: FontFamily::regular_bold_stacks(mono_stack.clone(), mono_bold_stack.clone()),
        size_body: 16.0,
        size_heading: 24.0,
        size_mono: 15.0,
    }
}

fn normalize_accent_color(color: u32) -> u32 {
    if color == 0 {
        return DEFAULT_ACCENT_COLOR;
    }
    let alpha = color_alpha(color);
    if alpha == 0 {
        return with_alpha(color, 0xff);
    }
    color
}

fn pick_accent_foreground(accent: u32) -> u32 {
    let brightness = color_red(accent) as f32 * 0.2126
        + color_green(accent) as f32 * 0.7152
        + color_blue(accent) as f32 * 0.0722;
    if brightness < 160.0 {
        WHITE
    } else {
        BLACK
    }
}

fn estimate_theme_dark(theme: &Theme) -> bool {
    let background = theme.colors.background;
    let luminance = color_red(background) as f32 * 0.2126
        + color_green(background) as f32 * 0.7152
        + color_blue(background) as f32 * 0.0722;
    luminance < 128.0
}

pub fn generate_theme(is_dark: bool, accent_color: u32) -> Theme {
    let fonts = default_fonts();
    let accent = normalize_accent_color(accent_color);
    let background = if is_dark {
        rgba(0x04, 0x0a, 0x14, 0xff)
    } else {
        rgba(0xf8, 0xfa, 0xfc, 0xff)
    };
    let surface = if is_dark {
        rgba(0x0f, 0x17, 0x28, 0xff)
    } else {
        WHITE
    };
    let text_primary = if is_dark {
        rgba(0xf8, 0xfa, 0xfc, 0xff)
    } else {
        rgba(0x0f, 0x17, 0x2a, 0xff)
    };
    let text_muted = if is_dark {
        rgba(0x94, 0xa3, 0xb8, 0xff)
    } else {
        rgba(0x47, 0x55, 0x69, 0xff)
    };
    let text_on_accent = pick_accent_foreground(accent);
    let border = if is_dark {
        rgba(0x24, 0x3b, 0x53, 0xff)
    } else {
        rgba(0xcb, 0xd5, 0xe1, 0xff)
    };
    let accent_hovered = if is_dark {
        mix_color(accent, WHITE, 0.14)
    } else {
        mix_color(accent, WHITE, 0.10)
    };
    let accent_pressed = if is_dark {
        mix_color(accent, BLACK, 0.24)
    } else {
        mix_color(accent, BLACK, 0.16)
    };
    let selection = with_alpha(accent, if is_dark { 0x40 } else { 0x33 });
    let scrollbar_track = if is_dark {
        rgba(0x12, 0x21, 0x33, 0xff)
    } else {
        rgba(0xe2, 0xe8, 0xf0, 0xff)
    };
    let scrollbar_thumb = if is_dark {
        mix_color(accent, surface, 0.55)
    } else {
        mix_color(accent, surface, 0.40)
    };
    let dialog_backdrop = if is_dark {
        rgba(0x00, 0x00, 0x00, 0x24)
    } else {
        rgba(0x00, 0x00, 0x00, 0x18)
    };
    let dialog_shadow = if is_dark {
        rgba(0x00, 0x00, 0x00, 0xd8)
    } else {
        rgba(0x00, 0x00, 0x00, 0x88)
    };
    let panel_shadow = with_alpha(
        dialog_shadow,
        (color_alpha(dialog_shadow) as f32 * 0.30).round() as u32,
    );
    let context_menu_panel_background = if is_dark {
        rgba(0x18, 0x1d, 0x26, 0xd8)
    } else {
        rgba(0xff, 0xff, 0xff, 0xdc)
    };
    let context_menu_panel_border_color = if is_dark {
        rgba(0xff, 0xff, 0xff, 0x10)
    } else {
        rgba(0x0f, 0x17, 0x2a, 0x14)
    };
    let context_menu_item_hover = if is_dark {
        rgba(0xff, 0xff, 0xff, 0x0c)
    } else {
        rgba(0x0f, 0x17, 0x2a, 0x08)
    };
    let context_menu_separator_color = if is_dark {
        rgba(0xff, 0xff, 0xff, 0x10)
    } else {
        rgba(0x0f, 0x17, 0x2a, 0x12)
    };
    let tool_tip_panel_background = if is_dark {
        rgba(0x11, 0x17, 0x20, 0xf0)
    } else {
        rgba(0xff, 0xff, 0xff, 0xf8)
    };
    let tool_tip_panel_border_color = if is_dark {
        rgba(0xff, 0xff, 0xff, 0x12)
    } else {
        rgba(0x0f, 0x17, 0x2a, 0x12)
    };

    Theme {
        colors: Colors {
            background,
            surface,
            text_primary,
            text_muted,
            text_on_accent,
            accent,
            accent_pressed,
            accent_hovered,
            border,
            selection,
            scrollbar_track,
            scrollbar_thumb,
            dialog_backdrop,
            dialog_shadow,
            panel_shadow,
            focus_ring: accent,
        },
        spacing: DEFAULT_SPACING,
        fonts: fonts.clone(),
        context_menu: ContextMenuTheme {
            panel_background: context_menu_panel_background,
            panel_border_color: context_menu_panel_border_color,
            panel_shadow_color: panel_shadow,
            panel_corner_radius: if is_dark { 16.0 } else { 14.0 },
            separator_color: context_menu_separator_color,
            shadow_offset_y: 12.0,
            shadow_blur: 28.0,
            shadow_spread: 0.0,
            item: ContextMenuItemTheme {
                background: rgba(0, 0, 0, 0),
                hover_background: context_menu_item_hover,
                text_color: text_primary,
                corner_radius: if is_dark { 10.0 } else { 9.0 },
                font_family: fonts.body_family.clone(),
                font_size: 13.0,
                height: 30.0,
                padding_left: 12.0,
                padding_top: 6.0,
                padding_right: 12.0,
                padding_bottom: 6.0,
            },
        },
        tool_tip: ToolTipTheme {
            panel_background: tool_tip_panel_background,
            panel_border_color: tool_tip_panel_border_color,
            panel_shadow_color: panel_shadow,
            panel_corner_radius: if is_dark { 12.0 } else { 10.0 },
            text_color: text_primary,
            font_family: fonts.body_family.clone(),
            font_size: 13.0,
            max_width: 280.0,
            padding_left: 10.0,
            padding_top: 7.0,
            padding_right: 10.0,
            padding_bottom: 7.0,
            shadow_offset_y: 10.0,
            shadow_blur: 24.0,
            shadow_spread: 0.0,
        },
    }
}

pub fn default_dark_theme() -> Theme {
    generate_theme(true, DEFAULT_ACCENT_COLOR)
}

pub fn default_light_theme() -> Theme {
    generate_theme(false, DEFAULT_ACCENT_COLOR)
}

pub fn current_theme() -> Theme {
    THEME_STATE.with(|slot| slot.borrow().signal.get())
}

pub fn subscribe(handler: impl Fn(Theme) + 'static) -> SubscriptionGuard {
    handler(current_theme());
    let guard = THEME_STATE.with(|slot| {
        let callback: Callback = Rc::new(move || handler(current_theme()));
        slot.borrow_mut().signal.subscribe(callback)
    });
    guard
}

pub fn bind_theme(handler: impl Fn(Theme) + 'static) -> SubscriptionGuard {
    subscribe(handler)
}

fn apply_theme(theme: Theme, source: ThemeSource, is_dark: bool) -> Theme {
    let callbacks = THEME_STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        state.theme_source = source;
        state.current_dark_mode = is_dark;
        state.signal.set(theme.clone())
    });
    if let Some(callbacks) = callbacks {
        for callback in callbacks {
            callback();
        }
    }
    theme
}

fn apply_system_theme() -> Theme {
    THEME_STATE
        .with(|slot| {
            let state = slot.borrow();
            generate_theme(state.system_dark_mode, state.system_accent_color)
        })
        .pipe(|theme| {
            let is_dark = estimate_theme_dark(&theme);
            apply_theme(theme, ThemeSource::System, is_dark)
        })
}

pub fn use_system_theme() -> Theme {
    THEME_STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        state.system_dark_mode = framework_host_services::fui_is_dark_mode();
        state.system_accent_color =
            normalize_accent_color(framework_host_services::fui_get_accent_color());
    });
    apply_system_theme()
}

pub fn use_custom_theme(theme: Theme) -> Theme {
    apply_theme(
        theme.clone(),
        ThemeSource::Custom,
        estimate_theme_dark(&theme),
    )
}

pub fn set_accent_color(color: u32) -> Theme {
    use_custom_theme(generate_theme(is_dark_mode(), color))
}

pub fn is_dark_mode() -> bool {
    THEME_STATE.with(|slot| slot.borrow().current_dark_mode)
}

pub fn is_using_system_theme() -> bool {
    THEME_STATE.with(|slot| slot.borrow().theme_source == ThemeSource::System)
}

pub fn handle_system_dark_mode_changed(is_dark: bool) -> Theme {
    THEME_STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        state.system_dark_mode = is_dark;
        if state.theme_source != ThemeSource::System {
            return state.signal.get();
        }
        state.system_accent_color =
            normalize_accent_color(framework_host_services::fui_get_accent_color());
        let theme = generate_theme(state.system_dark_mode, state.system_accent_color);
        drop(state);
        apply_theme(theme, ThemeSource::System, is_dark)
    })
}

pub fn handle_system_accent_color_changed(color: u32) -> Theme {
    THEME_STATE.with(|slot| {
        let mut state = slot.borrow_mut();
        state.system_accent_color = normalize_accent_color(color);
        if state.theme_source != ThemeSource::System {
            return state.signal.get();
        }
        let theme = generate_theme(state.system_dark_mode, state.system_accent_color);
        let is_dark = estimate_theme_dark(&theme);
        drop(state);
        apply_theme(theme, ThemeSource::System, is_dark)
    })
}

trait Pipe: Sized {
    fn pipe<T>(self, callback: impl FnOnce(Self) -> T) -> T {
        callback(self)
    }
}

impl<T> Pipe for T {}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
pub extern "C" fn __fui_on_system_dark_mode_changed(is_dark: bool) {
    handle_system_dark_mode_changed(is_dark);
}

#[cfg_attr(not(feature = "worker-runtime"), no_mangle)]
pub extern "C" fn __fui_on_system_accent_color_changed(color: u32) {
    handle_system_accent_color_changed(color);
}

#[cfg(test)]
mod tests {
    use super::{
        current_theme, handle_system_accent_color_changed, handle_system_dark_mode_changed,
        is_dark_mode, subscribe, use_system_theme,
    };
    use crate::ffi;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn uses_host_system_theme_values() {
        ffi::test::reset();
        ffi::test::set_system_dark_mode(false);
        ffi::test::set_system_accent_color(0xFF0000FF);
        let theme = use_system_theme();
        assert!(!is_dark_mode());
        assert_eq!(theme.colors.accent, 0xFF0000FF);
    }

    #[test]
    fn system_callbacks_update_active_theme() {
        ffi::test::reset();
        ffi::test::set_system_accent_color(0x00FF00FF);
        handle_system_dark_mode_changed(false);
        let theme = handle_system_accent_color_changed(0x112233FF);
        assert!(!is_dark_mode());
        assert_eq!(theme.colors.accent, 0x112233FF);
        assert_eq!(current_theme().colors.accent, 0x112233FF);
    }

    #[test]
    fn subscribe_invokes_immediately() {
        ffi::test::reset();
        let count = Rc::new(Cell::new(0));
        let counter = count.clone();
        let _guard = subscribe(move |_theme| {
            counter.set(counter.get() + 1);
        });
        assert_eq!(count.get(), 1);
    }
}
