/// Internationalization module - Japanese/English localization
use std::sync::atomic::{AtomicU8, Ordering};

static LOCALE: AtomicU8 = AtomicU8::new(0); // 0 = Japanese, 1 = English

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    Japanese = 0,
    English = 1,
}

impl Locale {
    pub fn label(&self) -> &'static str {
        match self {
            Locale::Japanese => "日本語",
            Locale::English => "English",
        }
    }

    pub fn all() -> &'static [Locale] {
        &[Locale::Japanese, Locale::English]
    }
}

pub fn set_locale(locale: Locale) {
    LOCALE.store(locale as u8, Ordering::Relaxed);
}

pub fn get_locale() -> Locale {
    match LOCALE.load(Ordering::Relaxed) {
        0 => Locale::Japanese,
        _ => Locale::English,
    }
}

/// Translate a key to the current locale
pub fn t(key: &str) -> &str {
    let locale = get_locale();
    match locale {
        Locale::Japanese => t_ja(key),
        Locale::English => t_en(key),
    }
}

fn t_ja(key: &str) -> &str {
    match key {
        // App
        "app.title" => "CANkoro - CAN/CAN-FD 解析ツール",
        "app.name" => "CANkoro",
        "app.connect" => "接続",
        "app.disconnect" => "切断",
        "app.start_logging" => "ログ開始",
        "app.stop_logging" => "ログ停止",
        "app.connected" => "接続済み",
        "app.disconnected" => "未接続",
        "app.rec" => "REC",
        "app.language" => "言語",

        // Errors
        "err.driver_not_available" => "Vectorドライバーが利用できません",
        "err.no_channels_enabled" => "チャンネルが有効になっていません",
        "err.activate_failed" => "アクティベート失敗",
        "err.open_port_failed" => "ポートオープン失敗",
        "err.dbc_load_error" => "DBCロードエラー",
        "err.blf_open_error" => "BLFオープンエラー",
        "err.asc_open_error" => "ASCオープンエラー",

        // Tabs
        "tab.channel_config" => "チャンネル設定",
        "tab.can_log" => "CANログ",
        "tab.graph" => "グラフ",
        "tab.tx_config" => "送信設定",
        "tab.tx_panels" => "送信パネル",
        "tab.panel_editor" => "パネルエディタ",

        // Channel Config
        "ch.title" => "チャンネル設定",
        "ch.dbc_file" => "DBCファイル:",
        "ch.none" => "(なし)",
        "ch.browse" => "参照...",
        "ch.no_channels" => "CANチャンネルが検出されませんでした。Vectorハードウェアの接続を確認してください。",
        "ch.mode" => "モード:",
        "ch.can" => "CAN",
        "ch.can_fd" => "CAN FD",
        "ch.bitrate" => "ビットレート:",
        "ch.data_bitrate" => "データビットレート:",
        "ch.transceiver" => "トランシーバー:",

        // Log View
        "log.title" => "CANログ",
        "log.total" => "合計",
        "log.displayed" => "表示",
        "log.auto_scroll" => "自動スクロール",
        "log.pause" => "一時停止",
        "log.resume" => "再開",
        "log.clear" => "クリア",
        "log.id_filter" => "IDフィルタ:",
        "log.timestamp" => "タイムスタンプ",
        "log.channel" => "Ch",
        "log.direction" => "方向",
        "log.id" => "ID",
        "log.dlc" => "DLC",
        "log.fd" => "FD",
        "log.brs" => "BRS",
        "log.data" => "データ",
        "log.rx" => "Rx",
        "log.tx" => "Tx",

        // Graph View
        "graph.title" => "シグナルグラフ",
        "graph.realtime" => "リアルタイム",
        "graph.playback" => "再生",
        "graph.select_signals" => "シグナル選択",
        "graph.fit_all" => "全体表示",
        "graph.auto_scroll" => "自動スクロール",
        "graph.c1" => "C1",
        "graph.c2" => "C2",
        "graph.diff" => "差分",
        "graph.clear_data" => "データクリア",
        "graph.load_log" => "ログ読込",
        "graph.play" => "再生",
        "graph.pause" => "一時停止",
        "graph.stop" => "停止",
        "graph.speed" => "速度:",
        "graph.frame" => "フレーム",
        "graph.cursor_analysis" => "カーソル解析",
        "graph.signal" => "シグナル",
        "graph.c1_value" => "C1値",
        "graph.c2_value" => "C2値",
        "graph.delta" => "差分",
        "graph.unit" => "単位",
        "graph.signal_selection" => "シグナル選択",
        "graph.no_dbc" => "DBCファイルが読み込まれていません。先にDBCファイルを読み込んでください。",
        "graph.time_axis" => "時間 [s]",

        // TX Config
        "tx.title" => "送信設定",
        "tx.save" => "設定を保存",
        "tx.load" => "設定を読込",
        "tx.forward_config" => "転送設定",
        "tx.add_forward" => "転送を追加",
        "tx.rx_channel" => "受信チャンネル:",
        "tx.tx_channel" => "送信チャンネル:",
        "tx.trapezoidal" => "台形波",
        "tx.add_trapezoidal" => "台形波を追加",
        "tx.message_id" => "メッセージID (16進):",
        "tx.signal" => "シグナル:",
        "tx.signal_name" => "シグナル名:",
        "tx.initial_value" => "初期値:",
        "tx.max_value" => "最大値:",
        "tx.rate" => "変化率 (値/秒):",
        "tx.hold_time" => "保持時間 (ms):",
        "tx.cycle_time" => "周期 (ms):",
        "tx.repeat" => "繰り返し",
        "tx.enable" => "有効",
        "tx.signal_modifications" => "シグナル変更",
        "tx.add_modification" => "変更を追加",
        "tx.msg" => "Msg:",
        "tx.sig" => "Sig:",
        "tx.val" => "値:",

        // Panel Editor
        "panel.title" => "パネルエディタ",
        "panel.new" => "新規パネル",
        "panel.name" => "パネル名:",
        "panel.add_widget" => "ウィジェット追加:",
        "panel.switch" => "スイッチ",
        "panel.value_box" => "数値ボックス",
        "panel.select_box" => "セレクトボックス",
        "panel.label" => "ラベル",
        "panel.add" => "追加",
        "panel.label_field" => "ラベル:",
        "panel.on_action" => "ONアクション",
        "panel.off_action" => "OFFアクション",
        "panel.message" => "メッセージ:",
        "panel.msg_id" => "Msg ID:",
        "panel.value" => "値:",
        "panel.text" => "テキスト:",
        "panel.font_size" => "フォントサイズ:",
        "panel.options" => "オプション:",
        "panel.new_option" => "新規オプション",
        "panel.add_option" => "+ オプション",
        "panel.enabled" => "有効",

        // File dialogs
        "file.asc" => "ASCファイル",
        "file.blf" => "BLFファイル",
        "file.dbc" => "DBCファイル",
        "file.tx_settings" => "送信設定",
        "file.log_files" => "ログファイル",

        _ => key,
    }
}

fn t_en(key: &str) -> &str {
    match key {
        // App
        "app.title" => "CANkoro - CAN/CAN-FD Analysis Tool",
        "app.name" => "CANkoro",
        "app.connect" => "Connect",
        "app.disconnect" => "Disconnect",
        "app.start_logging" => "Start Logging",
        "app.stop_logging" => "Stop Logging",
        "app.connected" => "Connected",
        "app.disconnected" => "Disconnected",
        "app.rec" => "REC",
        "app.language" => "Language",

        // Errors
        "err.driver_not_available" => "Vector driver not available",
        "err.no_channels_enabled" => "No channels enabled",
        "err.activate_failed" => "Activate failed",
        "err.open_port_failed" => "Open port failed",
        "err.dbc_load_error" => "DBC load error",
        "err.blf_open_error" => "BLF open error",
        "err.asc_open_error" => "ASC open error",

        // Tabs
        "tab.channel_config" => "Channel Config",
        "tab.can_log" => "CAN Log",
        "tab.graph" => "Graph",
        "tab.tx_config" => "TX Config",
        "tab.tx_panels" => "TX Panels",
        "tab.panel_editor" => "Panel Editor",

        // Channel Config
        "ch.title" => "Channel Configuration",
        "ch.dbc_file" => "DBC File:",
        "ch.none" => "(None)",
        "ch.browse" => "Browse...",
        "ch.no_channels" => "No CAN channels detected. Please check Vector hardware connection.",
        "ch.mode" => "Mode:",
        "ch.can" => "CAN",
        "ch.can_fd" => "CAN FD",
        "ch.bitrate" => "Bitrate:",
        "ch.data_bitrate" => "Data Bitrate:",
        "ch.transceiver" => "Transceiver:",

        // Log View
        "log.title" => "CAN Log",
        "log.total" => "Total",
        "log.displayed" => "Displayed",
        "log.auto_scroll" => "Auto-scroll",
        "log.pause" => "Pause",
        "log.resume" => "Resume",
        "log.clear" => "Clear",
        "log.id_filter" => "ID Filter:",
        "log.timestamp" => "Timestamp",
        "log.channel" => "Ch",
        "log.direction" => "Dir",
        "log.id" => "ID",
        "log.dlc" => "DLC",
        "log.fd" => "FD",
        "log.brs" => "BRS",
        "log.data" => "Data",
        "log.rx" => "Rx",
        "log.tx" => "Tx",

        // Graph View
        "graph.title" => "Signal Graph",
        "graph.realtime" => "Realtime",
        "graph.playback" => "Playback",
        "graph.select_signals" => "Select Signals",
        "graph.fit_all" => "Fit All",
        "graph.auto_scroll" => "Auto-scroll",
        "graph.c1" => "C1",
        "graph.c2" => "C2",
        "graph.diff" => "Diff",
        "graph.clear_data" => "Clear Data",
        "graph.load_log" => "Load Log",
        "graph.play" => "Play",
        "graph.pause" => "Pause",
        "graph.stop" => "Stop",
        "graph.speed" => "Speed:",
        "graph.frame" => "Frame",
        "graph.cursor_analysis" => "Cursor Analysis",
        "graph.signal" => "Signal",
        "graph.c1_value" => "C1 Value",
        "graph.c2_value" => "C2 Value",
        "graph.delta" => "Delta",
        "graph.unit" => "Unit",
        "graph.signal_selection" => "Signal Selection",
        "graph.no_dbc" => "No DBC file loaded. Please load a DBC file first.",
        "graph.time_axis" => "Time [s]",

        // TX Config
        "tx.title" => "Transmission Configuration",
        "tx.save" => "Save Settings",
        "tx.load" => "Load Settings",
        "tx.forward_config" => "Forward Configurations",
        "tx.add_forward" => "Add Forward",
        "tx.rx_channel" => "Rx Channel:",
        "tx.tx_channel" => "Tx Channel:",
        "tx.trapezoidal" => "Trapezoidal Wave",
        "tx.add_trapezoidal" => "Add Trapezoidal",
        "tx.message_id" => "Message ID (hex):",
        "tx.signal" => "Signal:",
        "tx.signal_name" => "Signal name:",
        "tx.initial_value" => "Initial value:",
        "tx.max_value" => "Max value:",
        "tx.rate" => "Rate (value/s):",
        "tx.hold_time" => "Hold time (ms):",
        "tx.cycle_time" => "Cycle time (ms):",
        "tx.repeat" => "Repeat",
        "tx.enable" => "Enable",
        "tx.signal_modifications" => "Signal Modifications",
        "tx.add_modification" => "Add Modification",
        "tx.msg" => "Msg:",
        "tx.sig" => "Sig:",
        "tx.val" => "Val:",

        // Panel Editor
        "panel.title" => "Panel Editor",
        "panel.new" => "New Panel",
        "panel.name" => "Panel name:",
        "panel.add_widget" => "Add widget:",
        "panel.switch" => "Switch",
        "panel.value_box" => "ValueBox",
        "panel.select_box" => "SelectBox",
        "panel.label" => "Label",
        "panel.add" => "Add",
        "panel.label_field" => "Label:",
        "panel.on_action" => "ON Action",
        "panel.off_action" => "OFF Action",
        "panel.message" => "Message:",
        "panel.msg_id" => "Msg ID:",
        "panel.value" => "Value:",
        "panel.text" => "Text:",
        "panel.font_size" => "Font size:",
        "panel.options" => "Options:",
        "panel.new_option" => "New Option",
        "panel.add_option" => "+ Option",
        "panel.enabled" => "Enabled",

        // File dialogs
        "file.asc" => "ASC files",
        "file.blf" => "BLF files",
        "file.dbc" => "DBC files",
        "file.tx_settings" => "TX Settings",
        "file.log_files" => "Log files",

        _ => key,
    }
}
