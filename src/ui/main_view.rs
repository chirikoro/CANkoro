/// Main view layout with tab-based navigation
use egui::{self, Ui};

use crate::i18n::t;

/// Available tabs in the main view
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainTab {
    ChannelConfig,
    LogView,
    GraphView,
    TxConfig,
    TxPanels,
    PanelEditor,
}

impl MainTab {
    pub fn label(&self) -> &str {
        match self {
            MainTab::ChannelConfig => t("tab.channel_config"),
            MainTab::LogView => t("tab.can_log"),
            MainTab::GraphView => t("tab.graph"),
            MainTab::TxConfig => t("tab.tx_config"),
            MainTab::TxPanels => t("tab.tx_panels"),
            MainTab::PanelEditor => t("tab.panel_editor"),
        }
    }
}

/// Draw the main tab bar and return the selected tab
pub fn draw_tab_bar(ui: &mut Ui, current_tab: &mut MainTab) {
    ui.horizontal(|ui| {
        for tab in [
            MainTab::ChannelConfig,
            MainTab::LogView,
            MainTab::GraphView,
            MainTab::TxConfig,
            MainTab::TxPanels,
            MainTab::PanelEditor,
        ] {
            if ui
                .selectable_label(*current_tab == tab, tab.label())
                .clicked()
            {
                *current_tab = tab;
            }
        }
    });
}

/// Draw the status bar at the bottom
pub fn draw_status_bar(
    ui: &mut Ui,
    connected: bool,
    rx_count: u64,
    tx_count: u64,
    logging: bool,
) {
    ui.horizontal(|ui| {
        let status_color = if connected {
            egui::Color32::from_rgb(0, 200, 0)
        } else {
            egui::Color32::from_rgb(200, 0, 0)
        };
        ui.colored_label(
            status_color,
            if connected { t("app.connected") } else { t("app.disconnected") },
        );
        ui.separator();
        ui.label(format!("Rx: {}", rx_count));
        ui.label(format!("Tx: {}", tx_count));
        ui.separator();
        if logging {
            ui.colored_label(egui::Color32::from_rgb(255, 100, 100), t("app.rec"));
        }
    });
}
