use eframe::egui;
use crate::processor::color;

pub fn show_band_selector(ui: &mut egui::Ui, label: &str, selected_path: &mut Option<std::path::PathBuf>, available_tiffs: &[std::path::PathBuf]) {
    // Безопасно получаем имя файла для отображения в кнопке списка
    let selected_text = selected_path
        .as_ref()
        .and_then(|p| p.file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Не выбран".to_string());

    egui::ComboBox::from_label(label)
        .selected_text(selected_text)
        .show_ui(ui, |ui| {
            // Опция сброса канала обратно в None ("Не выбран")
            ui.selectable_value(selected_path, None, "Не выбран");
            ui.separator();

            // Выводим все доступные файлы из папки
            for tiff_path in available_tiffs {
                if let Some(file_name) = tiff_path.file_name() {
                    let file_name_str = file_name.to_string_lossy().to_string();
                    ui.selectable_value(
                        selected_path,
                        Some(tiff_path.clone()),
                        file_name_str,
                    );
                }
            }
        });
}

pub fn show_filter_setting(ui: &mut egui::Ui, hue_min: &mut f32, hue_max: &mut f32) {
    ui.heading("Настройки фильтра");
    ui.separator();
    ui.add(egui::Slider::new(hue_min, 0.0..=360.0).text("Мин. Оттенок"));
    ui.add(egui::Slider::new(hue_max, 0.0..=360.0).text("Макс. Оттенок"));
    ui.separator();
}

pub fn show_error_message(ui: &mut egui::Ui, error: &Option<String>) {
    if let Some(err) = error {
        ui.add_space(15.0);
            ui.colored_label(egui::Color32::RED, err);
    }
}
pub fn show_image_viewer(ui: &mut egui::Ui, source_tex: &Option<egui::TextureHandle>, mask_tex: &Option<egui::TextureHandle>) {
    ui.columns(2, |columns| {
        // Левая колонка
        columns[0].vertical(|ui| {
            ui.label("Исходный спутниковый снимок:");
            ui.add_space(5.0);
            if let Some(texture) = source_tex {
                // Узнаем доступную ширину колонки
                let width = ui.available_width();
                // Высчитываем правильную высоту, сохраняя пропорции
                let height = width * (texture.size()[1] as f32 / texture.size()[0] as f32);
                
                // Правильный синтаксис для новых версий egui:
                ui.add(egui::Image::new(texture).fit_to_exact_size(egui::vec2(width, height)));
            } else {
                ui.label("Изображение не загружено...");
            }
        });
        
        // Правая колонка
        columns[1].vertical(|ui| {
            ui.label("Выделенная маска полей:");
            ui.add_space(5.0);
            if let Some(texture) = mask_tex {
                let width = ui.available_width();
                let height = width * (texture.size()[1] as f32 / texture.size()[0] as f32);
                
                // Правильный синтаксис для новых версий egui:
                ui.add(egui::Image::new(texture).fit_to_exact_size(egui::vec2(width, height)));
            } else {
                ui.label("Запустите анализ для получения маски...");
            }
        });
    });
}