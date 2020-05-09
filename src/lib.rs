#![crate_name = "coremidi"]
#![crate_type = "lib"]
#![doc(html_root_url = "https://chris-zen.github.io/coremidi/")]

/*!
This is a [CoreMIDI](https://developer.apple.com/reference/coremidi) library for Rust built on top of the low-level bindings [coremidi-sys](https://github.com/jonas-k/coremidi-sys).
CoreMIDI is a macOS framework that provides APIs for communicating with MIDI (Musical Instrument Digital Interface) devices, including hardware keyboards and synthesizers.

This library preserves the fundamental concepts behind the CoreMIDI framework, while being Rust idiomatic. This means that if you already know CoreMIDI, you will find very easy to start using it.

Please see the [examples](examples) for an idea on how it looks like, but if you are eager to see an example, this is how you would send some note:

```rust,no_run
extern crate coremidi;
use std::time::Duration;
use std::thread;
let client = coremidi::Client::new("example-client").unwrap();
let output_port = client.output_port("example-port").unwrap();
let destination = coremidi::Destination::from_index(0).unwrap();
let note_on = coremidi::PacketBuffer::new(0, &[0x90, 0x40, 0x7f]);
let note_off = coremidi::PacketBuffer::new(0, &[0x80, 0x40, 0x7f]);
output_port.send(&destination, &note_on).unwrap();
thread::sleep(Duration::from_millis(1000));
output_port.send(&destination, &note_off).unwrap();
```

If you are looking for a portable MIDI library then you can look into:

- [midir](https://github.com/Boddlnagg/midir) (which is using this lib)
- [portmidi-rs](https://github.com/musitdev/portmidi-rs)

For handling low level MIDI data you may look into:

- [midi-rs](https://github.com/samdoshi/midi-rs)
- [rimd](https://github.com/RustAudio/rimd)

*/

extern crate core_foundation_sys;
extern crate core_foundation;
extern crate coremidi_sys;

use core_foundation_sys::base::OSStatus;

use coremidi_sys::{
    MIDIObjectRef, MIDIFlushOutput, MIDIRestart, MIDIPacket, MIDIPacketList
};

/// A [MIDI Object](https://developer.apple.com/reference/coremidi/midiobjectref).
///
/// The base class of many CoreMIDI objects.
///
#[derive(PartialEq)]
pub struct Object(MIDIObjectRef);

/// A [MIDI client](https://developer.apple.com/reference/coremidi/midiclientref).
///
/// An object maintaining per-client state.
///
/// A simple example to create a Client:
///
/// ```rust,no_run
/// let client = coremidi::Client::new("example-client").unwrap();
/// ```
#[derive(Debug)]
pub struct Client {
    // Order is important, object needs to be dropped first
    object: Object,
    callback: BoxedCallback<Notification>,
}

// A lifetime-managed wrapper for callback functions
#[derive(Debug, PartialEq)]
struct BoxedCallback<T>(*mut Box<FnMut(&T)>);

impl<T> BoxedCallback<T> {
    fn new<F: FnMut(&T) + Send + 'static>(f: F) -> BoxedCallback<T> {
        BoxedCallback(Box::into_raw(Box::new(Box::new(f))))
    }

    fn null() -> BoxedCallback<T> {
        BoxedCallback(::std::ptr::null_mut())
    }

    fn raw_ptr(&mut self) -> *mut ::std::os::raw::c_void {
        self.0 as *mut ::std::os::raw::c_void
    }

    // must not be null
    unsafe fn call_from_raw_ptr(raw_ptr: *mut ::std::os::raw::c_void, arg: &T) {
        let callback = &mut *(raw_ptr as *mut Box<FnMut(&T)>);
        callback(arg);
    }
}

unsafe impl<T> Send for BoxedCallback<T> {}

impl<T> Drop for BoxedCallback<T> {
    fn drop(&mut self) {
        unsafe {
            if !self.0.is_null() {
                let _ = Box::from_raw(self.0);
            }
        }
    }
}

/// A MIDI connection port owned by a client.
/// See [MIDIPortRef](https://developer.apple.com/reference/coremidi/midiportref).
///
/// Ports can't be instantiated directly, but through a client.
///
#[derive(Debug)]
pub struct Port { object: Object }

/// An output [MIDI port](https://developer.apple.com/reference/coremidi/midiportref) owned by a client.
///
/// A simple example to create an output port and send a MIDI event:
///
/// ```rust,no_run
/// let client = coremidi::Client::new("example-client").unwrap();
/// let output_port = client.output_port("example-port").unwrap();
/// let destination = coremidi::Destination::from_index(0).unwrap();
/// let packets = coremidi::PacketBuffer::new(0, &[0x90, 0x40, 0x7f]);
/// output_port.send(&destination, &packets).unwrap();
/// ```
#[derive(Debug)]
pub struct OutputPort { port: Port }

/// An input [MIDI port](https://developer.apple.com/reference/coremidi/midiportref) owned by a client.
///
/// A simple example to create an input port:
///
/// ```rust,no_run
/// let client = coremidi::Client::new("example-client").unwrap();
/// let input_port = client.input_port("example-port", |packet_list| println!("{}", packet_list)).unwrap();
/// let source = coremidi::Source::from_index(0).unwrap();
/// input_port.connect_source(&source);
/// ```
#[derive(Debug)]
pub struct InputPort {
    // Note: the order is important here, port needs to be dropped first
    port: Port,
    callback: BoxedCallback<PacketList>,
}

/// A MIDI source or source, owned by an entity.
/// See [MIDIEndpointRef](https://developer.apple.com/reference/coremidi/midiendpointref).
///
/// You don't need to create an endpoint directly, instead you can create system sources and sources or virtual ones from a client.
///
#[derive(Debug)]
pub struct Endpoint { object: Object }

/// A [MIDI source](https://developer.apple.com/reference/coremidi/midiendpointref) owned by an entity.
///
/// A source can be created from an index like this:
///
/// ```rust,no_run
/// let source = coremidi::Destination::from_index(0).unwrap();
/// println!("The source at index 0 has display name '{}'", source.display_name().unwrap());
/// ```
///
#[derive(Debug)]
pub struct Destination { endpoint: Endpoint }

/// A [MIDI source](https://developer.apple.com/reference/coremidi/midiendpointref) owned by an entity.
///
/// A source can be created from an index like this:
///
/// ```rust,no_run
/// let source = coremidi::Source::from_index(0).unwrap();
/// println!("The source at index 0 has display name '{}'", source.display_name().unwrap());
/// ```
///
#[derive(Debug)]
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
#[derive(Debug)]
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
#[derive(Debug)]
pub struct VirtualDestination {
    // Note: the order is important here, endpoint needs to be dropped first
    endpoint: Endpoint,
    callback: BoxedCallback<PacketList>,
}

/// A [MIDI object](https://developer.apple.com/reference/coremidi/midideviceref).
///
/// A MIDI device or external device, containing entities.
///
#[derive(Debug)]
#[derive(PartialEq)]
pub struct Device { object: Object }

/// A [list of MIDI events](https://developer.apple.com/reference/coremidi/midipacketlist) being received from, or being sent to, one endpoint.
///
#[repr(C)]
pub struct PacketList {
    // NOTE: This type must only exist in the form of immutable references
    //       pointing to valid instances of MIDIPacketList.
    //       This type must NOT implement `Copy`!
    inner: PacketListInner,
    _do_not_construct: packets::alignment::Marker
}

#[repr(packed)]
struct PacketListInner {
    num_packets: u32,
    data: [MIDIPacket; 0]
}

impl PacketList {
    /// For internal usage only.
    /// Requires this instance to actually point to a valid MIDIPacketList
    unsafe fn as_ptr(&self) -> *mut MIDIPacketList {
        self as *const PacketList as *mut PacketList as *mut MIDIPacketList
    }
}

mod object;
mod devices;
mod client;
mod ports;
mod packets;
mod properties;
mod endpoints;
mod notifications;
pub use endpoints::destinations::Destinations;
pub use endpoints::sources::Sources;
pub use packets::{PacketListIterator, Packet, PacketBuffer};
pub use properties::{Properties, PropertyGetter, PropertySetter};
pub use notifications::Notification;

/// Unschedules previously-sent packets for all the endpoints.
/// See [MIDIFlushOutput](https://developer.apple.com/reference/coremidi/1495312-midiflushoutput).
///
pub fn flush() -> Result<(), OSStatus> {
    let status = unsafe { MIDIFlushOutput(0) };
    unit_result_from_status(status)
}

/// Stops and restarts MIDI I/O.
/// See [MIDIRestart](https://developer.apple.com/reference/coremidi/1495146-midirestart).
///
pub fn restart() -> Result<(), OSStatus> {
    let status = unsafe { MIDIRestart() };
    unit_result_from_status(status)
}

/// Convert an OSStatus into a Result<T, OSStatus> given a mapping closure
fn result_from_status<T, F: FnOnce() -> T>(status: OSStatus, f: F) -> Result<T, OSStatus> {
    match status {
        0 => Ok(f()),
        _ => Err(status),
    }
}

/// Convert an OSSStatus into a Result<(), OSStatus>
fn unit_result_from_status(status: OSStatus) -> Result<(), OSStatus> {
    result_from_status(status, || ())
}