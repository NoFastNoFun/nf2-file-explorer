use egui::Ui;
use std::path::PathBuf;
use crate::app::AppState;
use tokio::runtime::Runtime;

pub fn render_breadcrumb(ui: &mut Ui, app: &mut AppState, rt: &Runtime) {
    ui.horizontal(|ui| {
        let path = &app.navigation.current_path;
        let components: Vec<PathBuf> = path.components()
            .map(|c| c.as_os_str().to_os_string().into())
            .collect();

        let mut current_path = PathBuf::new();
        for (idx, component) in components.iter().enumerate() {
            if idx > 0 {
                ui.label(" > ");
            }

            current_path.push(component);
            let label = component.to_string_lossy();
            
            if ui.button(&*label).clicked() {
                rt.block_on(app.navigate_to(current_path.clone()));
            }
        }
    });
}

