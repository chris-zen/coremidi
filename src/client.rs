use core_foundation::string::CFString;
use core_foundation::base::{OSStatus, TCFType};

use coremidi_sys::{
    MIDIClientRef, MIDIClientCreate, MIDIClientDispose,
    MIDIPortRef, MIDIOutputPortCreate,
    MIDIEndpointRef, MIDISourceCreate
};

use std::mem;
use std::ptr;

use Client;
use OutputPort;
use Endpoint;
use VirtualSource;

impl Client {
    /// Creates a new CoreMIDI client.
    /// See [MIDIClientCreate](https://developer.apple.com/reference/coremidi/1495360-midiclientcreate).
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
    /// See [MIDIOutputPortCreate](https://developer.apple.com/reference/coremidi/1495166-midioutputportcreate).
    ///
    pub fn output_port(&self, name: &str) -> Result<OutputPort, OSStatus> {
        let output_port_name = CFString::new(name);
        let mut output_port: MIDIPortRef = unsafe { mem::uninitialized() };
        let status = unsafe { MIDIOutputPortCreate(
            self.0,
            output_port_name.as_concrete_TypeRef(),
            &mut output_port)
        };
        if status == 0 { Ok(OutputPort(output_port)) } else { Err(status) }
    }

    /// Creates a virtual source in the client.
    /// See [MIDISourceCreate](https://developer.apple.com/reference/coremidi/1495212-midisourcecreate).
    ///
    pub fn virtual_source(&self, name: &str) -> Result<VirtualSource, OSStatus> {
        let virtual_source_name = CFString::new(name);
        let mut virtual_source: MIDIEndpointRef = unsafe { mem::uninitialized() };
        let status = unsafe { MIDISourceCreate(
            self.0,
            virtual_source_name.as_concrete_TypeRef(),
            &mut virtual_source)
        };
        if status == 0 { Ok(VirtualSource { endpoint: Endpoint(virtual_source) }) } else { Err(status) }
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        unsafe { MIDIClientDispose(self.0) };
    }
}
