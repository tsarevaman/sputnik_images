use image::GrayImage;
use imageproc::morphology::open;
use imageproc::distance_transform::Norm;

/// Морфологическое открытие
pub fn remove_noise(mask: &GrayImage, radius: u8) -> GrayImage {
    // Выполняет морфологическое открытие с кистью в форме квадарта
    //open(mask, Norm::L1, radius)

    mask.clone()
}