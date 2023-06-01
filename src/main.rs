use std::{
    sync::mpsc::{self},
    time::Duration,
};

use clap::Parser;
use eframe::egui;
use egui::{Color32, RichText};
use env_logger::Env;
use log::{debug, info, warn};
use mediation::{MediationDataModel, MONITOR_LOG_LENGTH};
use midi_interface::get_midi_connection;
use midi_msg::{MidiMsg, ReceiverContext, SystemRealTimeMsg};
use midir::{Ignore, MidiInput};
use settings::Cli;
use tether_interface::{start_tether_agent, TetherSettings};

mod mediation;
mod midi_interface;
mod settings;
mod tether_interface;

fn main() {
    let cli = Cli::parse();
    env_logger::Builder::from_env(Env::default().default_filter_or(&cli.log_level))
        .filter_module("paho_mqtt", log::LevelFilter::Warn)
        .filter_module("egui_glow", log::LevelFilter::Warn)
        .filter_module("egui_winit", log::LevelFilter::Warn)
        .filter_module("eframe", log::LevelFilter::Warn)
        .init();

    // TODO: we don't really use these handles or join them
    // Might be useful for closing things down properly, though
    let mut handles = Vec::new();

    let (midi_tx, midi_rx) = mpsc::channel();
    let (tether_tx, tether_rx) = mpsc::channel();

    let mut model = MediationDataModel::new(midi_rx, tether_tx);

    if cli.tether_disable {
        warn!("Tether connection disabled; local-mode only");
    } else {
        handles.push(start_tether_agent(
            tether_rx,
            TetherSettings {
                host: cli.tether_host,
                username: cli.tether_username,
                password: cli.tether_password,
                role: cli.tether_role,
                id: cli.tether_id,
            },
        ));
    }

    for port in cli.midi_ports {
        let mut midi_input = MidiInput::new("midir reading input").expect("midir failure");
        midi_input.ignore(Ignore::None);

        let midi_tx = midi_tx.clone();
        let (midi_input_port, port_name) =
            get_midi_connection(&midi_input, port).expect("failed to open MIDI port");
        model.add_port(port, port_name);
        let midi_thread = std::thread::spawn(move || {
            let mut ctx = ReceiverContext::new();

            let _connection = midi_input
                .connect(
                    &midi_input_port,
                    "midir-read-input",
                    move |_timestamp, midi_bytes, _| {
                        let (msg, _len) = MidiMsg::from_midi_with_context(&midi_bytes, &mut ctx)
                            .expect("Not an error");

                        // Handle everything but spammy clock messages.
                        if let MidiMsg::SystemRealTime {
                            msg: SystemRealTimeMsg::TimingClock,
                        } = msg
                        {
                            // no-op
                        } else {
                            midi_tx
                                .send((port, msg))
                                .expect("failed to send on channel");
                            // println!("{}: {:?}", stamp, msg);
                        }
                    },
                    (),
                )
                .expect("failed to connect port");

            // TODO: this thread should end when required
            loop {
                std::thread::sleep(Duration::from_millis(1));
            }
            //     listen_for_midi(port, midi_tx);
        });
        handles.push(midi_thread);
    }

    // for handle in handles {
    //     handle.join().unwrap();
    // }

    if cli.headless_mode {
        info!("Running in headless mode; Ctrl+C to quit");
        loop {
            while let Ok((port_index, msg)) = &model.midi_rx.try_recv() {
                debug!("Last received message: {:?}", &msg);
                model.handle_incoming_midi(*port_index, msg);
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    } else {
        info!("Running graphics mode; close the window to quit");
        let options = eframe::NativeOptions {
            // initial_window_size: Some(egui::vec2(320.0, 240.0)),
            ..Default::default()
        };
        eframe::run_native(
            "Tether MIDI Mediator",
            options,
            Box::new(|_cc| Box::<MediationDataModel>::new(model)),
        )
        .expect("Failed to launch GUI");
        info!("GUI ended; exit now...");
        std::process::exit(0);
    }
}

impl eframe::App for MediationDataModel {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // TODO: continuous mode essential?
        ctx.request_repaint();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("MIDI Ports Connected");

            for (_key, info) in self.port_info.iter() {
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
                        ui.label(
                            RichText::new(&format!("{:.1}s ago", elapsed.as_secs_f32()))
                                .color(color),
                        );
                    }
                });
            }

            ui.separator();

            ui.heading(&format!(
                "Last {} (max) messages received",
                MONITOR_LOG_LENGTH
            ));

            if self.message_log.is_empty() {
                ui.label("Nothing received yet");
            } else {
                egui::ScrollArea::vertical()
                    .auto_shrink([true; 2])
                    .show(ui, |ui| {
                        for item in self.message_log.iter().rev() {
                            ui.label(item);
                        }
                    });
            }
        });

        if let Ok((port_index, msg)) = &self.midi_rx.try_recv() {
            debug!("GUI received MIDI message: {:?}", msg);
            self.handle_incoming_midi(*port_index, msg);
            // TODO: is this the right place to add a delay?
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}
