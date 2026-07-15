use std::fs;
use std::path::{Path, PathBuf};
use tiff::decoder::Decoder;
use tiff::tags::Tag;
use std::fs::File;
use tiff::encoder::{TiffEncoder, colortype};

/// Структура для хранения извлеченных геоданных
#[derive(Default)]
pub struct TiffMetadata {
    pub pixel_scale: Option<Vec<f64>>,
    pub tie_point: Option<Vec<f64>>,
    pub geokeys: Option<Vec<u16>>,
    pub geo_double_params: Option<Vec<f64>>, // Тег 34736
    pub geo_ascii_params: Option<String>,    // Тег 34737
}

// Функция для извлечения метаданных из TIFF файла
pub fn extract_tiff_metadata(path: &Path) -> TiffMetadata {
    let mut metadata = TiffMetadata::default();
    
    if let Ok(file) = fs::File::open(path) {
        if let Ok(mut decoder) = Decoder::new(file) {
            metadata.pixel_scale = decoder.get_tag(Tag::Unknown(33550)).ok().and_then(|v| v.into_f64_vec().ok());
            metadata.tie_point = decoder.get_tag(Tag::Unknown(33922)).ok().and_then(|v| v.into_f64_vec().ok());
            
            metadata.geokeys = decoder.get_tag(Tag::Unknown(34735)).ok().and_then(|v| v.into_u16_vec().ok())
                .or_else(|| {
                    decoder.get_tag(Tag::Unknown(34735)).ok()
                        .and_then(|v| v.into_u32_vec().ok())
                        .map(|vec| vec.into_iter().map(|k| k as u16).collect())
                });
            
            metadata.geo_double_params = decoder.get_tag(Tag::Unknown(34736)).ok().and_then(|v| v.into_f64_vec().ok());
            metadata.geo_ascii_params = decoder.get_tag(Tag::Unknown(34737)).ok().and_then(|v| v.into_string().ok());
        }
    }
    
    // Вывод в консоль
    println!("\nЧтение метаданных: {:?}", path.file_name().unwrap_or_default());
    
    if let Some(scale) = &metadata.pixel_scale {
        println!("  Pixel Scale (33550): {:?}", scale);
    } else {
        println!("  Pixel Scale (33550): Не найдено");
    }
    
    if let Some(tie) = &metadata.tie_point {
        println!("  Tie Point (33922): {:?}", tie);
    } else {
        println!("  Tie Point (33922): Не найдено");
    }
    
    if let Some(keys) = &metadata.geokeys {
        println!("  GeoKeys (34735): Найдено {} ключей", keys.len());
    } else {
        println!("  ❌ GeoKeys (34735): Не найдено (ОШИБКА ЧТЕНИЯ)");
    }

    if let Some(doubles) = &metadata.geo_double_params {
        println!("  GeoDoubleParams (34736): Найдено {} значений", doubles.len());
    }

    if let Some(_) = &metadata.geo_ascii_params {
        println!("  GeoAsciiParams (34737): Найдено");
    }
    
    metadata
}

/// Функция для поиска всех .tif файлов в директории
pub fn find_tiff_files(folder_path: &Path) -> Vec<PathBuf> {
    let mut found_tiffs = Vec::new();
    
    if let Ok(entries) = fs::read_dir(folder_path) {
        for entry in entries.flatten() {
            let file_path = entry.path();
            if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
                if ext.eq_ignore_ascii_case("tif") || ext.eq_ignore_ascii_case("tiff") {
                    found_tiffs.push(file_path);
                }
            }
        }
    }
    found_tiffs
}

pub fn save_tiff(
    path: &str, 
    width: u32, 
    height: u32, 
    pixels: &[u8], 
    pixel_scale: &[f64], 
    tie_point: &[f64], 
    geokeys: &[u16], // Строго u16
    geo_double_params: Option<&[f64]>,
    geo_ascii_params: Option<&str>
) {
    println!("\nСохранение маски: {}", path);
    println!("  Запись GeoKeys: {} ключей", geokeys.len());

    let file = match File::create(path) {
        Ok(f) => f,
        Err(e) => {
            println!("Ошибка создания файла: {}", e);
            return;
        }
    };

    let mut encoder = match TiffEncoder::new(file) {
        Ok(enc) => enc,
        Err(e) => {
            println!("Ошибка создания энкодера: {}", e);
            return;
        }
    };

    let mut image_encoder = encoder.new_image::<colortype::RGB8>(width, height).unwrap();
    
    image_encoder.encoder().write_tag(Tag::Unknown(33550), pixel_scale).unwrap();
    image_encoder.encoder().write_tag(Tag::Unknown(33922), tie_point).unwrap();
    image_encoder.encoder().write_tag(Tag::Unknown(34735), geokeys).unwrap();
    
    if let Some(doubles) = geo_double_params {
        image_encoder.encoder().write_tag(Tag::Unknown(34736), doubles).unwrap();
    }
    if let Some(ascii) = geo_ascii_params {
        image_encoder.encoder().write_tag(Tag::Unknown(34737), ascii).unwrap();
    }

    match image_encoder.write_data(pixels) {
        Ok(_) => println!("  Успешно сохранено с гео-тегами!"),
        Err(e) => println!("  Ошибка при записи пикселей: {}", e),
    }
}