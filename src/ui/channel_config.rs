/// Channel configuration UI
use egui::{self, Ui};

use crate::i18n::t;
use crate::vector::channel::CanMode;
use crate::vector::driver::HwChannelInfo;

/// Channel configuration state
#[derive(Debug, Clone)]
pub struct ChannelConfigState {
    pub channels: Vec<ChannelUiState>,
    pub selected_dbc_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ChannelUiState {
    pub hw_info: HwChannelInfo,
    pub enabled: bool,
    pub mode: CanMode,
    pub bitrate: u32,
    pub data_bitrate: u32,
}

impl ChannelConfigState {
    pub fn new() -> Self {
        Self {
            channels: Vec::new(),
            selected_dbc_path: None,
        }
    }

    pub fn set_channels(&mut self, hw_channels: Vec<HwChannelInfo>) {
        self.channels = hw_channels
            .into_iter()
            .map(|hw| ChannelUiState {
                hw_info: hw,
                enabled: false,
                mode: CanMode::Can,
                bitrate: 500000,
                data_bitrate: 2000000,
            })
            .collect();
    }
}

const BITRATES: &[u32] = &[
    10000, 20000, 33333, 50000, 83333, 100000, 125000, 250000, 500000, 800000, 1000000,
];

const FD_DATA_BITRATES: &[u32] = &[500000, 1000000, 2000000, 4000000, 5000000, 8000000];

pub fn draw_channel_config(ui: &mut Ui, state: &mut ChannelConfigState) -> bool {
    let mut config_changed = false;

    ui.heading(t("ch.title"));
    ui.separator();

    // DBC file selection
    ui.horizontal(|ui| {
        ui.label(t("ch.dbc_file"));
        if let Some(ref path) = state.selected_dbc_path {
            ui.label(path);
        } else {
            ui.label(t("ch.none"));
        }
        if ui.button(t("ch.browse")).clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter(t("file.dbc"), &["dbc"])
                .pick_file()
            {
                state.selected_dbc_path = Some(path.to_string_lossy().to_string());
                config_changed = true;
            }
        }
    });

    ui.separator();

    if state.channels.is_empty() {
        ui.label(t("ch.no_channels"));
        return false;
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        for ch in &mut state.channels {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    if ui.checkbox(&mut ch.enabled, "").changed() {
                        config_changed = true;
                    }
                    ui.strong(&ch.hw_info.name);
                    ui.label(format!(
                        "[{}] (S/N: {}, Ch: {})",
                        ch.hw_info.hw_type_name(),
                        ch.hw_info.serial_number,
                        ch.hw_info.hw_channel
                    ));
                });

                if ch.enabled {
                    ui.indent("ch_config", |ui| {
                        ui.horizontal(|ui| {
                            ui.label(t("ch.mode"));
                            if ui
                                .radio_value(&mut ch.mode, CanMode::Can, t("ch.can"))
                                .changed()
                            {
                                config_changed = true;
                            }
                            if ui
                                .radio_value(&mut ch.mode, CanMode::CanFd, t("ch.can_fd"))
                                .changed()
                            {
                                config_changed = true;
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label(t("ch.bitrate"));
                            egui::ComboBox::from_id_salt(format!("br_{}", ch.hw_info.channel_index))
                                .selected_text(format_bitrate(ch.bitrate))
                                .show_ui(ui, |ui| {
                                    for &br in BITRATES {
                                        if ui
                                            .selectable_value(
                                                &mut ch.bitrate,
                                                br,
                                                format_bitrate(br),
                                            )
                                            .changed()
                                        {
                                            config_changed = true;
                                        }
                                    }
                                });
                        });

                        if ch.mode == CanMode::CanFd {
                            ui.horizontal(|ui| {
                                ui.label(t("ch.data_bitrate"));
                                egui::ComboBox::from_id_salt(format!(
                                    "dbr_{}",
                                    ch.hw_info.channel_index
                                ))
                                .selected_text(format_bitrate(ch.data_bitrate))
                                .show_ui(ui, |ui| {
                                    for &br in FD_DATA_BITRATES {
                                        if ui
                                            .selectable_value(
                                                &mut ch.data_bitrate,
                                                br,
                                                format_bitrate(br),
                                            )
                                            .changed()
                                        {
                                            config_changed = true;
                                        }
                                    }
                                });
                            });
                        }

                        ui.horizontal(|ui| {
                            ui.label(t("ch.transceiver"));
                            ui.label(&ch.hw_info.transceiver_name);
                        });
                    });
                }
            });
        }
    });

    config_changed
}

fn format_bitrate(br: u32) -> String {
    if br >= 1_000_000 {
        format!("{} Mbit/s", br / 1_000_000)
    } else if br >= 1000 {
        format!("{} kbit/s", br / 1000)
    } else {
        format!("{} bit/s", br)
    }
}
