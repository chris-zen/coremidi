use core_foundation::base::OSStatus;

use coremidi_sys::{
    MIDIPortDispose,
    MIDISend
};

use OutputPort;
use Destination;
use PacketList;

impl OutputPort {
    /// Send a list of packets to a destination.
    ///
    pub fn send(&self, destination: &Destination, packet_list: &PacketList) -> Result<(), OSStatus> {
        let status = unsafe { MIDISend(
            self.0,
            destination.0,
            &packet_list.0)
        };
        if status == 0 { Ok(()) } else { Err(status) }
    }
}

impl Drop for OutputPort {
    fn drop(&mut self) {
        unsafe { MIDIPortDispose(self.0) };
    }
}
