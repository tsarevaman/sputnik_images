use image::GrayImage;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgricultureCalculateError {
    EmptyImage,
    InvalidDimensions,
}

impl fmt::Display for AgricultureCalculateError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Self::EmptyImage => write!(f, "Изображение не содержит данных"),
      Self::InvalidDimensions => write!(f, "Фотография пустая"),
    }
  }
}


pub fn calculate_area(mask: &GrayImage, size_of_pixel: f64) -> Result<f64, AgricultureCalculateError> {
    if mask.is_empty() {
        return Err(AgricultureCalculateError::EmptyImage);
    }

    let total_pixels = mask.width() * mask.height();

    if total_pixels <= 0 {
        return Err(AgricultureCalculateError::InvalidDimensions);
    }

    let area = (mask
        .pixels()
        .filter(|pixel| pixel[0] == 255)
        .count() as f64) * size_of_pixel;

    Ok(area)
}