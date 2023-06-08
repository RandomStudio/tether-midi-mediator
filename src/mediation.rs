use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender},
    time::SystemTime,
};

use circular_buffer::CircularBuffer;
use log::{debug, warn};
use midi_msg::{Channel, ControlChange, MidiMsg};

use serde::Serialize;
use tether_agent::rmp_serde::to_vec_named;

use crate::tether_interface::TetherStateMessage;

#[derive(Serialize, Debug)]
pub struct TetherNotePayload {
    pub channel: u8,
    pub note: u8,
    pub velocity: u8,
}

#[derive(Serialize, Debug)]
pub struct TetherControlChangePayload {
    pub channel: u8,
    pub controller: u8,
    pub value: u8,
}

#[derive(Debug)]
pub enum TetherMidiMessage {
    /// Already-encoded payload
    Raw(Vec<u8>),
    NoteOn(TetherNotePayload),
    NoteOff(TetherNotePayload),
    ControlChange(TetherControlChangePayload),
}

pub struct PortInformation {
    pub index: usize,
    pub full_name: String,
    pub last_received: SystemTime,
}

/// Port index, MIDI Message
pub type MidiReceiverPayload = (usize, MidiMsg);

/// Value, time
pub type LastControllerValue = u8;

pub const MONITOR_LOG_LENGTH: usize = 8;
pub struct MediationDataModel {
    pub message_log: CircularBuffer<MONITOR_LOG_LENGTH, String>,
    pub midi_rx: Receiver<MidiReceiverPayload>,
    pub tether_tx: Sender<TetherMidiMessage>,
    pub port_info: HashMap<String, PortInformation>,
    pub tether_connected: bool,
    pub tether_uri: Option<String>,
    pub tether_state_rx: Receiver<TetherStateMessage>,
    pub controller_relative_mode: bool,
    pub known_controller_values: HashMap<String, LastControllerValue>,
}

impl MediationDataModel {
    pub fn new(
        midi_rx: Receiver<MidiReceiverPayload>,
        tether_tx: Sender<TetherMidiMessage>,
        tether_state_rx: Receiver<TetherStateMessage>,
        controller_relative_mode: bool,
    ) -> Self {
        MediationDataModel {
            midi_rx,
            tether_tx,
            message_log: CircularBuffer::new(),
            port_info: HashMap::new(),
            tether_state_rx,
            tether_connected: false,
            tether_uri: None,
            controller_relative_mode,
            known_controller_values: HashMap::new(),
        }
    }

    pub fn add_port(&mut self, index: usize, full_name: String) {
        // let shortened_name = full_name.replace(" ", "_").trim().to_lowercase();
        let port_key = format!("{index}");
        // let full_name = String::from("unknown");
        self.port_info.insert(
            port_key,
            PortInformation {
                index,
                full_name,
                last_received: SystemTime::now(),
            },
        );
    }

    pub fn handle_incoming_midi(&mut self, port_index: usize, msg: &MidiMsg) {
        let raw_message_string = format!("{:?}", msg);
        let raw_payload = to_vec_named(&raw_message_string).expect("failed to encode raw payload");
        self.message_log.push_back(raw_message_string);
        self.tether_tx
            .send(TetherMidiMessage::Raw(raw_payload))
            .unwrap();

        match msg {
            MidiMsg::ChannelVoice { channel, msg } => {
                debug!("Channel {:?}, msg: {:?}", channel, msg);
                match msg {
                    midi_msg::ChannelVoiceMsg::NoteOn { note, velocity } => {
                        self.tether_tx
                            .send(TetherMidiMessage::NoteOn(TetherNotePayload {
                                channel: channel_to_int(*channel),
                                note: *note,
                                velocity: *velocity,
                            }))
                            .unwrap();
                        debug!("NoteOn {}, @ {}", note, velocity);
                    }
                    midi_msg::ChannelVoiceMsg::NoteOff { note, velocity } => {
                        self.tether_tx
                            .send(TetherMidiMessage::NoteOff(TetherNotePayload {
                                channel: channel_to_int(*channel),
                                note: *note,
                                velocity: *velocity,
                            }))
                            .unwrap();
                        debug!("NoteOff {}, @ {}", note, velocity);
                    }
                    midi_msg::ChannelVoiceMsg::ControlChange { control } => {
                        debug!("ControlChange message: {:?}", control);
                        match control {
                            ControlChange::Undefined { control, value } => {
                                let value = if self.controller_relative_mode {
                                    let key = format!("control");
                                    if let Some(prev_value) = self.known_controller_values.get(&key)
                                    {
                                        let increment: i16 = if *value < 64 {
                                            *value as i16
                                        } else {
                                            (*value as i16) - 128
                                        };
                                        let absolute_value: u8 = ((*prev_value as i16) + increment)
                                            .clamp(0, 127)
                                            .try_into()
                                            .expect("integer conversion failed");
                                        self.known_controller_values
                                            .insert(key, absolute_value as u8);
                                        absolute_value
                                    } else {
                                        self.known_controller_values.insert(key, *value);
                                        *value
                                    }
                                    // self.known_controller_values.insert(format!("{}", control), ())
                                } else {
                                    *value
                                };
                                self.tether_tx
                                    .send(TetherMidiMessage::ControlChange(
                                        TetherControlChangePayload {
                                            channel: channel_to_int(*channel),
                                            controller: *control,
                                            value,
                                        },
                                    ))
                                    .unwrap();
                                debug!("'Undefined' control change message: control = {control}, value = {value}");
                            }
                            _ => {
                                warn!("This type of ControlChange message not handled (yet)");
                            }
                        }
                    }
                    _ => {
                        warn!("This type of ChannelVoiceMessage not handled (yet)");
                    }
                }
            }
            _ => {
                debug!("unhandled midi message: {:?}", msg);
            }
        }
        self.update_port_info(port_index);
    }

    fn update_port_info(&mut self, index: usize) {
        for (key, info) in self.port_info.iter_mut() {
            if key.eq(&format!("{index}")) {
                info.last_received = SystemTime::now();
            }
        }
    }
}

fn channel_to_int(channel: Channel) -> u8 {
    match channel {
        Channel::Ch1 => 1,
        Channel::Ch2 => 2,
        Channel::Ch3 => 3,
        Channel::Ch4 => 4,
        Channel::Ch5 => 5,
        Channel::Ch6 => 6,
        Channel::Ch7 => 7,
        Channel::Ch8 => 8,
        Channel::Ch9 => 9,
        Channel::Ch10 => 10,
        Channel::Ch11 => 11,
        Channel::Ch12 => 12,
        Channel::Ch13 => 13,
        Channel::Ch14 => 14,
        Channel::Ch15 => 15,
        Channel::Ch16 => 16,
    }
}
