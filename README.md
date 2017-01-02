# coremidi

This is a [CoreMIDI](https://developer.apple.com/reference/coremidi) library for Rust built on top of the low-level bindings [coremidi-sys](https://github.com/jonas-k/coremidi-sys).
CoreMIDI is a Mac OSX framework that provides APIs for communicating with MIDI (Musical Instrument Digital Interface) devices, including hardware keyboards and synthesizers.

This library preserves the fundamental concepts behind the CoreMIDI framework, while being Rust idiomatic. This means that if you already know CoreMIDI, you will find very easy to start using it.

Please see the [examples](examples) for an idea on how it looks like, but if you are eager to see an example, this is how you would send some note:

```rust
extern crate coremidi;
use coremidi::{Client, Destinations, PacketBuffer};
use std::time::Duration;
use std::thread;
let client = Client::new("example-client").unwrap();
let output_port = client.output_port("example-port").unwrap();
let destination = Destinations::from_index(0);
let note_on = PacketBuffer::from_data(0, vec![0x90, 0x40, 0x7f]);
let note_off = PacketBuffer::from_data(0, vec![0x80, 0x40, 0x7f]);
output_port.send(&destination, &note_on).unwrap();
thread::sleep(Duration::from_millis(1000));
output_port.send(&destination, &note_off).unwrap();
```

If you are looking for a portable MIDI library then you can look into:
- [portmidi-rs](https://github.com/musitdev/portmidi-rs)
- [midir](https://github.com/Boddlnagg/midir)

For handling low level MIDI data you may look into:
- [midi-rs](https://github.com/samdoshi/midi-rs)
- [rimd](https://github.com/RustAudio/rimd)

**Please note that this is a work in progress project !**

# Installation

Add this to your *Cargo.toml*.

```toml
[dependencies]
coremidi = "^0.0.2"
```

# Examples

The examples can be run with:

```sh
cargo run --example send-notes
```

These are the provided examples:

- [endpoints](examples/system-endpoints.rs): how to enumerate sources and destinations.
- [send-notes](examples/send-notes.rs): how to create an output port and send MIDI messages.
- [virtual-source](examples/virtual-source.rs): how to create a virtual source and generate MIDI messages.
- [dump](examples/dump.rs): how to create an input port and receive MIDI messages.

# Roadmap

- [x] Enumerate destinations
- [x] Create output ports
- [x] Create a PacketList from MIDI bytes
- [x] Send a PacketList into an output port
- [x] Create virtual sources
- [x] Support a virtual source receiving a PacketList
- [x] Flush output
- [x] Enumerate sources
- [x] Create input ports
- [x] Support callbacks from input messages
- [x] Connect and disconnect sources
- [x] Add support to build PacketList (PacketBuffer)
- [ ] Create virtual destinations
- [ ] Support callbacks for virtual destinations
- [ ] Stop and restart MIDI I/O
- [ ] Support Sysex
- [ ] Support devices
- [ ] Support entities
- [ ] Support more MIDI Object properties (other than display name)
