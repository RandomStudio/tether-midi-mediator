use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(long = "loglevel",default_value_t=String::from("info"))]
    pub log_level: String,

    /// Flag to enable headless (no GUI) mode, suitable for server-type
    /// process
    #[arg(long = "headless")]
    pub headless_mode: bool,

    /// Flag to disable Tether connection
    #[arg(long = "tether.disable")]
    pub tether_disable: bool,

    /// The IP address of the Tether MQTT broker (server)
    #[arg(long = "tether.host", default_value_t=String::from("localhost"))]
    pub tether_host: String,

    /// Override role for Tether Agent
    #[arg(long = "tether.role", default_value_t=String::from("midi"))]
    pub tether_role: String,

    /// Override ID/group for Tether Agent
    #[arg(long = "tether.id")]
    pub tether_id: Option<String>,

    /// Enable translation of relative controller values into absolute values
    #[arg(long = "midi.relative")]
    pub relative_mode_enabled: bool,

    /// Disable lookup of "knob mapping" for device(s)
    #[arg(long = "knobs.disable", default_value_t = false)]
    pub knobs_disable: bool,

    /// Specify one or more MIDI ports by index, in any order
    #[clap()]
    pub midi_ports: Vec<usize>,
}
