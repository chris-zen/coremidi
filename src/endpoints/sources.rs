use core_foundation_sys::base::OSStatus;

use coremidi_sys::{
    MIDIReceived, MIDIEndpointDispose
};

use VirtualSource;
use PacketList;

impl VirtualSource {
    /// Distributes incoming MIDI from a source to the client input ports which are connected to that source.
    /// See [MIDIReceived](https://developer.apple.com/reference/coremidi/1495276-midireceived)
    ///
    pub fn received(&self, packet_list: &PacketList) -> Result<(), OSStatus> {
        let status = unsafe { MIDIReceived(
            self.0,
            &packet_list.0)
        };
        if status == 0 { Ok(()) } else { Err(status) }
    }
}

impl Drop for VirtualSource {
    fn drop(&mut self) {
        unsafe { MIDIEndpointDispose(self.0) };
    }
}
