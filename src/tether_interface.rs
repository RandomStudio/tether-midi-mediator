use std::{
    sync::mpsc::{Receiver, Sender},
    thread::JoinHandle,
};

use log::{debug, error};
use tether_agent::TetherAgent;

use crate::mediation::TetherMidiMessage;

#[derive(Clone)]
pub struct TetherSettings {
    pub host: std::net::IpAddr,
    pub username: Option<String>,
    pub password: Option<String>,
    pub role: String,
    pub id: Option<String>,
}

pub type TetherStateMessage = (bool, TetherSettings, Option<String>);
pub fn start_tether_agent(
    rx: Receiver<TetherMidiMessage>,
    tx: Sender<TetherStateMessage>,
    settings: TetherSettings,
) -> JoinHandle<()> {
    let agent = TetherAgent::new(
        &settings.role,
        if let Some(override_id) = &settings.id {
            Some(override_id)
        } else {
            None
        },
        Some(settings.host),
    );
    match agent.connect(None, None) {
        Ok(()) => {
            tx.send((
                agent.is_connected(),
                settings,
                Some(String::from(agent.broker_uri())),
            ))
            .expect("failed to send state");

            let note_on_output = agent
                .create_output_plug("notesOn", None, None)
                .expect("failed to create output plug");
            let note_off_output = agent
                .create_output_plug("notesOff", None, None)
                .expect("failed to create output plug");
            let cc_output = agent
                .create_output_plug("controlChange", None, None)
                .expect("failed to create output plug");
            let raw_output = agent
                .create_output_plug("raw", None, None)
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
                    }
                }
            })
        }
        Err(e) => {
            tx.send((false, settings, None))
                .expect("failed to send state");

            error!("Error connecting Tether Agent: {e}");
            panic!("Could not connect Tether");
        }
    }
}
