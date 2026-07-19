pub(crate) fn scalar_count(text: &str) -> u32 {
    text.chars().count() as u32
}

pub(crate) fn scalar_to_byte(text: &str, scalar_index: u32) -> u32 {
    let target = scalar_index.min(scalar_count(text)) as usize;
    if target == 0 {
        return 0;
    }
    text.char_indices()
        .nth(target)
        .map(|(byte, _)| byte as u32)
        .unwrap_or(text.len() as u32)
}

pub(crate) fn byte_to_scalar(text: &str, byte_index: u32) -> u32 {
    let target = (byte_index as usize).min(text.len());
    let mut scalar_index = 0;
    for (byte, character) in text.char_indices() {
        if target <= byte || target < byte + character.len_utf8() {
            return scalar_index;
        }
        scalar_index += 1;
    }
    scalar_index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_and_utf8_byte_offsets_round_trip_at_character_boundaries() {
        let text = "A你😀Z";
        let expected_bytes = [0, 1, 4, 8, 9];
        for (scalar, byte) in expected_bytes.into_iter().enumerate() {
            assert_eq!(scalar_to_byte(text, scalar as u32), byte);
            assert_eq!(byte_to_scalar(text, byte), scalar as u32);
        }
    }

    #[test]
    fn byte_offsets_inside_a_multibyte_scalar_clamp_to_its_leading_boundary() {
        let text = "A你😀Z";
        assert_eq!(byte_to_scalar(text, 2), 1);
        assert_eq!(byte_to_scalar(text, 6), 2);
    }
}
