/// DBC database - collection of messages and signals with lookup
use std::collections::HashMap;
use std::path::Path;

use super::message::DbcMessage;
use super::parser;

/// Database of CAN messages from a DBC file
#[derive(Debug, Clone)]
pub struct DbcDatabase {
    /// Messages indexed by ID
    messages: HashMap<u32, DbcMessage>,
    /// All messages in order
    message_list: Vec<DbcMessage>,
    /// Source file path
    pub file_path: Option<String>,
}

impl DbcDatabase {
    /// Load a DBC file
    pub fn load(path: &Path) -> Result<Self, String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read DBC file: {}", e))?;
        let messages = parser::parse_dbc(&content)?;

        let mut db = Self {
            messages: HashMap::new(),
            message_list: Vec::new(),
            file_path: Some(path.to_string_lossy().to_string()),
        };

        for msg in messages {
            db.messages.insert(msg.id, msg.clone());
            db.message_list.push(msg);
        }

        Ok(db)
    }

    /// Get a message by ID
    pub fn get_message(&self, id: u32) -> Option<&DbcMessage> {
        self.messages.get(&id)
    }

    /// Get all messages
    pub fn messages(&self) -> &[DbcMessage] {
        &self.message_list
    }

    /// Get all signal names across all messages
    pub fn all_signal_names(&self) -> Vec<(u32, String, String)> {
        let mut result = Vec::new();
        for msg in &self.message_list {
            for sig in &msg.signals {
                result.push((msg.id, msg.name.clone(), sig.name.clone()));
            }
        }
        result
    }

    /// Decode a CAN frame to physical values
    pub fn decode_frame(
        &self,
        id: u32,
        data: &[u8],
    ) -> Option<Vec<(String, f64, String)>> {
        self.messages.get(&id).map(|msg| {
            msg.signals
                .iter()
                .map(|sig| {
                    let value = sig.decode_physical(data);
                    (sig.name.clone(), value, sig.unit.clone())
                })
                .collect()
        })
    }
}
