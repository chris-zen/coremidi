# coremidi

CoreMIDI library for Rust built on top of the low-level bindings at [coremidi-sys](https://github.com/jonas-k/coremidi-sys).

This library tries to be as transparent as possible to the original CoreMIDI framework, while being as rust idiomatic as possible. This means that if you already know [CoreMIDI](https://developer.apple.com/reference/coremidi) you will find very easy to start using it.

Please see the [examples](examples) for an idea on how does it look like.

If you are looking for a portable MIDI library then you can look into:
- [midir](https://github.com/Boddlnagg/midir)
- [portmidi-rs](https://github.com/musitdev/portmidi-rs)

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

- [system-endpoints](examples/system-endpoints.rs): shows how to enumerate sources and destinations.
- [send-notes](examples/send-notes.rs): shows how to create an output port and send MIDI messages.
- [virtual-source](examples/virtual-source.rs): shows how to create a virtual source and generate MIDI messages.

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
- [ ] Create virtual destinations
- [ ] Support callbacks for virtual destinations
- [ ] Improve PacketList to support multiple packets and arbitrary sizes
- [ ] Stop and restart MIDI I/O
- [ ] Support Sysex
- [ ] Support devices
- [ ] Support entities
- [ ] Support more MIDI Object properties (other than display name)
