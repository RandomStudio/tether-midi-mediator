use std::error::Error;

use log::{info, warn};
use midir::{MidiInput, MidiInputPort};

pub fn get_midi_connection(
    midi_in: &MidiInput,
    preferred_port: usize,
) -> Result<(MidiInputPort, String), Box<dyn Error>> {
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
            for (i, p) in in_ports.iter().enumerate() {
                info!("{}: {}", i, midi_in.port_name(p).unwrap());
            }
            in_ports.get(preferred_port).expect("invalid port")
        }
    };

    let in_port_name = midi_in.port_name(in_port).expect("Failed to get port name");

    Ok((in_port.to_owned(), in_port_name))
}
