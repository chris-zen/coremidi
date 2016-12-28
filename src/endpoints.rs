use core_foundation::string::{CFString, CFStringRef};
use core_foundation::base::TCFType;

use coremidi_sys::{
    MIDIGetDestination,
    MIDIObjectGetStringProperty,
    kMIDIPropertyDisplayName,
    MIDIEndpointRef,
    ItemCount
};

use std::mem;

use Destination;

impl Destination {
    fn from_index(index: usize) -> Destination {
        Destination {
            endpoint_ref: unsafe { MIDIGetDestination(index as ItemCount) }
        }
    }

    pub fn get_display_name(self: &Self) -> Option<String> {
        get_display_name(self.endpoint_ref)
    }
}

fn get_display_name(endpoint: MIDIEndpointRef) -> Option<String> {
    let mut display_name_ref: CFStringRef = unsafe { mem::uninitialized() };
    let status = unsafe { MIDIObjectGetStringProperty(
        endpoint,
        kMIDIPropertyDisplayName,
        &mut display_name_ref) };
    if status == 0 {
        let display_name: CFString = unsafe { TCFType::wrap_under_create_rule(display_name_ref) };
        Some(format!("{}", display_name))
    }
    else {
        None
    }
}

pub mod destinations {
    use Destination;

    use coremidi_sys::MIDIGetNumberOfDestinations;

    pub fn count() -> usize {
        unsafe { MIDIGetNumberOfDestinations() as usize }
    }

    pub fn from_index(index: usize) -> Destination {
        assert!(index < count(), "Index greater than the available number of destinations");
        Destination::from_index(index)
    }
}
