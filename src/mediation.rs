use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender},
    time::SystemTime,
};

use circular_buffer::CircularBuffer;
use log::{debug, error, warn};
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
pub enum ControllerLabel {
    Numbered(u8),
    Special(String),
}

#[derive(Serialize, Debug)]
pub enum MidiValue {
    LowRes(u8),
    HiRes(u16),
}

#[derive(Serialize, Debug)]
pub struct TetherControlChangePayload {
    pub channel: u8,
    pub controller: ControllerLabel,
    pub value: MidiValue,
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

#[derive(PartialEq, Debug)]
pub enum ControllerValueMode {
    Absolute,
    Relative,
}

pub const MONITOR_LOG_LENGTH: usize = 16;
// pub const MAX_HIRES: u16 = 2u16.pow(13) - 128;
// pub const HALF_HIRES: u16 = MAX_HIRES / 2;

pub struct MediationDataModel {
    pub midi_message_log: CircularBuffer<MONITOR_LOG_LENGTH, String>,
    pub tether_message_log: CircularBuffer<MONITOR_LOG_LENGTH, String>,
    pub midi_rx: Receiver<MidiReceiverPayload>,
    pub tether_tx: Sender<TetherMidiMessage>,
    pub ports_metadata: HashMap<String, PortInformation>,
    pub tether_connected: bool,
    pub tether_uri: Option<String>,
    pub tether_state_rx: Receiver<TetherStateMessage>,
    pub controller_mode: ControllerValueMode,
    pub known_controller_values: HashMap<String, MidiValue>,
}

impl MediationDataModel {
    pub fn new(
        midi_rx: Receiver<MidiReceiverPayload>,
        tether_tx: Sender<TetherMidiMessage>,
        tether_state_rx: Receiver<TetherStateMessage>,
        controller_mode: ControllerValueMode,
    ) -> Self {
        MediationDataModel {
            midi_rx,
            tether_tx,
            midi_message_log: CircularBuffer::new(),
            tether_message_log: CircularBuffer::new(),
            ports_metadata: HashMap::new(),
            tether_state_rx,
            tether_connected: false,
            tether_uri: None,
            controller_mode,
            known_controller_values: HashMap::new(),
        }
    }

    pub fn add_port(&mut self, index: usize, full_name: String) {
        // let shortened_name = full_name.replace(" ", "_").trim().to_lowercase();
        let port_key = format!("{index}");
        // let full_name = String::from("unknown");
        self.ports_metadata.insert(
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
        self.midi_message_log.push_back(raw_message_string);
        self.tether_tx
            .send(TetherMidiMessage::Raw(raw_payload))
            .unwrap();

        match msg {
            MidiMsg::ChannelVoice { channel, msg } => {
                debug!("Channel {:?}, msg: {:?}", channel, msg);
                match msg {
                    midi_msg::ChannelVoiceMsg::NoteOn { note, velocity } => {
                        let out_msg = TetherNotePayload {
                            channel: channel_to_int(*channel),
                            note: *note,
                            velocity: *velocity,
                        };
                        self.tether_message_log.push_back(format!("{:?}", out_msg));
                        self.tether_tx
                            .send(TetherMidiMessage::NoteOn(out_msg))
                            .unwrap();
                        debug!("NoteOn {}, @ {}", note, velocity);
                    }
                    midi_msg::ChannelVoiceMsg::NoteOff { note, velocity } => {
                        let out_msg = TetherNotePayload {
                            channel: channel_to_int(*channel),
                            note: *note,
                            velocity: *velocity,
                        };
                        self.tether_message_log.push_back(format!("{:?}", out_msg));
                        self.tether_tx
                            .send(TetherMidiMessage::NoteOff(out_msg))
                            .unwrap();
                        debug!("NoteOff {}, @ {}", note, velocity);
                    }
                    midi_msg::ChannelVoiceMsg::ControlChange { control } => {
                        debug!("ControlChange message: {:?}", control);
                        match control {
                            ControlChange::Undefined { control, value } => {
                                self.send_control_change(
                                    ControllerLabel::Numbered(*control),
                                    MidiValue::LowRes(*value),
                                    channel,
                                );
                            }
                            ControlChange::ModWheel(value) => self.send_control_change(
                                ControllerLabel::Special("ModWheel".into()),
                                MidiValue::HiRes(*value),
                                channel,
                            ),
                            ControlChange::Breath(value) => self.send_control_change(
                                ControllerLabel::Special("Breath".into()),
                                MidiValue::HiRes(*value),
                                channel,
                            ),
                            ControlChange::UndefinedHighRes {
                                control1,
                                control2,
                                value,
                            } => self.send_control_change(
                                ControllerLabel::Special(format!(
                                    "UndefinedHighRes-{}-{}",
                                    control1, control2
                                )),
                                MidiValue::HiRes(*value),
                                channel,
                            ),
                            ControlChange::Foot(value) => self.send_control_change(
                                ControllerLabel::Special("Foot".into()),
                                MidiValue::HiRes(*value),
                                channel,
                            ),
                            ControlChange::Portamento(value) => self.send_control_change(
                                ControllerLabel::Special("Portamento".into()),
                                MidiValue::HiRes(*value),
                                channel,
                            ),
                            ControlChange::DataEntry(value) => self.send_control_change(
                                ControllerLabel::Special("DataEntry".into()),
                                MidiValue::HiRes(*value),
                                channel,
                            ),
                            ControlChange::Volume(value) => self.send_control_change(
                                ControllerLabel::Special("Volume".into()),
                                MidiValue::HiRes(*value),
                                channel,
                            ),
                            ControlChange::Balance(value) => self.send_control_change(
                                ControllerLabel::Special("Balance".into()),
                                MidiValue::HiRes(*value),
                                channel,
                            ),
                            ControlChange::Pan(value) => self.send_control_change(
                                ControllerLabel::Special("Pan".into()),
                                MidiValue::HiRes(*value),
                                channel,
                            ),
                            ControlChange::Expression(value) => self.send_control_change(
                                ControllerLabel::Special("Expression".into()),
                                MidiValue::HiRes(*value),
                                channel,
                            ),
                            ControlChange::Effect1(value) => self.send_control_change(
                                ControllerLabel::Special("Effect1".into()),
                                MidiValue::HiRes(*value),
                                channel,
                            ),
                            ControlChange::Effect2(value) => self.send_control_change(
                                ControllerLabel::Special("Effect2".into()),
                                MidiValue::HiRes(*value),
                                channel,
                            ),
                            ControlChange::GeneralPurpose1(value) => self.send_control_change(
                                ControllerLabel::Special("GeneralPurpose1".into()),
                                MidiValue::HiRes(*value),
                                channel,
                            ),
                            ControlChange::GeneralPurpose2(value) => self.send_control_change(
                                ControllerLabel::Special("GeneralPurpose2".into()),
                                MidiValue::HiRes(*value),
                                channel,
                            ),
                            ControlChange::GeneralPurpose3(value) => self.send_control_change(
                                ControllerLabel::Special("GeneralPurpose3".into()),
                                MidiValue::HiRes(*value),
                                channel,
                            ),
                            ControlChange::GeneralPurpose4(value) => self.send_control_change(
                                ControllerLabel::Special("GeneralPurpose4".into()),
                                MidiValue::HiRes(*value),
                                channel,
                            ),

                            // -------------------------------------------------------------
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

    fn send_control_change(
        &mut self,
        control_label: ControllerLabel,
        value: MidiValue,
        channel: &Channel,
    ) {
        let send_absolute_value: MidiValue = match self.controller_mode {
            ControllerValueMode::Relative => {
                let key = match &control_label {
                    ControllerLabel::Numbered(value) => value.to_string(),
                    ControllerLabel::Special(label) => label.clone(),
                };
                match value {
                    MidiValue::LowRes(x) => {
                        if let Some(prev_value) = self.known_controller_values.get(&key) {
                            let prev_value = match prev_value {
                                MidiValue::HiRes(x) => {
                                    error!("This value used to be u8");
                                    *x as u8
                                }
                                MidiValue::LowRes(x) => *x,
                            };
                            let increment = if x < 64 { x as i16 } else { x as i16 - 128 };
                            let absolute_value: i16 = (prev_value as i16 + increment).clamp(0, 127);
                            let absolute_value: u8 = absolute_value.try_into().unwrap_or(0);
                            self.known_controller_values
                                .insert(key, MidiValue::LowRes(absolute_value));
                            MidiValue::LowRes(absolute_value)
                        } else {
                            let last_known_value = MidiValue::LowRes(x);
                            self.known_controller_values.insert(key, last_known_value);
                            MidiValue::LowRes(x)
                        }
                    }
                    MidiValue::HiRes(x) => {
                        warn!("HiRes relative ignored (send as-is)");
                        MidiValue::HiRes(x)
                    }
                }
            }
            ControllerValueMode::Absolute => value,
        };
        let out_msg = TetherControlChangePayload {
            channel: channel_to_int(*channel),
            controller: control_label,
            value: send_absolute_value,
        };
        self.tether_message_log.push_back(format!("{:?}", out_msg));
        self.tether_tx
            .send(TetherMidiMessage::ControlChange(out_msg))
            .unwrap();
        // debug!(
        //     "'Undefined' control change message: control = {:?}, value = {}",
        //     &control_label, value
        // );
    }

    fn update_port_info(&mut self, index: usize) {
        for (key, info) in self.ports_metadata.iter_mut() {
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
