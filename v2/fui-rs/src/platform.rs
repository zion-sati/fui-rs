use crate::ffi;
use crate::generated::framework_host_services;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlatformFamily {
    Unknown = 0,
    Apple = 1,
    Windows = 2,
    Linux = 3,
}

pub fn device_pixel_ratio() -> f32 {
    unsafe { ffi::get_device_pixel_ratio() }
}

pub fn platform_family() -> PlatformFamily {
    match framework_host_services::fui_get_platform_family() {
        1 => PlatformFamily::Apple,
        2 => PlatformFamily::Windows,
        3 => PlatformFamily::Linux,
        _ => PlatformFamily::Unknown,
    }
}

pub fn is_coarse_pointer() -> bool {
    framework_host_services::fui_is_coarse_pointer()
}

pub fn primary_shortcut_modifier() -> u32 {
    match platform_family() {
        PlatformFamily::Apple => ffi::KeyModifier::Meta as u32,
        _ => ffi::KeyModifier::Ctrl as u32,
    }
}

pub fn word_navigation_modifier() -> u32 {
    match platform_family() {
        PlatformFamily::Apple => ffi::KeyModifier::Alt as u32,
        _ => ffi::KeyModifier::Ctrl as u32,
    }
}

pub fn line_boundary_modifier() -> u32 {
    match platform_family() {
        PlatformFamily::Apple => ffi::KeyModifier::Meta as u32,
        _ => 0,
    }
}

pub fn document_boundary_modifier() -> u32 {
    match platform_family() {
        PlatformFamily::Apple => ffi::KeyModifier::Meta as u32,
        _ => ffi::KeyModifier::Ctrl as u32,
    }
}

fn has_modifier(modifiers: u32, expected: u32) -> bool {
    expected != 0 && (modifiers & expected) != 0
}

pub fn has_primary_shortcut_modifier(modifiers: u32) -> bool {
    has_modifier(modifiers, primary_shortcut_modifier())
}

pub fn has_word_navigation_modifier(modifiers: u32) -> bool {
    has_modifier(modifiers, word_navigation_modifier())
}

pub fn has_line_boundary_modifier(modifiers: u32) -> bool {
    has_modifier(modifiers, line_boundary_modifier())
}

pub fn has_document_boundary_modifier(modifiers: u32) -> bool {
    has_modifier(modifiers, document_boundary_modifier())
}

fn format_shortcut_key_token(key: &str, platform_family: PlatformFamily) -> String {
    match key {
        "ArrowLeft" => {
            if platform_family == PlatformFamily::Apple {
                "←".to_string()
            } else {
                "Left".to_string()
            }
        }
        "ArrowRight" => {
            if platform_family == PlatformFamily::Apple {
                "→".to_string()
            } else {
                "Right".to_string()
            }
        }
        "ArrowUp" => {
            if platform_family == PlatformFamily::Apple {
                "↑".to_string()
            } else {
                "Up".to_string()
            }
        }
        "ArrowDown" => {
            if platform_family == PlatformFamily::Apple {
                "↓".to_string()
            } else {
                "Down".to_string()
            }
        }
        "PageUp" => "PgUp".to_string(),
        "PageDown" => "PgDn".to_string(),
        _ if key.chars().count() == 1 => key.to_uppercase(),
        _ => key.to_string(),
    }
}

fn append_shortcut_modifier_tokens(
    tokens: &mut Vec<String>,
    modifiers: u32,
    platform: PlatformFamily,
) {
    if platform == PlatformFamily::Apple {
        if (modifiers & ffi::KeyModifier::Ctrl as u32) != 0 {
            tokens.push("⌃".to_string());
        }
        if (modifiers & ffi::KeyModifier::Alt as u32) != 0 {
            tokens.push("⌥".to_string());
        }
        if (modifiers & ffi::KeyModifier::Shift as u32) != 0 {
            tokens.push("⇧".to_string());
        }
        if (modifiers & ffi::KeyModifier::Meta as u32) != 0 {
            tokens.push("⌘".to_string());
        }
        return;
    }

    if (modifiers & ffi::KeyModifier::Ctrl as u32) != 0 {
        tokens.push("Ctrl".to_string());
    }
    if (modifiers & ffi::KeyModifier::Alt as u32) != 0 {
        tokens.push("Alt".to_string());
    }
    if (modifiers & ffi::KeyModifier::Shift as u32) != 0 {
        tokens.push("Shift".to_string());
    }
    if (modifiers & ffi::KeyModifier::Meta as u32) != 0 {
        tokens.push("Meta".to_string());
    }
}

pub fn format_shortcut_label(key: &str, modifiers: u32) -> String {
    let platform = platform_family();
    let mut tokens = Vec::new();
    append_shortcut_modifier_tokens(&mut tokens, modifiers, platform);
    tokens.push(format_shortcut_key_token(key, platform));
    if platform == PlatformFamily::Apple {
        tokens.join("")
    } else {
        tokens.join("+")
    }
}

pub fn format_primary_shortcut_label(key: &str) -> String {
    format_shortcut_label(key, primary_shortcut_modifier())
}

pub fn format_undo_shortcut_label() -> String {
    format_primary_shortcut_label("z")
}

pub fn format_redo_shortcut_label() -> String {
    match platform_family() {
        PlatformFamily::Apple => format_shortcut_label(
            "z",
            primary_shortcut_modifier() | ffi::KeyModifier::Shift as u32,
        ),
        _ => format_primary_shortcut_label("y"),
    }
}

fn matches_shortcut_key(key: &str, expected: &str) -> bool {
    key.eq_ignore_ascii_case(expected)
}

pub fn is_undo_shortcut(key: &str, modifiers: u32) -> bool {
    (modifiers & ffi::KeyModifier::Shift as u32) == 0
        && has_primary_shortcut_modifier(modifiers)
        && matches_shortcut_key(key, "z")
}

pub fn is_redo_shortcut(key: &str, modifiers: u32) -> bool {
    match platform_family() {
        PlatformFamily::Apple => {
            has_primary_shortcut_modifier(modifiers)
                && (modifiers & ffi::KeyModifier::Shift as u32) != 0
                && matches_shortcut_key(key, "z")
        }
        _ => has_primary_shortcut_modifier(modifiers) && matches_shortcut_key(key, "y"),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        document_boundary_modifier, is_coarse_pointer, is_redo_shortcut, is_undo_shortcut,
        line_boundary_modifier, platform_family, PlatformFamily,
    };
    use crate::ffi;

    #[test]
    fn returns_mock_device_pixel_ratio() {
        ffi::test::reset();
        ffi::test::set_device_pixel_ratio(2.5);
        assert_eq!(super::device_pixel_ratio(), 2.5);
    }

    #[test]
    fn reports_platform_family_and_pointer_mode() {
        ffi::test::reset();
        ffi::test::set_platform_family(1);
        ffi::test::set_coarse_pointer(true);
        assert_eq!(platform_family(), PlatformFamily::Apple);
        assert!(is_coarse_pointer());
        assert_eq!(line_boundary_modifier(), ffi::KeyModifier::Meta as u32);
        assert_eq!(document_boundary_modifier(), ffi::KeyModifier::Meta as u32);
        assert!(is_undo_shortcut("z", ffi::KeyModifier::Meta as u32));
        assert!(is_redo_shortcut(
            "z",
            ffi::KeyModifier::Meta as u32 | ffi::KeyModifier::Shift as u32
        ));
    }
}
