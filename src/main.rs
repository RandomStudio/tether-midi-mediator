use std::{
    error::Error,
    io::stdin,
    sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};

use clap::{command, Parser};
use eframe::egui;
use env_logger::Env;
use log::{debug, error, info, warn};
use midi_msg::{ControlChange, MidiMsg, ReceiverContext};
use midir::{Ignore, MidiInput};
use tether_agent::TetherAgent;

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

    let mut handles = Vec::new();

    let (midi_tx, midi_rx) = mpsc::channel();
    let (tether_tx, tether_rx) = mpsc::channel();

    let mut model = Model::new(midi_rx, tether_tx);

    if cli.tether_disable {
        warn!("Tether connection disabled; local-mode only");
    } else {
        let agent = TetherAgent::new("midi", None, None);
        match agent.connect(None, None) {
            Ok(()) => {
                let tether_thread = std::thread::spawn(move || loop {
                    println!("Checking...");
                    if let Ok(msg) = tether_rx.recv() {
                        debug!("Tether Thread received message via Model: {:?}", msg);
                    }
                    // std::thread::sleep(Duration::from_millis(500));
                });
                handles.push(tether_thread);
            }
            Err(e) => {
                error!("Error connecting Tether Agent: {e}");
                panic!("Could not connect Tether");
            }
        }
    }

    for port in cli.midi_ports {
        let midi_tx = midi_tx.clone();
        let midi_thread = std::thread::spawn(move || match get_midi_input(port, midi_tx) {
            Ok(_) => (),
            Err(err) => error!("MIDI port Error: {}", err),
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
            Box::new(|_cc| Box::<Model>::new(model)),
        )
        .expect("Failed to launch GUI");
        info!("GUI ended; exit now...");
        std::process::exit(0);
    }
}

fn get_midi_input(preferred_port: usize, tx: mpsc::Sender<MidiMsg>) -> Result<(), Box<dyn Error>> {
    let mut input = String::new();

    let mut midi_in = MidiInput::new("midir reading input")?;
    midi_in.ignore(Ignore::None);

    // Get an input port (read from console if multiple are available)
    let in_ports = midi_in.ports();
    let in_port = match in_ports.len() {
        0 => return Err("no input port found".into()),
        1 => {
            warn!(
                "Choosing the only available input port: {}",
                midi_in.port_name(&in_ports[0]).unwrap()
            );
            &in_ports[0]
        }
        _ => {
            debug!("Available input ports:");
            for (i, p) in in_ports.iter().enumerate() {
                info!("{}: {}", i, midi_in.port_name(p).unwrap());
            }
            in_ports.get(preferred_port).ok_or("invalid port")?
        }
    };

    let in_port_name = midi_in.port_name(in_port)?;

    let mut ctx = ReceiverContext::new();
    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _conn_in = midi_in.connect(
        in_port,
        "midir-read-input",
        move |_stamp, midi_bytes, _| {
            let (msg, _len) =
                MidiMsg::from_midi_with_context(&midi_bytes, &mut ctx).expect("Not an error");

            tx.send(msg).expect("failed to send on MIDI thread");
        },
        (),
    )?;

    info!(
        "MIDI connection open, reading input from '{}'.",
        in_port_name
    );

    input.clear();
    stdin().read_line(&mut input)?; // wait for next enter key press

    warn!("Closing connection");
    Ok(())
}

struct Model {
    last_msg_received: String,
    midi_rx: Receiver<MidiMsg>,
    tether_tx: Sender<MidiMsg>,
}

impl Model {
    pub fn new(midi_rx: Receiver<MidiMsg>, tether_tx: Sender<MidiMsg>) -> Self {
        Model {
            midi_rx,
            tether_tx,
            last_msg_received: "".to_owned(),
        }
    }

    pub fn handle_incoming_midi(&mut self, msg: &MidiMsg) {
        self.last_msg_received = format!("{:?}", msg);
        match self.tether_tx.send(msg.clone()) {
            Ok(()) => {}
            Err(e) => {
                error!("tether_tx SendError: {}", e);
            }
        }

        // match msg {
        //     MidiMsg::ChannelVoice { channel, msg } => {
        //         debug!("Channel {:?}, msg: {:?}", channel, msg);
        //         match msg {
        //             midi_msg::ChannelVoiceMsg::NoteOn { note, velocity } => {
        //                 debug!("NoteOn {}, @ {}", note, velocity);
        //             }
        //             midi_msg::ChannelVoiceMsg::NoteOff { note, velocity } => {
        //                 debug!("NoteOff {}, @ {}", note, velocity);
        //             }
        //             midi_msg::ChannelVoiceMsg::ControlChange { control } => {
        //                 debug!("ControlChange message: {:?}", control);
        //                 match control {
        //                     ControlChange::Undefined { control, value } => {
        //                         debug!("'Undefined' control change message: control = {control}, value = {value}");
        //                     }
        //                     _ => {
        //                         warn!("This type of ControlChange message not handled (yet)");
        //                     }
        //                 }
        //             }
        //             _ => {
        //                 warn!("This type of ChannelVoiceMessage not handled (yet)");
        //             }
        //         }
        //     }
        //     _ => {
        //         debug!("unhandled midi message: {:?}", msg);
        //     }
        // }
    }
}

impl eframe::App for Model {
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
