use std::{sync::mpsc, time::Duration};

use log::{debug, info, warn};
use midi_msg::{MidiMsg, ReceiverContext};
use midir::{Ignore, MidiInput};

pub fn listen_for_midi(preferred_port: usize, tx: mpsc::Sender<MidiMsg>) {
    let mut midi_in = MidiInput::new("midir reading input").expect("midir failure");
    midi_in.ignore(Ignore::None);

    // Get an input port (read from console if multiple are available)
    let in_ports = midi_in.ports();
    let in_port = match in_ports.len() {
        0 => panic!("no input port found"),
        1 => {
            warn!(
                "Choosing the only available input port: {}",
                midi_in.port_name(&in_ports[0]).unwrap()
            );
            &in_ports[0]
        }
        _ => {
            debug!("Available input ports:");
            for (i, p) in in_ports.iter().enumerate() {
                info!("{}: {}", i, midi_in.port_name(p).unwrap());
            }
            in_ports.get(preferred_port).expect("invalid port")
        }
    };

    let in_port_name = midi_in.port_name(in_port).expect("Failed to get port name");

    let mut ctx = ReceiverContext::new();
    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _conn_in = midi_in.connect(
        in_port,
        "midir-read-input",
        move |_stamp, midi_bytes, _| {
            let (msg, _len) =
                MidiMsg::from_midi_with_context(&midi_bytes, &mut ctx).expect("Not an error");

            tx.send(msg).expect("failed to send on MIDI thread");
        },
        (),
    );

    info!(
        "MIDI connection open, reading input from '{}'.",
        in_port_name
    );

    loop {
        std::thread::sleep(Duration::from_millis(1));
    }

    // warn!("Closing connection");
}
