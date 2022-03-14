use std::ops::Deref;
use coremidi_sys::MIDIObjectRef;

use crate::object::Object;

/// A [MIDI object](https://developer.apple.com/documentation/coremidi/midideviceref).
///
/// A MIDI device or external device, containing entities.
///
#[derive(Debug, PartialEq)]
pub struct Device {
    pub(crate) object: Object,
}

impl Device {
    pub(crate) fn new(object_ref: MIDIObjectRef) -> Self {
        Self {
            object: Object(object_ref),
        }
    }
}

impl Deref for Device {
    type Target = Object;

    fn deref(&self) -> &Object {
        &self.object
    }
}

impl From<Object> for Device {
    fn from(object: Object) -> Self {
        Self::new(object.0)
    }
}
