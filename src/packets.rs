use coremidi_sys::{MIDIPacket, MIDIPacketNext, MIDITimeStamp};
use coremidi_sys::{MIDIPacketList, MIDIPacketListAdd, MIDIPacketListInit};

use std::fmt;
use std::mem::size_of;
use std::ops::Deref;
use std::slice;

pub type Timestamp = u64;

#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
pub mod alignment {
    pub type Marker = [u32; 0]; // ensures 4-byte alignment (on ARM)
    pub const NEEDS_ALIGNMENT: bool = true;
}

#[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
pub mod alignment {
    pub type Marker = [u8; 0]; // unaligned
    pub const NEEDS_ALIGNMENT: bool = false;
}

/// A collection of simultaneous MIDI events.
/// See [MIDIPacket](https://developer.apple.com/reference/coremidi/midipacket).
///
#[repr(C)]
pub struct Packet {
    // NOTE: At runtime this type must only be used behind immutable references
    //       that point to valid instances of MIDIPacket (mutable references would allow mem::swap).
    //       This type must NOT implement `Copy`!
    //       On ARM, this must be 4-byte aligned.
    inner: PacketInner,
    _alignment_marker: alignment::Marker,
}

#[repr(packed)]
struct PacketInner {
    timestamp: MIDITimeStamp,
    length: u16,
    data: [u8; 0], // zero-length, because we cannot make this type bigger without knowing how much data there actually is
}

impl Packet {
    /// Get the packet timestamp.
    ///
    pub fn timestamp(&self) -> Timestamp {
        self.inner.timestamp as Timestamp
    }

    /// Get the packet data. This method just gives raw MIDI bytes. You would need another
    /// library to decode them and work with higher level events.
    ///
    ///
    /// The following example:
    ///
    /// ```
    /// let packet_list = &coremidi::PacketBuffer::new(0, &[0x90, 0x40, 0x7f]);
    /// for packet in packet_list.iter() {
    ///   for byte in packet.data() {
    ///     print!(" {:x}", byte);
    ///   }
    /// }
    /// ```
    ///
    /// will print:
    ///
    /// ```text
    ///  90 40 7f
    /// ```
    pub fn data(&self) -> &[u8] {
        let data_ptr = self.inner.data.as_ptr();
        let data_len = self.inner.length as usize;
        unsafe { slice::from_raw_parts(data_ptr, data_len) }
    }
}

impl fmt::Debug for Packet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let result = write!(
            f,
            "Packet(ptr={:x}, ts={:016x}, data=[",
            self as *const _ as usize,
            self.timestamp() as u64
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

/// A [list of MIDI events](https://developer.apple.com/reference/coremidi/midipacketlist) being received from, or being sent to, one endpoint.
///
#[repr(C)]
pub struct PacketList {
    // NOTE: This type must only exist in the form of immutable references
    //       pointing to valid instances of MIDIPacketList.
    //       This type must NOT implement `Copy`!
    inner: PacketListInner,
    _do_not_construct: alignment::Marker,
}

#[repr(packed)]
struct PacketListInner {
    num_packets: u32,
    data: [MIDIPacket; 0],
}

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
        self.len() == 0
    }

    /// Get the number of packets in the list.
    ///
    pub fn len(&self) -> usize {
        self.inner.num_packets as usize
    }

    /// Get an iterator for the packets in the list.
    ///
    pub fn iter(&self) -> PacketListIterator {
        PacketListIterator {
            count: self.len(),
            packet_ptr: std::ptr::addr_of!(self.inner.data) as *const MIDIPacket,
            _phantom: ::std::marker::PhantomData::default(),
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
        let num_packets = self.inner.num_packets;
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
}

/// A mutable `PacketList` builder.
///
/// A `PacketList` is an inmmutable reference to a [MIDIPacketList](https://developer.apple.com/reference/coremidi/midipacketlist) structure,
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
    /// let buffer = coremidi::PacketBuffer::new(0, &[0x90, 0x3c, 0x7f]);
    /// assert_eq!(buffer.len(), 1)
    /// ```
    pub fn new(time: MIDITimeStamp, data: &[u8]) -> Self {
        let capacity = data.len() + Self::PACKET_LIST_HEADER_SIZE + Self::PACKET_HEADER_SIZE;
        let mut storage = Storage::with_capacity(capacity);
        let packet_list_ptr = unsafe { storage.as_mut_ptr::<MIDIPacketList>() };
        let current_packet_ptr = unsafe { MIDIPacketListInit(packet_list_ptr) };
        let current_packet_ptr = unsafe {
            MIDIPacketListAdd(
                packet_list_ptr,
                storage.capacity() as u64,
                current_packet_ptr,
                time,
                data.len() as u64,
                data.as_ptr(),
            )
        };
        let current_packet_offset =
            (current_packet_ptr as *const u8 as usize) - (packet_list_ptr as *const u8 as usize);
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
        let packet_list_ptr = unsafe { storage.as_mut_ptr() };
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
    pub fn push_data(&mut self, time: MIDITimeStamp, data: &[u8]) -> &mut Self {
        let packet_size = Self::packet_size(data.len());
        let next_packet_offset = self.next_packet_offset();
        unsafe {
            // We ensure capacity for the worst case as if there was no merge with the current packet
            self.storage
                .ensure_capacity(next_packet_offset + packet_size);
        }
        let packet_list_ptr = unsafe { self.storage.as_mut_ptr::<MIDIPacketList>() };
        let current_packet_ptr = unsafe {
            (packet_list_ptr as *const u8).add(self.current_packet_offset) as *mut MIDIPacket
        };
        let current_packet_ptr = unsafe {
            MIDIPacketListAdd(
                packet_list_ptr,
                self.storage.capacity() as u64,
                current_packet_ptr,
                time,
                data.len() as u64,
                data.as_ptr(),
            )
        };
        self.current_packet_offset =
            (current_packet_ptr as *const u8 as usize) - (packet_list_ptr as *const u8 as usize);

        self
    }

    /// Clears the buffer, removing all packets.
    /// Note that this method has no effect on the allocated capacity of the buffer.
    pub fn clear(&mut self) {
        unsafe {
            self.as_mut_ref().inner.num_packets = 0;
        }
        self.current_packet_offset = Self::PACKET_LIST_HEADER_SIZE;
    }

    #[inline]
    fn last_packet(&self) -> &Packet {
        assert!(!self.as_ref().is_empty());
        let packets_slice = self.storage.get_slice::<u8>();
        let packet_slot = &packets_slice[self.current_packet_offset..];
        unsafe { &*(packet_slot.as_ptr() as *const Packet) }
    }

    #[inline]
    fn next_packet_offset(&self) -> usize {
        if self.as_ref().is_empty() {
            self.current_packet_offset
        } else {
            let data_len = self.last_packet().inner.length as usize;
            let next_offset = self.current_packet_offset + Self::packet_size(data_len);
            if alignment::NEEDS_ALIGNMENT {
                (next_offset + 3) & !(3usize)
            } else {
                next_offset
            }
        }
    }

    #[inline]
    fn packet_size(data_len: usize) -> usize {
        Self::PACKET_HEADER_SIZE + data_len
    }

    #[inline]
    unsafe fn as_mut_ref(&mut self) -> &mut PacketList {
        &mut *self.storage.as_mut_ptr::<PacketList>()
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

pub(crate) enum Storage {
    /// Inline stores the data directly on the stack, if it is small enough.
    /// NOTE: using u32 ensures correct alignment (required on ARM)
    Inline([u32; Storage::INLINE_SIZE / 4]),
    /// External is used whenever the size of the data exceeds INLINE_PACKET_BUFFER_SIZE.
    /// This means that the size of the contained vector is always greater than INLINE_PACKET_BUFFER_SIZE.
    External(Vec<u32>),
}

impl Storage {
    pub(crate) const INLINE_SIZE: usize = (size_of::<Vec<u32>>() + 3) & !(3usize); // must be divisible by 4

    #[inline]
    #[allow(clippy::uninit_vec)]
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        if capacity <= Self::INLINE_SIZE {
            Self::Inline([0; Self::INLINE_SIZE / 4])
        } else {
            let u32_len = ((capacity - 1) / 4) + 1;
            let mut buffer = Vec::with_capacity(u32_len);
            unsafe {
                buffer.set_len(u32_len);
            }
            Storage::External(buffer)
        }
    }

    #[inline]
    pub(crate) fn capacity(&self) -> usize {
        match *self {
            Storage::Inline(ref inline) => inline.len() * 4,
            Storage::External(ref vec) => vec.len() * 4,
        }
    }

    #[inline]
    pub(crate) fn get_slice<T>(&self) -> &[T] {
        unsafe {
            match *self {
                Storage::Inline(ref inline) => slice::from_raw_parts(
                    inline.as_ptr() as *const T,
                    inline.len() * 4 / size_of::<T>(),
                ),
                Storage::External(ref vec) => {
                    slice::from_raw_parts(vec.as_ptr() as *const T, vec.len() * 4 / size_of::<T>())
                }
            }
        }
    }

    #[inline]
    pub(crate) fn get_slice_mut<T>(&mut self) -> &mut [T] {
        unsafe {
            match *self {
                Storage::Inline(ref mut inline) => slice::from_raw_parts_mut(
                    inline.as_mut_ptr() as *mut T,
                    inline.len() * 4 / size_of::<T>(),
                ),
                Storage::External(ref mut vec) => slice::from_raw_parts_mut(
                    vec.as_mut_ptr() as *mut T,
                    vec.len() * 4 / size_of::<T>(),
                ),
            }
        }
    }

    /// Call this only with larger length values (won't make the buffer smaller)
    #[allow(clippy::uninit_vec)]
    pub(crate) unsafe fn ensure_capacity(&mut self, capacity: usize) {
        if capacity < Self::INLINE_SIZE || capacity < self.get_slice::<u8>().len() {
            return;
        }

        let vec_capacity = ((capacity - 1) / 4) + 1;
        let vec: Option<Vec<u32>> = match *self {
            Storage::Inline(ref inline) => {
                let mut v = Vec::with_capacity(vec_capacity);
                v.extend_from_slice(inline);
                v.set_len(vec_capacity);
                Some(v)
            }
            Storage::External(ref mut vec) => {
                let current_len = vec.len();
                vec.reserve(vec_capacity - current_len);
                vec.set_len(vec_capacity);
                None
            }
        };

        // to prevent borrow-check errors, this must come after the `match`
        if let Some(v) = vec {
            *self = Storage::External(v);
        }
    }

    #[inline]
    pub(crate) unsafe fn as_ptr<T>(&self) -> *const T {
        self.get_slice().as_ptr() as *const T
    }

    #[inline]
    pub(crate) unsafe fn as_mut_ptr<T>(&mut self) -> *mut T {
        self.get_slice_mut().as_ptr() as *const T as *mut T
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use coremidi_sys::{MIDIPacketList, MIDITimeStamp};
    use std::mem;

    #[test]
    pub fn packet_struct_layout() {
        let expected_align = if super::alignment::NEEDS_ALIGNMENT {
            4
        } else {
            1
        };
        assert_eq!(expected_align, mem::align_of::<Packet>());
        assert_eq!(expected_align, mem::align_of::<PacketList>());

        let dummy_packet: Packet = unsafe { mem::zeroed() };
        let ptr = &dummy_packet as *const _ as *const u8;
        assert_eq!(
            PacketBuffer::PACKET_HEADER_SIZE,
            dummy_packet.inner.data.as_ptr() as usize - ptr as usize
        );

        let dummy_packet_list: PacketList = unsafe { mem::zeroed() };
        let ptr = &dummy_packet_list as *const _ as *const u8;
        assert_eq!(
            PacketBuffer::PACKET_LIST_HEADER_SIZE,
            std::ptr::addr_of!(dummy_packet_list.inner.data) as usize - ptr as usize
        );
    }

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
            packet_buf.storage.get_slice().as_ptr() as *const _ as *const MIDIPacketList
        );
    }

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
