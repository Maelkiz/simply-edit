use image::{DynamicImage, Rgba, RgbaImage, imageops::FilterType};

/// Longest side, in pixels, of the image screens transform on every keypress.
///
/// Terminal panes rarely show more than this, and keeping per-frame work small is what keeps
/// held-down keys responsive.
pub(crate) const PREVIEW_MAX_SIDE: u32 = 1024;

const CHECKER_TILE: u32 = 8;
const CHECKER_LIGHT: Rgba<u8> = Rgba([153, 153, 153, 255]);
const CHECKER_DARK: Rgba<u8> = Rgba([102, 102, 102, 255]);

/// Downscale `img` so its longest side is at most [`PREVIEW_MAX_SIDE`]. Never upscales.
pub(crate) fn prepare_base(img: &DynamicImage) -> RgbaImage {
    let long = img.width().max(img.height());
    if long <= PREVIEW_MAX_SIDE {
        return img.to_rgba8();
    }
    img.resize(PREVIEW_MAX_SIDE, PREVIEW_MAX_SIDE, FilterType::Lanczos3)
        .to_rgba8()
}

/// Blend `img` over a grey checkerboard so transparent regions are visible, returning an opaque
/// image.
pub(crate) fn checkerboard_composite(img: &RgbaImage) -> RgbaImage {
    RgbaImage::from_fn(img.width(), img.height(), |x, y| {
        let bg = if (x / CHECKER_TILE + y / CHECKER_TILE).is_multiple_of(2) {
            CHECKER_LIGHT
        } else {
            CHECKER_DARK
        };
        let px = img.get_pixel(x, y);
        let a = px[3] as u32;
        let blend = |fg: u8, bg: u8| ((fg as u32 * a + bg as u32 * (255 - a) + 127) / 255) as u8;
        Rgba([
            blend(px[0], bg[0]),
            blend(px[1], bg[1]),
            blend(px[2], bg[2]),
            255,
        ])
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prepare_base_caps_long_side() {
        let img = DynamicImage::new_rgba8(4000, 1000);
        let base = prepare_base(&img);
        assert_eq!(base.width(), PREVIEW_MAX_SIDE);
        assert_eq!(base.height(), 256);
    }

    #[test]
    fn test_prepare_base_caps_portrait() {
        let img = DynamicImage::new_rgba8(500, 2048);
        let base = prepare_base(&img);
        assert_eq!(base.height(), PREVIEW_MAX_SIDE);
        assert_eq!(base.width(), 250);
    }

    #[test]
    fn test_prepare_base_small_unchanged() {
        let img = DynamicImage::new_rgba8(40, 30);
        let base = prepare_base(&img);
        assert_eq!(base.dimensions(), (40, 30));
    }

    #[test]
    fn test_checkerboard_keeps_opaque_pixels() {
        let img = RgbaImage::from_pixel(2, 2, Rgba([10, 20, 30, 255]));
        let out = checkerboard_composite(&img);
        assert_eq!(*out.get_pixel(1, 1), Rgba([10, 20, 30, 255]));
    }

    #[test]
    fn test_checkerboard_shows_tiles_under_transparency() {
        let img = RgbaImage::from_pixel(16, 16, Rgba([255, 0, 0, 0]));
        let out = checkerboard_composite(&img);
        assert_eq!(*out.get_pixel(0, 0), CHECKER_LIGHT);
        assert_eq!(*out.get_pixel(CHECKER_TILE, 0), CHECKER_DARK);
        assert_eq!(*out.get_pixel(CHECKER_TILE, CHECKER_TILE), CHECKER_LIGHT);
    }
}
