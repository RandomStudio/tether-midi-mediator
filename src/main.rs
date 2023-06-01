use std::{
    error::Error,
    io::{stdin, stdout, Write},
    sync::mpsc::{self, Receiver},
    time::Duration,
};

use clap::{command, Parser};
use eframe::egui;
use env_logger::Env;
use log::{debug, error, info, warn};
use midi_msg::{ControlChange, MidiMsg, ReceiverContext, SystemRealTimeMsg};
use midir::{Ignore, MidiInput};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(long = "loglevel",default_value_t=String::from("info"))]
    log_level: String,

    #[arg(long = "headless")]
    headless_mode: bool,

    #[arg(long = "midi.port", default_value_t = 0)]
    midi_port: usize,
}

fn main() {
    let cli = Cli::parse();
    env_logger::Builder::from_env(Env::default().default_filter_or(&cli.log_level)).init();

    let mut handles = Vec::new();

    let (tx, rx) = mpsc::channel();

    let midi_tx = tx.clone();
    let midi_thread = std::thread::spawn(move || match get_midi_input(cli.midi_port, midi_tx) {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err),
    });
    handles.push(midi_thread);

    if cli.headless_mode {
        info!("Running in headless mode; no graphic output");
        for received in rx {
            println!("Received MIDI message: {:?}", received);
            // std::thread::sleep(Duration::from_millis(1));
        }
    } else {
        info!("Running graphics mode");
        let options = eframe::NativeOptions {
            // initial_window_size: Some(egui::vec2(320.0, 240.0)),
            ..Default::default()
        };
        eframe::run_native(
            "My egui App",
            options,
            Box::new(|_cc| Box::<Model>::new(Model::new(rx))),
        )
        .expect("Failed to launch GUI");
    }

    for handle in handles {
        handle.join().expect("failed to join thread handle");
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
            info!(
                "Choosing the only available input port: {}",
                midi_in.port_name(&in_ports[0]).unwrap()
            );
            &in_ports[0]
        }
        _ => {
            debug!("\nAvailable input ports:");
            for (i, p) in in_ports.iter().enumerate() {
                println!("{}: {}", i, midi_in.port_name(p).unwrap());
            }
            in_ports.get(preferred_port).ok_or("invalid port")?
            // print!("Please select input port: ");
            // stdout().flush()?;
            // let mut input = String::new();
            // stdin().read_line(&mut input)?;
            // in_ports
            //     .get(input.trim().parse::<usize>()?)
            //     .ok_or("invalid input port selected")?
        }
    };

    println!("\nOpening connection");
    let in_port_name = midi_in.port_name(in_port)?;

    let mut ctx = ReceiverContext::new();
    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _conn_in = midi_in.connect(
        in_port,
        "midir-read-input",
        move |stamp, midi_bytes, _| {
            let (msg, _len) =
                MidiMsg::from_midi_with_context(&midi_bytes, &mut ctx).expect("Not an error");

            tx.send(msg);

            // // Print everything but spammy clock messages.
            // if let MidiMsg::SystemRealTime {
            //     msg: SystemRealTimeMsg::TimingClock,
            // } = msg
            // {
            //     // no-op
            // } else {
            //     println!("{}: {:?}", stamp, msg);
            // }

            // match msg {
            //     MidiMsg::ChannelVoice { channel, msg } => {
            //         println!("Channel {:?}, msg: {:?}", channel, msg);
            //         match msg {
            //             midi_msg::ChannelVoiceMsg::NoteOn { note, velocity } => {
            //                 println!("NoteOn {}, @ {}", note, velocity);
            //             }
            //             midi_msg::ChannelVoiceMsg::NoteOff { note, velocity } => {
            //                 println!("NoteOff {}, @ {}", note, velocity);
            //             }
            //             midi_msg::ChannelVoiceMsg::ControlChange { control } => {
            //                 println!("ControlChange message: {:?}", control);
            //                 match control {
            //                     ControlChange::Undefined { control, value } => {
            //                         println!("'Undefined' control change message: control = {control}, value = {value}");
            //                     },
            //                     _ => {
            //                         warn!("This type of ControlChange message not handled (yet)")
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
        },
        (),
    )?;

    println!(
        "Connection open, reading input from '{}' (press enter to exit) ...",
        in_port_name
    );

    input.clear();
    stdin().read_line(&mut input)?; // wait for next enter key press

    println!("Closing connection");
    Ok(())
}

struct Model {
    last_msg_received: String,
    rx: Receiver<MidiMsg>,
}

impl Model {
    pub fn new(rx: Receiver<MidiMsg>) -> Self {
        Model {
            rx,
            last_msg_received: "".to_owned(),
        }
    }
}

impl eframe::App for Model {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Tether Midi Mediator");
            ui.label("Last message received:");
            ui.small(&self.last_msg_received);
        });

        if let Ok(msg) = &self.rx.try_recv() {
            println!("GUI received MIDI message: {:?}", msg);
            self.last_msg_received = format!("{:?}", msg);
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}
