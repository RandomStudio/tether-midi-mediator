use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub struct NotePayload {
    pub channel: u8,
    pub note: u8,
    pub velocity: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ControllerLabel {
    Numbered(u8),
    Special(String),
}

#[derive(Serialize, Debug, Clone)]
pub enum MidiValue {
    LowRes(u8),
    HighRes(u16),
}

#[derive(Serialize, Debug)]
pub struct ControlChangePayload {
    pub channel: u8,
    pub controller: ControllerLabel,
    pub value: MidiValue,
}

#[derive(Serialize, Debug)]
pub struct KnobPayload {
    pub index: u8,
    pub position: f32,
}

#[derive(Debug, Serialize)]
pub enum TetherMidiMessage {
    /// Already-encoded payload
    Raw(Vec<u8>),
    NoteOn(NotePayload),
    NoteOff(NotePayload),
    ControlChange(ControlChangePayload),
    Knob(KnobPayload),
}
