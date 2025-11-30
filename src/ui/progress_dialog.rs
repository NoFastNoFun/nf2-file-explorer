use egui::{Context, Window, ProgressBar};

pub struct ProgressState {
    pub show: bool,
    pub operation: String,
    pub current: u64,
    pub total: u64,
    pub cancelled: bool,
}

impl ProgressState {
    pub fn new() -> Self {
        Self {
            show: false,
            operation: String::new(),
            current: 0,
            total: 0,
            cancelled: false,
        }
    }

    pub fn start(&mut self, operation: String, total: u64) {
        self.show = true;
        self.operation = operation;
        self.current = 0;
        self.total = total;
        self.cancelled = false;
    }

    pub fn update(&mut self, current: u64) {
        self.current = current;
    }

    pub fn finish(&mut self) {
        self.show = false;
        self.operation.clear();
        self.current = 0;
        self.total = 0;
    }

    pub fn cancel(&mut self) {
        self.cancelled = true;
    }

    pub fn progress(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            (self.current as f32 / self.total as f32).min(1.0)
        }
    }
}

pub fn render_progress_dialog(ctx: &Context, progress: &mut ProgressState) {
    if !progress.show {
        return;
    }

    let mut open = true;
    Window::new("Progress")
        .collapsible(false)
        .resizable(false)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.label(&progress.operation);
            ui.separator();
            
            let progress_bar = ProgressBar::new(progress.progress())
                .show_percentage();
            ui.add(progress_bar);
            
            ui.label(format!("{} / {} bytes", progress.current, progress.total));
            
            ui.separator();
            
            if ui.button("Cancel").clicked() {
                progress.cancel();
            }
        });

    if !open {
        progress.cancel();
    }
}


