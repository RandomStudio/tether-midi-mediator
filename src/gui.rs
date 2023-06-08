use std::time::Duration;

use egui::{Color32, RichText};

use crate::mediation::{MediationDataModel, MONITOR_LOG_LENGTH};

pub fn render_gui(model: &mut MediationDataModel, ui: &mut egui::Ui) {
    ui.heading("Tether Agent");
    if model.tether_connected {
        ui.label(RichText::new("Connected").color(Color32::GREEN));
        if let Some(uri) = &model.tether_uri {
            ui.label(uri);
        }
    } else {
        ui.label(RichText::new("Not connected").color(Color32::RED));
    }
    ui.separator();

    ui.heading("MIDI Ports Connected");

    for (_key, info) in model.ports_metadata.iter() {
        ui.horizontal(|ui| {
            ui.label(&format!("PORT #{}: \"{}\"", info.index, info.full_name));
            if let Ok(elapsed) = info.last_received.elapsed() {
                let color = if elapsed > Duration::from_secs(15) {
                    Color32::DARK_GRAY
                } else if elapsed > Duration::from_secs(1) {
                    Color32::GRAY
                } else {
                    Color32::GREEN
                };
                ui.label(RichText::new(format!("{:.0}s ago", elapsed.as_secs_f32())).color(color));
            }
        });
    }

    ui.separator();

    ui.columns(2, |columns| {
        // Left: MIDI IN
        columns[0].heading(&format!(
            "Last {} (max) MIDI messages received",
            MONITOR_LOG_LENGTH
        ));

        if model.midi_message_log.is_empty() {
            columns[0].label("Nothing received yet");
        } else {
            egui::ScrollArea::vertical()
                .auto_shrink([true; 2])
                .id_source("left")
                .show(&mut columns[0], |ui| {
                    for item in model.midi_message_log.iter().rev() {
                        ui.label(item);
                    }
                });
        }

        // Right: Tether OUT
        columns[1].heading(&format!(
            "Last {} (max) Tether messages translated",
            MONITOR_LOG_LENGTH
        ));

        if model.tether_message_log.is_empty() {
            columns[1].label("Nothing sent yet");
        } else {
            egui::ScrollArea::vertical()
                .auto_shrink([true; 2])
                .id_source("right")
                .show(&mut columns[1], |ui| {
                    for item in model.tether_message_log.iter().rev() {
                        ui.label(item);
                    }
                });
        }
    });
}
