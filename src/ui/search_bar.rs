use egui::{Ui, TextEdit};
use crate::app::AppState;
use tokio::runtime::Runtime;
use std::time::{Duration, Instant};

pub fn render_search_bar(ui: &mut Ui, app: &mut AppState, rt: &Runtime) {
    ui.horizontal(|ui| {
        ui.label("Search:");
        let response = ui.add(
            TextEdit::singleline(&mut app.search.options.query)
                .desired_width(200.0)
                .hint_text("Enter search query")
        );

        let mut should_search = response.changed();

        let recursive_response = ui.checkbox(&mut app.search.options.recursive, "Recursive");
        if recursive_response.changed() {
            should_search = true;
        }

        let global_response = ui.checkbox(&mut app.search.options.global, "Global (all drives)");
        if global_response.changed() {
            should_search = true;
        }

        let match_case_response = ui.checkbox(&mut app.search.options.match_case, "Match case");
        if match_case_response.changed() {
            should_search = true;
        }

        let file_only_response = ui.checkbox(&mut app.search.options.file_only, "Files only");
        if file_only_response.changed() {
            should_search = true;
        }

        let dir_only_response = ui.checkbox(&mut app.search.options.dir_only, "Folders only");
        if dir_only_response.changed() {
            should_search = true;
        }

        let regex_response = ui.checkbox(&mut app.search.options.use_regex, "Regex");
        if regex_response.changed() {
            should_search = true;
        }

        if should_search && !app.search.options.query.is_empty() {
            app.search.debounce_timer = Some(Instant::now());
        }

        // Check if debounce timer has elapsed
        if let Some(timer) = app.search.debounce_timer {
            if timer.elapsed() >= Duration::from_millis(400) {
                if !app.search.options.query.is_empty() {
                    rt.block_on(app.perform_search());
                }
                app.search.debounce_timer = None;
            }
        }

        if ui.button("Clear").clicked() {
            app.search.options.query.clear();
            app.search.is_searching = false;
            app.search.results.clear();
            app.search.debounce_timer = None;
        }
    });
}

