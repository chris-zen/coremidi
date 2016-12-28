use core_foundation::string::CFString;
use core_foundation::base::{OSStatus, TCFType};

use coremidi_sys::{
    MIDIClientRef, MIDIClientCreate, MIDIClientDispose,
    MIDIPortRef, MIDIOutputPortCreate
};

use std::mem;
use std::ptr;

use Client;
use OutputPort;

impl Client {
    /// Creates a new CoreMIDI client.
    ///
    pub fn new(name: &str) -> Result<Client, OSStatus> {
        let client_name = CFString::new(name);
        let mut client: MIDIClientRef = unsafe { mem::uninitialized() };
        let status = unsafe { MIDIClientCreate(
            client_name.as_concrete_TypeRef(),
            None, ptr::null_mut(),
            &mut client)
        };
        if status == 0 { Ok(Client(client)) } else { Err(status) }
    }

    /// Creates an output port through which the client may send outgoing MIDI messages to any MIDI destination.
    ///
    pub fn create_output_port(&self, name: &str) -> Result<OutputPort, OSStatus> {
        let output_port_name = CFString::new(name);
        let mut output_port: MIDIPortRef = unsafe { mem::uninitialized() };
        let status = unsafe { MIDIOutputPortCreate(
            self.0,
            output_port_name.as_concrete_TypeRef(),
            &mut output_port)
        };
        if status == 0 { Ok(OutputPort(output_port)) } else { Err(status) }
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        unsafe { MIDIClientDispose(self.0) };
    }
}
