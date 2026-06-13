use eframe::egui;
use crate::UdpStudioState;

impl UdpStudioState {
    pub fn show_multicast(&mut self, ui: &mut egui::Ui) {
        let mut join_trigger = None;
        let mut leave_trigger = None;

        ui.vertical(|ui| {
            // Listener Status Warning
            if !self.is_listening {
                egui::Frame::NONE
                    .fill(egui::Color32::from_rgb(45, 20, 20))
                    .corner_radius(egui::CornerRadius::same(4))
                    .inner_margin(egui::Margin::same(10))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.colored_label(egui::Color32::from_rgb(255, 120, 120), "⚠️ Listener Offline:");
                            ui.label("You must Bind to a local port in the title bar first before joining multicast groups.");
                        });
                    });
                ui.add_space(10.0);
            }

            // 1. Join Multicast Group Form
            egui::Frame::NONE
                .fill(ui.visuals().widgets.inactive.bg_fill)
                .corner_radius(egui::CornerRadius::same(6))
                .inner_margin(egui::Margin::same(12))
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.strong("🌐 Join a Multicast Group");
                        ui.add_space(8.0);

                        egui::Grid::new("multicast_join_grid")
                            .num_columns(2)
                            .spacing([10.0, 8.0])
                            .show(ui, |ui| {
                                ui.label("Multicast IP:");
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.multicast_input_addr)
                                        .desired_width(180.0)
                                        .hint_text("e.g. 224.0.23.0")
                                );
                                ui.end_row();

                                ui.label("Local Interface IP:");
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.multicast_input_interface)
                                        .desired_width(180.0)
                                        .hint_text("e.g. 0.0.0.0")
                                );
                                ui.end_row();
                            });

                        ui.add_space(10.0);

                        // Quick Presets
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Quick Presets:").weak().size(11.0));
                            
                            if ui.button("ECHONET Lite").on_hover_text("Join 224.0.23.0").clicked() {
                                self.multicast_input_addr = "224.0.23.0".to_string();
                                self.multicast_input_interface = "0.0.0.0".to_string();
                            }
                            if ui.button("SSDP").on_hover_text("Join 239.255.255.250").clicked() {
                                self.multicast_input_addr = "239.255.255.250".to_string();
                                self.multicast_input_interface = "0.0.0.0".to_string();
                            }
                            if ui.button("mDNS").on_hover_text("Join 224.0.0.251").clicked() {
                                self.multicast_input_addr = "224.0.0.251".to_string();
                                self.multicast_input_interface = "0.0.0.0".to_string();
                            }
                            if ui.button("All-Nodes").on_hover_text("Join 224.0.0.1").clicked() {
                                self.multicast_input_addr = "224.0.0.1".to_string();
                                self.multicast_input_interface = "0.0.0.0".to_string();
                            }
                        });

                        ui.add_space(12.0);

                        let join_btn = ui.add_enabled(
                            self.is_listening,
                            egui::Button::new("➕ Join Multicast Group").min_size(egui::vec2(150.0, 26.0))
                        );
                        if join_btn.clicked() {
                            join_trigger = Some((
                                self.multicast_input_addr.trim().to_string(),
                                self.multicast_input_interface.trim().to_string()
                            ));
                        }
                    });
                });

            ui.add_space(15.0);
            ui.separator();
            ui.add_space(10.0);

            // 2. Joined Groups List
            ui.strong("👥 Currently Joined Groups");
            ui.add_space(6.0);

            egui::ScrollArea::vertical().id_salt("multicast_list_scroll").show(ui, |ui| {
                if self.multicast_groups.is_empty() {
                    ui.colored_label(
                        egui::Color32::from_rgb(120, 130, 140),
                        egui::RichText::new("No active multicast memberships on this socket.").italics()
                    );
                } else {
                    egui::Grid::new("multicast_joined_grid")
                        .num_columns(3)
                        .spacing([15.0, 8.0])
                        .show(ui, |ui| {
                            // Table Header
                            ui.colored_label(egui::Color32::from_rgb(180, 190, 200), egui::RichText::new("Multicast Address").strong());
                            ui.colored_label(egui::Color32::from_rgb(180, 190, 200), egui::RichText::new("Interface Address").strong());
                            ui.colored_label(egui::Color32::from_rgb(180, 190, 200), egui::RichText::new("Action").strong());
                            ui.end_row();

                            for group in &self.multicast_groups {
                                ui.label(&group.multi_addr);
                                ui.label(&group.interface_addr);
                                
                                if ui.button("🗑 Leave").clicked() {
                                    leave_trigger = Some((group.multi_addr.clone(), group.interface_addr.clone()));
                                }
                                ui.end_row();
                            }
                        });
                }
            });
        });

        // Apply mutations outside borrow scopes
        if let Some((m_addr, i_addr)) = join_trigger {
            if m_addr.is_empty() || i_addr.is_empty() {
                self.logs.push(crate::types::LogEntry {
                    timestamp: chrono::Local::now(),
                    direction: crate::types::LogDirection::SystemError,
                    address: std::net::SocketAddr::from(([0, 0, 0, 0], 0)),
                    data: "Multicast address and Interface address cannot be empty.".to_string().into_bytes(),
                });
            } else {
                self.udp_worker.send(crate::udp_worker::UdpCommand::JoinMulticast {
                    multi_addr: m_addr,
                    interface_addr: i_addr,
                });
            }
        }

        if let Some((m_addr, i_addr)) = leave_trigger {
            self.udp_worker.send(crate::udp_worker::UdpCommand::LeaveMulticast {
                multi_addr: m_addr,
                interface_addr: i_addr,
            });
        }
    }
}
