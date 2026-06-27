use eframe::egui;
use crate::UdpStudioState;

impl UdpStudioState {
    pub fn show_multicast(&mut self, ui: &mut egui::Ui) {
        crate::locales::init_translations();
        let lang_id = self.language_id();
        let tr = |key: &str| {
            egui_i18n::set_language(&lang_id);
            egui_i18n::tr!(key)
        };
        let tr_args = |key: &str, args: &std::collections::HashMap<std::borrow::Cow<'static, str>, egui_i18n::fluent_bundle::FluentValue<'_>>| {
            egui_i18n::set_language(&lang_id);
            let mut fluent_args = egui_i18n::fluent::FluentArgs::new();
            for (k, v) in args {
                fluent_args.set(k.as_ref(), v.clone());
            }
            egui_i18n::translate_fluent(key, &fluent_args)
        };

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
                            ui.colored_label(egui::Color32::from_rgb(255, 120, 120), tr("mc-status-offline"));
                            ui.label(tr("mc-status-offline-tip"));
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
                        ui.strong(tr("mc-join-title"));
                        ui.add_space(8.0);

                        egui::Grid::new("multicast_join_grid")
                            .num_columns(2)
                            .spacing([10.0, 8.0])
                            .show(ui, |ui| {
                                ui.label(tr("mc-label-multicast-ip"));
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.multicast_input_addr)
                                        .desired_width(180.0)
                                        .hint_text("e.g. 224.0.23.0")
                                );
                                ui.end_row();

                                ui.label(tr("mc-label-interface-ip"));
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
                            ui.label(egui::RichText::new(tr("mc-label-presets")).weak().size(11.0));
                            
                            let mut args_ac = std::collections::HashMap::new();
                            args_ac.insert(std::borrow::Cow::Borrowed("ip"), "224.0.23.0".into());
                            if ui.button("ECHONET Lite").on_hover_text(tr_args("mc-preset-tip", &args_ac)).clicked() {
                                self.multicast_input_addr = "224.0.23.0".to_string();
                                self.multicast_input_interface = "0.0.0.0".to_string();
                            }

                            let mut args_ssdp = std::collections::HashMap::new();
                            args_ssdp.insert(std::borrow::Cow::Borrowed("ip"), "239.255.255.250".into());
                            if ui.button("SSDP").on_hover_text(tr_args("mc-preset-tip", &args_ssdp)).clicked() {
                                self.multicast_input_addr = "239.255.255.250".to_string();
                                self.multicast_input_interface = "0.0.0.0".to_string();
                            }

                            let mut args_mdns = std::collections::HashMap::new();
                            args_mdns.insert(std::borrow::Cow::Borrowed("ip"), "224.0.0.251".into());
                            if ui.button("mDNS").on_hover_text(tr_args("mc-preset-tip", &args_mdns)).clicked() {
                                self.multicast_input_addr = "224.0.0.251".to_string();
                                self.multicast_input_interface = "0.0.0.0".to_string();
                            }

                            let mut args_nodes = std::collections::HashMap::new();
                            args_nodes.insert(std::borrow::Cow::Borrowed("ip"), "224.0.0.1".into());
                            if ui.button("All-Nodes").on_hover_text(tr_args("mc-preset-tip", &args_nodes)).clicked() {
                                self.multicast_input_addr = "224.0.0.1".to_string();
                                self.multicast_input_interface = "0.0.0.0".to_string();
                            }
                        });

                        ui.add_space(12.0);

                        let join_btn = ui.add_enabled(
                            self.is_listening,
                            egui::Button::new(tr("mc-btn-join")).min_size(egui::vec2(150.0, 26.0))
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
            ui.strong(tr("mc-title-joined-list"));
            ui.add_space(6.0);

            egui::ScrollArea::vertical().id_salt("multicast_list_scroll").show(ui, |ui| {
                if self.multicast_groups.is_empty() {
                    ui.colored_label(
                        egui::Color32::from_rgb(120, 130, 140),
                        egui::RichText::new(tr("mc-no-memberships")).italics()
                    );
                } else {
                    egui::Grid::new("multicast_joined_grid")
                        .num_columns(3)
                        .spacing([15.0, 8.0])
                        .show(ui, |ui| {
                            // Table Header
                            ui.colored_label(egui::Color32::from_rgb(180, 190, 200), egui::RichText::new(tr("mc-hdr-multicast-addr")).strong());
                            ui.colored_label(egui::Color32::from_rgb(180, 190, 200), egui::RichText::new(tr("mc-hdr-interface-addr")).strong());
                            ui.colored_label(egui::Color32::from_rgb(180, 190, 200), egui::RichText::new(tr("mc-hdr-action")).strong());
                            ui.end_row();

                            for group in &self.multicast_groups {
                                ui.label(&group.multi_addr);
                                ui.label(&group.interface_addr);
                                
                                if ui.button(tr("mc-btn-leave")).clicked() {
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
                self.add_system_error(tr("mc-err-empty-fields"));
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
