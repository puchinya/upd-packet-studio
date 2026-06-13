use crate::types::{Collection, PacketDefinition, PayloadType};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedConfig {
    pub collections: Vec<Collection>,
    pub listener_addr: String,
    pub composer_target: String,
    pub composer_payload_type: PayloadType,
    pub composer_payload: String,
}

impl SavedConfig {
    pub fn load() -> Self {
        if let Ok(content) = std::fs::read_to_string("updexp_config.json") {
            if let Ok(config) = serde_json::from_str::<Self>(&content) {
                return config;
            }
        }
        
        Self {
            collections: vec![
                Collection {
                    id: "default_col_1".to_string(),
                    name: "ECHONET Lite Queries".to_string(),
                    is_expanded: true,
                    requests: vec![
                        PacketDefinition {
                            id: "default_1".to_string(),
                            name: "Aircon Get Operation".to_string(),
                            target: "127.0.0.1:3610".to_string(),
                            payload_type: PayloadType::Hex,
                            payload: "10 81 00 01 05 FF 01 01 30 01 62 01 80 00".to_string(),
                        },
                        PacketDefinition {
                            id: "default_2".to_string(),
                            name: "Node Profile Get".to_string(),
                            target: "127.0.0.1:3610".to_string(),
                            payload_type: PayloadType::Hex,
                            payload: "10 81 00 02 05 FF 01 0E F0 01 62 01 D6 00".to_string(),
                        },
                    ],
                },
                Collection {
                    id: "default_col_2".to_string(),
                    name: "General UDP Tests".to_string(),
                    is_expanded: true,
                    requests: vec![
                        PacketDefinition {
                            id: "default_3".to_string(),
                            name: "Local Loopback Ping".to_string(),
                            target: "127.0.0.1:9000".to_string(),
                            payload_type: PayloadType::Text,
                            payload: "Ping!".to_string(),
                        },
                    ],
                },
            ],
            listener_addr: "0.0.0.0:9000".to_string(),
            composer_target: "127.0.0.1:9000".to_string(),
            composer_payload_type: PayloadType::Text,
            composer_payload: "Hello from Composer!".to_string(),
        }
    }

    pub fn save(&self) {
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write("updexp_config.json", content);
        }
    }
}
