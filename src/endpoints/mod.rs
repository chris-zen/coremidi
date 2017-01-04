use core_foundation_sys::base::OSStatus;

use coremidi_sys::MIDIFlushOutput;

use std::ops::Deref;

use Object;
use Endpoint;

impl Endpoint {
    /// Unschedules previously-sent packets.
    /// See [MIDIFlushOutput](https://developer.apple.com/reference/coremidi/1495312-midiflushoutput).
    ///
    pub fn flush(&self) -> Result<(), OSStatus> {
        let status = unsafe { MIDIFlushOutput(self.object.0) };
        if status == 0 { Ok(()) } else { Err(status) }
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

pub mod destinations;
pub mod sources;
