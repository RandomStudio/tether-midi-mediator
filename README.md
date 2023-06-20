# Tether Midi Mediator

Turns incoming MIDI (from your plugged-in device, or virtual device) into outgoing Tether messages in a standard form on plugs with standard names.

## Running
Download a release for your OS, and run it!

Available MIDI input ports will be detected automatically.

## TODO
- [x] Handle "relative mode" knob controller values, e.g. Akai APC Key25
- [x] Display incoming MIDI messages AND outgoing Tether messages
- [x] Make it possible to list ports, optionally specify inputs
- [ ] Convert, and possibly visualise, MIDI note numbers -> actual notes
- [ ] Convert the other way, i.e. Tether Messages -> Midi Output