use std::{sync::mpsc, time::Duration};

use clap::Parser;
use eframe::egui;
use env_logger::Env;
use gui::render_gui;
use log::{debug, info, warn};
use mediation::MediationDataModel;
use midi_interface::{get_midi_connection, midi_listener_thread};
use midir::{Ignore, MidiInput};
use settings::Cli;
use tether_interface::{start_tether_agent, TetherSettings};

mod gui;
mod mediation;
mod midi_interface;
mod settings;
mod tether_interface;

fn list_midi_ports() -> anyhow::Result<()> {
    let mut midi_input = MidiInput::new("midir reading input").expect("midir failure");
    midi_input.ignore(Ignore::None);

    for (i, p) in midi_input.ports().iter().enumerate() {
        println!("{}: {}", i, midi_input.port_name(p)?);
    }
    Ok(())
}
fn main() {
    let cli = Cli::parse();

    list_midi_ports().expect("failed to list MIDI ports");

    if cli.midi_ports.is_empty() {
        panic!("You must provide at least one MIDI port index(es), e.g. \"./tether-midi-mediator 1 2\"")
    }

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
    let (tether_state_tx, tether_state_rx) = mpsc::channel();

    let tether_settings = TetherSettings {
        host: cli.tether_host,
        username: cli.tether_username,
        password: cli.tether_password,
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
        cli.relative_mode_enabled,
    );

    for port in cli.midi_ports {
        let mut midi_input = MidiInput::new("midir reading input").expect("midir failure");
        midi_input.ignore(Ignore::None);

        let midi_tx = midi_tx.clone();
        let (midi_input_port, port_name) =
            get_midi_connection(&midi_input, port).expect("failed to open MIDI port");
        model.add_port(port, port_name);
        handles.push(midi_listener_thread(
            midi_input,
            midi_input_port,
            midi_tx,
            port,
        ));
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
            render_gui(self, ui);
        });

        if let Ok((is_connected, _settings_used, broker_uri)) = &self.tether_state_rx.try_recv() {
            self.tether_connected = *is_connected;
            self.tether_uri = broker_uri.clone();
        }

        if let Ok((port_index, msg)) = &self.midi_rx.try_recv() {
            debug!("GUI received MIDI message: {:?}", msg);
            self.handle_incoming_midi(*port_index, msg);
            // TODO: is this the right place to add a delay?
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}
