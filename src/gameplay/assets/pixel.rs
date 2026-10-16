//! Full-range conversion from original RGB555/RGB565 channels to SDL RGBA.
//!
//! Simply shifting the bits leaves white at 248/252, unlike a displayed 16-bit
//! game framebuffer. Replicate high bits into the low bits, as SDL's pixel-format
//! conversion does, preserving both black and full-intensity white.

/// Expands an already masked five-bit channel to the full eight-bit range.
pub const fn expand5(value: u8) -> u8 {
    (value << 3_u8) | (value >> 2_u8)
}
/// Expands an already masked six-bit channel to the full eight-bit range.
pub const fn expand6(value: u8) -> u8 {
    (value << 2_u8) | (value >> 4_u8)
}

/// Applies the release's 16/32 blend with black in original channel space.
/// Alpha is untouched: transparent silhouette and shadow-run opacity survive.
pub fn shade_explored(pixels: &mut [u8], six_bit_green: bool) {
    for pixel in pixels.chunks_exact_mut(4_usize) {
        pixel[0] = expand5((pixel[0] >> 3_u8).div_euclid(2_u8));
        pixel[1] = if six_bit_green {
            expand6((pixel[1] >> 2_u8).div_euclid(2_u8))
        } else {
            expand5((pixel[1] >> 3_u8).div_euclid(2_u8))
        };
        pixel[2] = expand5((pixel[2] >> 3_u8).div_euclid(2_u8));
    }
}

#[cfg(test)]
mod tests {
    use super::{expand5, expand6, shade_explored};

    #[test]
    fn original_fog_blends_packed_channels_before_expansion_and_preserves_alpha() {
        let mut white = [255_u8, 255_u8, 255_u8, 255_u8, 0_u8, 0_u8, 0_u8, 128_u8];
        shade_explored(&mut white, true);
        assert_eq!(
            white,
            [123_u8, 125_u8, 123_u8, 255_u8, 0_u8, 0_u8, 0_u8, 128_u8],
            "RGB565 fog with a separate shadow pixel"
        );
        let mut white555 = [255_u8, 255_u8, 255_u8, 255_u8];
        shade_explored(&mut white555, false);
        assert_eq!(
            white555,
            [123_u8, 123_u8, 123_u8, 255_u8],
            "RGB555 keeps five green bits"
        );
    }

    #[test]
    fn original_channel_expansion_preserves_endpoints_and_intermediate_values() {
        assert_eq!(
            (expand5(0_u8), expand5(31_u8)),
            (0_u8, 255_u8),
            "five-bit endpoints"
        );
        assert_eq!(
            (expand6(0_u8), expand6(63_u8)),
            (0_u8, 255_u8),
            "six-bit endpoints"
        );
        assert_eq!(
            (expand5(26_u8), expand6(50_u8)),
            (214_u8, 203_u8),
            "original display brightness, not dark 208/200 shifted values"
        );
    }
}
