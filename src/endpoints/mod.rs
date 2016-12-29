use core_foundation_sys::base::OSStatus;

use coremidi_sys::MIDIFlushOutput;

use Endpoint;
use properties;

impl Endpoint {
    /// Get the display name for the destination endpoint.
    ///
    pub fn display_name(&self) -> Option<String> {
        properties::get_display_name(self.0)
    }

    /// Unschedules previously-sent packets.
    /// See [MIDIFlushOutput](https://developer.apple.com/reference/coremidi/1495312-midiflushoutput).
    ///
    pub fn flush(&self) -> Result<(), OSStatus> {
        let status = unsafe { MIDIFlushOutput(self.0) };
        if status == 0 { Ok(()) } else { Err(status) }
    }
}

pub mod destinations;
pub mod sources;
