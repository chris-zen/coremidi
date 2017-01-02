#![allow(non_snake_case, non_upper_case_globals, non_camel_case_types)]

use core_foundation_sys::base::OSStatus;
use core_foundation_sys::string::CFStringRef;

use coremidi_sys::{
    UInt16, UInt32, Byte, MIDITimeStamp,
    MIDIClientRef, MIDIEndpointRef, MIDIPortRef
};

use std::mem;

pub type MIDIReadProc =
    ::std::option::Option<extern "C" fn(pktlist: *const MIDIPacketList,
                                        readProcRefCon: *mut ::libc::c_void,
                                        srcConnRefCon: *mut ::libc::c_void)
                              -> ()>;

pub const MAX_PACKET_DATA_LENGTH: usize = 0xffffusize;

#[repr(C)]
#[repr(packed)]
#[derive(Copy)]
pub struct Struct_MIDIPacket {
    pub timeStamp: MIDITimeStamp,
    pub length: UInt16,
    pub data: [Byte; MAX_PACKET_DATA_LENGTH],
}
impl ::std::clone::Clone for Struct_MIDIPacket {
    fn clone(&self) -> Self { *self }
}
impl ::std::default::Default for Struct_MIDIPacket {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}

pub type MIDIPacket = Struct_MIDIPacket;

#[repr(C)]
#[repr(packed)]
#[derive(Copy)]
pub struct Struct_MIDIPacketList {
    pub numPackets: UInt32,
    pub packet: [MIDIPacket; 1usize],
}
impl ::std::clone::Clone for Struct_MIDIPacketList {
    fn clone(&self) -> Self { *self }
}
impl ::std::default::Default for Struct_MIDIPacketList {
    fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}

pub type MIDIPacketList = Struct_MIDIPacketList;

extern "C" {
    pub fn MIDISend(port: MIDIPortRef, dest: MIDIEndpointRef,
                    pktlist: *const MIDIPacketList) -> OSStatus;

    pub fn MIDIReceived(src: MIDIEndpointRef, pktlist: *const MIDIPacketList) -> OSStatus;

    pub fn MIDIInputPortCreate(client: MIDIClientRef, portName: CFStringRef,
                               readProc: MIDIReadProc,
                               refCon: *mut ::libc::c_void,
                               outPort: *mut MIDIPortRef) -> OSStatus;

    pub fn MIDIPacketListInit(pktlist: *mut MIDIPacketList) -> *mut MIDIPacket;
}

pub unsafe fn MIDIPacketNext(packet: *const MIDIPacket) -> *const MIDIPacket {
    let ptr = packet as *const u8;
    let offset = mem::size_of::<MIDITimeStamp>() as isize
        + mem::size_of::<UInt16>() as isize
        + (*packet).length as isize;
    ptr.offset(offset) as *const MIDIPacket
}
