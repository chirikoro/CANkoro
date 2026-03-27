/// Transmission configuration UI
use egui::{self, Ui};

use crate::config::tx_settings::{ForwardSetting, TrapezoidalSetting, TxSettings};
use crate::dbc::database::DbcDatabase;

/// Draw the transmission configuration panel
pub fn draw_tx_config(
    ui: &mut Ui,
    settings: &mut TxSettings,
    dbc: &Option<DbcDatabase>,
    available_channels: &[(u32, String)],
) -> Vec<TxAction> {
    let mut actions = Vec::new();

    ui.heading("Transmission Configuration");
    ui.separator();

    // Save / Load buttons
    ui.horizontal(|ui| {
        if ui.button("Save Settings").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("TX Settings", &["json"])
                .save_file()
            {
                if let Err(e) = settings.save(&path) {
                    log::error!("Failed to save TX settings: {}", e);
                }
            }
        }
        if ui.button("Load Settings").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("TX Settings", &["json"])
                .pick_file()
            {
                match TxSettings::load(&path) {
                    Ok(loaded) => *settings = loaded,
                    Err(e) => log::error!("Failed to load TX settings: {}", e),
                }
            }
        }
    });

    ui.separator();

    // Forward configurations
    ui.collapsing("Forward Configurations", |ui| {
        if ui.button("Add Forward").clicked() {
            settings.forwards.push(ForwardSetting {
                name: format!("Forward {}", settings.forwards.len() + 1),
                enabled: false,
                rx_channel_index: 0,
                tx_channel_index: 1,
                message_filter: Vec::new(),
                modifications: Vec::new(),
            });
        }

        let mut remove_idx = None;
        for (i, fwd) in settings.forwards.iter_mut().enumerate() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut fwd.name);
                    if ui.checkbox(&mut fwd.enabled, "Enable").changed() {
                        if fwd.enabled {
                            actions.push(TxAction::StartForward(i));
                        } else {
                            actions.push(TxAction::StopForward(i));
                        }
                    }
                    if ui.button("X").clicked() {
                        remove_idx = Some(i);
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Rx Channel:");
                    egui::ComboBox::from_id_salt(format!("fwd_rx_{}", i))
                        .selected_text(
                            available_channels
                                .get(fwd.rx_channel_index as usize)
                                .map(|(_, name)| name.as_str())
                                .unwrap_or("?"),
                        )
                        .show_ui(ui, |ui| {
                            for (idx, (_, name)) in available_channels.iter().enumerate() {
                                ui.selectable_value(
                                    &mut fwd.rx_channel_index,
                                    idx as u32,
                                    name,
                                );
                            }
                        });

                    ui.label("Tx Channel:");
                    egui::ComboBox::from_id_salt(format!("fwd_tx_{}", i))
                        .selected_text(
                            available_channels
                                .get(fwd.tx_channel_index as usize)
                                .map(|(_, name)| name.as_str())
                                .unwrap_or("?"),
                        )
                        .show_ui(ui, |ui| {
                            for (idx, (_, name)) in available_channels.iter().enumerate() {
                                ui.selectable_value(
                                    &mut fwd.tx_channel_index,
                                    idx as u32,
                                    name,
                                );
                            }
                        });
                });

                // Signal modifications
                if let Some(ref dbc_db) = dbc {
                    draw_signal_modifications(ui, &mut fwd.modifications, dbc_db, i);
                }
            });
        }
        if let Some(idx) = remove_idx {
            if settings.forwards[idx].enabled {
                actions.push(TxAction::StopForward(idx));
            }
            settings.forwards.remove(idx);
        }
    });

    ui.separator();

    // Trapezoidal wave configurations
    ui.collapsing("Trapezoidal Wave", |ui| {
        if ui.button("Add Trapezoidal").clicked() {
            settings.trapezoidals.push(TrapezoidalSetting {
                name: format!("Trapezoid {}", settings.trapezoidals.len() + 1),
                enabled: false,
                tx_channel_index: 0,
                message_id: 0,
                signal_name: String::new(),
                initial_value: 0.0,
                max_value: 100.0,
                rate: 10.0,
                hold_time_ms: 1000,
                cycle_time_ms: 10,
                repeat: true,
                base_data: vec![0u8; 8],
                is_fd: false,
                dlc: 8,
            });
        }

        let mut remove_idx = None;
        for (i, trap) in settings.trapezoidals.iter_mut().enumerate() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut trap.name);
                    if ui.checkbox(&mut trap.enabled, "Enable").changed() {
                        if trap.enabled {
                            actions.push(TxAction::StartTrapezoidal(i));
                        } else {
                            actions.push(TxAction::StopTrapezoidal(i));
                        }
                    }
                    if ui.button("X").clicked() {
                        remove_idx = Some(i);
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Tx Channel:");
                    egui::ComboBox::from_id_salt(format!("trap_tx_{}", i))
                        .selected_text(
                            available_channels
                                .get(trap.tx_channel_index as usize)
                                .map(|(_, name)| name.as_str())
                                .unwrap_or("?"),
                        )
                        .show_ui(ui, |ui| {
                            for (idx, (_, name)) in available_channels.iter().enumerate() {
                                ui.selectable_value(
                                    &mut trap.tx_channel_index,
                                    idx as u32,
                                    name,
                                );
                            }
                        });
                });

                // Message & signal selection from DBC
                if let Some(ref dbc_db) = dbc {
                    ui.horizontal(|ui| {
                        ui.label("Message ID (hex):");
                        let mut id_str = format!("{:03X}", trap.message_id);
                        if ui.text_edit_singleline(&mut id_str).changed() {
                            if let Ok(id) = u32::from_str_radix(&id_str, 16) {
                                trap.message_id = id;
                            }
                        }
                    });

                    if let Some(msg) = dbc_db.get_message(trap.message_id) {
                        ui.horizontal(|ui| {
                            ui.label("Signal:");
                            egui::ComboBox::from_id_salt(format!("trap_sig_{}", i))
                                .selected_text(&trap.signal_name)
                                .show_ui(ui, |ui| {
                                    for sig in &msg.signals {
                                        if ui
                                            .selectable_value(
                                                &mut trap.signal_name,
                                                sig.name.clone(),
                                                format!(
                                                    "{} [{:.1}..{:.1}] {}",
                                                    sig.name, sig.min, sig.max, sig.unit
                                                ),
                                            )
                                            .changed()
                                        {
                                            trap.initial_value = sig.min;
                                            trap.max_value = sig.max;
                                        }
                                    }
                                });
                        });

                        if let Some(sig) = msg.get_signal(&trap.signal_name) {
                            ui.horizontal(|ui| {
                                ui.label("Initial value:");
                                ui.add(
                                    egui::DragValue::new(&mut trap.initial_value)
                                        .range(sig.min..=sig.max)
                                        .speed(0.1),
                                );
                                ui.label(&sig.unit);
                            });
                            ui.horizontal(|ui| {
                                ui.label("Max value:");
                                ui.add(
                                    egui::DragValue::new(&mut trap.max_value)
                                        .range(sig.min..=sig.max)
                                        .speed(0.1),
                                );
                                ui.label(&sig.unit);
                            });
                        }
                    }
                } else {
                    ui.horizontal(|ui| {
                        ui.label("Message ID (hex):");
                        let mut id_str = format!("{:03X}", trap.message_id);
                        if ui.text_edit_singleline(&mut id_str).changed() {
                            if let Ok(id) = u32::from_str_radix(&id_str, 16) {
                                trap.message_id = id;
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Signal name:");
                        ui.text_edit_singleline(&mut trap.signal_name);
                    });
                }

                ui.horizontal(|ui| {
                    ui.label("Rate (value/s):");
                    ui.add(egui::DragValue::new(&mut trap.rate).speed(0.1));
                });
                ui.horizontal(|ui| {
                    ui.label("Hold time (ms):");
                    ui.add(egui::DragValue::new(&mut trap.hold_time_ms).speed(10.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Cycle time (ms):");
                    ui.add(egui::DragValue::new(&mut trap.cycle_time_ms).speed(1.0));
                });
                ui.checkbox(&mut trap.repeat, "Repeat");
                ui.checkbox(&mut trap.is_fd, "CAN FD");
            });
        }
        if let Some(idx) = remove_idx {
            if settings.trapezoidals[idx].enabled {
                actions.push(TxAction::StopTrapezoidal(idx));
            }
            settings.trapezoidals.remove(idx);
        }
    });

    actions
}

fn draw_signal_modifications(
    ui: &mut Ui,
    modifications: &mut Vec<crate::can::transmitter::SignalModification>,
    dbc: &DbcDatabase,
    fwd_index: usize,
) {
    ui.collapsing(
        format!("Signal Modifications ({})", modifications.len()),
        |ui| {
            if ui.button("Add Modification").clicked() {
                modifications.push(crate::can::transmitter::SignalModification {
                    message_id: 0,
                    signal_name: String::new(),
                    value: 0.0,
                });
            }

            let mut remove_idx = None;
            for (j, modification) in modifications.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.label("Msg:");
                    let mut id_str = format!("{:03X}", modification.message_id);
                    if ui
                        .add(egui::TextEdit::singleline(&mut id_str).desired_width(60.0))
                        .changed()
                    {
                        if let Ok(id) = u32::from_str_radix(&id_str, 16) {
                            modification.message_id = id;
                        }
                    }

                    ui.label("Sig:");
                    if let Some(msg) = dbc.get_message(modification.message_id) {
                        egui::ComboBox::from_id_salt(format!("mod_sig_{}_{}", fwd_index, j))
                            .selected_text(&modification.signal_name)
                            .show_ui(ui, |ui| {
                                for sig in &msg.signals {
                                    ui.selectable_value(
                                        &mut modification.signal_name,
                                        sig.name.clone(),
                                        &sig.name,
                                    );
                                }
                            });

                        if let Some(sig) = msg.get_signal(&modification.signal_name) {
                            ui.label("Val:");
                            ui.add(
                                egui::DragValue::new(&mut modification.value)
                                    .range(sig.min..=sig.max)
                                    .speed(0.1),
                            );
                            ui.label(&sig.unit);
                        }
                    } else {
                        ui.text_edit_singleline(&mut modification.signal_name);
                        ui.add(egui::DragValue::new(&mut modification.value).speed(0.1));
                    }

                    if ui.button("X").clicked() {
                        remove_idx = Some(j);
                    }
                });
            }
            if let Some(idx) = remove_idx {
                modifications.remove(idx);
            }
        },
    );
}

/// Actions generated by the TX config UI
pub enum TxAction {
    StartForward(usize),
    StopForward(usize),
    StartTrapezoidal(usize),
    StopTrapezoidal(usize),
}
