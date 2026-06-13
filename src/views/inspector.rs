use eframe::egui;
use crate::UdpStudioState;
use crate::types::{LogDirection, InspectorProtocol, to_hex_dump};

struct EchonetProperty {
    epc: u8,
    pdc: u8,
    edt: Vec<u8>,
}

impl UdpStudioState {
    pub fn show_inspector(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            if let Some(idx) = self.selected_log_idx {
                if idx < self.logs.len() {
                    let entry = &self.logs[idx];
                    
                    // Selected log metadata header
                    ui.horizontal(|ui| {
                        ui.label(format!("Timestamp: {}", entry.timestamp.format("%Y-%m-%d %H:%M:%S.%3f")));
                        ui.separator();
                        let dir_label = match entry.direction {
                            LogDirection::Sent => "Sent To:",
                            LogDirection::Received => "Received From:",
                            LogDirection::SystemInfo => "Event target:",
                            LogDirection::SystemError => "Error target:",
                        };
                        ui.label(format!("{} {}", dir_label, entry.address));
                        ui.separator();
                        ui.label(format!("Size: {} bytes", entry.data.len()));
                    });
                    
                    ui.add_space(8.0);
                    
                    // Protocol Selector
                    ui.horizontal(|ui| {
                        ui.label("Decode As:");
                        ui.selectable_value(&mut self.inspector_protocol, InspectorProtocol::Raw, "🔌 Raw (Hex)");
                        ui.selectable_value(&mut self.inspector_protocol, InspectorProtocol::TextAscii, "📝 Text (ASCII)");
                        ui.selectable_value(&mut self.inspector_protocol, InspectorProtocol::EchonetLite, "💡 ECHONET Lite");
                    });
                    
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Decode details
                    match self.inspector_protocol {
                        InspectorProtocol::Raw => {
                            ui.label("Hex Dump View:");
                            ui.add_space(4.0);
                            egui::ScrollArea::vertical()
                                .id_salt("hex_dump_scroll")
                                .show(ui, |ui| {
                                    let mut dump = to_hex_dump(&entry.data);
                                    ui.add(
                                        egui::TextEdit::multiline(&mut dump)
                                            .font(egui::TextStyle::Monospace)
                                            .code_editor()
                                            .desired_width(ui.available_width())
                                            .interactive(false)
                                    );
                                });
                        }
                        InspectorProtocol::TextAscii => {
                            ui.label("ASCII Text View (with control code visualizers):");
                            ui.add_space(4.0);
                            egui::ScrollArea::vertical()
                                .id_salt("ascii_scroll")
                                .show(ui, |ui| {
                                    let mut ascii_rep = to_ascii_inspector(&entry.data);
                                    ui.add(
                                        egui::TextEdit::multiline(&mut ascii_rep)
                                            .font(egui::TextStyle::Monospace)
                                            .desired_width(ui.available_width())
                                            .interactive(false)
                                    );
                                });
                        }
                        InspectorProtocol::EchonetLite => {
                            ui.label("ECHONET Lite Protocol Decode:");
                            ui.add_space(6.0);
                            
                            if entry.data.len() < 12 {
                                ui.colored_label(
                                    egui::Color32::from_rgb(255, 100, 100),
                                    "⚠️ Packet too short to be a valid ECHONET Lite frame (min 12 bytes)."
                                );
                            } else {
                                let ehd1 = entry.data[0];
                                let ehd2 = entry.data[1];
                                
                                if ehd1 != 0x10 {
                                    ui.colored_label(
                                        egui::Color32::from_rgb(255, 180, 100),
                                        format!("⚠️ EHD1 is 0x{:02X} (Expected 0x10 for ECHONET Lite)", ehd1)
                                    );
                                    ui.add_space(4.0);
                                }
                                
                                let tid_h = entry.data[2];
                                let tid_l = entry.data[3];
                                let seoj = &entry.data[4..7];
                                let deoj = &entry.data[7..10];
                                let esv = entry.data[10];
                                let opc = entry.data[11];
                                
                                // Frame structure grid
                                egui::Grid::new("el_inspector_grid")
                                    .num_columns(2)
                                    .spacing([12.0, 6.0])
                                    .show(ui, |ui| {
                                        ui.label("EHD1 (Header 1):");
                                        ui.monospace(format!("0x{:02X} (ECHONET Lite)", ehd1));
                                        ui.end_row();
                                        
                                        ui.label("EHD2 (Header 2):");
                                        ui.monospace(format!("0x{:02X} (Format {})", ehd2, if ehd2 == 0x81 { "1" } else { "2" }));
                                        ui.end_row();
                                        
                                        ui.label("Transaction ID (TID):");
                                        ui.monospace(format!("0x{:02X}{:02X}", tid_h, tid_l));
                                        ui.end_row();
                                        
                                        ui.label("Source Object (SEOJ):");
                                        ui.label(translate_object(seoj));
                                        ui.end_row();
                                        
                                        ui.label("Dest Object (DEOJ):");
                                        ui.label(translate_object(deoj));
                                        ui.end_row();
                                        
                                        ui.label("Service Code (ESV):");
                                        ui.label(translate_esv(esv));
                                        ui.end_row();
                                        
                                        ui.label("Property Count (OPC):");
                                        ui.monospace(format!("{}", opc));
                                        ui.end_row();
                                    });
                                
                                ui.add_space(10.0);
                                ui.strong("Parsed Properties:");
                                ui.add_space(4.0);
                                
                                // Parse properties
                                let mut properties = Vec::new();
                                let mut curr_offset = 12;
                                let mut is_malformed = false;
                                
                                for _ in 0..opc {
                                    if curr_offset + 2 > entry.data.len() {
                                        is_malformed = true;
                                        break;
                                    }
                                    let epc = entry.data[curr_offset];
                                    let pdc = entry.data[curr_offset + 1];
                                    
                                    curr_offset += 2;
                                    
                                    if curr_offset + (pdc as usize) > entry.data.len() {
                                        is_malformed = true;
                                        break;
                                    }
                                    
                                    let edt = entry.data[curr_offset..curr_offset + (pdc as usize)].to_vec();
                                    curr_offset += pdc as usize;
                                    
                                    properties.push(EchonetProperty { epc, pdc, edt });
                                }
                                
                                // Render properties
                                egui::ScrollArea::vertical().id_salt("el_props_scroll").show(ui, |ui| {
                                    for (prop_idx, prop) in properties.iter().enumerate() {
                                        egui::Frame::NONE
                                            .fill(ui.visuals().widgets.inactive.bg_fill)
                                            .corner_radius(egui::CornerRadius::same(4))
                                            .inner_margin(egui::Margin::symmetric(10, 8))
                                            .show(ui, |ui| {
                                                ui.vertical(|ui| {
                                                    ui.horizontal(|ui| {
                                                        ui.strong(format!("#{}:", prop_idx + 1));
                                                        ui.label("EPC:");
                                                        ui.monospace(format!("0x{:02X}", prop.epc));
                                                        ui.label(translate_epc(prop.epc));
                                                    });
                                                    ui.add_space(2.0);
                                                    ui.horizontal(|ui| {
                                                        ui.label("PDC:");
                                                        ui.monospace(format!("{}", prop.pdc));
                                                        ui.separator();
                                                        ui.label("EDT:");
                                                        ui.monospace(translate_edt(prop.epc, &prop.edt));
                                                    });
                                                });
                                            });
                                        ui.add_space(6.0);
                                    }
                                    
                                    if is_malformed {
                                        ui.colored_label(
                                            egui::Color32::from_rgb(255, 100, 100),
                                            "⚠️ Malformed ECHONET Lite properties: Packet truncated."
                                        );
                                    }
                                });
                            }
                        }
                    }
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Select a log item in the logs panel to inspect its contents");
                });
            }
        });
    }
}

// ASCII Formatter: highlights control characters
fn to_ascii_inspector(bytes: &[u8]) -> String {
    let mut result = String::new();
    for &b in bytes {
        if b.is_ascii_graphic() || b == b' ' {
            result.push(b as char);
        } else {
            let label = match b {
                0 => "[NUL]",
                1 => "[SOH]",
                2 => "[STX]",
                3 => "[ETX]",
                4 => "[EOT]",
                9 => "[TAB]",
                10 => "[LF]",
                13 => "[CR]",
                _ => &format!("[0x{:02X}]", b),
            };
            result.push_str(label);
        }
    }
    result
}

// ECHONET Lite Translators
fn translate_object(obj_bytes: &[u8]) -> String {
    if obj_bytes.len() != 3 {
        return "Unknown".to_string();
    }
    let group = obj_bytes[0];
    let class = obj_bytes[1];
    let instance = obj_bytes[2];
    
    let name = match (group, class) {
        (0x05, 0xFF) => "Controller",
        (0x0E, 0xF0) => "Node Profile",
        (0x01, 0x30) => "Home Air Conditioner",
        (0x02, 0x88) => "Smart Meter",
        _ => "Custom/Unknown Device",
    };
    format!("{} (0x{:02X} {:02X} {:02X})", name, group, class, instance)
}

fn translate_esv(esv: u8) -> String {
    match esv {
        0x60 => "SetI (Set Property - No Response Required)".to_string(),
        0x61 => "SetC (Set Property - Response Required)".to_string(),
        0x62 => "Get (Get Property Value)".to_string(),
        0x63 => "INF_REQ (Property Value Write Request)".to_string(),
        0x71 => "Set_Res (Set Property Response)".to_string(),
        0x72 => "Get_Res (Get Property Response)".to_string(),
        0x73 => "INF (Inform Property Value)".to_string(),
        0x74 => "INFC (Inform Property Value Response)".to_string(),
        0x50 => "SetI_SNA (Set SNA - No Response)".to_string(),
        0x51 => "SetC_SNA (Set SNA Response)".to_string(),
        0x52 => "Get_SNA (Get SNA Response)".to_string(),
        0x53 => "INF_SNA (Inform SNA Response)".to_string(),
        _ => format!("Unknown Service (0x{:02X})", esv),
    }
}

fn translate_epc(epc: u8) -> String {
    match epc {
        0x80 => "-> Operation Status".to_string(),
        0x81 => "-> Installation Location".to_string(),
        0x82 => "-> Standard Version Info".to_string(),
        0x83 => "-> Identification Number".to_string(),
        0x88 => "-> Fault Status".to_string(),
        0x8A => "-> Manufacturer Code".to_string(),
        0xB0 => "-> Operation Mode".to_string(),
        0xC0 => "-> Set Temperature".to_string(),
        0xC1 => "-> Set Temp Cooling".to_string(),
        0xD6 => "-> Self-node instance list".to_string(),
        0xD7 => "-> Self-node class list".to_string(),
        _ => "".to_string(),
    }
}

fn translate_edt(epc: u8, edt: &[u8]) -> String {
    if edt.is_empty() {
        return "Empty".to_string();
    }
    match epc {
        0x80 => {
            if edt[0] == 0x30 {
                "ON (0x30)".to_string()
            } else if edt[0] == 0x31 {
                "OFF (0x31)".to_string()
            } else {
                format!("Unknown (0x{:02X})", edt[0])
            }
        }
        0x88 => {
            if edt[0] == 0x41 {
                "Fault (0x41)".to_string()
            } else if edt[0] == 0x42 {
                "Normal (0x42)".to_string()
            } else {
                format!("Unknown (0x{:02X})", edt[0])
            }
        }
        0xB0 => {
            match edt[0] {
                0x41 => "Automatic (0x41)".to_string(),
                0x42 => "Cooling (0x42)".to_string(),
                0x43 => "Heating (0x43)".to_string(),
                0x44 => "Dehumidifying (0x44)".to_string(),
                0x45 => "Air Circulator (0x45)".to_string(),
                _ => format!("Unknown (0x{:02X})", edt[0]),
            }
        }
        0xC0 | 0xC1 => {
            format!("{} °C (0x{:02X})", edt[0], edt[0])
        }
        _ => {
            edt.iter().map(|b| format!("{:02X}", b)).collect::<Vec<String>>().join(" ")
        }
    }
}
