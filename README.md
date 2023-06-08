# Tether Midi Mediator

Turns incoming MIDI (from your plugged-in device, or virtual device) into outgoing Tether messages in a standard form on plugs with standard names.

## Running
For now, you **must** provide one or more MIDI Port index numbers when launching, for example:
```
./tether-midi-mediator 1 2
```
or,
```
cargo run -- 1 2
```

## TODO
- [ ] Handle "relative mode" knob controller values, e.g. Akai APC Key25
- [ ] Make it possible to list ports and add them (and save them?) via GUI; no need for command line args
- [ ] Convert, and possibly visualise, MIDI note numbers -> actual notes
- [ ] Convert the other way, i.e. Tether Messages -> Midi Output