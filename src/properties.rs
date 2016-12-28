use core_foundation::string::{CFString, CFStringRef};
use core_foundation::base::TCFType;

use coremidi_sys::{
    MIDIObjectRef, MIDIObjectGetStringProperty,
    kMIDIPropertyDisplayName
};

use std::mem;

pub fn get_display_name(object_ref: MIDIObjectRef) -> Option<String> {
    let mut display_name_ref: CFStringRef = unsafe { mem::uninitialized() };
    let status = unsafe { MIDIObjectGetStringProperty(
        object_ref,
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
