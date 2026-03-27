/// User-customizable transmission panel with widgets
use egui::{self, Ui};

use crate::config::tx_settings::{PanelWidget, TxPanel};
use crate::dbc::database::DbcDatabase;

/// Draw a transmission panel with its widgets
pub fn draw_tx_panel(
    ui: &mut Ui,
    panel: &mut TxPanel,
    _dbc: &Option<DbcDatabase>,
) -> Vec<PanelEvent> {
    let mut events = Vec::new();

    ui.group(|ui| {
        ui.heading(&panel.name);
        ui.separator();

        for widget in &mut panel.widgets {
            match widget {
                PanelWidget::Switch {
                    label,
                    state,
                    on_modification,
                    off_modification,
                    ..
                } => {
                    ui.horizontal(|ui| {
                        if ui.toggle_value(state, label.as_str()).changed() {
                            if *state {
                                if let Some(ref modification) = on_modification {
                                    events.push(PanelEvent::SignalChanged {
                                        message_id: modification.message_id,
                                        signal_name: modification.signal_name.clone(),
                                        value: modification.value,
                                    });
                                }
                            } else {
                                if let Some(ref modification) = off_modification {
                                    events.push(PanelEvent::SignalChanged {
                                        message_id: modification.message_id,
                                        signal_name: modification.signal_name.clone(),
                                        value: modification.value,
                                    });
                                }
                            }
                        }
                    });
                }
                PanelWidget::ValueBox {
                    label,
                    value,
                    min,
                    max,
                    unit,
                    message_id,
                    signal_name,
                    ..
                } => {
                    ui.horizontal(|ui| {
                        ui.label(label.as_str());
                        if ui
                            .add(
                                egui::DragValue::new(value)
                                    .range(*min..=*max)
                                    .speed(0.1),
                            )
                            .changed()
                        {
                            if let (Some(mid), Some(ref sname)) = (message_id, signal_name) {
                                events.push(PanelEvent::SignalChanged {
                                    message_id: *mid,
                                    signal_name: sname.clone(),
                                    value: *value,
                                });
                            }
                        }
                        ui.label(unit.as_str());
                    });
                }
                PanelWidget::SelectBox {
                    label,
                    options,
                    selected_index,
                    message_id,
                    signal_name,
                    ..
                } => {
                    ui.horizontal(|ui| {
                        ui.label(label.as_str());
                        let current_text = options
                            .get(*selected_index)
                            .map(|(text, _)| text.as_str())
                            .unwrap_or("(none)");
                        egui::ComboBox::from_label("")
                            .selected_text(current_text)
                            .show_ui(ui, |ui| {
                                for (idx, (text, val)) in options.iter().enumerate() {
                                    if ui
                                        .selectable_value(selected_index, idx, text.as_str())
                                        .changed()
                                    {
                                        if let (Some(mid), Some(ref sname)) =
                                            (&*message_id, &*signal_name)
                                        {
                                            events.push(PanelEvent::SignalChanged {
                                                message_id: *mid,
                                                signal_name: sname.clone(),
                                                value: *val,
                                            });
                                        }
                                    }
                                }
                            });
                    });
                }
                PanelWidget::Label {
                    text, font_size, ..
                } => {
                    ui.label(
                        egui::RichText::new(text.as_str()).size(*font_size),
                    );
                }
            }
        }
    });

    events
}

/// Events generated by panel widgets
#[derive(Debug)]
pub enum PanelEvent {
    SignalChanged {
        message_id: u32,
        signal_name: String,
        value: f64,
    },
}
