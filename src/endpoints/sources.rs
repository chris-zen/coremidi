use core_foundation_sys::base::OSStatus;

use coremidi_sys::{
    MIDIReceived
};

use std::ops::Deref;

use Endpoint;
use VirtualSource;
use PacketList;

impl VirtualSource {
    /// Distributes incoming MIDI from a source to the client input ports which are connected to that source.
    /// See [MIDIReceived](https://developer.apple.com/reference/coremidi/1495276-midireceived)
    ///
    pub fn received(&self, packet_list: &PacketList) -> Result<(), OSStatus> {
        let status = unsafe { MIDIReceived(
            self.endpoint.0,
            &packet_list.0)
        };
        if status == 0 { Ok(()) } else { Err(status) }
    }
}

impl Deref for VirtualSource {
    type Target = Endpoint;

    fn deref(&self) -> &Endpoint {
        &self.endpoint
    }
}
