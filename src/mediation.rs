use std::sync::mpsc::{Receiver, Sender};

use log::error;
use midi_msg::MidiMsg;

pub struct MediationDataModel {
    pub last_msg_received: String,
    pub midi_rx: Receiver<MidiMsg>,
    pub tether_tx: Sender<MidiMsg>,
}

impl MediationDataModel {
    pub fn new(midi_rx: Receiver<MidiMsg>, tether_tx: Sender<MidiMsg>) -> Self {
        MediationDataModel {
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
