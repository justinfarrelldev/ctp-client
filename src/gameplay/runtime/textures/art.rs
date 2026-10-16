//! Original RIM names from `controlpanel.ldl`, `Orders.txt` and `uniticon.txt`.

use crate::gameplay::assets::pixel::{expand5, expand6};
use crate::gameplay::assets::rim::RgbaImage;
use crate::gameplay::{GameplayError, asset_error};
use sdl2::pixels::Color;

/// Complete artwork required by the first ancient-era gameplay interface.
pub(super) const UI_NAMES: &[&str] = &[
    "uptb00.rim",
    "uptg32.rim",
    "uptbtest.rim",
    "uptb02.rim",
    "uptb03.rim",
    "uptg01e.rim",
    "uptg05a.rim",
    "uptg05b.rim",
    "uptg05c.rim",
    "uptg20e.rim",
    "upic34.rim",
    "upic33.rim",
    "upic35.rim",
    "upic36.rim",
    "upfg01.rim",
    "upfg03.rim",
    "upfg04.rim",
    "upfg05.rim",
    "upfg06.rim",
    "upfg50.rim",
    "upst01.rim",
    "upcb14u.rim",
    "upcb14d.rim",
    "upcb14h.rim",
    "upcb14x.rim",
    "upcb05u.rim",
    "upcb06u.rim",
    "upcb07u.rim",
    "upcb08u.rim",
    "upcb09u.rim",
    "upcb10u.rim",
    "upcb11u.rim",
    "upcb60u.rim",
    "uppb01u.rim",
    "upta01ur.rim",
    "upta01u.rim",
    "upta01ul.rim",
    "upsp01bl.rim",
    "upsp01bm.rim",
    "upsp01br.rim",
    "upsp01cu.rim",
    "upbt02x.rim",
    "upbt02u.rim",
    "upbt02d.rim",
    "upbt02h.rim",
    "upsi26.rim",
    "upsi27.rim",
    "upsi28.rim",
    "upsi29.rim",
    "upsi04.rim",
    "upsi02.rim",
    "upsi30.rim",
    "upup002l.rim",
    "upup003l.rim",
    "uprb01.rim",
    "uprb11.rim",
    "uprb02.rim",
    "uprb44.rim",
    "uprb22.rim",
    "uprb04.rim",
    "uprb33.rim",
    "uprb03.rim",
    "upcb20.rim",
    "upcb20u.rim",
    "upcb21.rim",
    "upcb21u.rim",
    "upcb22.rim",
    "upcb22u.rim",
    "upcb23.rim",
    "upcb23u.rim",
    "upcb24.rim",
    "upcb24u.rim",
    "upcb25.rim",
    "upcb25u.rim",
    "upcb26.rim",
    "upcb27.rim",
];

/// Tiles an installed pattern into a declared LDL control without stretching it.
pub(super) fn tiled_pattern(
    pattern: &RgbaImage,
    width: u32,
    height: u32,
) -> Result<RgbaImage, GameplayError> {
    if pattern.width == 0_u32 || pattern.height == 0_u32 {
        return Err(asset_error("empty UI pattern".to_owned()));
    }
    let pitch = usize::try_from(width)
        .ok()
        .and_then(|value| value.checked_mul(4_usize))
        .ok_or_else(|| asset_error("UI pattern pitch overflow".to_owned()))?;
    let length = usize::try_from(height)
        .ok()
        .and_then(|value| value.checked_mul(pitch))
        .ok_or_else(|| asset_error("UI pattern size overflow".to_owned()))?;
    let mut data = Vec::with_capacity(length);
    for y in 0_u32..height {
        for x in 0_u32..width {
            let offset = usize::try_from(y.checked_rem(pattern.height).unwrap_or(0_u32))
                .map_or(0_usize, |value| value)
                .saturating_mul(pattern.pitch)
                .saturating_add(
                    usize::try_from(x.checked_rem(pattern.width).unwrap_or(0_u32))
                        .map_or(0_usize, |value| value)
                        .saturating_mul(4_usize),
                );
            data.extend_from_slice(
                pattern
                    .data
                    .get(offset..offset.saturating_add(4_usize))
                    .ok_or_else(|| {
                        asset_error("UI pattern pixel buffer is truncated".to_owned())
                    })?,
            );
        }
    }
    Ok(RgbaImage {
        width,
        height,
        pitch,
        data,
    })
}

/// Reproduces `c3_Button`'s default two-pixel `primitives_BevelRect16` pass.
/// Light/dark edges are blended 16/32 in the original 565 channel space.
pub(super) fn apply_original_bevel(image: &mut RgbaImage, level: u32, pressed: bool) {
    let level = level.min(image.width.min(image.height).div_euclid(2_u32));
    for inset in 0_u32..level {
        let right = image.width.saturating_sub(1_u32).saturating_sub(inset);
        let bottom = image.height.saturating_sub(1_u32).saturating_sub(inset);
        for offset in 0_u32
            ..image
                .width
                .saturating_sub(1_u32)
                .saturating_sub(inset.saturating_mul(2_u32))
        {
            bevel_pixel(image, inset.saturating_add(offset), inset, !pressed);
            bevel_pixel(image, right.saturating_sub(offset), bottom, pressed);
        }
        for offset in 0_u32
            ..image
                .height
                .saturating_sub(1_u32)
                .saturating_sub(inset.saturating_mul(2_u32))
        {
            bevel_pixel(
                image,
                inset,
                inset.saturating_add(1_u32).saturating_add(offset),
                !pressed,
            );
            bevel_pixel(image, right, inset.saturating_add(offset), pressed);
        }
    }
}

/// Blends one original 565 pixel without double-shading the bevel corners.
fn bevel_pixel(image: &mut RgbaImage, x: u32, y: u32, light: bool) {
    let offset = usize::try_from(y)
        .map_or(0_usize, |value| value)
        .saturating_mul(image.pitch)
        .saturating_add(
            usize::try_from(x)
                .map_or(0_usize, |value| value)
                .saturating_mul(4_usize),
        );
    if let Some(pixel) = image.data.get_mut(offset..offset.saturating_add(4_usize)) {
        for (channel, shift, maximum) in [
            (0_usize, 3_u8, 31_u8),
            (1_usize, 2_u8, 63_u8),
            (2_usize, 3_u8, 31_u8),
        ] {
            let value = pixel[channel] >> shift;
            let mixed = value
                .saturating_add(if light { maximum } else { 0_u8 })
                .div_euclid(2_u8);
            pixel[channel] = if channel == 1_usize {
                expand6(mixed)
            } else {
                expand5(mixed)
            };
        }
    }
}

/// Reproduces `TileSet`'s 16-bit map-icon conversion, including skip and mask keys.
pub(super) fn colorize_map_icon(image: &RgbaImage, color: Color) -> RgbaImage {
    let mut result = image.clone();
    for pixel in result.data.chunks_exact_mut(4) {
        if pixel[0] == 0_u8 && pixel[1] == 0_u8 && pixel[2] == 0_u8 {
            pixel[3] = 0_u8;
        } else if pixel[0] >= 248_u8 && pixel[1] == 0_u8 && pixel[2] >= 248_u8 {
            pixel.copy_from_slice(&[0_u8, 0_u8, 0_u8, 128_u8]);
        } else if pixel[0] == 0_u8 && pixel[1] >= 248_u8 && pixel[2] >= 248_u8 {
            pixel.copy_from_slice(&[color.r, color.g, color.b, 255_u8]);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::{apply_original_bevel, colorize_map_icon};
    use crate::gameplay::assets::rim::RgbaImage;
    use sdl2::pixels::Color;

    #[test]
    fn original_c3_bevel_uses_two_quantized_pixel_edges_not_a_stretched_frame() {
        let mut image = RgbaImage {
            width: 6_u32,
            height: 6_u32,
            pitch: 24_usize,
            data: [214_u8, 203_u8, 198_u8, 255_u8].repeat(36_usize),
        };
        apply_original_bevel(&mut image, 2_u32, false);
        assert_eq!(
            &image.data[..4_usize],
            &[231_u8, 227_u8, 222_u8, 255_u8],
            "raised top/left source blend"
        );
        assert_eq!(
            &image.data[140_usize..144_usize],
            &[107_u8, 101_u8, 99_u8, 255_u8],
            "raised bottom/right source blend"
        );
        assert_eq!(
            &image.data[56_usize..60_usize],
            &[214_u8, 203_u8, 198_u8, 255_u8],
            "inner artwork untouched"
        );
        let mut pressed = RgbaImage {
            width: 6_u32,
            height: 6_u32,
            pitch: 24_usize,
            data: [214_u8, 203_u8, 198_u8, 255_u8].repeat(36_usize),
        };
        apply_original_bevel(&mut pressed, 2_u32, true);
        assert_eq!(
            &pressed.data[..4_usize],
            &image.data[140_usize..144_usize],
            "pressing inverts the source bevel"
        );
    }

    #[test]
    fn map_icon_colorization_preserves_shading_and_skips_black() {
        let image = RgbaImage {
            width: 4_u32,
            height: 1_u32,
            pitch: 16_usize,
            data: vec![
                0_u8, 0, 0, 255, 0, 252, 248, 255, 248, 252, 248, 255, 80, 100, 80, 255,
            ],
        };
        let colored = colorize_map_icon(&image, Color::RGB(0_u8, 252_u8, 0_u8));
        assert_eq!(
            colored.data,
            vec![
                0_u8, 0, 0, 0, 0, 252, 0, 255, 248, 252, 248, 255, 80, 100, 80, 255
            ],
            "only cyan is replaced and black is transparent"
        );
        assert_eq!(image.data[3], 255_u8, "source is not mutated");
    }
}
