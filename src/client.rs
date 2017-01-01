use core_foundation::string::CFString;
use core_foundation::base::{OSStatus, TCFType};

use coremidi_sys::{
    MIDIClientRef, MIDIClientCreate, MIDIClientDispose,
    MIDIPortRef, MIDIOutputPortCreate,
    MIDIEndpointRef, MIDISourceCreate
};

use coremidi_sys_ext::{
    MIDIPacketList, MIDIInputPortCreate
};

use std::mem;
use std::ptr;

use Client;
use Port;
use InputPort;
use OutputPort;
use Endpoint;
use VirtualSource;
use PacketList;

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
        let port_name = CFString::new(name);
        let mut port_ref: MIDIPortRef = unsafe { mem::uninitialized() };
        let status = unsafe { MIDIOutputPortCreate(
            self.0,
            port_name.as_concrete_TypeRef(),
            &mut port_ref)
        };
        if status == 0 { Ok(OutputPort { port: Port(port_ref) }) } else { Err(status) }
    }

    /// Creates an input port through which the client may receive incoming MIDI messages from any MIDI source.
    /// See [MIDIInputPortCreate](https://developer.apple.com/reference/coremidi/1495225-midiinputportcreate).
    ///
    pub fn input_port<F>(&self, name: &str, callback: &F) -> Result<InputPort, OSStatus>
            where F: Fn(PacketList) {

        extern "C" fn read_proc<F: Fn(PacketList)>(
                pktlist: *const MIDIPacketList,
                read_proc_ref_con: *mut ::libc::c_void,
                _: *mut ::libc::c_void) { //srcConnRefCon

            let _ = ::std::panic::catch_unwind(|| unsafe {
                let packet_list = PacketList(*pktlist);
                let ref callback = *(read_proc_ref_con as *const F);
                // println!("read_proc: pl={:x}, mpl={:x}", &packet_list as *const _ as usize, pktlist as usize);
                callback(packet_list);
            });
        }

        let port_name = CFString::new(name);
        let mut port_ref: MIDIPortRef = unsafe { mem::uninitialized() };
        let status = unsafe { MIDIInputPortCreate(
            self.0,
            port_name.as_concrete_TypeRef(),
            Some(read_proc::<F> as extern "C" fn(_, _, _)),
            callback as *const _ as *mut ::libc::c_void,
            &mut port_ref)
        };
        if status == 0 { Ok(InputPort { port: Port(port_ref) }) } else { Err(status) }
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
