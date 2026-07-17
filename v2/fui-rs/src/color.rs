pub const fn rgba(red: u32, green: u32, blue: u32, alpha: u32) -> u32 {
    ((red & 0xff) << 24) | ((green & 0xff) << 16) | ((blue & 0xff) << 8) | (alpha & 0xff)
}

pub const fn rgb(red: u32, green: u32, blue: u32) -> u32 {
    rgba(red, green, blue, 0xff)
}

pub const fn with_alpha(color: u32, alpha: u32) -> u32 {
    (color & 0xffff_ff00) | (alpha & 0xff)
}

pub(crate) fn color_red(color: u32) -> u32 {
    (color >> 24) & 0xff
}

pub(crate) fn color_green(color: u32) -> u32 {
    (color >> 16) & 0xff
}

pub(crate) fn color_blue(color: u32) -> u32 {
    (color >> 8) & 0xff
}

pub(crate) fn color_alpha(color: u32) -> u32 {
    color & 0xff
}

fn clamp_unit(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 0.5 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * ((2.0 / 3.0) - t) * 6.0;
    }
    p
}

pub fn hsl_to_color(hue: f32, saturation: f32, lightness: f32) -> u32 {
    let normalized_hue = hue % 360.0;
    let hue = if normalized_hue < 0.0 {
        normalized_hue + 360.0
    } else {
        normalized_hue
    };
    let saturation = clamp_unit(saturation);
    let lightness = clamp_unit(lightness);
    if saturation <= 0.0 {
        let channel = (lightness * 255.0) as u32;
        return rgb(channel, channel, channel);
    }

    let hue_fraction = hue / 360.0;
    let q = if lightness < 0.5 {
        lightness * (1.0 + saturation)
    } else {
        lightness + saturation - (lightness * saturation)
    };
    let p = 2.0 * lightness - q;
    rgba(
        (clamp_unit(hue_to_rgb(p, q, hue_fraction + 1.0 / 3.0)) * 255.0) as u32,
        (clamp_unit(hue_to_rgb(p, q, hue_fraction)) * 255.0) as u32,
        (clamp_unit(hue_to_rgb(p, q, hue_fraction - 1.0 / 3.0)) * 255.0) as u32,
        0xff,
    )
}

fn mix_channel(from: u32, to: u32, amount: f32) -> u32 {
    let weight = clamp_unit(amount);
    (from as f32 + ((to as f32 - from as f32) * weight)).round() as u32
}

pub fn mix_color(from: u32, to: u32, amount: f32) -> u32 {
    rgba(
        mix_channel(color_red(from), color_red(to), amount),
        mix_channel(color_green(from), color_green(to), amount),
        mix_channel(color_blue(from), color_blue(to), amount),
        mix_channel(color_alpha(from), color_alpha(to), amount),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_helpers_match_effindom_rgba_packing() {
        assert_eq!(rgb(0x12, 0x34, 0x56), 0x123456ff);
        assert_eq!(rgba(0x12, 0x34, 0x56, 0x78), 0x12345678);
        assert_eq!(with_alpha(0x123456ff, 0x78), 0x12345678);
    }

    #[test]
    fn color_helpers_mask_channels_and_interpolate() {
        assert_eq!(rgba(0x112, 0x134, 0x156, 0x178), 0x12345678);
        assert_eq!(mix_color(rgb(0, 0, 0), rgb(255, 255, 255), 0.5), rgba(128, 128, 128, 255));
        assert_eq!(hsl_to_color(0.0, 1.0, 0.5), rgb(255, 0, 0));
    }
}
