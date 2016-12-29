/*!
CoreMIDI library for Rust built on top of the low-level bindings [coremidi-sys](https://github.com/jonas-k/coremidi-sys).

This library tries to be as transparent as possible to the original CoreMIDI framework, while being as rust idiomatic as possible. This means that if you already know [CoreMIDI](https://developer.apple.com/reference/coremidi) you will find very easy to start using it.

*/

extern crate core_foundation_sys;
extern crate core_foundation;
extern crate coremidi_sys;

use coremidi_sys::{MIDIClientRef, MIDIPortRef, MIDIEndpointRef, MIDIPacketList};

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

/// A [MIDI destination](https://developer.apple.com/reference/coremidi/midiendpointref) owned by an entity.
///
/// A destination can be created from an index:
///
/// ```
/// let destination = coremidi::Destination::from_index(0);
/// println!("The destination at index 0 has display name '{}'", destination.get_display_name());
/// ```
///
pub struct Destination(MIDIEndpointRef);

/// A [MIDI virtual source](https://developer.apple.com/reference/coremidi/1495212-midisourcecreate) owned by a client.
///
/// A virtual source can be created like:
///
/// ```
/// let client = coremidi::Client::new("example-client").unwrap();
/// let source = client.virtual_source("example-source").unwrap();
/// ```
///
pub struct VirtualSource(MIDIEndpointRef);

/// A [list of MIDI events](https://developer.apple.com/reference/coremidi/midipacketlist) being received from, or being sent to, one endpoint.
///
pub struct PacketList(MIDIPacketList);

mod client;
mod ports;
mod packets;
mod properties;
mod endpoints;
pub use endpoints::destinations::Destinations;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
