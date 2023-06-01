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