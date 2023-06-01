use std::{
    sync::mpsc::{self},
    time::Duration,
};

use clap::{command, Parser};
use eframe::egui;
use env_logger::Env;
use log::{debug, info, warn};
use mediation::MediationDataModel;
use midi_interface::listen_for_midi;
use tether_interface::start_tether_agent;

mod mediation;
mod midi_interface;
mod tether_interface;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(long = "loglevel",default_value_t=String::from("info"))]
    log_level: String,

    /// Flag to enable headless (no GUI) mode, suitable for server-type
    /// process
    #[arg(long = "headless")]
    headless_mode: bool,

    /// Flag to disable Tether connection
    #[arg(long = "tether.disable")]
    tether_disable: bool,

    /// Specify one or more MIDI ports by index, in any order
    #[clap()]
    midi_ports: Vec<usize>,
}

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
        handles.push(start_tether_agent(tether_rx));
    }

    for port in cli.midi_ports {
        let midi_tx = midi_tx.clone();
        let midi_thread = std::thread::spawn(move || {
            listen_for_midi(port, midi_tx);
        });
        handles.push(midi_thread);
    }

    if cli.headless_mode {
        info!("Running in headless mode; Ctrl+C to quit");
        loop {
            while let Ok(msg) = &model.midi_rx.try_recv() {
                model.handle_incoming_midi(msg);
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
            "My egui App",
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
            ui.heading("Tether Midi Mediator");
            ui.label("Last message received:");
            ui.small(&self.last_msg_received);
        });

        if let Ok(msg) = &self.midi_rx.try_recv() {
            debug!("GUI received MIDI message: {:?}", msg);
            self.handle_incoming_midi(msg);
            // TODO: is this the right place to add a delay?
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}
