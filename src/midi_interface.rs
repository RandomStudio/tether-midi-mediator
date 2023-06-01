use std::{error::Error, sync::mpsc, time::Duration};

use log::{info, warn};
use midi_msg::{MidiMsg, ReceiverContext, SystemRealTimeMsg};
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

pub fn midi_listener_thread(
    midi_input: MidiInput,
    midi_input_port: midir::MidiInputPort,
    midi_tx: mpsc::Sender<(usize, MidiMsg)>,
    port: usize,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut ctx = ReceiverContext::new();

        let _connection = midi_input
            .connect(
                &midi_input_port,
                "midir-read-input",
                move |_timestamp, midi_bytes, _| {
                    let (msg, _len) = MidiMsg::from_midi_with_context(midi_bytes, &mut ctx)
                        .expect("Not an error");

                    // Handle everything but spammy clock messages.
                    if let MidiMsg::SystemRealTime {
                        msg: SystemRealTimeMsg::TimingClock,
                    } = msg
                    {
                        // no-op
                    } else {
                        midi_tx
                            .send((port, msg))
                            .expect("failed to send on channel");
                        // println!("{}: {:?}", stamp, msg);
                    }
                },
                (),
            )
            .expect("failed to connect port");

        // TODO: this thread should end when required
        loop {
            std::thread::sleep(Duration::from_millis(1));
        }
        //     listen_for_midi(port, midi_tx);
    })
}
