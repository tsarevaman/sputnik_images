use eframe::egui;
use image::{DynamicImage, RgbImage};
use crate::processor::{analyzer, color, filter};
use std::{path::{Path, PathBuf}};
use rfd::FileDialog;
use crate::io::file_utils;
use crate::app::components;
use crate::processor::metrics;

/// Структура для хранения настроек
pub struct AnalyzerSettings {
    pub hue_min: f32,   
    pub hue_max: f32,
}

impl Default for AnalyzerSettings {
    fn default() -> Self {
        Self { hue_min: 30.0, hue_max: 90.0 }
    }
}

/// Структура для хранения данных анализа изображения
pub struct ProjectData {
    pub paths: Option<BandPaths>,
    pub source_image: Option<DynamicImage>,
    pub mask: Option<RgbImage>,
    pub field_area_pixels: f64,
    pub pixel_scale: Option<Vec<f64>>,
    pub tie_point: Option<Vec<f64>>,
    pub geokeys: Option<Vec<u16>>,

    pub geo_double_params: Option<Vec<f64>>, 
    pub geo_ascii_params: Option<String>,
}

impl ProjectData {
    pub fn new() -> Self {
        Self {
            paths: None,
            source_image: None,
            mask: None,
            field_area_pixels: 0.0,
            pixel_scale: None,
            tie_point: None,
            geokeys: None,

            geo_double_params: None,
            geo_ascii_params: None,
        }
    }
}

/// Состояние окна
#[derive(Default)]
pub struct UiState {
    pub source_texture: Option<egui::TextureHandle>,
    pub mask_texture: Option<egui::TextureHandle>,
    pub last_error: Option<String>, // Для вывода ошибок пользователю
}

/// Главная структура окна
#[derive(Default)]
pub struct SatelliteApp {
    pub settings: AnalyzerSettings,
    pub project: Option<ProjectData>,
    pub ui: UiState,
    pub ndvi_threshold: f32,
}

#[derive(Default)]
pub struct BandPaths {
    pub available_tiffs: Vec<PathBuf>,
    pub red_tiff: Option<PathBuf>,
    pub green_tiff: Option<PathBuf>,
    pub blue_tiff: Option<PathBuf>,
    pub nir_tiff: Option<PathBuf>,
}

impl SatelliteApp {
    fn load_image_from_path(&mut self, ctx: &egui::Context, path: &Path) {
        match image::open(path) {
            Ok(img) => {
                let rgba_img = img.to_rgba8();
                let color_img = egui::ColorImage::from_rgba_unmultiplied(
                    [rgba_img.width() as usize, rgba_img.height() as usize], 
                    rgba_img.as_raw()
                );
                
                self.ui.source_texture = Some(ctx.load_texture(
                    "source_image",
                    color_img,
                    Default::default()
                ));

                let mut project = ProjectData::new();
                project.source_image = Some(img);
                
                // Вызов file_utils
                let metadata = file_utils::extract_tiff_metadata(path);
                project.pixel_scale = metadata.pixel_scale;
                project.tie_point = metadata.tie_point;
                project.geokeys = metadata.geokeys;
                project.geo_double_params = metadata.geo_double_params;
                project.geo_ascii_params = metadata.geo_ascii_params;

                self.project = Some(project);
                self.ui.mask_texture = None;
                self.ui.last_error = None;
            },
            Err(e) => {
                let err_msg = format!("Ошибка открытия файла {:?}: {}", path, e);
                self.ui.last_error = Some(err_msg);
            }
        }
    }


    fn load_tiff_from_path(&mut self, ctx: &egui::Context, red_path: &PathBuf, green_path: &PathBuf, blue_path: &PathBuf) {
        if let (Ok(red_img), Ok(green_img), Ok(blue_img)) = (
            image::open(red_path),
            image::open(green_path),
            image::open(blue_path)
        ) {
            // Создание холста
            let mut combined_img = RgbImage::new(red_img.width(), red_img.height());

            let r_layer = red_img.to_luma16();
            let g_layer = green_img.to_luma16();
            let b_layer = blue_img.to_luma16();

            // Множитель яркости. Для Landsat 16-bit обычно хватает умножения на 3.0 или 4.0
            let brightness: f32 = 4.0; 

            for (x, y, pixel) in combined_img.enumerate_pixels_mut() {
                // Достаем 16-битное значение, умножаем на яркость, 
                // делим на 256 (чтобы перевести в 8-бит для экрана) и обрезаем лишнее
                let r = ((r_layer.get_pixel(x, y)[0] as f32 * brightness) / 256.0).min(255.0) as u8;
                let g = ((g_layer.get_pixel(x, y)[0] as f32 * brightness) / 256.0).min(255.0) as u8;
                let b = ((b_layer.get_pixel(x, y)[0] as f32 * brightness) / 256.0).min(255.0) as u8;
                
                *pixel = image::Rgb([r, g, b]);
            }

            let dynamic_img = DynamicImage::ImageRgb8(combined_img.clone());

            let color_img = egui::ColorImage::from_rgb(
                [combined_img.width() as usize, combined_img.height() as usize],
                combined_img.as_raw()
            );

            self.ui.source_texture = Some(ctx.load_texture(
                "combined_image",
                color_img,
                Default::default()
            ));

            if let Some(project) = &mut self.project {
                project.source_image = Some(dynamic_img);
                let metadata = file_utils::extract_tiff_metadata(red_path);
                project.pixel_scale = metadata.pixel_scale;
                project.tie_point = metadata.tie_point;
                project.geokeys = metadata.geokeys;
                project.geo_double_params = metadata.geo_double_params;
                project.geo_ascii_params = metadata.geo_ascii_params;
            }

            self.ui.last_error = None;

        } else {
            self.ui.last_error = Some("Ошибка чтения файлов. Проверьте, не повреждены ли tiff снимки.".to_string());
        }
    }
}

impl eframe::App for SatelliteApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Drag and drop
        let mut dropped_file_path = None;
        ctx.input(|i| {
            if let Some(file) = i.raw.dropped_files.first() {
                if let Some(path) = &file.path {
                    dropped_file_path = Some(path.clone());
                }
            }
        });

        if let Some(path) = dropped_file_path {
            self.load_image_from_path(ctx, &path);
        }
        
        egui::SidePanel::left("settings_panel").show(ctx, |ui| {
            components::show_filter_setting(ui, &mut self.settings.hue_min, &mut self.settings.hue_max);
            ui.add_space(20.0);
            
            if ui.button("Загрузить обычное фото").clicked() {
                if let Some(path) = FileDialog::new().pick_file() {
                    self.load_image_from_path(ctx, Path::new(&path));
                }
            }

            ui.add_space(10.0);

            if ui.button("Загрузить tiff снимок").clicked() {
                if let Some(folder_path) = FileDialog::new().pick_folder() {
                    
                    // Вызов file_utils
                    let found_tiffs = file_utils::find_tiff_files(&folder_path);
                    
                    let mut project = ProjectData::new();
                    let band_paths = BandPaths {
                        available_tiffs: found_tiffs,
                        ..Default::default()
                    };

                    project.paths = Some(band_paths);
                    self.project = Some(project);

                    self.ui.last_error = None;
                    self.ui.mask_texture = None;
                }
            }

            ui.horizontal(|ui| {
                ui.label("Порог NDVI:");
                // Создаем ползунок от -1.0 до 1.0 (стандартный диапазон NDVI)
                ui.add(egui::Slider::new(&mut self.ndvi_threshold, -1.0..=1.0).step_by(0.05));
            });

            ui.add_space(20.0);
                
            if let Some(project) = &mut self.project {
                if let Some(band_paths) = &mut project.paths {
                    ui.group(|ui| {
                        ui.label("Сопоставление спектральных каналов:");
                        ui.add_space(5.0);

                        components::show_band_selector(ui, "Красный канал (Red)", &mut band_paths.red_tiff, &band_paths.available_tiffs);
                        ui.add_space(5.0);
                        
                        components::show_band_selector(ui, "Зеленый канал (Green)", &mut band_paths.green_tiff, &band_paths.available_tiffs);
                        ui.add_space(5.0);

                        components::show_band_selector(ui, "Синий канал (Blue)", &mut band_paths.blue_tiff, &band_paths.available_tiffs);
                        ui.add_space(5.0);

                        components::show_band_selector(ui, "Инфракрасный канал (NIR)", &mut band_paths.nir_tiff, &band_paths.available_tiffs);
                    });
                }
            }

            if ui.button("Собрать цветное изображение").clicked() {
                if let Some(project) = &mut self.project {
                    if let Some(band_paths) = &project.paths {
                        
                        if let (Some(red_path), Some(green_path), Some(blue_path)) = 
                            (&band_paths.red_tiff, &band_paths.green_tiff, &band_paths.blue_tiff) 
                        {
                            let r_clone = red_path.clone();
                            let g_clone = green_path.clone();
                            let b_clone = blue_path.clone();

                            self.load_tiff_from_path(ctx, &r_clone, &g_clone, &b_clone);
                        } else {
                            self.ui.last_error = Some("Пожалуйста, выберите Red, Green и Blue каналы!".to_string());
                        }
                        
                    }
                }
            }
            
            ui.add_space(20.0);

            if ui.button("Анализировать снимок").clicked() {
                if let Some(project) = &mut self.project {
                    if let Some(source_image) = &project.source_image {
                        
                        // ВАЖНО: Мы больше не вычисляем здесь pixel_size. 
                        // Мы просто передаем 1.0, чтобы функция посчитала количество (штуки) пикселей.
                        match analyzer::run_analysis(source_image, self.settings.hue_min, self.settings.hue_max, 1.0) {
                            Ok(result) => {
                                // Сохраняем математические результаты в проект
                                project.field_area_pixels = result.area_pixels;
                                project.mask = Some(result.overlaid_image.clone());
                                
                                // Сохраняем визуальные результаты в UI
                                let color_img = egui::ColorImage::from_rgb(
                                    [result.overlaid_image.width() as usize, result.overlaid_image.height() as usize],
                                    result.overlaid_image.as_raw()
                                );

                                self.ui.mask_texture = Some(ctx.load_texture(
                                    "mask_image",
                                    color_img,
                                    Default::default()
                                ));

                                self.ui.last_error = None; // Успех
                            },
                            Err(e) => {
                                self.ui.last_error = Some(format!("Ошибка при анализе: {}", e));
                            }
                        }
                    } else {
                        self.ui.last_error = Some("Изображение в проекте отсутствует!".to_string());
                    }
                } else {
                    self.ui.last_error = Some("Сначала выберите файл!".to_string());
                }
            }

            ui.add_space(20.0);
            if ui.button("Рассчитать NDVI маску").clicked() {
                if let Some(project) = &mut self.project {
                    if let Some(paths) = &project.paths {
                        if let (Some(red), Some(nir)) = (&paths.red_tiff, &paths.nir_tiff) {
                            
                            // Вызываем расчет маски
                            match color::ndvi_filter_image(red, nir, self.ndvi_threshold) {
                                Ok(raw_mask) => {
                                    // 1. Убираем шум
                                    let clean_mask = filter::remove_noise(&raw_mask, 8);
                                    
                                    // ВАЖНО: Передаем 1.0 как f64 и убираем лишнее приведение типов (as f64)
                                    match metrics::calculate_area(&clean_mask, 1.0) {
                                        Ok(s) => project.field_area_pixels = s,
                                        Err(e) => println!("Ошибка расчета площади: {}", e),
                                    }

                                    // 2. Берем исходное фото для наложения
                                    if let Some(source_img) = &project.source_image {
                                        let source_rgb = source_img.to_rgb8();
                                        
                                        // 3. Создаем наложение (функция overlay_mask должна быть в color.rs)
                                        let overlaid = color::overlay_mask(&source_rgb, &clean_mask);
                                        
                                        // 4. Сохраняем в проект
                                        project.mask = Some(overlaid.clone());
                                        
                                        // 5. Подготовка для отображения
                                        let color_img = egui::ColorImage::from_rgb(
                                            [overlaid.width() as usize, overlaid.height() as usize],
                                            overlaid.as_raw()
                                        );
                                        
                                        self.ui.mask_texture = Some(ctx.load_texture(
                                            "mask_image",
                                            color_img,
                                            Default::default()
                                        ));
                                        
                                        self.ui.last_error = None; 
                                    } else {
                                        self.ui.last_error = Some("Сначала загрузите спутниковый снимок!".to_string());
                                    }
                                }
                                Err(e) => {
                                    self.ui.last_error = Some(format!("Ошибка расчета NDVI: {}", e));
                                }
                            }
                        } else {
                            self.ui.last_error = Some("Для NDVI нужны Red и NIR каналы!".to_string());
                        }
                    }
                }
            }
 
            // Расчет и вывод площади в зависимости от наличия метаданных масштаба пикселя
            if let Some(project) = &self.project {
                if let Some(scale) = &project.pixel_scale {
                    // Перевод площади если есть метаданные
                    let pixel_area_m2 = scale[0] * scale[1];
                    let total_area_m2 = project.field_area_pixels * pixel_area_m2;
                    let area_hectares = total_area_m2 / 10000.0;

                    // Вывод площади в гектарах
                    ui.label(format!("Площадь полей: {:.2} га", area_hectares));
                    
                    // Вывод площади в пикселях
                    ui.small(format!("({:.0} пикселей)", project.field_area_pixels));
                } else {
                    // Если нет метаданных выводяться площадь только в пикселях
                    ui.label(format!("Площадь полей: {:.0} пикселей", project.field_area_pixels));
                }
            } else {
                // Если проект еще не загружен
                ui.label("Площадь полей: 0 пикселей");
            }
            
            ui.add_space(10.0);

            if ui.button("Сохранить результат").clicked() {
                if let Some(project) = &self.project {
                    if let Some(mask) = &project.mask {
                        if let Some(path) = FileDialog::new().save_file() {
                            if let (Some(scale), Some(tie), Some(keys)) = 
                                (&project.pixel_scale, &project.tie_point, &project.geokeys) 
                            {

                        let double_params = project.geo_double_params.as_deref();
                        let ascii_params = project.geo_ascii_params.as_deref();
                        file_utils::save_tiff(
                            path.to_str().unwrap(), 
                            mask.width(), 
                            mask.height(), 
                            &mask.as_raw(), 
                            scale, 
                            tie, 
                            keys,
                            double_params,
                            ascii_params
                        );
                            } else {
                                match mask.save(&path) {
                                    Ok(_) => self.ui.last_error = None,
                                    Err(e) => self.ui.last_error = Some(format!("Ошибка сохранения: {}", e))
                                }
                            }
                        }
                    } else {
                        self.ui.last_error = Some("Нет маски для сохранения. Запустите анализ.".to_string());
                    }
                } else {
                    self.ui.last_error = Some("Нет открытого проекта.".to_string());
                }
            }

            // Вывод ошибок в интерфейс
            components::show_error_message(ui, &self.ui.last_error);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Анализ спутникового снимка");
            ui.separator();
            ui.add_space(10.0);
            components::show_image_viewer(ui, &self.ui.source_texture, &self.ui.mask_texture);
        });
    }
}