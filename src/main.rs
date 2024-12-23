use std::{sync::mpsc, time::Duration};

use clap::Parser;
use eframe::egui;
use egui::Vec2;
use env_logger::Env;
use gui::render_gui;
use log::{debug, info, warn};
use mediation::{ControllerValueMode, MediationDataModel};
use midi_interface::{get_midi_connection, midi_listener_thread};
use midir::{Ignore, MidiInput};
use settings::Cli;
use tether_interface::{start_tether_agent, TetherSettings};

mod gui;
mod mediation;
mod midi_interface;
mod settings;
mod tether_interface;

fn list_midi_ports() -> anyhow::Result<Vec<usize>> {
    let mut midi_input = MidiInput::new("midir reading input").expect("midir failure");
    midi_input.ignore(Ignore::None);

    let mut port_indexes = Vec::new();
    for (i, p) in midi_input.ports().iter().enumerate() {
        info!(
            "Available MIDI port: #{} = {}",
            i,
            midi_input
                .port_name(p)
                .expect("failed to retrieve port name")
        );
        port_indexes.push(i);
    }
    Ok(port_indexes)
}
fn main() {
    let cli = Cli::parse();

    env_logger::Builder::from_env(Env::default().default_filter_or(&cli.log_level))
        .filter_module("rumqttc", log::LevelFilter::Warn)
        .filter_module("tether_agent", log::LevelFilter::Warn)
        .init();
    let available_port_indexes = list_midi_ports().expect("failed to list MIDI ports");

    let listen_ports = if cli.midi_ports.is_empty() {
        warn!("No ports specified; will listen on all available MIDI Inputs");
        available_port_indexes
    } else {
        info!("MIDI ports specified: {:?}", cli.midi_ports);
        cli.midi_ports
    };

    // TODO: we don't really use these handles or join them
    // Might be useful for closing things down properly, though
    let mut handles = Vec::new();

    let (midi_tx, midi_rx) = mpsc::channel();
    let (tether_tx, tether_rx) = mpsc::channel();
    let (tether_state_tx, tether_state_rx) = mpsc::channel();

    let tether_settings = TetherSettings {
        host: cli.tether_host,
        role: cli.tether_role,
        id: cli.tether_id,
    };

    if cli.tether_disable {
        warn!("Tether connection disabled; local-mode only");
    } else {
        handles.push(start_tether_agent(
            tether_rx,
            tether_state_tx,
            tether_settings,
        ));
    }

    let mut model = MediationDataModel::new(
        midi_rx,
        tether_tx,
        tether_state_rx,
        if cli.relative_mode_enabled {
            ControllerValueMode::Relative
        } else {
            ControllerValueMode::Absolute
        },
    );

    for port in listen_ports {
        let mut midi_input = MidiInput::new("midir reading input").expect("midir failure");
        midi_input.ignore(Ignore::None);

        let midi_tx = midi_tx.clone();
        let (midi_input_port, port_name) =
            get_midi_connection(&midi_input, port).expect("failed to open MIDI port");
        model.add_port(port, port_name.clone());
        if !cli.knobs_disable {
            match model.add_knob_mapping(&port_name) {
                Ok(_) => info!(
                    "Added automatic knob mapping for device \"{}\" OK",
                    &port_name
                ),
                Err(_) => warn!("Could not find mapping for device \"{}\"", &port_name),
            }
        }
        handles.push(midi_listener_thread(
            midi_input,
            midi_input_port,
            midi_tx,
            port,
        ));
    }

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
            viewport: egui::ViewportBuilder::default().with_inner_size(Vec2::new(1280., 500.)),
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
            render_gui(self, ui);
        });

        if let Ok((is_connected, _settings_used, broker_uri)) = &self.tether_state_rx.try_recv() {
            self.tether_connected = *is_connected;
            self.tether_uri = broker_uri.clone();
        }

        while let Ok((port_index, msg)) = &self.midi_rx.try_recv() {
            debug!("GUI received MIDI message: {:?}", msg);
            self.handle_incoming_midi(*port_index, msg);
            // std::thread::sleep(Duration::from_millis(1));
        }
    }
}
