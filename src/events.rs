use crate::protocol::Protocol;
use coremidi_sys::{MIDIEventList, MIDIEventPacket, MIDIEventPacketNext};
use std::marker::PhantomData;
use std::slice;

pub type Timestamp = u64;

/// A variable-length list of MIDI event packets
/// See [MIDIEventList](https://developer.apple.com/documentation/coremidi/midieventlist)
///
pub struct EventList(MIDIEventList);

impl EventList {
    pub fn protocol(&self) -> Protocol {
        Protocol::from(self.0.protocol)
    }

    /// Check if the packet list is empty.
    ///
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the number of packets in the list.
    ///
    pub fn len(&self) -> usize {
        self.0.numPackets as usize
    }

    /// Get an iterator for the packets in the list.
    ///
    pub fn iter(&self) -> EventListIter {
        EventListIter {
            count: self.len(),
            packet_ptr: std::ptr::addr_of!(self.0.packet) as *const MIDIEventPacket,
            _phantom: PhantomData,
        }
    }
}

// TODO impl Debug for EventList
// TODO impl Display for EventList

pub struct EventListIter<'a> {
    count: usize,
    packet_ptr: *const MIDIEventPacket,
    _phantom: PhantomData<&'a MIDIEventPacket>,
}

impl<'a> Iterator for EventListIter<'a> {
    type Item = &'a EventPacket;

    fn next(&mut self) -> Option<&'a EventPacket> {
        if self.count > 0 {
            let packet = unsafe { &*(self.packet_ptr as *const EventPacket) };
            self.count -= 1;
            self.packet_ptr = unsafe { MIDIEventPacketNext(self.packet_ptr) };
            Some(packet)
        } else {
            None
        }
    }
}

pub struct EventPacket(MIDIEventPacket);

impl EventPacket {
    pub fn timestamp(&self) -> Timestamp {
        self.0.timeStamp as Timestamp
    }

    /// Get the packet data. This method just gives raw MIDI words. You would need another
    /// library to decode them and work with higher level events.
    ///
    pub fn data(&self) -> &[u32] {
        let data_ptr = self.0.words.as_ptr();
        let data_len = self.0.wordCount as usize;
        unsafe { slice::from_raw_parts(data_ptr, data_len) }
    }
}

// TODO impl Debug for EventPacket
// TODO impl Display for EventPacket

#[cfg(test)]
mod tests {
    // TODO
}
