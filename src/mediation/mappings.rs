use log::{debug, info};
use serde::Deserialize;

use super::messages::ControllerLabel;
use anyhow::anyhow;

#[derive(Deserialize, Clone, Debug)]
pub struct KnobMapping {
    pub channel: u8,
    pub controller: ControllerLabel,
}

#[derive(Deserialize, Clone, Debug)]
pub struct DeviceWithMapping {
    name: String,
    knobs: Vec<KnobMapping>,
}

pub fn load_knob_mappings(name: &str) -> anyhow::Result<Vec<KnobMapping>> {
    let json_str = include_str!("../../mappings/knobs.json");
    let all_mappings = serde_json::from_str::<Vec<DeviceWithMapping>>(json_str)
        .expect("failed to load knob mappings");
    info!("...Loaded {} knob mappings OK", all_mappings.len());
    match all_mappings.iter().find(|x| x.name == name) {
        Some(device) => {
            let knobs = device.clone().knobs;
            Ok(knobs)
        }
        None => Err(anyhow!("Could not find device with name  {}", name)),
    }
}
