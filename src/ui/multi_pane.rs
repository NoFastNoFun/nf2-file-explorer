use egui::{Ui, Context, Rect, Vec2};
use std::path::PathBuf;
use crate::ui::tabs::Tab;
use crate::ui::file_list::FileListView;
use crate::ui::search_bar;
use crate::app::AppState;
use crate::fs::directory::FileEntry;
use tokio::runtime::Runtime;

#[derive(Clone, Copy, PartialEq)]
pub enum PaneLayout {
    Single,
    DualHorizontal,
    DualVertical,
}

pub struct PaneState {
    pub current_path: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected_indices: Vec<usize>,
    pub tabs: Vec<Tab>,
    pub active_tab_index: usize,
    pub file_list_view: FileListView,
    pub filter_text: String,
    pub loading: bool,
    pub error_message: Option<String>,
}

impl PaneState {
    pub fn new(path: PathBuf) -> Self {
        let mut tabs = Vec::new();
        let tab = Tab::new(path.clone());
        tabs.push(tab);
        
        Self {
            current_path: path,
            entries: Vec::new(),
            selected_indices: Vec::new(),
            tabs,
            active_tab_index: 0,
            file_list_view: FileListView::new(),
            filter_text: String::new(),
            loading: false,
            error_message: None,
        }
    }

    pub fn active_tab(&self) -> Option<&Tab> {
        self.tabs.get(self.active_tab_index)
    }

    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.get_mut(self.active_tab_index)
    }

    pub fn add_tab(&mut self, path: PathBuf) {
        let tab = Tab::new(path);
        self.tabs.push(tab);
        self.active_tab_index = self.tabs.len() - 1;
    }

    pub fn close_tab(&mut self, index: usize) {
        if self.tabs.len() <= 1 {
            return;
        }
        self.tabs.remove(index);
        if self.active_tab_index >= self.tabs.len() {
            self.active_tab_index = self.tabs.len().saturating_sub(1);
        }
        if let Some(tab) = self.tabs.get(self.active_tab_index) {
            self.current_path = tab.path.clone();
        }
    }

    pub fn switch_to_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_tab_index = index;
            if let Some(tab) = self.tabs.get(index) {
                self.current_path = tab.path.clone();
            }
        }
    }
}

pub struct MultiPaneView {
    pub layout: PaneLayout,
    pub left_pane: PaneState,
    pub right_pane: Option<PaneState>,
    pub split_ratio: f32,
    pub is_dragging_splitter: bool,
}

impl MultiPaneView {
    pub fn new() -> Self {
        let current_path = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("C:\\"));
        
        Self {
            layout: PaneLayout::Single,
            left_pane: PaneState::new(current_path),
            right_pane: None,
            split_ratio: 0.5,
            is_dragging_splitter: false,
        }
    }

    pub fn active_pane_mut(&mut self) -> &mut PaneState {
        &mut self.left_pane
    }

    pub fn toggle_dual_pane(&mut self) {
        match self.layout {
            PaneLayout::Single => {
                self.layout = PaneLayout::DualHorizontal;
                let current_path = self.left_pane.current_path.clone();
                self.right_pane = Some(PaneState::new(current_path));
            }
            _ => {
                self.layout = PaneLayout::Single;
                self.right_pane = None;
            }
        }
    }

    pub fn set_layout(&mut self, layout: PaneLayout) {
        self.layout = layout;
        if layout == PaneLayout::Single {
            self.right_pane = None;
        } else if self.right_pane.is_none() {
            let current_path = self.left_pane.current_path.clone();
            self.right_pane = Some(PaneState::new(current_path));
        }
    }

    pub fn render_tabs(ui: &mut Ui, pane: &mut PaneState, rt: &Runtime, app: &mut AppState, is_active_pane: bool) {
        ui.horizontal(|ui| {
            let mut tabs_to_close = Vec::new();
            let mut tab_to_switch = None;
            
            for (idx, tab) in pane.tabs.iter().enumerate() {
                let is_active = idx == pane.active_tab_index;
                let response = ui.selectable_label(is_active, &tab.title);
                
                if response.clicked() {
                    tab_to_switch = Some(idx);
                    if is_active_pane {
                        if let Some(tab) = pane.tabs.get(idx) {
                            rt.block_on(app.navigate_to(tab.path.clone()));
                        }
                    }
                }

                if response.middle_clicked() {
                    tabs_to_close.push(idx);
                }

                if ui.button("×").clicked() && pane.tabs.len() > 1 {
                    tabs_to_close.push(idx);
                }
            }
            
            for idx in tabs_to_close {
                pane.close_tab(idx);
            }
            
            if let Some(idx) = tab_to_switch {
                pane.switch_to_tab(idx);
            }

            if ui.button("+").clicked() {
                pane.add_tab(pane.current_path.clone());
            }
        });
    }

    pub fn render(
        &mut self,
        ui: &mut Ui,
        ctx: &Context,
        app: &mut AppState,
        rt: &Runtime,
    ) {
        match self.layout {
            PaneLayout::Single => {
                self.render_single_pane(ui, ctx, app, rt);
            }
            PaneLayout::DualHorizontal => {
                self.render_dual_horizontal(ui, ctx, app, rt);
            }
            PaneLayout::DualVertical => {
                self.render_dual_vertical(ui, ctx, app, rt);
            }
        }
    }

    fn render_single_pane(
        &mut self,
        ui: &mut Ui,
        ctx: &Context,
        app: &mut AppState,
        rt: &Runtime,
    ) {
        ui.vertical(|ui| {
            Self::render_tabs(ui, &mut self.left_pane, rt, app, true);
            ui.separator();
            
            search_bar::render_search_bar(ui, app, rt);
            ui.separator();

            if self.left_pane.loading {
                ui.centered_and_justified(|ui| {
                    ui.spinner();
                    ui.label("Loading...");
                });
            } else if let Some(ref error) = self.left_pane.error_message {
                ui.colored_label(egui::Color32::RED, error);
            } else {
                let old_path = app.navigation.current_path.clone();
                let old_entries = app.entries.clone();
                let old_selected = app.selected_indices.clone();
                let old_loading = app.loading;
                let old_error = app.ui.error_message.clone();
                
                app.navigation.current_path = self.left_pane.current_path.clone();
                app.entries = self.left_pane.entries.clone();
                app.selected_indices = self.left_pane.selected_indices.clone();
                app.loading = self.left_pane.loading;
                app.ui.error_message = self.left_pane.error_message.clone();
                
                self.left_pane.file_list_view.render(ui, app, rt, ctx);
                
                self.left_pane.selected_indices = app.selected_indices.clone();
                self.left_pane.entries = app.entries.clone();
                
                app.navigation.current_path = old_path;
                app.entries = old_entries;
                app.selected_indices = old_selected;
                app.loading = old_loading;
                app.ui.error_message = old_error;
            }
        });
    }

    fn render_dual_horizontal(
        &mut self,
        ui: &mut Ui,
        ctx: &Context,
        app: &mut AppState,
        rt: &Runtime,
    ) {
        let available_rect = ui.available_rect_before_wrap();
        let splitter_width = 4.0;
        let left_width = available_rect.width() * self.split_ratio - splitter_width / 2.0;
        let right_width = available_rect.width() * (1.0 - self.split_ratio) - splitter_width / 2.0;

        ui.horizontal(|ui| {
            ui.set_width(left_width);
            ui.vertical(|ui| {
                Self::render_tabs(ui, &mut self.left_pane, rt, app, true);
                ui.separator();
                search_bar::render_search_bar(ui, app, rt);
                ui.separator();
                
                let old_path = app.navigation.current_path.clone();
                let old_entries = app.entries.clone();
                let old_selected = app.selected_indices.clone();
                let old_loading = app.loading;
                let old_error = app.ui.error_message.clone();
                
                app.navigation.current_path = self.left_pane.current_path.clone();
                app.entries = self.left_pane.entries.clone();
                app.selected_indices = self.left_pane.selected_indices.clone();
                app.loading = self.left_pane.loading;
                app.ui.error_message = self.left_pane.error_message.clone();
                
                self.left_pane.file_list_view.render(ui, app, rt, ctx);
                
                self.left_pane.selected_indices = app.selected_indices.clone();
                self.left_pane.entries = app.entries.clone();
                
                app.navigation.current_path = old_path;
                app.entries = old_entries;
                app.selected_indices = old_selected;
                app.loading = old_loading;
                app.ui.error_message = old_error;
            });

            let splitter_rect = Rect::from_min_size(
                ui.cursor().min + Vec2::new(0.0, 0.0),
                Vec2::new(splitter_width, available_rect.height()),
            );
            let splitter_response = ui.allocate_rect(splitter_rect, egui::Sense::drag());
            
            ui.painter().rect_filled(
                splitter_rect,
                0.0,
                egui::Color32::from_gray(100),
            );

            if splitter_response.dragged() {
                let delta = splitter_response.drag_delta().x;
                let total_width = available_rect.width();
                self.split_ratio = ((self.split_ratio * total_width + delta) / total_width)
                    .clamp(0.1, 0.9);
            }

            ui.set_width(right_width);
            ui.vertical(|ui| {
                if let Some(ref mut right_pane) = self.right_pane {
                    Self::render_tabs(ui, right_pane, rt, app, false);
                    ui.separator();
                    
                    let old_path = app.navigation.current_path.clone();
                    let old_entries = app.entries.clone();
                    let old_selected = app.selected_indices.clone();
                    let old_loading = app.loading;
                    let old_error = app.ui.error_message.clone();
                    
                    app.navigation.current_path = right_pane.current_path.clone();
                    app.entries = right_pane.entries.clone();
                    app.selected_indices = right_pane.selected_indices.clone();
                    app.loading = right_pane.loading;
                    app.ui.error_message = right_pane.error_message.clone();
                    
                    right_pane.file_list_view.render(ui, app, rt, ctx);
                    
                    right_pane.selected_indices = app.selected_indices.clone();
                    right_pane.entries = app.entries.clone();
                    
                    app.navigation.current_path = old_path;
                    app.entries = old_entries;
                    app.selected_indices = old_selected;
                    app.loading = old_loading;
                    app.ui.error_message = old_error;
                }
            });
        });
    }

    fn render_dual_vertical(
        &mut self,
        ui: &mut Ui,
        ctx: &Context,
        app: &mut AppState,
        rt: &Runtime,
    ) {
        let available_rect = ui.available_rect_before_wrap();
        let splitter_height = 4.0;
        let top_height = available_rect.height() * self.split_ratio - splitter_height / 2.0;
        let bottom_height = available_rect.height() * (1.0 - self.split_ratio) - splitter_height / 2.0;

        ui.vertical(|ui| {
            ui.set_height(top_height);
            ui.vertical(|ui| {
                Self::render_tabs(ui, &mut self.left_pane, rt, app, true);
                ui.separator();
                search_bar::render_search_bar(ui, app, rt);
                ui.separator();
                
                let old_path = app.navigation.current_path.clone();
                let old_entries = app.entries.clone();
                let old_selected = app.selected_indices.clone();
                let old_loading = app.loading;
                let old_error = app.ui.error_message.clone();
                
                app.navigation.current_path = self.left_pane.current_path.clone();
                app.entries = self.left_pane.entries.clone();
                app.selected_indices = self.left_pane.selected_indices.clone();
                app.loading = self.left_pane.loading;
                app.ui.error_message = self.left_pane.error_message.clone();
                
                self.left_pane.file_list_view.render(ui, app, rt, ctx);
                
                self.left_pane.selected_indices = app.selected_indices.clone();
                self.left_pane.entries = app.entries.clone();
                
                app.navigation.current_path = old_path;
                app.entries = old_entries;
                app.selected_indices = old_selected;
                app.loading = old_loading;
                app.ui.error_message = old_error;
            });

            let splitter_rect = Rect::from_min_size(
                ui.cursor().min + Vec2::new(0.0, 0.0),
                Vec2::new(available_rect.width(), splitter_height),
            );
            let splitter_response = ui.allocate_rect(splitter_rect, egui::Sense::drag());
            
            ui.painter().rect_filled(
                splitter_rect,
                0.0,
                egui::Color32::from_gray(100),
            );

            if splitter_response.dragged() {
                let delta = splitter_response.drag_delta().y;
                let total_height = available_rect.height();
                self.split_ratio = ((self.split_ratio * total_height + delta) / total_height)
                    .clamp(0.1, 0.9);
            }

            ui.set_height(bottom_height);
            ui.vertical(|ui| {
                if let Some(ref mut right_pane) = self.right_pane {
                    Self::render_tabs(ui, right_pane, rt, app, false);
                    ui.separator();
                    
                    let old_path = app.navigation.current_path.clone();
                    let old_entries = app.entries.clone();
                    let old_selected = app.selected_indices.clone();
                    let old_loading = app.loading;
                    let old_error = app.ui.error_message.clone();
                    
                    app.navigation.current_path = right_pane.current_path.clone();
                    app.entries = right_pane.entries.clone();
                    app.selected_indices = right_pane.selected_indices.clone();
                    app.loading = right_pane.loading;
                    app.ui.error_message = right_pane.error_message.clone();
                    
                    right_pane.file_list_view.render(ui, app, rt, ctx);
                    
                    right_pane.selected_indices = app.selected_indices.clone();
                    right_pane.entries = app.entries.clone();
                    
                    app.navigation.current_path = old_path;
                    app.entries = old_entries;
                    app.selected_indices = old_selected;
                    app.loading = old_loading;
                    app.ui.error_message = old_error;
                }
            });
        });
    }
}

