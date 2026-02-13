use std::fmt;
use std::marker::PhantomData;
use std::ops::Deref;
use std::slice;

use coremidi_sys::{
    MIDIPacket, MIDIPacketList, MIDIPacketListAdd, MIDIPacketListInit, MIDIPacketNext,
};

use crate::events::Storage;

pub use crate::events::Timestamp;

/// A [list of MIDI events](https://developer.apple.com/documentation/coremidi/midipacketlist) being received from, or being sent to, one endpoint.
///
pub struct PacketList(MIDIPacketList);

impl PacketList {
    /// For internal usage only.
    /// Requires this instance to actually point to a valid MIDIPacketList
    pub(crate) unsafe fn as_ptr(&self) -> *mut MIDIPacketList {
        self as *const PacketList as *mut PacketList as *mut MIDIPacketList
    }
}

impl PacketList {
    /// Check if the packet list is empty.
    ///
    pub fn is_empty(&self) -> bool {
        self.0.numPackets == 0
    }

    /// Get the number of packets in the list.
    ///
    pub fn len(&self) -> usize {
        self.0.numPackets as usize
    }

    /// Get an iterator for the packets in the list.
    ///
    pub fn iter(&self) -> PacketListIterator<'_> {
        PacketListIterator {
            count: self.len(),
            packet_ptr: std::ptr::addr_of!(self.0.packet) as *const MIDIPacket,
            _phantom: PhantomData,
        }
    }
}

impl fmt::Debug for PacketList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let result = write!(f, "PacketList(ptr={:x}, packets=[", unsafe {
            self.as_ptr() as usize
        });
        self.iter()
            .enumerate()
            .fold(result, |prev_result, (i, packet)| match prev_result {
                Err(err) => Err(err),
                Ok(()) => {
                    let sep = if i != 0 { ", " } else { "" };
                    write!(f, "{}{:?}", sep, packet)
                }
            })
            .and_then(|_| write!(f, "])"))
    }
}

impl fmt::Display for PacketList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let num_packets = self.len();
        let result = write!(f, "PacketList(len={})", num_packets);
        self.iter()
            .fold(result, |prev_result, packet| match prev_result {
                Err(err) => Err(err),
                Ok(()) => write!(f, "\n  {}", packet),
            })
    }
}

pub struct PacketListIterator<'a> {
    count: usize,
    packet_ptr: *const MIDIPacket,
    _phantom: ::std::marker::PhantomData<&'a Packet>,
}

impl<'a> Iterator for PacketListIterator<'a> {
    type Item = &'a Packet;

    fn next(&mut self) -> Option<&'a Packet> {
        if self.count > 0 {
            let packet = unsafe { &*(self.packet_ptr as *const Packet) };
            self.count -= 1;
            self.packet_ptr = unsafe { MIDIPacketNext(self.packet_ptr) };
            Some(packet)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count, Some(self.count))
    }
}

impl ExactSizeIterator for PacketListIterator<'_> {
    fn len(&self) -> usize {
        self.count
    }
}

impl<'a> IntoIterator for &'a PacketList {
    type Item = &'a Packet;
    type IntoIter = PacketListIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// A collection of simultaneous MIDI events.
/// See [MIDIPacket](https://developer.apple.com/documentation/coremidi/midipacket).
///
pub struct Packet(MIDIPacket);

impl Packet {
    /// Get the packet timestamp.
    ///
    pub fn timestamp(&self) -> Timestamp {
        self.0.timeStamp as Timestamp
    }

    /// Get the packet data. This method just gives raw MIDI bytes. You would need another
    /// library to decode them and work with higher level events.
    ///
    /// ```
    /// let packet_list = &coremidi::PacketBuffer::new(0, &[0x90, 0x40, 0x7f]);
    /// let data: Vec<u8> = packet_list.iter().map(|packet| packet.data().to_vec()).flatten().collect();
    /// assert_eq!(data, vec![0x90, 0x40, 0x7f])
    /// ```
    pub fn data(&self) -> &[u8] {
        let data_ptr = self.0.data.as_ptr();
        let data_len = self.0.length as usize;
        unsafe { slice::from_raw_parts(data_ptr, data_len) }
    }
}

impl fmt::Debug for Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let result = write!(
            f,
            "Packet(ptr={:x}, ts={:016x}, data=[",
            self as *const _ as usize,
            self.timestamp()
        );
        let result = self
            .data()
            .iter()
            .enumerate()
            .fold(result, |prev_result, (i, b)| match prev_result {
                Err(err) => Err(err),
                Ok(()) => {
                    let sep = if i > 0 { ", " } else { "" };
                    write!(f, "{}{:02x}", sep, b)
                }
            });
        result.and_then(|_| write!(f, "])"))
    }
}

impl fmt::Display for Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let result = write!(f, "{:016x}:", self.timestamp());
        self.data()
            .iter()
            .fold(result, |prev_result, b| match prev_result {
                Err(err) => Err(err),
                Ok(()) => write!(f, " {:02x}", b),
            })
    }
}

/// A mutable `PacketList` builder.
///
/// A `PacketList` is an immutable reference to a [MIDIPacketList](https://developer.apple.com/documentation/coremidi/midipacketlist) structure,
/// while a `PacketBuffer` is a mutable structure that allows to build a `PacketList` by adding packets.
/// It dereferences to a `PacketList`, so it can be used whenever a `PacketList` is needed.
///
pub struct PacketBuffer {
    storage: Storage,
    current_packet_offset: usize,
}

impl PacketBuffer {
    const PACKET_LIST_HEADER_SIZE: usize = 4; // MIDIPacketList::numPackets: UInt32
    const PACKET_HEADER_SIZE: usize = 8 +     // MIDIPacket::timeStamp: MIDITimeStamp/UInt64
            2; // MIDIPacket::length: UInt16

    /// Create a `PacketBuffer` with a single packet containing the provided timestamp and data.
    ///
    /// According to the official documentation for CoreMIDI, the timestamp represents
    /// the time at which the events are to be played, where zero means "now".
    /// The timestamp applies to the first MIDI byte in the packet.
    ///
    /// Example on how to create a `PacketBuffer` with a single packet for a MIDI note on for C-5:
    ///
    /// ```
    /// use coremidi::PacketBuffer;
    /// let buffer = PacketBuffer::new(0, &[0x90, 0x3c, 0x7f]);
    /// assert_eq!(buffer.len(), 1);
    /// assert_eq!(buffer.iter().next().map(|packet| packet.data().to_vec()), Some(vec![0x90, 0x3c, 0x7f]))
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the buffer capacity is insufficient for the packet (CoreMIDI returns null from `MIDIPacketListAdd`).
    pub fn new(timestamp: Timestamp, data: &[u8]) -> Self {
        let capacity = data.len() + Self::PACKET_LIST_HEADER_SIZE + Self::PACKET_HEADER_SIZE;
        let mut storage = Storage::with_capacity(capacity);
        let packet_list_ptr = unsafe { storage.as_mut_ptr::<MIDIPacketList>() };
        let current_packet_ptr = unsafe { MIDIPacketListInit(packet_list_ptr) };
        let current_packet_ptr = unsafe {
            MIDIPacketListAdd(
                packet_list_ptr,
                storage.capacity() as u64,
                current_packet_ptr,
                timestamp,
                data.len() as u64,
                data.as_ptr(),
            )
        };
        assert!(
            !current_packet_ptr.is_null(),
            "insufficient buffer capacity for packet"
        );
        let current_packet_offset = unsafe {
            (current_packet_ptr as *const u8).offset_from(packet_list_ptr as *const u8) as usize
        };

        Self {
            storage,
            current_packet_offset,
        }
    }

    /// Create an empty `PacketBuffer` with no packets.
    ///
    /// Example on how to create an empty `PacketBuffer`
    /// with a capacity for 128 bytes in total (including headers):
    ///
    /// ```
    /// let buffer = coremidi::PacketBuffer::with_capacity(128);
    /// assert_eq!(buffer.len(), 0);
    /// assert_eq!(buffer.capacity(), 128);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        let capacity = std::cmp::max(capacity, Storage::INLINE_SIZE);
        let mut storage = Storage::with_capacity(capacity);
        let packet_list_ptr = unsafe { storage.as_mut_ptr::<MIDIPacketList>() };
        let current_packet_ptr = unsafe { MIDIPacketListInit(packet_list_ptr) };
        let current_packet_offset =
            (current_packet_ptr as *const u8 as usize) - (packet_list_ptr as *const u8 as usize);

        Self {
            storage,
            current_packet_offset,
        }
    }

    /// Get underlying buffer capacity in bytes
    pub fn capacity(&self) -> usize {
        self.storage.capacity()
    }

    /// Add a new event containing the provided timestamp and data.
    ///
    /// According to the official documentation for CoreMIDI, the timestamp represents
    /// the time at which the events are to be played, where zero means "now".
    /// The timestamp applies to the first MIDI byte in the packet.
    ///
    /// An event must not have a timestamp that is smaller than that of a previous event
    /// in the same `PacketList`
    ///
    /// Example:
    ///
    /// ```
    /// let mut chord = coremidi::PacketBuffer::new(0, &[0x90, 0x3c, 0x7f]);
    /// chord.push_data(0, &[0x90, 0x40, 0x7f]);
    /// assert_eq!(chord.len(), 1);
    /// let repr = format!("{}", &chord as &coremidi::PacketList);
    /// assert_eq!(repr, "PacketList(len=1)\n  0000000000000000: 90 3c 7f 90 40 7f");
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the buffer capacity is insufficient for the packet (CoreMIDI returns null from `MIDIPacketListAdd`).
    pub fn push_data(&mut self, timestamp: Timestamp, data: &[u8]) -> &mut Self {
        self.ensure_capacity(data.len());

        let packet_list_ptr = unsafe { self.storage.as_mut_ptr::<MIDIPacketList>() };
        let current_packet_ptr = unsafe {
            self.storage.as_ptr::<u8>().add(self.current_packet_offset) as *mut MIDIPacket
        };

        let current_packet_ptr = unsafe {
            MIDIPacketListAdd(
                packet_list_ptr,
                self.storage.capacity() as u64,
                current_packet_ptr,
                timestamp,
                data.len() as u64,
                data.as_ptr(),
            )
        };
        assert!(
            !current_packet_ptr.is_null(),
            "insufficient buffer capacity for packet"
        );

        self.current_packet_offset = unsafe {
            (current_packet_ptr as *const u8).offset_from(packet_list_ptr as *const u8) as usize
        };

        self
    }

    /// Clears the buffer, removing all packets.
    /// Note that this method has no effect on the allocated capacity of the buffer.
    pub fn clear(&mut self) {
        let packet_list_ptr = unsafe { self.storage.as_mut_ptr::<MIDIPacketList>() };
        let current_packet_ptr = unsafe { MIDIPacketListInit(packet_list_ptr) };
        self.current_packet_offset = unsafe {
            (current_packet_ptr as *const u8).offset_from(packet_list_ptr as *const u8) as usize
        };
    }

    fn ensure_capacity(&mut self, data_len: usize) {
        let next_capacity = self.aligned_bytes_len() + Self::PACKET_HEADER_SIZE + data_len;

        // We ensure capacity for the worst case as if there was no merge with the current packet
        self.storage.ensure_capacity(next_capacity);
    }

    #[inline]
    fn aligned_bytes_len(&self) -> usize {
        if self.as_ref().is_empty() {
            self.current_packet_offset
        } else {
            let storage_start_ptr = unsafe { self.storage.as_ptr::<u8>() };
            let current_packet = unsafe {
                &*(storage_start_ptr.add(self.current_packet_offset) as *const MIDIPacket)
            };
            let current_packet_data_ptr = current_packet.data.as_ptr();
            let data_offset = current_packet_data_ptr as usize - storage_start_ptr as usize;
            let data_len = current_packet.length as usize;
            (data_offset + data_len + 3) & !3
        }
    }
}

impl AsRef<PacketList> for PacketBuffer {
    #[inline]
    fn as_ref(&self) -> &PacketList {
        unsafe { &*self.storage.as_ptr::<PacketList>() }
    }
}

impl Deref for PacketBuffer {
    type Target = PacketList;

    #[inline]
    fn deref(&self) -> &PacketList {
        self.as_ref()
    }
}

impl Clone for PacketBuffer {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            current_packet_offset: self.current_packet_offset,
        }
    }
}

impl fmt::Debug for PacketBuffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let packet_list: &PacketList = self.as_ref();
        f.debug_struct("PacketBuffer")
            .field("capacity", &self.capacity())
            .field("packets", &packet_list.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use coremidi_sys::{MIDIPacketList, MIDITimeStamp};

    #[test]
    pub fn single_packet_alloc_inline() {
        let packet_buf = PacketBuffer::new(42, &[0x90u8, 0x40, 0x7f]);
        if let Storage::External(_) = packet_buf.storage {
            panic!("A single 3-byte message must not be allocated externally")
        }
    }

    #[test]
    fn packet_buffer_deref() {
        let packet_buf = PacketBuffer::new(42, &[0x90u8, 0x40, 0x7f]);
        let packet_list: &PacketList = &packet_buf;
        assert_eq!(
            unsafe { packet_list.as_ptr() as *const MIDIPacketList },
            unsafe { packet_buf.storage.as_ptr::<MIDIPacketList>() },
        );
    }

    // FIXME
    #[test]
    fn packet_list_length() {
        let mut packet_buf = PacketBuffer::new(42, &[0x90u8, 0x40, 0x7f]);
        packet_buf.push_data(43, &[0x91u8, 0x40, 0x7f]);
        packet_buf.push_data(44, &[0x80u8, 0x40, 0x7f]);
        packet_buf.push_data(45, &[0x81u8, 0x40, 0x7f]);
        assert_eq!(packet_buf.len(), 4);
    }

    #[test]
    fn packet_buffer_empty_with_capacity() {
        let packet_buf = PacketBuffer::with_capacity(128);
        assert_eq!(packet_buf.capacity(), 128);
        assert_eq!(packet_buf.len(), 0);
    }

    #[test]
    fn packet_buffer_with_capacity_zero() {
        let packet_buf = PacketBuffer::with_capacity(0);
        assert_eq!(packet_buf.capacity(), Storage::INLINE_SIZE);
        assert_eq!(packet_buf.len(), 0);
    }

    #[test]
    fn packet_buffer_with_capacity() {
        let mut packet_buf = PacketBuffer::with_capacity(128);
        packet_buf.push_data(43, &[0x91u8, 0x40, 0x7f]);
        packet_buf.push_data(44, &[0x80u8, 0x40, 0x7f]);
        packet_buf.push_data(45, &[0x81u8, 0x40, 0x7f]);
        assert_eq!(packet_buf.capacity(), 128);
        assert_eq!(packet_buf.len(), 3);
    }

    // FIXME
    #[test]
    fn packet_buffer_clear() {
        let mut packet_buf = PacketBuffer::new(42, &[0x90u8, 0x40, 0x7f]);
        packet_buf.push_data(43, &[0x91u8, 0x40, 0x7f]);
        packet_buf.push_data(44, &[0x80u8, 0x40, 0x7f]);
        packet_buf.push_data(45, &[0x81u8, 0x40, 0x7f]);
        assert_eq!(packet_buf.len(), 4);
        packet_buf.clear();
        assert_eq!(packet_buf.len(), 0);
    }

    #[test]
    fn compare_equal_timestamps() {
        unsafe {
            compare_packet_list(vec![
                (42, vec![0x90, 0x40, 0x7f]),
                (42, vec![0x90, 0x41, 0x7f]),
                (42, vec![0x90, 0x42, 0x7f]),
            ])
        }
    }

    // FIXME
    #[test]
    fn compare_different_timestamps() {
        unsafe {
            compare_packet_list(vec![
                (42, vec![0x90, 0x40, 0x7f]),
                (43, vec![0x90, 0x40, 0x7f]),
                (44, vec![0x90, 0x40, 0x7f]),
            ])
        }
    }

    // FIXME
    #[test]
    fn compare_sysex_single() {
        unsafe {
            compare_packet_list(vec![
                (42, vec![0x90, 0x40, 0x7f]),
                (42, vec![0xF0, 0x01, 0x01, 0x01, 0x01, 0x01, 0xF7]), // sysex
                (42, vec![0x90, 0x41, 0x7f]),
            ])
        }
    }

    // FIXME
    #[test]
    fn compare_sysex_split1() {
        unsafe {
            compare_packet_list(vec![
                (42, vec![0x90, 0x40, 0x7f]),
                (42, vec![0xF0, 0x01, 0x01, 0x01, 0x01]), // sysex part 1
                (42, vec![0x01, 0xF7]),                   // sysex part 2
                (42, vec![0x90, 0x41, 0x7f]),
            ])
        }
    }

    #[test]
    fn compare_sysex_split2() {
        unsafe {
            compare_packet_list(vec![
                (42, vec![0x90, 0x40, 0x7f]),
                (42, vec![0xF0, 0x01, 0x01, 0x01, 0x01]), // sysex part 1
                (42, vec![0x01, 0x01, 0x01]),             // sysex part 2
                (42, vec![0x01, 0xF7]),                   // sysex part 3
                (42, vec![0x90, 0x41, 0x7f]),
            ])
        }
    }

    #[test]
    fn compare_sysex_malformed() {
        unsafe {
            compare_packet_list(vec![
                (42, vec![0x90, 0x40, 0x7f]),
                (42, vec![0xF0, 0x01, 0x01, 0x01, 0x01]), // sysex part 1
                (42, vec![0x01, 0x01, 0x01]),             // sysex part 2
                (42, vec![0x90, 0x41, 0x7f]),
            ])
        }
    }

    #[test]
    fn compare_sysex_long() {
        let mut sysex = vec![0xF0];
        sysex.resize(301, 0x01);
        sysex.push(0xF7);
        unsafe {
            compare_packet_list(vec![
                (42, vec![0x90, 0x40, 0x7f]),
                (43, vec![0x90, 0x41, 0x7f]),
                (43, sysex),
            ])
        }
    }

    #[test]
    fn packet_list_iter_exact_size() {
        let mut packet_buf = PacketBuffer::new(42, &[0x90, 0x40, 0x7f]);
        packet_buf.push_data(43, &[0x80, 0x40, 0x7f]);

        let mut iter = packet_buf.iter();
        assert_eq!(iter.len(), 2);
        iter.next();
        assert_eq!(iter.len(), 1);
        iter.next();
        assert_eq!(iter.len(), 0);
    }

    #[test]
    fn packet_list_into_iterator() {
        let packet_buf = PacketBuffer::new(42, &[0x90, 0x40, 0x7f]);
        let packet_list: &PacketList = &packet_buf;

        let data: Vec<Vec<u8>> = packet_list.into_iter().map(|p| p.data().to_vec()).collect();
        assert_eq!(data, vec![vec![0x90, 0x40, 0x7f]]);
    }

    #[test]
    fn packet_display() {
        let packet_buf = PacketBuffer::new(0, &[0x90, 0x3c, 0x7f]);
        let packet = packet_buf.iter().next().unwrap();
        let display = format!("{}", packet);
        assert_eq!(display, "0000000000000000: 90 3c 7f");
    }

    #[test]
    fn packet_list_display() {
        let packet_buf = PacketBuffer::new(0, &[0x90, 0x3c, 0x7f]);
        let display = format!("{}", &packet_buf as &PacketList);
        assert!(display.starts_with("PacketList(len=1)"));
        assert!(display.contains("90 3c 7f"));
    }

    #[test]
    fn packet_buffer_clone() {
        let mut original = PacketBuffer::new(42, &[0x90, 0x40, 0x7f]);
        original.push_data(43, &[0x80, 0x40, 0x7f]);

        let cloned = original.clone();

        assert_eq!(cloned.len(), original.len());
        assert_eq!(cloned.capacity(), original.capacity());
        for (o, c) in original.iter().zip(cloned.iter()) {
            assert_eq!(o.timestamp(), c.timestamp());
            assert_eq!(o.data(), c.data());
        }
    }

    #[test]
    fn packet_buffer_debug() {
        let packet_buf = PacketBuffer::new(42, &[0x90, 0x40, 0x7f]);
        let debug = format!("{:?}", packet_buf);
        assert!(debug.contains("PacketBuffer"));
        assert!(debug.contains("capacity"));
        assert!(debug.contains("packets: 1"));
    }

    #[test]
    fn packet_buffer_empty_iter_exact_size() {
        let packet_buf = PacketBuffer::with_capacity(64);
        assert_eq!(packet_buf.iter().len(), 0);
        assert!(packet_buf.iter().next().is_none());
    }

    /// Compares the results of building a PacketList using our PacketBuffer API
    /// and the native API (MIDIPacketListAdd, etc).
    unsafe fn compare_packet_list(packets: Vec<(MIDITimeStamp, Vec<u8>)>) {
        // allocate a buffer on the stack for building the list using native methods
        const BUFFER_SIZE: usize = 65536; // maximum allowed size
        let buffer: &mut [u8] = &mut [0; BUFFER_SIZE];
        let pkt_list_ptr = buffer.as_mut_ptr() as *mut MIDIPacketList;

        // build the list
        let mut pkt_ptr = MIDIPacketListInit(pkt_list_ptr);
        for pkt in &packets {
            pkt_ptr = MIDIPacketListAdd(
                pkt_list_ptr,
                BUFFER_SIZE as u64,
                pkt_ptr,
                pkt.0,
                pkt.1.len() as u64,
                pkt.1.as_ptr(),
            );
            assert!(!pkt_ptr.is_null());
        }
        let list_native = &*(pkt_list_ptr as *const _ as *const PacketList);

        // build the PacketBuffer, containing the same packets
        let mut packet_buf = PacketBuffer::new(packets[0].0, &packets[0].1);
        for pkt in &packets[1..] {
            packet_buf.push_data(pkt.0, &pkt.1);
        }

        // print buffer contents for debugging purposes
        let packet_buf_slice = packet_buf.storage.get_slice::<u8>();
        println!(
            "native: {:?}",
            buffer[0..packet_buf_slice.len()]
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<String>>()
                .join(" ")
        );
        println!(
            "buffer: {:?}",
            packet_buf_slice
                .iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<String>>()
                .join(" ")
        );

        let list: &PacketList = &packet_buf;

        // check if the contents match
        assert_eq!(
            list_native.len(),
            list.len(),
            "PacketList lengths must match"
        );
        for (n, p) in list_native.iter().zip(list.iter()) {
            assert_eq!(n.data(), p.data());
        }
    }
}
