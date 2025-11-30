use egui::Ui;
use crate::app::AppState;

pub fn render_status_bar(ui: &mut Ui, app: &AppState) {
    ui.horizontal(|ui| {
        let selected_count = app.selected_indices.len();
        let total_count = app.entries.len();

        if selected_count > 0 {
            ui.label(format!("{} selected", selected_count));
        } else {
            ui.label(format!("{} items", total_count));
        }

        ui.separator();

        if selected_count > 0 {
            let total_size: u64 = app
                .get_selected_entries()
                .iter()
                .map(|e| e.size)
                .sum();
            ui.label(format!("Total size: {}", format_bytes(total_size)));
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(app.navigation.current_path.to_string_lossy().to_string());
        });
    });
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

