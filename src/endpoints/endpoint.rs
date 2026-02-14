use core_foundation_sys::base::OSStatus;
use std::ops::Deref;

use coremidi_sys::{MIDIEndpointRef, MIDIFlushOutput};

use crate::object::Object;
use crate::unit_result_from_status;

/// A MIDI source or destination, owned by an entity.
/// See [MIDIEndpointRef](https://developer.apple.com/documentation/coremidi/midiendpointref).
///
/// You don't need to create an endpoint directly, instead you can create system sources or virtual ones from a client.
///
#[derive(Debug, Hash, Eq, PartialEq)]
pub struct Endpoint {
    pub(crate) object: Object,
}

impl Endpoint {
    pub(crate) fn new(endpoint_ref: MIDIEndpointRef) -> Self {
        Self {
            object: Object(endpoint_ref),
        }
    }

    /// Unschedules previously-sent packets.
    /// See [MIDIFlushOutput](https://developer.apple.com/documentation/coremidi/1495312-midiflushoutput).
    ///
    pub fn flush(&self) -> Result<(), OSStatus> {
        let status = unsafe { MIDIFlushOutput(self.object.0) };
        unit_result_from_status(status)
    }
}

impl AsRef<Object> for Endpoint {
    fn as_ref(&self) -> &Object {
        &self.object
    }
}

impl Deref for Endpoint {
    type Target = Object;

    fn deref(&self) -> &Object {
        &self.object
    }
}
