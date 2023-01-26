
mot - MIDI and OSC Tools
------------------------

mot consists of several MIDI and OSC command line tools. These are mainly of interest to debug and check OSC messages and MIDI devices. There are applications to:
* midi_echo: prints MIDI messages coming from a connected MIDI device.
* osc_echo: prints OSC messages arriving at a certain UDP port.
* midi_to_osc: a MIDI to OSC bridge which sends MIDI messages coming from a connected MIDI device to an OSC target.
* midi_roundtrip_latency: measure MIDI round-trip latency.

## Install MIDI and OSC Tools mot


## Command line applications in mot

~~~~~~
mot - Midi and OSC Tools

Usage: mot <COMMAND>

Commands:
  midi_to_osc             Transport MIDI over OSC
  midi_echo               Print incoming MIDI messages.
  osc_echo                Print incoming OSC messages.
  midi_roundtrip_latency  Test MIDI roundtrip latency
  help                    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
~~~~~~

### MIDI echo



~~~~~~
mot midi_echo -l
#Listing MIDI input devices:
# Available MIDI input ports:
# 0: Teensy MIDI Port 1
mot midi_echo 0
~~~~~~

### OSC echo

### MIDI to OSC bridge

### MIDI 


