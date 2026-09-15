//! U²-Net inference via `tract`, a pure-Rust ONNX runtime.
//!
//! Unlike the flood fill in the parent module, this segments a subject from any
//! background — including the gradients the algorithmic mode provably cannot
//! handle — at the cost of a ~168 MB model and a few seconds per image.

use std::path::Path;

use image::{DynamicImage, GrayImage, Rgba, RgbaImage};
use tract_onnx::prelude::*;

/// Side length of the square input U²-Net was trained on.
const INPUT_SIZE: usize = 320;

/// ImageNet normalisation, matching rembg's preprocessing.
const MEAN: [f32; 3] = [0.485, 0.456, 0.406];
const STD: [f32; 3] = [0.229, 0.224, 0.225];

/// An optimised, runnable U²-Net plan.
///
/// This is `Sync`, so a single loaded model can be shared by reference across
/// threads.
pub(crate) type CutoutModel = std::sync::Arc<TypedRunnableModel>;

pub(crate) fn load_model(path: &Path) -> Result<CutoutModel, String> {
    tract_onnx::onnx()
        .model_for_path(path)
        .and_then(|m| m.with_input_fact(0, f32::fact([1, 3, INPUT_SIZE, INPUT_SIZE]).into()))
        .and_then(|m| m.into_optimized())
        .and_then(|m| m.into_runnable())
        .map_err(|e| {
            format!(
                "cutout: failed to load the background removal model '{}': {e}",
                path.display()
            )
        })
}

/// Builds the NCHW input tensor U²-Net expects.
///
/// rembg scales by the image's own maximum channel value rather than by a fixed
/// 255; matching that matters on dark images, where dividing by 255 would shift
/// the whole input away from the distribution the network was trained on.
fn preprocess(img: &DynamicImage) -> Tensor {
    let resized = image::imageops::resize(
        &img.to_rgb8(),
        INPUT_SIZE as u32,
        INPUT_SIZE as u32,
        image::imageops::FilterType::Triangle,
    );

    let max = resized.iter().copied().max().unwrap_or(0).max(1) as f32;

    tract_ndarray::Array4::from_shape_fn((1, 3, INPUT_SIZE, INPUT_SIZE), |(_, c, y, x)| {
        let pixel = resized.get_pixel(x as u32, y as u32);
        (pixel[c] as f32 / max - MEAN[c]) / STD[c]
    })
    .into()
}

/// Turns U²-Net's first output (`d0`) into an alpha mask at `width`x`height`.
///
/// The raw output is unbounded, so it is min-max normalised before being
/// quantised — the same step rembg performs.
fn postprocess(output: &TValue, width: u32, height: u32) -> Result<GrayImage, String> {
    let d0 = output
        .to_plain_array_view::<f32>()
        .map_err(|e| format!("cutout: unexpected model output: {e}"))?;

    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for &v in d0.iter() {
        lo = lo.min(v);
        hi = hi.max(v);
    }
    // A constant output carries no information; treat it as fully opaque
    // rather than dividing by zero.
    let span = if hi > lo { hi - lo } else { 1.0 };

    let mut mask = GrayImage::new(INPUT_SIZE as u32, INPUT_SIZE as u32);
    for y in 0..INPUT_SIZE {
        for x in 0..INPUT_SIZE {
            let v = ((d0[[0, 0, y, x]] - lo) / span).clamp(0.0, 1.0);
            mask.put_pixel(x as u32, y as u32, image::Luma([(v * 255.0).round() as u8]));
        }
    }

    Ok(image::imageops::resize(
        &mask,
        width,
        height,
        image::imageops::FilterType::Triangle,
    ))
}

/// Applies `mask` to `img` as its alpha channel.
///
/// `mask` must already match `img`'s dimensions.
pub(crate) fn apply_mask(img: &DynamicImage, mask: &GrayImage) -> RgbaImage {
    let rgba = img.to_rgba8();
    let mut out = RgbaImage::new(rgba.width(), rgba.height());
    for (x, y, pixel) in rgba.enumerate_pixels() {
        let alpha = mask.get_pixel(x, y).0[0];
        out.put_pixel(x, y, Rgba([pixel[0], pixel[1], pixel[2], alpha]));
    }
    out
}

pub(crate) fn neural_cutout(model: &CutoutModel, img: &DynamicImage) -> Result<RgbaImage, String> {
    let outputs = model
        .run(tvec!(preprocess(img).into()))
        .map_err(|e| format!("cutout: background removal failed: {e}"))?;
    let mask = postprocess(&outputs[0], img.width(), img.height())?;
    Ok(apply_mask(img, &mask))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Luma;

    #[test]
    fn test_apply_mask_uses_the_mask_as_alpha() {
        let img = DynamicImage::ImageRgba8(RgbaImage::from_pixel(3, 2, Rgba([10, 20, 30, 255])));
        let mut mask = GrayImage::new(3, 2);
        mask.put_pixel(0, 0, Luma([0]));
        mask.put_pixel(1, 0, Luma([128]));
        mask.put_pixel(2, 0, Luma([255]));

        let out = apply_mask(&img, &mask);
        assert_eq!(out.get_pixel(0, 0).0, [10, 20, 30, 0]);
        assert_eq!(out.get_pixel(1, 0).0, [10, 20, 30, 128]);
        assert_eq!(out.get_pixel(2, 0).0, [10, 20, 30, 255]);
    }

    #[test]
    fn test_apply_mask_preserves_colour_and_dimensions() {
        let img = DynamicImage::ImageRgba8(RgbaImage::from_pixel(4, 5, Rgba([7, 8, 9, 255])));
        let mask = GrayImage::from_pixel(4, 5, Luma([200]));

        let out = apply_mask(&img, &mask);
        assert_eq!((out.width(), out.height()), (4, 5));
        assert_eq!(out.get_pixel(3, 4).0, [7, 8, 9, 200]);
    }

    #[test]
    fn test_preprocess_produces_the_shape_the_model_expects() {
        let img = DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
            64,
            48,
            image::Rgb([120, 130, 140]),
        ));
        let tensor = preprocess(&img);
        assert_eq!(tensor.shape(), [1, 3, INPUT_SIZE, INPUT_SIZE]);
    }

    #[test]
    fn test_preprocess_normalises_against_the_image_maximum() {
        // A uniform image's brightest channel becomes 1.0 before ImageNet
        // normalisation, so each channel lands on (1 - mean) / std.
        let img =
            DynamicImage::ImageRgb8(image::RgbImage::from_pixel(8, 8, image::Rgb([60, 0, 0])));
        let tensor = preprocess(&img);
        let view = tensor.to_plain_array_view::<f32>().expect("f32 tensor");
        let expected = (1.0 - MEAN[0]) / STD[0];
        assert!((view[[0, 0, 0, 0]] - expected).abs() < 1e-4);
    }

    #[test]
    fn test_preprocess_does_not_divide_by_zero_on_a_black_image() {
        let img = DynamicImage::ImageRgb8(image::RgbImage::from_pixel(8, 8, image::Rgb([0, 0, 0])));
        let tensor = preprocess(&img);
        let view = tensor.to_plain_array_view::<f32>().expect("f32 tensor");
        assert!(view.iter().all(|v| v.is_finite()));
    }
}
