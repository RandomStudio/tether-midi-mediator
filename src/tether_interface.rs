use std::{sync::mpsc::Receiver, thread::JoinHandle};

use log::{debug, error};
use midi_msg::MidiMsg;
use serde::Serialize;
use tether_agent::TetherAgent;

use crate::mediation::TetherMidiMessage;

pub fn start_tether_agent(rx: Receiver<TetherMidiMessage>) -> JoinHandle<()> {
    let agent = TetherAgent::new("midi", None, None);
    match agent.connect(None, None) {
        Ok(()) => {
            let notes_output = agent
                .create_output_plug("notes", None, None)
                .expect("failed to create output plug");
            let cc_output = agent
                .create_output_plug("controlChange", None, None)
                .expect("failed to create output plug");
            let raw_output = agent
                .create_output_plug("raw", None, None)
                .expect("failed to create output plug");
            let tether_thread = std::thread::spawn(move || loop {
                if let Ok(msg) = rx.recv() {
                    // agent.encode_and_publish(plug, data)
                    debug!("Tether Thread received message via Model: {:?}", &msg);
                    match msg {
                        TetherMidiMessage::Raw(payload) => {
                            agent.publish(&raw_output, Some(&payload)).unwrap()
                        }
                        TetherMidiMessage::ControlChange(cc_payload) => {
                            agent.encode_and_publish(&cc_output, cc_payload).unwrap();
                        }
                        TetherMidiMessage::NoteOn(nn_payload) => {
                            agent.encode_and_publish(&notes_output, nn_payload).unwrap();
                        }
                    }
                }
            });
            tether_thread
        }
        Err(e) => {
            error!("Error connecting Tether Agent: {e}");
            panic!("Could not connect Tether");
        }
    }
}
