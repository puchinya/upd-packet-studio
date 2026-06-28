use std::collections::HashMap;
use std::path::Path;
use crate::mra_defs::{RawMraClass, PropertyInfo, ClassInfo};

#[derive(Clone)]
pub struct MraDatabase {
    pub classes: HashMap<(u8, u8), ClassInfo>,
}

impl MraDatabase {
    pub fn load_empty() -> Self {
        Self { classes: HashMap::new() }
    }

    pub fn load() -> Self {
        let mut db = Self {
            classes: HashMap::new(),
        };
        
        let base_path = Path::new("assets/mra/MRA_v1.4.0");
        if !base_path.exists() {
            return db;
        }

        // 1. Load super class (0x0000)
        let super_class_path = base_path.join("superClass").join("0x0000.json");
        let super_class_props = if let Ok(content) = std::fs::read_to_string(&super_class_path) {
            if let Ok(raw_class) = serde_json::from_str::<RawMraClass>(&content) {
                raw_class.el_properties.into_iter().map(|p| {
                    let epc = u8::from_str_radix(p.epc.trim_start_matches("0x"), 16).unwrap_or(0);
                    let info = PropertyInfo {
                        epc,
                        name_ja: p.property_name.ja,
                        name_en: p.property_name.en,
                        description_ja: p.descriptions.as_ref().map(|d| d.ja.clone()),
                        description_en: p.descriptions.as_ref().map(|d| d.en.clone()),
                    };
                    (epc, info)
                }).collect::<HashMap<u8, PropertyInfo>>()
            } else {
                HashMap::new()
            }
        } else {
            HashMap::new()
        };

        // Helper to load a class from a file path
        let load_class_file = |path: &Path| -> Option<RawMraClass> {
            let content = std::fs::read_to_string(path).ok()?;
            serde_json::from_str::<RawMraClass>(&content).ok()
        };

        // 2. Load node profile (0x0EF0)
        let node_profile_path = base_path.join("nodeProfile").join("0x0EF0.json");
        if let Some(raw_class) = load_class_file(&node_profile_path) {
            let mut props = super_class_props.clone();
            for p in raw_class.el_properties {
                let epc = u8::from_str_radix(p.epc.trim_start_matches("0x"), 16).unwrap_or(0);
                props.insert(epc, PropertyInfo {
                    epc,
                    name_ja: p.property_name.ja,
                    name_en: p.property_name.en,
                    description_ja: p.descriptions.as_ref().map(|d| d.ja.clone()),
                    description_en: p.descriptions.as_ref().map(|d| d.en.clone()),
                });
            }
            db.classes.insert((0x0E, 0xF0), ClassInfo {
                group_code: 0x0E,
                class_code: 0xF0,
                name_ja: raw_class.class_name.ja,
                name_en: raw_class.class_name.en,
                properties: props,
            });
        }

        // 3. Load device classes
        let devices_dir = base_path.join("devices");
        if let Ok(entries) = std::fs::read_dir(devices_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Some(raw_class) = load_class_file(&path) {
                        let eoj_hex = raw_class.eoj.trim_start_matches("0x");
                        if eoj_hex.len() == 4 {
                            if let (Ok(g), Ok(c)) = (
                                u8::from_str_radix(&eoj_hex[0..2], 16),
                                u8::from_str_radix(&eoj_hex[2..4], 16)
                            ) {
                                let mut props = super_class_props.clone();
                                for p in raw_class.el_properties {
                                    let epc = u8::from_str_radix(p.epc.trim_start_matches("0x"), 16).unwrap_or(0);
                                    props.insert(epc, PropertyInfo {
                                        epc,
                                        name_ja: p.property_name.ja,
                                        name_en: p.property_name.en,
                                        description_ja: p.descriptions.as_ref().map(|d| d.ja.clone()),
                                        description_en: p.descriptions.as_ref().map(|d| d.en.clone()),
                                    });
                                }
                                db.classes.insert((g, c), ClassInfo {
                                    group_code: g,
                                    class_code: c,
                                    name_ja: raw_class.class_name.ja,
                                    name_en: raw_class.class_name.en,
                                    properties: props,
                                });
                            }
                        }
                    }
                }
            }
        }

        db
    }
}
