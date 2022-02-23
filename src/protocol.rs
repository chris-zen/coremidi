use coremidi_sys::MIDIProtocolID;

/// The [MIDI Protocol](https://developer.apple.com/documentation/coremidi/midiprotocolid) to use for messages
///
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Protocol {
    /// MIDI 1.0
    Midi10,

    /// MIDI 2.0
    Midi20,

    /// Reserved for future protocols not supported by this crate yet
    Unsupported(MIDIProtocolID),
}

impl From<MIDIProtocolID> for Protocol {
    fn from(protocol_id: MIDIProtocolID) -> Self {
        match protocol_id as ::std::os::raw::c_uint {
            coremidi_sys::kMIDIProtocol_1_0 => Protocol::Midi10,
            coremidi_sys::kMIDIProtocol_2_0 => Protocol::Midi20,
            _ => Protocol::Unsupported(protocol_id),
        }
    }
}

impl From<Protocol> for MIDIProtocolID {
    fn from(protocol: Protocol) -> Self {
        match protocol {
            Protocol::Midi10 => coremidi_sys::kMIDIProtocol_1_0 as MIDIProtocolID,
            Protocol::Midi20 => coremidi_sys::kMIDIProtocol_2_0 as MIDIProtocolID,
            Protocol::Unsupported(protocol_id) => protocol_id,
        }
    }
}
