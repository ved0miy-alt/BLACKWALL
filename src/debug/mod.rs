/// Debug UI using egui
use egui::{Context, Color32, RichText};
use glam::Vec3;

pub struct DebugUI {
    pub visible: bool,
    pub fps: f32,
    pub chunk_count: usize,
    pub point_count: usize,
    pub camera_pos: Vec3,
    pub generation_time_ms: f32,

    // Network parameters display
    pub density: f32,
    pub chaos: f32,
    pub flow: f32,
    pub entropy: f32,
    pub packet_rate: f32,
    pub energy: f32,
    pub frequency: f32,
    pub curvature: f32,

    // Traffic stats
    pub packets_per_sec: f32,
    pub bytes_per_sec: f32,
    pub tcp_count: usize,
    pub udp_count: usize,
}

impl Default for DebugUI {
    fn default() -> Self {
        Self {
            visible: true,
            fps: 0.0,
            chunk_count: 0,
            point_count: 0,
            camera_pos: Vec3::ZERO,
            generation_time_ms: 0.0,
            density: 0.0,
            chaos: 0.0,
            flow: 0.0,
            entropy: 0.0,
            packet_rate: 0.0,
            energy: 0.0,
            frequency: 0.0,
            curvature: 0.0,
            packets_per_sec: 0.0,
            bytes_per_sec: 0.0,
            tcp_count: 0,
            udp_count: 0,
        }
    }
}

impl DebugUI {
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn ui(&mut self, ctx: &Context) {
        if !self.visible {
            return;
        }

        // Configure style to remove background and separators
        let mut style = (*ctx.style()).clone();
        style.visuals.window_fill = egui::Color32::TRANSPARENT;
        style.visuals.panel_fill = egui::Color32::TRANSPARENT;
        style.visuals.window_stroke = egui::Stroke::NONE;
        style.visuals.widgets.noninteractive.bg_stroke = egui::Stroke::NONE;
        ctx.set_style(style);

        // Top panel centered horizontally, no background, no separator
        egui::TopBottomPanel::top("network_stats")
            .frame(egui::Frame::none())
            .show_separator_line(false)
            .show(ctx, |ui| {
                ui.add_space(20.0);

                ui.horizontal(|ui| {
                    // Center the content horizontally
                    let available_width = ui.available_width();
                    let content_width = 600.0;
                    let spacing = (available_width - content_width) / 2.0;
                    ui.add_space(spacing.max(0.0));

                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            // Left column - traffic stats
                            ui.vertical(|ui| {
                                ui.label(RichText::new("NETWORK TRAFFIC").color(Color32::from_rgb(255, 0, 0)).size(16.0).strong());
                                ui.add_space(8.0);

                                ui.label(RichText::new(format!("Packets/sec: {:.0}", self.packets_per_sec))
                                    .color(Color32::from_rgb(255, 50, 50))
                                    .size(14.0));

                                ui.label(RichText::new(format!("Bytes/sec: {:.0}", self.bytes_per_sec))
                                    .color(Color32::from_rgb(255, 50, 50))
                                    .size(14.0));

                                ui.label(RichText::new(format!("TCP: {}", self.tcp_count))
                                    .color(Color32::from_rgb(255, 80, 80))
                                    .size(13.0));

                                ui.label(RichText::new(format!("UDP: {}", self.udp_count))
                                    .color(Color32::from_rgb(255, 80, 80))
                                    .size(13.0));
                            });

                            ui.add_space(40.0);

                            // Middle column - parameters
                            ui.vertical(|ui| {
                                ui.label(RichText::new("PARAMETERS").color(Color32::from_rgb(255, 0, 0)).size(16.0).strong());
                                ui.add_space(8.0);

                                ui.label(RichText::new(format!("density: {:.3}", self.density))
                                    .color(Color32::from_rgb(255, 100, 100))
                                    .size(13.0));

                                ui.label(RichText::new(format!("chaos: {:.3}", self.chaos))
                                    .color(Color32::from_rgb(255, 100, 100))
                                    .size(13.0));

                                ui.label(RichText::new(format!("flow: {:.3}", self.flow))
                                    .color(Color32::from_rgb(255, 100, 100))
                                    .size(13.0));

                                ui.label(RichText::new(format!("entropy: {:.3}", self.entropy))
                                    .color(Color32::from_rgb(255, 100, 100))
                                    .size(13.0));
                            });

                            ui.add_space(40.0);

                            // Right column - more parameters + stats
                            ui.vertical(|ui| {
                                ui.label(RichText::new(format!("packet_rate: {:.3}", self.packet_rate))
                                    .color(Color32::from_rgb(255, 100, 100))
                                    .size(13.0));

                                ui.label(RichText::new(format!("energy: {:.3}", self.energy))
                                    .color(Color32::from_rgb(255, 100, 100))
                                    .size(13.0));

                                ui.label(RichText::new(format!("frequency: {:.3}", self.frequency))
                                    .color(Color32::from_rgb(255, 100, 100))
                                    .size(13.0));

                                ui.label(RichText::new(format!("curvature: {:.3}", self.curvature))
                                    .color(Color32::from_rgb(255, 100, 100))
                                    .size(13.0));

                                ui.add_space(5.0);

                                ui.label(RichText::new(format!("FPS: {:.1}", self.fps))
                                    .color(Color32::from_rgb(255, 80, 80))
                                    .size(13.0));
                            });
                        });
                    });
                });

                ui.add_space(10.0);
            });
    }
}
