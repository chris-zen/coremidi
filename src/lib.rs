extern crate core_foundation_sys;
extern crate core_foundation;
extern crate coremidi_sys;

use core_foundation::string::CFString;
use core_foundation::base::{OSStatus, TCFType};

use coremidi_sys::{
    MIDIClientRef, MIDIClientCreate, MIDIClientDispose,
    MIDIPortRef, MIDIOutputPortCreate, MIDIPortDispose,
    MIDIEndpointRef, MIDIPacketList, MIDISend
};

use std::mem;
use std::ptr;

pub mod packets;
mod endpoints;
pub use endpoints::destinations;

pub struct PacketList(MIDIPacketList);

pub struct Destination { endpoint_ref: MIDIEndpointRef }

pub struct Client { client_ref: MIDIClientRef }

pub struct OutputPort { port_ref: MIDIPortRef }


impl Client {
    pub fn new(name: &str) -> Result<Client, OSStatus> {
        let client_name = CFString::new(name);
        let mut client: MIDIClientRef = unsafe { mem::uninitialized() };
        let status = unsafe { MIDIClientCreate(
            client_name.as_concrete_TypeRef(),
            None, ptr::null_mut(),
            &mut client)
        };
        if status == 0 { Ok(Client { client_ref: client }) } else { Err(status) }
    }

    pub fn create_output_port(self: &Self, name: &str) -> Result<OutputPort, OSStatus> {
        OutputPort::new(self, name)
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        unsafe { MIDIClientDispose(self.client_ref) };
    }
}

impl OutputPort {
    fn new(client: &Client, name: &str) -> Result<OutputPort, OSStatus> {
        let output_port_name = CFString::new(name);
        let mut output_port: MIDIPortRef = unsafe { mem::uninitialized() };
        let status = unsafe { MIDIOutputPortCreate(
            client.client_ref,
            output_port_name.as_concrete_TypeRef(),
            &mut output_port)
        };
        if status == 0 { Ok(OutputPort { port_ref: output_port }) } else { Err(status) }
    }

    pub fn send(self: &Self, destination: &Destination, packet_list: &PacketList) -> Result<(), OSStatus> {
        let status = unsafe { MIDISend(
            self.port_ref,
            destination.endpoint_ref,
            &packet_list.0)
        };
        if status == 0 { Ok(()) } else { Err(status) }
    }
}

impl Drop for OutputPort {
    fn drop(&mut self) {
        unsafe { MIDIPortDispose(self.port_ref) };
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
