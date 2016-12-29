/*!
CoreMIDI library for Rust built on top of the low-level bindings [coremidi-sys](https://github.com/jonas-k/coremidi-sys).

This library tries to be as transparent as possible to the original CoreMIDI framework, while being as rust idiomatic as possible. This means that if you already know [CoreMIDI](https://developer.apple.com/reference/coremidi) you will find very easy to start using it.

*/

extern crate core_foundation_sys;
extern crate core_foundation;
extern crate coremidi_sys;

use core_foundation_sys::base::OSStatus;
use coremidi_sys::{MIDIClientRef, MIDIPortRef, MIDIEndpointRef, MIDIPacketList, MIDIFlushOutput};

/// A [MIDI client](https://developer.apple.com/reference/coremidi/midiclientref).
///
/// A simple example to create a Client:
///
/// ```
/// let client = coremidi::Client::new("example-client").unwrap();
/// ```
pub struct Client(MIDIClientRef);

/// An output [MIDI connection port](https://developer.apple.com/reference/coremidi/midiportref) owned by a client.
///
/// A simple example to create an output port and send a MIDI event:
///
/// ```
/// let client = coremidi::Client::new("example-client").unwrap();
/// let output_port = client.output_port("example-port").unwrap();
/// let destination = coremidi::Destination::from_index(0);
/// let packets = coremidi::PacketList::from_data(0, vec![0x90, 0x40, 0x7f]);
/// output_port.send(&destination, &packets).unwrap();
/// ```
pub struct OutputPort(MIDIPortRef);

/// A MIDI source or destination, owned by an entity.
/// See [MIDIEndpointRef](https://developer.apple.com/reference/coremidi/midiendpointref).
///
/// You don't need to create an endpoint directly, instead you can create system sources and destinations or virtual ones from a client.
///
pub struct Endpoint(MIDIEndpointRef);

/// A [MIDI destination](https://developer.apple.com/reference/coremidi/midiendpointref) owned by an entity.
///
/// A destination can be created from an index like this:
///
/// ```
/// let destination = coremidi::Destination::from_index(0);
/// println!("The destination at index 0 has display name '{}'", destination.get_display_name());
/// ```
///
pub struct Destination { endpoint: Endpoint }

/// A [MIDI virtual source](https://developer.apple.com/reference/coremidi/1495212-midisourcecreate) owned by a client.
///
/// A virtual source can be created like:
///
/// ```
/// let client = coremidi::Client::new("example-client").unwrap();
/// let source = client.virtual_source("example-source").unwrap();
/// ```
///
pub struct VirtualSource { endpoint: Endpoint }

/// A [list of MIDI events](https://developer.apple.com/reference/coremidi/midipacketlist) being received from, or being sent to, one endpoint.
///
pub struct PacketList(MIDIPacketList);

mod client;
mod ports;
mod packets;
mod properties;
mod endpoints;
pub use endpoints::destinations::Destinations;

/// Unschedules previously-sent packets for all the endpoints.
/// See [MIDIFlushOutput](https://developer.apple.com/reference/coremidi/1495312-midiflushoutput).
///
pub fn flush() -> Result<(), OSStatus> {
    let status = unsafe { MIDIFlushOutput(0) };
    if status == 0 { Ok(()) } else { Err(status) }
}
