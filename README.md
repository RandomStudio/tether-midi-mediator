# Tether Midi Mediator

Turns incoming MIDI (from your plugged-in device, or virtual device) into outgoing Tether messages in a standard form on plugs with standard names.

## Running
Download a [release](https://github.com/RandomStudio/tether-midi-mediator/releases) for your OS, and run it!

Available MIDI input ports will be detected automatically.

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