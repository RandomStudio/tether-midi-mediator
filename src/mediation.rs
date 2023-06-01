use std::sync::mpsc::{Receiver, Sender};

use log::{debug, warn};
use midi_msg::{Channel, ControlChange, MidiMsg};

use serde::Serialize;
use tether_agent::rmp_serde::to_vec_named;

#[derive(Serialize, Debug)]
// #[serde(rename_all = "camelCase")]
pub struct TetherNoteOnPayload {
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
    NoteOn(TetherNoteOnPayload),
    ControlChange(TetherControlChangePayload),
}

pub struct MediationDataModel {
    pub last_msg_received: String,
    pub midi_rx: Receiver<MidiMsg>,
    pub tether_tx: Sender<TetherMidiMessage>,
}

impl MediationDataModel {
    pub fn new(midi_rx: Receiver<MidiMsg>, tether_tx: Sender<TetherMidiMessage>) -> Self {
        MediationDataModel {
            midi_rx,
            tether_tx,
            last_msg_received: "".to_owned(),
        }
    }

    pub fn handle_incoming_midi(&mut self, msg: &MidiMsg) {
        let raw_message_string = format!("{:?}", msg);
        let raw_payload = to_vec_named(&raw_message_string).expect("failed to encode raw payload");
        self.last_msg_received = raw_message_string.to_owned();
        self.tether_tx
            .send(TetherMidiMessage::Raw(raw_payload))
            .unwrap();

        match msg {
            MidiMsg::ChannelVoice { channel, msg } => {
                debug!("Channel {:?}, msg: {:?}", channel, msg);
                match msg {
                    midi_msg::ChannelVoiceMsg::NoteOn { note, velocity } => {
                        self.tether_tx
                            .send(TetherMidiMessage::NoteOn(TetherNoteOnPayload {
                                channel: channel_to_int(*channel),
                                note: *note,
                                velocity: *velocity,
                            }))
                            .unwrap();
                        debug!("NoteOn {}, @ {}", note, velocity);
                    }
                    midi_msg::ChannelVoiceMsg::NoteOff { note, velocity } => {
                        debug!("NoteOff {}, @ {}", note, velocity);
                    }
                    midi_msg::ChannelVoiceMsg::ControlChange { control } => {
                        debug!("ControlChange message: {:?}", control);
                        match control {
                            ControlChange::Undefined { control, value } => {
                                self.tether_tx
                                    .send(TetherMidiMessage::ControlChange(
                                        TetherControlChangePayload {
                                            channel: channel_to_int(*channel),
                                            controller: *control,
                                            value: *value,
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
