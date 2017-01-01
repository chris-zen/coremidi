use coremidi_sys::{
    MIDITimeStamp, UInt16
};

use coremidi_sys_ext::{
    MIDIPacketList, MIDIPacket, MIDIPacketNext
};

use std::ptr;
use std::fmt;

use PacketList;

pub struct Packet(*const MIDIPacket);

impl Packet {
    pub fn timestamp(&self) -> u64 {
        self.packet().timeStamp as u64
    }

    pub fn length(&self) -> usize {
        self.packet().length as usize
    }

    pub fn iter(&self) -> PacketIterator {
        PacketIterator {
            count: self.length(),
            data: &self.packet().data[0]
        }
    }

    fn packet(&self) -> &MIDIPacket {
        unsafe { &*self.0 }
    }
}

impl fmt::Debug for Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pkt = unsafe { *self.0 };
        let result = write!(f, "Packet(ptr={:x}, ts={:018x}, data=[",
                            self.0 as usize, pkt.timeStamp as u64);
        let indices = 0..(pkt.length as usize);
        let result = indices.fold(result, |prev_result, i| {
            match prev_result {
                Err(err) => Err(err),
                Ok(()) => {
                    let sep = if i > 0 { ", " } else { " " };
                    write!(f, "{}{:02x}", sep, pkt.data[i])
                }
            }
        });
        result.and_then(|_| write!(f, "])"))
    }
}

impl fmt::Display for Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pkt = unsafe { *self.0 };
        let result = write!(f, "{:018x}:", pkt.timeStamp as u64);
        let indices = 0..(pkt.length as usize);
        indices.fold(result, |prev_result, i| {
            match prev_result {
                Err(err) => Err(err),
                Ok(()) => write!(f, " {:02x}", pkt.data[i])
            }
        })
    }
}

pub struct PacketIterator {
    count: usize,
    data: *const u8
}

impl Iterator for PacketIterator {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        if self.count > 0 {
            let d: u8 = unsafe { *self.data };
            self.data = unsafe { self.data.offset(1) };
            self.count -= 1;
            Some(d)
        }
        else {
            None
        }
    }
}

impl PacketList {
    /// Build a PacketList with a single Packet from a timestamp and a vector of MIDI bytes.
    ///
    /// This is still in development so for now the amount of MIDI bytes is limited to 256.
    ///
    pub fn from_data(timestamp: u64, data: Vec<u8>) -> PacketList {
        let len = data.len();
        // TODO Allocate the packet list in the heap and remove the limit of 256 bytes.
        assert!(len < 256usize, "The maximum number of bytes supported per packet is 256");
        let mut packet = MIDIPacket {
            timeStamp: timestamp as MIDITimeStamp,
            length: len as UInt16,
            data: [0; 256]
        };
        packet.data[0..len].clone_from_slice(&data);
        PacketList(MIDIPacketList { numPackets: 1, packet: [packet]})
    }

    pub fn length(&self) -> usize {
        self.0.numPackets as usize
    }

    pub fn iter(&self) -> PacketListIterator {
        let packet_ptr: *const MIDIPacket = if self.0.numPackets > 0 {
            &self.0.packet[0]
        } else {
            ptr::null::<MIDIPacket>()
        };
        PacketListIterator {
            count: self.0.numPackets as usize,
            packet_ptr: packet_ptr
        }
    }
}

impl fmt::Debug for PacketList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let result = write!(f, "PacketList(ptr={:x}, packets=[", &self.0 as *const _ as usize);
        self.iter().fold(result, |prev_result, packet| {
            match prev_result {
                Err(err) => Err(err),
                Ok(()) => {
                    let sep = if prev_result != result { ", " } else { "" };
                    write!(f, "{}{:?}", sep, packet)
                }
            }
        }).and_then(|_| write!(f, "])"))
    }
}

impl fmt::Display for PacketList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let result = write!(f, "PacketList(len={})", self.0.numPackets);
        self.iter().fold(result, |prev_result, packet| {
            match prev_result {
                Err(err) => Err(err),
                Ok(()) => write!(f, "\n  {}", packet)
            }
        })
    }
}

pub struct PacketListIterator {
    count: usize,
    packet_ptr: *const MIDIPacket
}

impl Iterator for PacketListIterator {
    type Item = Packet;

    fn next(&mut self) -> Option<Packet> {
        if self.count > 0 {
            let packet = Packet(self.packet_ptr);
            self.count -= 1;
            self.packet_ptr = unsafe { MIDIPacketNext(self.packet_ptr) };
            Some(packet)
        }
        else {
            None
        }
    }
}
