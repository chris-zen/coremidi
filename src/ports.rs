use core_foundation::base::OSStatus;

use coremidi_sys::{
    MIDIPortConnectSource, MIDIPortDisconnectSource, MIDIPortDispose
};

use coremidi_sys::{
    MIDISend
};

use std::ptr;
use std::ops::Deref;

use Object;
use Port;
use OutputPort;
use InputPort;
use Destination;
use Source;
use PacketList;

impl Deref for Port {
    type Target = Object;

    fn deref(&self) -> &Object {
        &self.object
    }
}

impl Drop for Port {
    fn drop(&mut self) {
        unsafe { MIDIPortDispose(self.object.0) };
    }
}

impl OutputPort {
    /// Send a list of packets to a destination.
    /// See [MIDISend](https://developer.apple.com/reference/coremidi/1495289-midisend).
    ///
    pub fn send(&self, destination: &Destination, packet_list: &PacketList) -> Result<(), OSStatus> {
        let status = unsafe { MIDISend(
            self.port.object.0,
            destination.endpoint.object.0,
            packet_list.0)
        };
        if status == 0 { Ok(()) } else { Err(status) }
    }
}

impl Deref for OutputPort {
    type Target = Port;

    fn deref(&self) -> &Port {
        &self.port
    }
}

impl InputPort {

    pub fn connect_source(&self, source: &Source) -> Result<(), OSStatus> {
        let status = unsafe { MIDIPortConnectSource(
            self.object.0,
            source.object.0,
            ptr::null_mut())
        };
        if status == 0 { Ok(()) } else { Err(status) }
    }

    pub fn disconnect_source(&self, source: &Source) -> Result<(), OSStatus> {
        let status = unsafe { MIDIPortDisconnectSource(
            self.object.0,
            source.object.0)
        };
        if status == 0 { Ok(()) } else { Err(status) }
    }
}

impl Deref for InputPort {
    type Target = Port;

    fn deref(&self) -> &Port {
        &self.port
    }
}
