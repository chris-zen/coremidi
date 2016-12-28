use coremidi_sys::{
    MIDIPacketList, MIDIPacket, MIDITimeStamp, UInt16
};

use PacketList;

impl PacketList {
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
}
