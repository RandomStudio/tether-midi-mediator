use std::{
    sync::mpsc::{self},
    time::Duration,
};

use clap::Parser;
use eframe::egui;
use egui::{Color32, RichText};
use env_logger::Env;
use log::{debug, info, warn};
use mediation::MediationDataModel;
use midi_interface::listen_for_midi;
use midi_msg::{MidiMsg, ReceiverContext, SystemRealTimeMsg};
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
        let midi_tx = midi_tx.clone();
        let (midi_system, midi_input_port, port_name) =
            listen_for_midi(port).expect("failed to open MIDI port");
        model.add_port(port, port_name);
        let midi_thread = std::thread::spawn(move || {
            let mut ctx = ReceiverContext::new();

            let _connection = midi_system
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
                model.handle_incoming_midi(*port_index, msg);
                debug!("Last received message: {}", &model.last_msg_received);
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
            ui.heading("Last message received:");
            ui.label(&self.last_msg_received);

            ui.separator();

            ui.heading("MIDI Ports Connected");

            for (_key, info) in self.port_info.iter() {
                ui.horizontal(|ui| {
                    ui.label(&format!("{} :{}", info.index, info.full_name));
                    if let Ok(elapsed) = info.last_received.elapsed() {
                        let color = if elapsed > Duration::from_secs(3) {
                            Color32::RED
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
        });

        if let Ok((port_index, msg)) = &self.midi_rx.try_recv() {
            debug!("GUI received MIDI message: {:?}", msg);
            self.handle_incoming_midi(*port_index, msg);
            // TODO: is this the right place to add a delay?
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}
