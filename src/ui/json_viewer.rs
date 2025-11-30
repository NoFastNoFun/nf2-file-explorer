use egui::{Ui, CollapsingHeader};
use serde_json::Value;

pub fn render_json_viewer(ui: &mut Ui, content: &str) {
    match serde_json::from_str::<Value>(content) {
        Ok(json) => {
            render_json_value(ui, &json, "", 0);
        }
        Err(_) => {
            ui.label("Invalid JSON");
        }
    }
}

fn render_json_value(ui: &mut Ui, value: &Value, key: &str, depth: usize) {
    match value {
        Value::Object(map) => {
            let header_text = if key.is_empty() { "Object" } else { key };
            CollapsingHeader::new(header_text)
                .default_open(depth < 2)
                .show(ui, |ui| {
                    for (k, v) in map {
                        render_json_value(ui, v, k, depth + 1);
                    }
                });
        }
        Value::Array(arr) => {
            let header_text = if key.is_empty() { "Array" } else { key };
            CollapsingHeader::new(format!("{} [{}]", header_text, arr.len()))
                .default_open(depth < 2)
                .show(ui, |ui| {
                    for (i, v) in arr.iter().enumerate() {
                        render_json_value(ui, v, &format!("[{}]", i), depth + 1);
                    }
                });
        }
        Value::String(s) => {
            ui.horizontal(|ui| {
                if !key.is_empty() {
                    ui.label(format!("{}: ", key));
                }
                ui.label(format!("\"{}\"", s));
            });
        }
        Value::Number(n) => {
            ui.horizontal(|ui| {
                if !key.is_empty() {
                    ui.label(format!("{}: ", key));
                }
                ui.label(n.to_string());
            });
        }
        Value::Bool(b) => {
            ui.horizontal(|ui| {
                if !key.is_empty() {
                    ui.label(format!("{}: ", key));
                }
                ui.label(b.to_string());
            });
        }
        Value::Null => {
            ui.horizontal(|ui| {
                if !key.is_empty() {
                    ui.label(format!("{}: ", key));
                }
                ui.label("null");
            });
        }
    }
}

pub fn render_yaml_viewer(ui: &mut Ui, content: &str) {
    match serde_yaml::from_str::<Value>(content) {
        Ok(yaml) => {
            render_json_value(ui, &yaml, "", 0);
        }
        Err(_) => {
            ui.label("Invalid YAML");
        }
    }
}

