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

**This version doesn't work because of a bug in coremidi-sys.
 Please see [this PR](https://github.com/jonas-k/coremidi-sys/pull/3)
 for more information on an ongoing fix for it.**

# Installation

Add this to your *Cargo.toml*.

```toml
[dependencies]
coremidi = "^0.0.1"
```

# Examples

The examples can be run with:

```sh
cargo run --example send-notes
```

- [send-notes](examples/send-notes.rs): shows how to create an output port and send MIDI messages.

# Roadmap

[x] Enumerate destinations
[x] Create output ports
[x] Create a PacketList from MIDI bytes
[x] Send a PacketList into an output port
[ ] Create a source virtual port
[ ] Send a PacketList into a source virtual port
[ ] Flush an output port
[ ] Enumerate sources
[ ] Create input ports
[ ] Support callbacks from input messages
[ ] Create a destination virtual port
[ ] Support callbacks for destination virtual ports
[ ] Connect and disconnect sources
[ ] Stop and restart MIDI I/O
[ ] Improve PacketList to support multiple packets and arbitrary sizes
[ ] Support Sysex
[ ] Support devices
[ ] Support entities
[ ] Support more MIDI Object properties (other than display name)
