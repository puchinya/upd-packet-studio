mod udp_worker;
mod types;
mod config;
mod styling;
mod views;

use std::net::SocketAddr;
use std::sync::mpsc::{Receiver, channel};
use chrono::Local;
use eframe::egui;
use egui_dock::{DockArea, DockState};

use udp_worker::{UdpWorker, UdpCommand, UdpEvent};
use types::{Tab, LogEntry, LogDirection, PayloadType, parse_hex_to_bytes, Collection, MulticastGroup, InspectorProtocol, LogExportFormat};
use config::SavedConfig;
use styling::setup_custom_styles;

pub struct UdpStudioState {
    pub(crate) collections: Vec<Collection>,
    pub(crate) selected_request_id: Option<String>,
    pub(crate) composer_selected_collection_idx: usize,
    
    // Composer tab inputs
    pub(crate) composer_target: String,
    pub(crate) composer_payload_type: PayloadType,
    pub(crate) composer_payload: String,
    pub(crate) composer_name: String,
    
    // Logs tab inputs
    pub(crate) logs: Vec<LogEntry>,
    pub(crate) selected_log_idx: Option<usize>,
    pub(crate) filter_text: String,
    pub(crate) auto_scroll: bool,
    pub(crate) log_export_format: LogExportFormat,
    pub(crate) filtered_indices: Vec<usize>,
    
    // Listener settings
    pub(crate) listener_addr: String,
    pub(crate) is_listening: bool,
    pub(crate) bound_addr: Option<String>,
    pub(crate) listener_error: Option<String>,
    
    // Channels & Worker
    pub(crate) udp_worker: UdpWorker,
    pub(crate) rx_event: Receiver<UdpEvent>,

    // ECHONET Lite Helper state
    pub(crate) el_tid: String,
    pub(crate) el_seoj: String,
    pub(crate) el_deoj_preset: usize,
    pub(crate) el_deoj_custom: String,
    pub(crate) el_esv_preset: usize,
    pub(crate) el_epc_preset: usize,
    pub(crate) el_epc_custom: String,
    pub(crate) el_edt: String,
    pub(crate) el_show_helper: bool,

    // Multicast fields
    pub(crate) multicast_groups: Vec<MulticastGroup>,
    pub(crate) multicast_input_addr: String,
    pub(crate) multicast_input_interface: String,

    // Inspector fields
    pub(crate) inspector_protocol: InspectorProtocol,
}

impl UdpStudioState {
    pub(crate) fn save_config(&self) {
        let config = SavedConfig {
            collections: self.collections.clone(),
            listener_addr: self.listener_addr.clone(),
            composer_target: self.composer_target.clone(),
            composer_payload_type: self.composer_payload_type,
            composer_payload: self.composer_payload.clone(),
        };
        config.save();
    }

    pub(crate) fn push_log(&mut self, entry: LogEntry) {
        self.logs.push(entry);
        self.update_filtered_indices();
    }

    pub(crate) fn add_system_info(&mut self, msg: String) {
        let entry = LogEntry::new(
            Local::now(),
            LogDirection::SystemInfo,
            SocketAddr::from(([0, 0, 0, 0], 0)),
            msg.into_bytes(),
        );
        self.push_log(entry);
    }

    pub(crate) fn add_system_error(&mut self, msg: String) {
        let entry = LogEntry::new(
            Local::now(),
            LogDirection::SystemError,
            SocketAddr::from(([0, 0, 0, 0], 0)),
            msg.into_bytes(),
        );
        self.push_log(entry);
    }

    pub(crate) fn update_filtered_indices(&mut self) {
        self.filtered_indices = self.logs
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                if self.filter_text.is_empty() {
                    return true;
                }
                entry.address_str.contains(&self.filter_text)
            })
            .map(|(idx, _)| idx)
            .collect();
    }

    pub(crate) fn send_packet(&mut self, target: &str, payload_type: PayloadType, payload: &str) {
        let data_res = match payload_type {
            PayloadType::Text => Ok(payload.as_bytes().to_vec()),
            PayloadType::Hex => parse_hex_to_bytes(payload),
        };
        
        match data_res {
            Ok(data) => {
                if data.is_empty() {
                    self.add_system_error("Cannot send empty packet.".to_string());
                    return;
                }
                self.udp_worker.send(UdpCommand::Send {
                    target: target.to_string(),
                    data,
                });
            }
            Err(e) => {
                self.add_system_error(format!("Hex parsing error: {}", e));
            }
        }
    }
}

struct MyTabViewer<'a> {
    state: &'a mut UdpStudioState,
}

impl<'a> egui_dock::TabViewer for MyTabViewer<'a> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            Tab::Collections => "📁 Collections".into(),
            Tab::Sender => "🚀 Composer".into(),
            Tab::LogViewer => "📊 Logs".into(),
            Tab::Inspector => "🔍 Inspector".into(),
            Tab::Multicast => "🌐 Multicast".into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            Tab::Collections => self.state.show_collections(ui),
            Tab::Sender => self.state.show_sender(ui),
            Tab::LogViewer => self.state.show_log_viewer(ui),
            Tab::Inspector => self.state.show_inspector(ui),
            Tab::Multicast => self.state.show_multicast(ui),
        }
    }
}

struct MainApp {
    dock_state: DockState<Tab>,
    state: UdpStudioState,
}

impl MainApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_custom_styles(&cc.egui_ctx);
        
        let config = SavedConfig::load();
        
        // Setup initial docking layout tree (3-column split where center is stacked)
        let mut dock_state = DockState::new(vec![Tab::Collections]);
        let surface = dock_state.main_surface_mut();
        
        // Split right to place LogViewer in the middle
        let [_left, middle] = surface.split_right(egui_dock::NodeIndex::root(), 0.25, vec![Tab::LogViewer]);
        // Split middle to place Sender on the right. 'center' now points to the leaf node with Tab::LogViewer.
        let [center, right] = surface.split_right(middle, 0.60, vec![Tab::Sender]);
        // Split center below to place Inspector at the bottom of the center column
        let [_, _] = surface.split_below(center, 0.55, vec![Tab::Inspector]);
        // Split right below to place Multicast at the bottom of the right column
        let [_, _] = surface.split_below(right, 0.55, vec![Tab::Multicast]);
        
        let (tx_event, rx_event) = channel();
        let udp_worker = UdpWorker::spawn(tx_event, cc.egui_ctx.clone());
        
        let state = UdpStudioState {
            collections: config.collections,
            selected_request_id: None,
            composer_selected_collection_idx: 0,
            composer_target: config.composer_target,
            composer_payload_type: config.composer_payload_type,
            composer_payload: config.composer_payload,
            composer_name: String::new(),
            logs: Vec::new(),
            selected_log_idx: None,
            filter_text: String::new(),
            auto_scroll: true,
            log_export_format: LogExportFormat::Csv,
            filtered_indices: Vec::new(),
            listener_addr: config.listener_addr,
            is_listening: false,
            bound_addr: None,
            listener_error: None,
            udp_worker,
            rx_event,
            el_tid: "0001".to_string(),
            el_seoj: "05FF01".to_string(),
            el_deoj_preset: 0,
            el_deoj_custom: "013001".to_string(),
            el_esv_preset: 0,
            el_epc_preset: 0,
            el_epc_custom: "80".to_string(),
            el_edt: "30".to_string(),
            el_show_helper: false,
            multicast_groups: Vec::new(),
            multicast_input_addr: "224.0.23.0".to_string(),
            multicast_input_interface: "0.0.0.0".to_string(),
            inspector_protocol: InspectorProtocol::Raw,
        };

        Self {
            dock_state,
            state,
        }
    }
}

// macOS style window control circles
fn circle_button(ui: &mut egui::Ui, color: egui::Color32) -> egui::Response {
    let size = egui::vec2(12.0, 12.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let is_hovered = response.hovered();
        let is_pressed = is_hovered && ui.input(|i| i.pointer.primary_down());
        
        let fill_color = if is_pressed {
            color.linear_multiply(0.7)
        } else if is_hovered {
            color.linear_multiply(0.9)
        } else {
            color
        };
        
        painter.circle_filled(rect.center(), 6.0, fill_color);
    }
    response
}

impl eframe::App for MainApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx();
        
        // Handle all incoming events from the background UDP worker thread
        while let Ok(event) = self.state.rx_event.try_recv() {
            match event {
                UdpEvent::Bound(addr) => {
                    self.state.is_listening = true;
                    self.state.bound_addr = Some(addr.to_string());
                    self.state.listener_addr = addr.to_string();
                    self.state.listener_error = None;
                    self.state.add_system_info(format!("Listening socket bound to {}", addr));
                    self.state.save_config();
                }
                UdpEvent::Unbound => {
                    self.state.is_listening = false;
                    self.state.bound_addr = None;
                    self.state.multicast_groups.clear();
                    self.state.add_system_info("Listening socket unbound".to_string());
                }
                UdpEvent::Sent { to, data, timestamp } => {
                    let entry = LogEntry::new(timestamp, LogDirection::Sent, to, data);
                    self.state.push_log(entry);
                }
                UdpEvent::Received { from, data, timestamp } => {
                    let entry = LogEntry::new(timestamp, LogDirection::Received, from, data);
                    self.state.push_log(entry);
                    ctx.request_repaint();
                }
                UdpEvent::Error(err_msg) => {
                    if !self.state.is_listening {
                        self.state.listener_error = Some(err_msg.clone());
                        self.state.multicast_groups.clear();
                    }
                    self.state.add_system_error(err_msg);
                }
                UdpEvent::MulticastJoined { multi_addr, interface_addr } => {
                    self.state.multicast_groups.push(MulticastGroup {
                        multi_addr: multi_addr.clone(),
                        interface_addr: interface_addr.clone(),
                    });
                    self.state.add_system_info(format!("Joined multicast group {} on interface {}", multi_addr, interface_addr));
                }
                UdpEvent::MulticastLeft { multi_addr, interface_addr } => {
                    self.state.multicast_groups.retain(|g| !(g.multi_addr == *multi_addr && g.interface_addr == *interface_addr));
                    self.state.add_system_info(format!("Left multicast group {} on interface {}", multi_addr, interface_addr));
                }
            }
        }

        // Outer window container with rounded corners (enabled by transparent viewport option)
        egui::Frame::NONE
            .fill(egui::Color32::from_rgb(20, 24, 30))
            .corner_radius(egui::CornerRadius::same(12))
            .show(ui, |ui| {
                // Set spacing between panel elements to zero so they align perfectly
                ui.style_mut().spacing.item_spacing = egui::vec2(0.0, 0.0);

                // Custom Title Bar Panel (Mac Style header with integrated socket listener setup)
                egui::Panel::top("custom_title_bar")
                    .frame(egui::Frame::default()
                        .fill(egui::Color32::from_rgb(15, 18, 22))
                        .corner_radius(egui::CornerRadius {
                            nw: 12,
                            ne: 12,
                            sw: 0,
                            se: 0,
                        })
                        .inner_margin(egui::Margin::symmetric(12, 8)))
                    .show_inside(ui, |ui| {
                        // Title bar background drag/double-click action covering the entire bar area
                        let title_bar_rect = ui.max_rect();
                        let drag_resp = ui.interact(title_bar_rect, ui.id().with("title_bar_drag"), egui::Sense::drag());
                        if drag_resp.dragged_by(egui::PointerButton::Primary) {
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
                        }
                        if drag_resp.double_clicked() {
                            let is_maximized = ui.ctx().input(|i| i.viewport().maximized.unwrap_or(false));
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
                        }

                        ui.horizontal(|ui| {
                            // Traffic lights window controls
                            ui.horizontal(|ui| {
                                if circle_button(ui, egui::Color32::from_rgb(255, 95, 86)).on_hover_text("Close").clicked() {
                                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                                }
                                ui.add_space(2.0);
                                if circle_button(ui, egui::Color32::from_rgb(255, 189, 46)).on_hover_text("Minimize").clicked() {
                                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                                }
                                ui.add_space(2.0);
                                if circle_button(ui, egui::Color32::from_rgb(39, 201, 63)).on_hover_text("Maximize").clicked() {
                                    let is_maximized = ui.ctx().input(|i| i.viewport().maximized.unwrap_or(false));
                                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
                                }
                            });
                            
                            ui.add_space(15.0);
                            
                            // Application title
                            ui.strong("UDP Packet Studio");
                            
                            ui.add_space(20.0);
                            ui.separator();
                            ui.add_space(15.0);
                            
                            // Integrated Socket Bind Controls
                            ui.label("Bind Address:");
                            let text_res = ui.add(egui::TextEdit::singleline(&mut self.state.listener_addr).desired_width(130.0));
                            if text_res.changed() {
                                self.state.save_config();
                            }
                            
                            ui.add_space(5.0);
                            
                            if self.state.is_listening {
                                if ui.button("⏹ Stop").clicked() {
                                    self.state.udp_worker.send(UdpCommand::Unbind);
                                }
                                ui.add_space(5.0);
                                ui.colored_label(egui::Color32::from_rgb(100, 255, 100), "🟢 Active");
                                if let Some(ref addr) = self.state.bound_addr {
                                    ui.label(format!("({})", addr));
                                }
                            } else {
                                if ui.button("▶ Bind").clicked() {
                                    self.state.listener_error = None;
                                    self.state.udp_worker.send(UdpCommand::Bind(self.state.listener_addr.clone()));
                                }
                                ui.add_space(5.0);
                                ui.colored_label(egui::Color32::from_rgb(255, 90, 90), "🔴 Offline");
                            }
                            
                            if let Some(ref err) = self.state.listener_error {
                                ui.add_space(10.0);
                                ui.colored_label(egui::Color32::from_rgb(255, 90, 90), format!("⚠️ {}", err));
                            }
                        });
                    });

                // Bottom status bar panel
                egui::Panel::bottom("bottom_status_bar")
                    .frame(egui::Frame::default()
                        .fill(egui::Color32::from_rgb(15, 18, 22))
                        .corner_radius(egui::CornerRadius {
                            nw: 0,
                            ne: 0,
                            sw: 12,
                            se: 12,
                        })
                        .inner_margin(egui::Margin::symmetric(12, 6)))
                    .show_inside(ui, |ui| {
                        ui.horizontal(|ui| {
                            if self.state.is_listening {
                                ui.colored_label(egui::Color32::from_rgb(100, 255, 100), "🟢 Active");
                                if let Some(ref addr) = self.state.bound_addr {
                                    ui.label(format!("Bound: {}", addr));
                                }
                                ui.add_space(10.0);
                                ui.separator();
                                ui.add_space(10.0);
                                ui.colored_label(egui::Color32::from_rgb(140, 200, 255), "📣 Broadcast Enabled");
                            } else {
                                ui.colored_label(egui::Color32::from_rgb(255, 90, 90), "🔴 Offline");
                                ui.label("Socket not bound");
                            }
                            
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(format!("Logged packets: {}", self.state.logs.len()));
                            });
                        });
                    });

                // Main docking control area inside central panel
                let mut viewer = MyTabViewer { state: &mut self.state };
                DockArea::new(&mut self.dock_state)
                    .show_inside(ui, &mut viewer);
            });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default()
            .with_title("UDP Packet Studio")
            .with_inner_size([1100.0, 700.0])
            .with_decorations(false) // borderless window
            .with_transparent(true),
        ..Default::default()
    };
    
    eframe::run_native(
        "UDP Packet Studio",
        options,
        Box::new(|cc| Ok(Box::new(MainApp::new(cc)))),
    )
}
