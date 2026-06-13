use eframe::egui;

// Styling system
pub fn setup_custom_styles(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    
    // Custom flat dark-theme color tokens
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(20, 24, 30);
    visuals.widgets.noninteractive.weak_bg_fill = egui::Color32::from_rgb(26, 32, 40);
    visuals.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(45, 55, 72));
    
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(32, 42, 54);
    visuals.widgets.inactive.weak_bg_fill = egui::Color32::from_rgb(32, 42, 54);
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 75, 95));
    
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(45, 59, 76);
    visuals.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(45, 59, 76);
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 149, 237));
    
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(60, 78, 102);
    visuals.widgets.active.weak_bg_fill = egui::Color32::from_rgb(60, 78, 102);
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 149, 237));
    
    visuals.selection.bg_fill = egui::Color32::from_rgb(100, 149, 237);
    visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::WHITE);
    
    visuals.panel_fill = egui::Color32::from_rgb(20, 24, 30);
    visuals.window_fill = egui::Color32::from_rgb(26, 32, 40);
    
    visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(6);
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(4);
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(4);
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(4);
    visuals.window_corner_radius = egui::CornerRadius::same(8);
    
    ctx.set_visuals(visuals);
    
    // Tweak spacing and rounding for standard layouts
    let mut style = (*ctx.global_style()).clone();
    style.spacing.item_spacing = egui::vec2(10.0, 10.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);
    style.spacing.window_margin = egui::Margin::same(12);
    ctx.set_global_style(style);
}
