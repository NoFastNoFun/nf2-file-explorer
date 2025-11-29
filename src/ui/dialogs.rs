use egui::{Context, Window};
use std::path::PathBuf;

pub enum DialogAction {
    Confirm,
    Cancel,
    None,
}

pub struct DeleteDialog {
    pub show: bool,
    pub file_count: usize,
    pub file_names: Vec<String>,
}

impl DeleteDialog {
    pub fn new() -> Self {
        Self {
            show: false,
            file_count: 0,
            file_names: Vec::new(),
        }
    }

    pub fn show(&mut self, file_count: usize, file_names: Vec<String>) {
        self.show = true;
        self.file_count = file_count;
        self.file_names = file_names;
    }

    pub fn render(&mut self, ctx: &Context) -> DialogAction {
        if !self.show {
            return DialogAction::None;
        }

        let mut action = DialogAction::None;
        let mut open = true;

        Window::new("Confirm Delete")
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    if self.file_count == 1 {
                        ui.label(format!("Are you sure you want to delete '{}'?", 
                            self.file_names.first().unwrap_or(&"this item".to_string())));
                    } else {
                        ui.label(format!("Are you sure you want to delete {} items?", self.file_count));
                        if self.file_count <= 5 {
                            ui.separator();
                            egui::ScrollArea::vertical()
                                .max_height(100.0)
                                .show(ui, |ui| {
                                    for name in &self.file_names {
                                        ui.label(format!("  • {}", name));
                                    }
                                });
                        }
                    }

                    ui.label("This action cannot be undone.");

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("Delete").clicked() {
                            action = DialogAction::Confirm;
                            self.show = false;
                        }
                        if ui.button("Cancel").clicked() {
                            action = DialogAction::Cancel;
                            self.show = false;
                        }
                    });
                });
            });

        if !open {
            self.show = false;
            if matches!(action, DialogAction::None) {
                action = DialogAction::Cancel;
            }
        }

        action
    }
}

pub struct OverwriteDialog {
    pub show: bool,
    pub file_name: String,
    pub existing_path: PathBuf,
    pub new_path: PathBuf,
}

impl OverwriteDialog {
    pub fn new() -> Self {
        Self {
            show: false,
            file_name: String::new(),
            existing_path: PathBuf::new(),
            new_path: PathBuf::new(),
        }
    }

    pub fn show(&mut self, file_name: String, existing_path: PathBuf, new_path: PathBuf) {
        self.show = true;
        self.file_name = file_name;
        self.existing_path = existing_path;
        self.new_path = new_path;
    }

    pub fn render(&mut self, ctx: &Context) -> DialogAction {
        if !self.show {
            return DialogAction::None;
        }

        let mut action = DialogAction::None;
        let mut open = true;

        Window::new("File Already Exists")
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.label(format!("A file named '{}' already exists.", self.file_name));
                    ui.label("Do you want to replace it?");

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("Replace").clicked() {
                            action = DialogAction::Confirm;
                            self.show = false;
                        }
                        if ui.button("Skip").clicked() {
                            action = DialogAction::Cancel;
                            self.show = false;
                        }
                    });
                });
            });

        if !open {
            self.show = false;
            if matches!(action, DialogAction::None) {
                action = DialogAction::Cancel;
            }
        }

        action
    }
}

