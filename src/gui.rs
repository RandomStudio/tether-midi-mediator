use std::time::Duration;

use egui::{Color32, RichText};

use crate::mediation::{MediationDataModel, MONITOR_LOG_LENGTH};

pub fn render_gui(model: &mut MediationDataModel, ui: &mut egui::Ui) {
    ui.heading("MIDI Ports Connected");

    for (_key, info) in model.port_info.iter() {
        ui.horizontal(|ui| {
            ui.label(&format!("PORT #{}: \"{}\"", info.index, info.full_name));
            if let Ok(elapsed) = info.last_received.elapsed() {
                let color = if elapsed > Duration::from_secs(5) {
                    Color32::RED
                } else if elapsed > Duration::from_secs(1) {
                    Color32::LIGHT_YELLOW
                } else {
                    Color32::GREEN
                };
                ui.label(RichText::new(&format!("{:.1}s ago", elapsed.as_secs_f32())).color(color));
            }
        });
    }

    ui.separator();

    ui.heading(&format!(
        "Last {} (max) messages received",
        MONITOR_LOG_LENGTH
    ));

    if model.message_log.is_empty() {
        ui.label("Nothing received yet");
    } else {
        egui::ScrollArea::vertical()
            .auto_shrink([true; 2])
            .show(ui, |ui| {
                for item in model.message_log.iter().rev() {
                    ui.label(item);
                }
            });
    }
}
