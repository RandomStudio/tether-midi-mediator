# Tether Midi Mediator

Turns incoming MIDI (from your plugged-in device, or virtual device) into outgoing Tether messages in a standard form on plugs with standard names.

## Running
Download a [release](https://github.com/RandomStudio/tether-midi-mediator/releases) for your OS, and run it!

Available MIDI input ports will be detected automatically.

## Tether MIDI Messages
This Agent translates MIDI into standardised Tether Messages on standard plugs.

- **ControlChange** (knob/slider) MIDI input
  - Published on the plug `"controlChange"`
  - Keys are
    - `channel`: MIDI channel
    - `control`: knob/slider number/ID
    - `value`: absolute value 0-127
  - Additionally, `"knobs"` if "knob mapping" is enabled (see below)
- **Note On** MIDI input
  - Published on the plug `"notesOn"`
  - Keys are
    - `channel`: MIDI channel
    - `note`: MIDI note number
    - `velocity`: volume/pressure 0-127
- **Note Of** MIDI input
  - Published on the plug `"notesOff"`
  - Keys are
    - `channel`: MIDI channel
    - `note`: MIDI note number
    - `velocity`: volume/pressure 0-127 (typically 0)

In addition, the "untranslated" MIDI message, as parsed by the underyling [midi-msg](https://crates.io/crates/midi-msg) library, is published on a plug `"raw"`.
## Absolute vs Relative mode
When it comes to Control Change messages, some MIDI controllers send absolute values from 0-127 depending on the knob position. This is the "standard" way.

Other controllers, particularly those with "endless" knobs (no stop points), send values meant to be interpreted as relative increase/decrease values. Tether Midi Mediator automatically handles this situation when in Relative mode (either select this in the GUI or pass `--midi.relative` flag from command line). The behaviour in relative mode is:
- Values in the lower half of the range (0-63) are interpreted as increment speeds
- Values in the upper half of the range (64-127) are interpreted as decrement speeds
- Any "previously known" value for the same channel+controller is recalled and the increment/decrement is applied; the range is also clamped if necessary
- The `"controlChange"` plug will publish only the absolute values; the `"raw"` plug will contain the original values sent by the controller

Relative-mode is not standardised for MIDI devices. The above mechanism has been tested with an Akai APC Key25 Mk2. Submit an Issue if you think alternative algorithms should be possible.

## Knob Mappings
Some devices (listed in `mappings/knobs.json`) will be matched automatically against their device name for a "knob mapping". This simply means that known ControlChange values are matched against a known order of "knobs" labelled 0, 1, 2, etc.

Incoming ControlChange MIDI messages with a known "knob mapping" will additionally generate messages on a "knobs" OutputPlug which encodes an `index` and `position` (normalised float value between `0.0` and `1.0`).

## CLI options
You can change various settings using the command line. Append `--help` for more details.

For example:
 - `--headless`: run without a GUI - great for server / console-based use
 - `--tether.disable`: don't try to connect to MQTT Broker at all
 - Numbers following params, eg. `./tether-midi-mediator 0 1` will only use MIDI input ports 0 and 1

## TODO
- [x] Handle "relative mode" knob controller values, e.g. Akai APC Key25
- [x] Display incoming MIDI messages AND outgoing Tether messages
- [x] Make it possible to list ports, optionally specify inputs
- [ ] Convert, and possibly visualise, MIDI note numbers -> actual notes
- [ ] Convert the other way, i.e. Tether Messages -> Midi Output
