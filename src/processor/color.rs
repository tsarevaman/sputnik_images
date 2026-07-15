use std::path::PathBuf;
use image::{GenericImageView, GrayImage, RgbImage};
use palette::{FromColor, Hsv, Srgb};

///Функция создает из RGB изображения HSV
pub fn filter_image(img: &RgbImage, min_hue: f32, max_hue: f32) -> GrayImage {
    let (width, height) = img.dimensions();

    // Создается вектор из сырых пикселей
    let raw_pixels = img
        .pixels()
        .map(|pixel| {
            let r = pixel[0] as f32 / 255.0;
            let g = pixel[1] as f32 / 255.0;
            let b = pixel[2] as f32 / 255.0;

            let rgb = Srgb::new(r, g,b );
            let hsv = Hsv::from_color(rgb);
            let hue = hsv.hue.into_positive_degrees();

            if hue >= min_hue && hue <= max_hue {
                255
            } else {
                0
            }
        })
        .collect();

    // Все сырые пиксели собираются в черно белую картинку
    GrayImage::from_raw(width, height, raw_pixels).unwrap()
}

pub fn colored_filter_image(img: &RgbImage, mask: &GrayImage) -> RgbImage {
    let mut result = img.clone();

    result.pixels_mut()
        .zip(mask.pixels())
        .for_each(|(rgb_pixel, mask_pixel)| {
            if mask_pixel[0] != 255 {
                rgb_pixel[0] >>= 4;
                rgb_pixel[1] >>= 4;
                rgb_pixel[2] >>= 4;
            }
        });
    
    result
}

pub fn ndvi_filter_image(red_path: &PathBuf, nir_path: &PathBuf, threshold: f32) -> Result<GrayImage, std::io::Error> {
    let red_img = image::open(red_path).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    let nir_img = image::open(nir_path).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    if red_img.dimensions() != nir_img.dimensions() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Изображения каналов имеют разный размер!"));
    }

    let red_layer = red_img.to_luma16();
    let nir_layer = nir_img.to_luma16();    

    let raw_pixel = nir_layer.as_raw()
        .iter()
        .zip(red_layer.as_raw().iter())
        .map(|(&nir, &red)| {  
            let nir_val = nir as f32;  
            let red_val = red as f32;
            let denominator = nir_val + red_val;
            if denominator == 0.0 {
                0 // Избегаем деления на ноль
            } else {
                let ndvi = (nir_val - red_val) / denominator;
                
                // Если индекс выше порога, пиксель белый, иначе черный
                if ndvi >= threshold {
                    255
                } else {
                    0
                }
            }
        })
        .collect();
        let (width, height) = red_img.dimensions();
        GrayImage::from_raw(width, height, raw_pixel).ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Ошибка при сборке изображения"))
}

pub fn overlay_mask(source: &RgbImage, mask: &GrayImage) -> RgbImage {
    let mut output = source.clone();
    
    output.pixels_mut()
        .zip(mask.pixels())
        .for_each(|(pixel, mask_pixel)| {
            if mask_pixel[0] != 255 {
                pixel[0] = 0;
                pixel[1] = 0;
                pixel[2] = 0;

                // pixel[0] >>= 2; 
                // pixel[1] >>= 2;
                // pixel[2] >>= 2;
            }
        });

    output
}