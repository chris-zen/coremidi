#![crate_name = "coremidi"]
#![crate_type = "lib"]
#![doc(html_root_url = "https://chris-zen.github.io/coremidi/")]

/*!
This is a [CoreMIDI](https://developer.apple.com/reference/coremidi) library for Rust built on top of the low-level bindings [coremidi-sys](https://github.com/jonas-k/coremidi-sys).
CoreMIDI is a Mac OSX framework that provides APIs for communicating with MIDI (Musical Instrument Digital Interface) devices, including hardware keyboards and synthesizers.

This library preserves the fundamental concepts behind the CoreMIDI framework, while being Rust idiomatic. This means that if you already know CoreMIDI, you will find very easy to start using it.

Please see the [examples](examples) for an idea on how it looks like, but if you are eager to see an example, this is how you would send some note:

```rust,no_run
extern crate coremidi;
use std::time::Duration;
use std::thread;
let client = coremidi::Client::new("example-client").unwrap();
let output_port = client.output_port("example-port").unwrap();
let destination = coremidi::Destination::from_index(0);
let note_on = coremidi::PacketBuffer::from_data(0, vec![0x90, 0x40, 0x7f]);
let note_off = coremidi::PacketBuffer::from_data(0, vec![0x80, 0x40, 0x7f]);
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

*/

extern crate core_foundation_sys;
extern crate core_foundation;
extern crate coremidi_sys;
extern crate libc;

use core_foundation_sys::base::OSStatus;

use coremidi_sys::{
    MIDIClientRef, MIDIPortRef, MIDIEndpointRef, MIDIFlushOutput
};

use coremidi_sys_ext::{
    MIDIPacketList
};

/// A [MIDI client](https://developer.apple.com/reference/coremidi/midiclientref).
///
/// A simple example to create a Client:
///
/// ```rust,no_run
/// let client = coremidi::Client::new("example-client").unwrap();
/// ```
pub struct Client(MIDIClientRef);

/// A MIDI connection port owned by a client.
/// See [MIDIPortRef](https://developer.apple.com/reference/coremidi/midiportref).
///
/// Ports can't be instantiated directly, but through a client.
///
pub struct Port(MIDIPortRef);

/// An output [MIDI port](https://developer.apple.com/reference/coremidi/midiportref) owned by a client.
///
/// A simple example to create an output port and send a MIDI event:
///
/// ```rust,no_run
/// let client = coremidi::Client::new("example-client").unwrap();
/// let output_port = client.output_port("example-port").unwrap();
/// let destination = coremidi::Destination::from_index(0);
/// let packets = coremidi::PacketBuffer::from_data(0, vec![0x90, 0x40, 0x7f]);
/// output_port.send(&destination, &packets).unwrap();
/// ```
pub struct OutputPort { port: Port }

/// An input [MIDI port](https://developer.apple.com/reference/coremidi/midiportref) owned by a client.
///
/// A simple example to create an input port:
///
/// ```rust,no_run
/// let client = coremidi::Client::new("example-client").unwrap();
/// let input_port = client.input_port("example-port", |packet_list| println!("{}", packet_list)).unwrap();
/// let source = coremidi::Source::from_index(0);
/// input_port.connect_source(&source);
/// ```
pub struct InputPort { port: Port }

/// A MIDI source or source, owned by an entity.
/// See [MIDIEndpointRef](https://developer.apple.com/reference/coremidi/midiendpointref).
///
/// You don't need to create an endpoint directly, instead you can create system sources and sources or virtual ones from a client.
///
pub struct Endpoint(MIDIEndpointRef);

/// A [MIDI source](https://developer.apple.com/reference/coremidi/midiendpointref) owned by an entity.
///
/// A source can be created from an index like this:
///
/// ```rust,no_run
/// let source = coremidi::Destination::from_index(0);
/// println!("The source at index 0 has display name '{}'", source.display_name().unwrap());
/// ```
///
pub struct Destination { endpoint: Endpoint }

/// A [MIDI source](https://developer.apple.com/reference/coremidi/midiendpointref) owned by an entity.
///
/// A source can be created from an index like this:
///
/// ```rust,no_run
/// let source = coremidi::Source::from_index(0);
/// println!("The source at index 0 has display name '{}'", source.display_name().unwrap());
/// ```
///
pub struct Source { endpoint: Endpoint }

/// A [MIDI virtual source](https://developer.apple.com/reference/coremidi/1495212-midisourcecreate) owned by a client.
///
/// A virtual source can be created like:
///
/// ```rust,no_run
/// let client = coremidi::Client::new("example-client").unwrap();
/// let source = client.virtual_source("example-source").unwrap();
/// ```
///
pub struct VirtualSource { endpoint: Endpoint }

/// A [MIDI virtual destination](https://developer.apple.com/reference/coremidi/1495347-mididestinationcreate) owned by a client.
///
/// A virtual destination can be created like:
///
/// ```rust,no_run
/// let client = coremidi::Client::new("example-client").unwrap();
/// client.virtual_destination("example-destination", |packet_list| println!("{}", packet_list)).unwrap();
/// ```
///
pub struct VirtualDestination { endpoint: Endpoint }

/// A [list of MIDI events](https://developer.apple.com/reference/coremidi/midipacketlist) being received from, or being sent to, one endpoint.
///
pub struct PacketList(*const MIDIPacketList);

mod coremidi_sys_ext;
mod client;
mod ports;
mod packets;
mod properties;
mod endpoints;
pub use endpoints::destinations::Destinations;
pub use endpoints::sources::Sources;
pub use packets::PacketBuffer;

/// Unschedules previously-sent packets for all the endpoints.
/// See [MIDIFlushOutput](https://developer.apple.com/reference/coremidi/1495312-midiflushoutput).
///
pub fn flush() -> Result<(), OSStatus> {
    let status = unsafe { MIDIFlushOutput(0) };
    if status == 0 { Ok(()) } else { Err(status) }
}
