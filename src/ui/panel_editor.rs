/// Panel editor - allows users to create/edit transmission panels
use egui::{self, Ui};

use crate::config::tx_settings::{PanelWidget, TxPanel};
use crate::dbc::database::DbcDatabase;
use crate::i18n::t;

/// Widget type for creation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewWidgetType {
    Switch,
    ValueBox,
    SelectBox,
    Label,
}

pub struct PanelEditorState {
    pub editing_panel_index: Option<usize>,
    pub new_widget_type: NewWidgetType,
    pub widget_counter: u32,
}

impl PanelEditorState {
    pub fn new() -> Self {
        Self {
            editing_panel_index: None,
            new_widget_type: NewWidgetType::Switch,
            widget_counter: 0,
        }
    }
}

impl Default for PanelEditorState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn draw_panel_editor(
    ui: &mut Ui,
    panels: &mut Vec<TxPanel>,
    editor_state: &mut PanelEditorState,
    dbc: &Option<DbcDatabase>,
) {
    ui.heading(t("panel.title"));
    ui.separator();

    // Panel list
    ui.horizontal(|ui| {
        if ui.button(t("panel.new")).clicked() {
            panels.push(TxPanel {
                name: format!("Panel {}", panels.len() + 1),
                widgets: Vec::new(),
            });
            editor_state.editing_panel_index = Some(panels.len() - 1);
        }
    });

    let mut remove_panel = None;
    for (i, panel) in panels.iter().enumerate() {
        ui.horizontal(|ui| {
            if ui.selectable_label(
                editor_state.editing_panel_index == Some(i),
                &panel.name,
            ).clicked() {
                editor_state.editing_panel_index = Some(i);
            }
            if ui.small_button("X").clicked() {
                remove_panel = Some(i);
            }
        });
    }
    if let Some(idx) = remove_panel {
        panels.remove(idx);
        editor_state.editing_panel_index = None;
    }

    ui.separator();

    // Edit selected panel
    if let Some(panel_idx) = editor_state.editing_panel_index {
        if panel_idx < panels.len() {
            let panel = &mut panels[panel_idx];

            ui.horizontal(|ui| {
                ui.label(t("panel.name"));
                ui.text_edit_singleline(&mut panel.name);
            });

            ui.separator();

            // Add widget controls
            ui.horizontal(|ui| {
                ui.label(t("panel.add_widget"));
                ui.radio_value(&mut editor_state.new_widget_type, NewWidgetType::Switch, t("panel.switch"));
                ui.radio_value(&mut editor_state.new_widget_type, NewWidgetType::ValueBox, t("panel.value_box"));
                ui.radio_value(&mut editor_state.new_widget_type, NewWidgetType::SelectBox, t("panel.select_box"));
                ui.radio_value(&mut editor_state.new_widget_type, NewWidgetType::Label, t("panel.label"));

                if ui.button(t("panel.add")).clicked() {
                    editor_state.widget_counter += 1;
                    let id = format!("w_{}", editor_state.widget_counter);
                    let widget = match editor_state.new_widget_type {
                        NewWidgetType::Switch => PanelWidget::Switch {
                            id,
                            label: t("panel.switch").to_string(),
                            x: 0.0,
                            y: 0.0,
                            width: 100.0,
                            height: 30.0,
                            on_modification: None,
                            off_modification: None,
                            state: false,
                        },
                        NewWidgetType::ValueBox => PanelWidget::ValueBox {
                            id,
                            label: t("panel.value_box").to_string(),
                            x: 0.0,
                            y: 0.0,
                            width: 150.0,
                            height: 30.0,
                            message_id: None,
                            signal_name: None,
                            value: 0.0,
                            min: 0.0,
                            max: 100.0,
                            unit: String::new(),
                        },
                        NewWidgetType::SelectBox => PanelWidget::SelectBox {
                            id,
                            label: t("panel.select_box").to_string(),
                            x: 0.0,
                            y: 0.0,
                            width: 150.0,
                            height: 30.0,
                            options: vec![("Option 1".to_string(), 0.0)],
                            selected_index: 0,
                            message_id: None,
                            signal_name: None,
                        },
                        NewWidgetType::Label => PanelWidget::Label {
                            id,
                            text: t("panel.label").to_string(),
                            x: 0.0,
                            y: 0.0,
                            width: 100.0,
                            height: 25.0,
                            font_size: 14.0,
                        },
                    };
                    panel.widgets.push(widget);
                }
            });

            ui.separator();

            // Widget list with properties
            let mut remove_widget = None;
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (i, widget) in panel.widgets.iter_mut().enumerate() {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            let type_name = match widget {
                                PanelWidget::Switch { .. } => t("panel.switch"),
                                PanelWidget::ValueBox { .. } => t("panel.value_box"),
                                PanelWidget::SelectBox { .. } => t("panel.select_box"),
                                PanelWidget::Label { .. } => t("panel.label"),
                            };
                            ui.strong(format!("[{}] {}", type_name, widget.id()));

                            if ui.small_button("X").clicked() {
                                remove_widget = Some(i);
                            }
                        });

                        draw_widget_properties(ui, widget, dbc, i);
                    });
                }
            });
            if let Some(idx) = remove_widget {
                panel.widgets.remove(idx);
            }
        }
    }
}

fn draw_widget_properties(
    ui: &mut Ui,
    widget: &mut PanelWidget,
    dbc: &Option<DbcDatabase>,
    widget_idx: usize,
) {
    match widget {
        PanelWidget::Switch { label, on_modification, off_modification, .. } => {
            ui.horizontal(|ui| {
                ui.label(t("panel.label_field"));
                ui.text_edit_singleline(label);
            });
            draw_modification_config(ui, t("panel.on_action"), on_modification, dbc, widget_idx, "on");
            draw_modification_config(ui, t("panel.off_action"), off_modification, dbc, widget_idx, "off");
        }
        PanelWidget::ValueBox {
            label, min, max, unit, message_id, signal_name, ..
        } => {
            ui.horizontal(|ui| {
                ui.label(t("panel.label_field"));
                ui.text_edit_singleline(label);
            });

            draw_signal_link(ui, message_id, signal_name, min, max, unit, dbc, widget_idx);
        }
        PanelWidget::SelectBox {
            label, options, message_id, signal_name, ..
        } => {
            ui.horizontal(|ui| {
                ui.label(t("panel.label_field"));
                ui.text_edit_singleline(label);
            });

            // Link to signal
            let mut dummy_min = 0.0;
            let mut dummy_max = 100.0;
            let mut dummy_unit = String::new();
            draw_signal_link(ui, message_id, signal_name, &mut dummy_min, &mut dummy_max, &mut dummy_unit, dbc, widget_idx);

            // Options
            ui.label(t("panel.options"));
            let mut remove_opt = None;
            for (j, (text, val)) in options.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(text);
                    ui.add(egui::DragValue::new(val).speed(0.1));
                    if ui.small_button("X").clicked() {
                        remove_opt = Some(j);
                    }
                });
            }
            if let Some(idx) = remove_opt {
                options.remove(idx);
            }
            if ui.small_button(t("panel.add_option")).clicked() {
                options.push((t("panel.new_option").to_string(), 0.0));
            }
        }
        PanelWidget::Label { text, font_size, .. } => {
            ui.horizontal(|ui| {
                ui.label(t("panel.text"));
                ui.text_edit_singleline(text);
            });
            ui.horizontal(|ui| {
                ui.label(t("panel.font_size"));
                ui.add(egui::DragValue::new(font_size).range(8.0..=32.0));
            });
        }
    }
}

fn draw_modification_config(
    ui: &mut Ui,
    label: &str,
    modification: &mut Option<crate::can::transmitter::SignalModification>,
    dbc: &Option<DbcDatabase>,
    widget_idx: usize,
    suffix: &str,
) {
    ui.collapsing(label, |ui| {
        let has_mod = modification.is_some();
        let mut enabled = has_mod;
        if ui.checkbox(&mut enabled, t("panel.enabled")).changed() {
            if enabled && !has_mod {
                *modification = Some(crate::can::transmitter::SignalModification {
                    message_id: 0,
                    signal_name: String::new(),
                    value: 0.0,
                });
            } else if !enabled {
                *modification = None;
            }
        }

        if let Some(ref mut m) = modification {
            ui.horizontal(|ui| {
                ui.label(t("panel.msg_id"));
                let mut id_str = format!("{:03X}", m.message_id);
                if ui.add(egui::TextEdit::singleline(&mut id_str).desired_width(60.0)).changed() {
                    if let Ok(id) = u32::from_str_radix(&id_str, 16) {
                        m.message_id = id;
                    }
                }
            });

            if let Some(ref dbc_db) = dbc {
                if let Some(msg) = dbc_db.get_message(m.message_id) {
                    ui.horizontal(|ui| {
                        ui.label(t("tx.signal"));
                        egui::ComboBox::from_id_salt(format!("mod_{}_{}", widget_idx, suffix))
                            .selected_text(&m.signal_name)
                            .show_ui(ui, |ui| {
                                for sig in &msg.signals {
                                    ui.selectable_value(
                                        &mut m.signal_name,
                                        sig.name.clone(),
                                        &sig.name,
                                    );
                                }
                            });
                    });

                    if let Some(sig) = msg.get_signal(&m.signal_name) {
                        ui.horizontal(|ui| {
                            ui.label(t("panel.value"));
                            ui.add(
                                egui::DragValue::new(&mut m.value)
                                    .range(sig.min..=sig.max)
                                    .speed(0.1),
                            );
                            ui.label(&sig.unit);
                        });
                    }
                }
            }
        }
    });
}

fn draw_signal_link(
    ui: &mut Ui,
    message_id: &mut Option<u32>,
    signal_name: &mut Option<String>,
    min: &mut f64,
    max: &mut f64,
    unit: &mut String,
    dbc: &Option<DbcDatabase>,
    widget_idx: usize,
) {
    if let Some(ref dbc_db) = dbc {
        ui.horizontal(|ui| {
            ui.label(t("panel.message"));
            let mut mid = message_id.unwrap_or(0);
            let mut id_str = format!("{:03X}", mid);
            if ui.add(egui::TextEdit::singleline(&mut id_str).desired_width(60.0)).changed() {
                if let Ok(id) = u32::from_str_radix(&id_str, 16) {
                    *message_id = Some(id);
                    mid = id;
                }
            }

            if let Some(msg) = dbc_db.get_message(mid) {
                ui.label(t("tx.signal"));
                let mut sname = signal_name.clone().unwrap_or_default();
                egui::ComboBox::from_id_salt(format!("siglink_{}", widget_idx))
                    .selected_text(&sname)
                    .show_ui(ui, |ui| {
                        for sig in &msg.signals {
                            if ui.selectable_value(&mut sname, sig.name.clone(), &sig.name).changed() {
                                *min = sig.min;
                                *max = sig.max;
                                *unit = sig.unit.clone();
                            }
                        }
                    });
                *signal_name = Some(sname);
            }
        });
    }
}
