use std::{
    sync::mpsc::{Receiver, Sender},
    thread::JoinHandle,
    time::Duration,
};

use log::debug;
use tether_agent::{PlugOptionsBuilder, TetherAgentOptionsBuilder};

use crate::mediation::messages::TetherMidiMessage;

#[derive(Clone)]
pub struct TetherSettings {
    pub host: String,
    pub role: String,
    pub id: Option<String>,
}

pub type TetherStateMessage = (bool, TetherSettings, Option<String>);
pub fn start_tether_agent(
    rx: Receiver<TetherMidiMessage>,
    tx: Sender<TetherStateMessage>,
    settings: TetherSettings,
) -> JoinHandle<()> {
    let mut agent = TetherAgentOptionsBuilder::new(&settings.role)
        .host(Some(&settings.host))
        .id(settings.id.as_deref())
        .build()
        .expect("failed to set up Tether Agent");
    tx.send((agent.is_connected(), settings, Some(agent.broker_uri())))
        .expect("failed to send state");

    let note_on_output = PlugOptionsBuilder::create_output("notesOn")
        .qos(Some(1))
        .build(&mut agent)
        .expect("failed to create output plug");
    let note_off_output = PlugOptionsBuilder::create_output("notesOff")
        .qos(Some(1))
        .build(&mut agent)
        .expect("failed to create output plug");
    let cc_output = PlugOptionsBuilder::create_output("controlChange")
        .qos(Some(0))
        .build(&mut agent)
        .expect("failed to create output plug");
    let knob_output = PlugOptionsBuilder::create_output("knobs")
        .qos(Some(0))
        .build(&mut agent)
        .expect("failed to create output plug");
    let raw_output = PlugOptionsBuilder::create_output("raw")
        .qos(Some(0))
        .build(&mut agent)
        .expect("failed to create output plug");
    std::thread::spawn(move || loop {
        if let Ok(msg) = rx.recv() {
            debug!("Tether Thread received message via Model: {:?}", &msg);
            match msg {
                TetherMidiMessage::Raw(payload) => {
                    agent.publish(&raw_output, Some(&payload)).unwrap()
                }
                TetherMidiMessage::ControlChange(cc_payload) => {
                    agent.encode_and_publish(&cc_output, cc_payload).unwrap();
                }
                TetherMidiMessage::NoteOn(n_payload) => {
                    agent
                        .encode_and_publish(&note_on_output, n_payload)
                        .unwrap();
                }
                TetherMidiMessage::NoteOff(n_payload) => {
                    agent
                        .encode_and_publish(&note_off_output, n_payload)
                        .unwrap();
                }
                TetherMidiMessage::Knob(k_payload) => {
                    agent.encode_and_publish(&knob_output, &k_payload).unwrap();
                }
            }
        } else {
            std::thread::sleep(Duration::from_millis(1));
        }
    })
}
