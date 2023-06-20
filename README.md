# Tether Midi Mediator

Turns incoming MIDI (from your plugged-in device, or virtual device) into outgoing Tether messages in a standard form on plugs with standard names.

## Running
Download a [release](https://github.com/RandomStudio/tether-midi-mediator/releases) for your OS, and run it!

Available MIDI input ports will be detected automatically.

## Absolute vs Relative mode
When it comes to Control Change messages, some MIDI controllers send absolute values from 0-127 depending on the knob position. This is the "standard" way.

Other controllers, particularly those with "endless" knobs (no stop points), send values meant to be interpreted as relative increase/decrease values. Tether Midi Mediator automatically handles this situation when in Relative mode (either select this in the GUI or pass `--midi.relative` flag from command line). The behaviour in relative mode is:
- Values in the lower half of the range (0-63) are interpreted as increment speeds
- Values in the upper half of the range (64-127) are interpreted as decrement speeds
- Any "previously known" value for the same channel+controller is recalled and the increment/decrement is applied; the range is also clamped if necessary

Relative-mode is not standardised for MIDI devices. The above mechanism has been tested with an Akai APC Key25 Mk2. Submit an Issue if you think alternative algorithms should be possible.
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