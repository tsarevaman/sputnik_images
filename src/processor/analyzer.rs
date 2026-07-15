use image::{DynamicImage, RgbImage};
use crate::processor::{color, filter, metrics};

pub struct AnalysisResult {
    pub overlaid_image: RgbImage,
    pub area_pixels: f64,
}

pub fn run_analysis(source_images: &DynamicImage, hue_min: f32, hue_max: f32, pixel_size: f64) -> Result<AnalysisResult, String> {
    // Подготовка изображения
    let rgb_img = source_images.to_rgb8();

    // Получение маски
    let raw_mask = color::filter_image(&rgb_img, hue_min, hue_max);
    let clean_mask = filter::remove_noise(&raw_mask, 2);

    let area_pixels = metrics::calculate_area(&clean_mask, pixel_size).map_err(|e| e.to_string())?;
    
    // Наложение маски на оригинальное фото
    let overlaid_image = color::colored_filter_image(&rgb_img, &clean_mask);

    Ok(AnalysisResult {
        overlaid_image,
        area_pixels,
    })
}