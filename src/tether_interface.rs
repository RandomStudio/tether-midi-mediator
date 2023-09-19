use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender},
    thread::JoinHandle,
    time::Duration,
};

use log::{debug, error};
use tether_agent::{PlugDefinition, PlugOptionsBuilder, TetherAgent, TetherAgentOptionsBuilder};

use crate::{
    mediation::{TetherControlChangePayload, TetherMidiMessage},
    settings::Cli,
};

pub struct TetherInterface {}

/// isConnected, URI if connected
pub type TetherStateMessage = (bool, Option<String>);

impl TetherInterface {
    pub fn new(
        settings: &Cli,
        rx: Receiver<TetherMidiMessage>,
        tx: Sender<TetherStateMessage>,
    ) -> Self {
        let agent = TetherAgentOptionsBuilder::new(&settings.tether_role)
            .id_optional(settings.tether_id.clone())
            .host_optional(settings.tether_host.clone())
            .username_optional(settings.tether_username.clone())
            .password_optional(settings.tether_password.clone())
            .build()
            .expect("failed to connect Tether");

        tx.send((agent.is_connected(), Some(String::from(agent.broker_uri()))))
            .expect("failed to send state");

        let note_on_output = PlugOptionsBuilder::create_output("notesOn")
            .qos(2)
            .build(&agent)
            .expect("failed to create output plug");
        let note_off_output = PlugOptionsBuilder::create_output("notesOff")
            .qos(2)
            .build(&agent)
            .expect("failed to create output plug");
        let cc_output = PlugOptionsBuilder::create_output("controlChange")
            .qos(1)
            .build(&agent)
            .expect("failed to create output plug");
        let raw_output = PlugOptionsBuilder::create_output("raw")
            .qos(0)
            .build(&agent)
            .expect("failed to create output plug");

        let input_plugs: HashMap<String, PlugDefinition> = HashMap::from([
            (
                "notesOn".into(),
                PlugOptionsBuilder::create_input("notesOn")
                    .build(&agent)
                    .unwrap(),
            ),
            (
                "notesOff".into(),
                PlugOptionsBuilder::create_input("notesOff")
                    .build(&agent)
                    .unwrap(),
            ),
            (
                "controlChange".into(),
                PlugOptionsBuilder::create_input("controlChange")
                    .build(&agent)
                    .unwrap(),
            ),
        ]);

        std::thread::spawn(move || loop {
            let mut work_done = false;

            while let Some((plug_name, message)) = agent.check_messages() {
                work_done = true;
                debug!("************************* Tether Thread received message via Tether");

                if let Some(matched_plug) = input_plugs.get(&plug_name) {
                    match matched_plug.name() {
                        "notesOn" => todo!(),
                        "notesOff" => todo!(),
                        "controlChange" => {
                            let control_change_message =
                                rmp_serde::from_slice::<TetherControlChangePayload>(
                                    message.payload(),
                                );
                            if let Ok(parsed) = control_change_message {
                                debug!("Incoming Control Change Message: {:?}", parsed);
                            }
                        }
                        &_ => {
                            error!("failed to match plug name")
                        }
                    }
                }
            }

            while let Ok(msg) = rx.recv() {
                work_done = true;
                debug!("Tether Thread received message via Model: {:?}", &msg);
                match msg {
                    TetherMidiMessage::Raw(payload) => {
                        agent.publish(&raw_output, Some(&payload)).unwrap()
                    }
                    TetherMidiMessage::ControlChange(cc_payload) => {
                        agent.encode_and_publish(&cc_output, cc_payload).unwrap()
                    }
                    TetherMidiMessage::NoteOn(n_payload) => agent
                        .encode_and_publish(&note_on_output, n_payload)
                        .unwrap(),
                    TetherMidiMessage::NoteOff(n_payload) => agent
                        .encode_and_publish(&note_off_output, n_payload)
                        .unwrap(),
                }
            }
            if !work_done {
                std::thread::sleep(Duration::from_millis(1));
            }
        });

        TetherInterface {}
    }
}

// pub fn start_tether_agent(
//     rx: Receiver<TetherMidiMessage>,
//     tx: Sender<TetherStateMessage>,
// ) ->  {
//     let agent = TetherAgentOptionsBuilder::new(&settings.role)
//         .id_optional(settings.id)
//         .build()
//         .expect("failed to connect Tether");

//     tx.send((
//         agent.is_connected(),
//         settings,
//         Some(String::from(agent.broker_uri())),
//     ))
//     .expect("failed to send state");

//     let note_on_output = PlugOptionsBuilder::create_output("notesOn")
//         .qos(2)
//         .build(&agent)
//         .expect("failed to create output plug");
//     let note_off_output = PlugOptionsBuilder::create_output("notesOff")
//         .qos(2)
//         .build(&agent)
//         .expect("failed to create output plug");
//     let cc_output = PlugOptionsBuilder::create_output("controlChange")
//         .qos(1)
//         .build(&agent)
//         .expect("failed to create output plug");
//     let raw_output = PlugOptionsBuilder::create_output("raw")
//         .qos(0)
//         .build(&agent)
//         .expect("failed to create output plug");

//     std::thread::spawn(move || loop {
//         if let Ok(msg) = rx.recv() {
//             debug!("Tether Thread received message via Model: {:?}", &msg);
//             match msg {
//                 TetherMidiMessage::Raw(payload) => {
//                     agent.publish(&raw_output, Some(&payload)).unwrap()
//                 }
//                 TetherMidiMessage::ControlChange(cc_payload) => {
//                     agent.encode_and_publish(&cc_output, cc_payload).unwrap();
//                 }
//                 TetherMidiMessage::NoteOn(n_payload) => {
//                     agent
//                         .encode_and_publish(&note_on_output, n_payload)
//                         .unwrap();
//                 }
//                 TetherMidiMessage::NoteOff(n_payload) => {
//                     agent
//                         .encode_and_publish(&note_off_output, n_payload)
//                         .unwrap();
//                 }
//             }
//         } else {
//             std::thread::sleep(Duration::from_millis(1));
//         }
//     })
// }
