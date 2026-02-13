use std::fmt::Formatter;

use coremidi_sys::MIDIProtocolID;

/// The [MIDI Protocol](https://developer.apple.com/documentation/coremidi/midiprotocolid) to use for messages
///
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    /// MIDI 1.0
    Midi10,

    /// MIDI 2.0
    Midi20,

    /// Reserved for future protocols not known by this crate yet
    /// Please don't use it, unless really needed.
    Unknown(MIDIProtocolID),
}

impl From<MIDIProtocolID> for Protocol {
    fn from(protocol_id: MIDIProtocolID) -> Self {
        match protocol_id as ::std::os::raw::c_uint {
            coremidi_sys::kMIDIProtocol_1_0 => Protocol::Midi10,
            coremidi_sys::kMIDIProtocol_2_0 => Protocol::Midi20,
            _ => Protocol::Unknown(protocol_id),
        }
    }
}

impl From<Protocol> for MIDIProtocolID {
    fn from(protocol: Protocol) -> Self {
        match protocol {
            Protocol::Midi10 => coremidi_sys::kMIDIProtocol_1_0 as MIDIProtocolID,
            Protocol::Midi20 => coremidi_sys::kMIDIProtocol_2_0 as MIDIProtocolID,
            Protocol::Unknown(protocol_id) => protocol_id,
        }
    }
}

impl std::fmt::Debug for Protocol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Midi10 => write!(f, "MIDI 1.0"),
            Self::Midi20 => write!(f, "MIDI 2.0"),
            Self::Unknown(protocol_id) => write!(f, "Unknown({})", protocol_id),
        }
    }
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_midi_protocol_id_known() {
        assert_eq!(
            Protocol::from(coremidi_sys::kMIDIProtocol_1_0 as MIDIProtocolID),
            Protocol::Midi10
        );
        assert_eq!(
            Protocol::from(coremidi_sys::kMIDIProtocol_2_0 as MIDIProtocolID),
            Protocol::Midi20
        );
    }

    #[test]
    fn from_midi_protocol_id_unknown() {
        let unknown = Protocol::from(0xFFFF as MIDIProtocolID);
        assert_eq!(unknown, Protocol::Unknown(0xFFFF));
    }

    #[test]
    fn into_midi_protocol_id_roundtrip() {
        let id_10: MIDIProtocolID = Protocol::Midi10.into();
        assert_eq!(Protocol::from(id_10), Protocol::Midi10);

        let id_20: MIDIProtocolID = Protocol::Midi20.into();
        assert_eq!(Protocol::from(id_20), Protocol::Midi20);

        let id_unknown: MIDIProtocolID = Protocol::Unknown(0xBEEF).into();
        assert_eq!(id_unknown, 0xBEEF);
    }

    #[test]
    fn debug_format() {
        assert_eq!(format!("{:?}", Protocol::Midi10), "MIDI 1.0");
        assert_eq!(format!("{:?}", Protocol::Midi20), "MIDI 2.0");
        assert_eq!(format!("{:?}", Protocol::Unknown(99)), "Unknown(99)");
    }

    #[test]
    fn display_format() {
        assert_eq!(format!("{}", Protocol::Midi10), "MIDI 1.0");
        assert_eq!(format!("{}", Protocol::Midi20), "MIDI 2.0");
    }

    #[test]
    fn clone_and_copy() {
        let p = Protocol::Midi20;
        let p2 = p;
        assert_eq!(p, p2);
    }

    #[test]
    fn equality() {
        assert_eq!(Protocol::Midi10, Protocol::Midi10);
        assert_ne!(Protocol::Midi10, Protocol::Midi20);
        assert_ne!(Protocol::Unknown(1), Protocol::Unknown(2));
        assert_eq!(Protocol::Unknown(1), Protocol::Unknown(1));
    }
}
