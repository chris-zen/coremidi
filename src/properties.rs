use core_foundation::string::{CFString, CFStringRef};
use core_foundation::base::{TCFType, OSStatus};
use core_foundation::base::{CFGetRetainCount, CFTypeRef, CFIndex};

use coremidi_sys::*;

use std::mem;

use {
    Object,
    result_from_status,
    unit_result_from_status,
};

pub trait PropertyGetter<T> {
    fn value_from(&self, object: &Object) -> Result<T, OSStatus>;
}

pub trait PropertySetter<T> {
    fn set_value(&self, object: &Object, value: T) -> Result<(), OSStatus>;
}

/// Because Property structs can be constructed from strings that have been
/// passed in from the user or are constants CFStringRefs from CoreMidi, we 
/// need to abstract over how we store their keys.
enum PropertyKeyStorage {
    Owned(CFString),
    Constant(CFStringRef)
}

impl PropertyKeyStorage {
    /// Return a raw CFStringRef pointing to this property key
    fn as_string_ref(&self) -> CFStringRef {
        match self {
            PropertyKeyStorage::Owned(owned) => owned.as_concrete_TypeRef(),
            PropertyKeyStorage::Constant(constant) => *constant,
        }
    }

    /// For checking the retain count when debugging
    #[allow(dead_code)]
    fn retain_count(&self) -> CFIndex {
        match self {
            PropertyKeyStorage::Owned(owned) => owned.retain_count(),
            PropertyKeyStorage::Constant(constant) => unsafe { CFGetRetainCount(*constant as CFTypeRef) },
        }
    }
}

/// A MIDI object property which value is an String
///
pub struct StringProperty(PropertyKeyStorage);

impl StringProperty {
    pub fn new(name: &str) -> Self {
        StringProperty(PropertyKeyStorage::Owned(CFString::new(name)))
    }

    /// Note: Should only be used internally with predefined CoreMidi constants,
    /// since it does not bump the retain count of the CFStringRef.
    fn from_constant_string_ref(string_ref: CFStringRef) -> Self {
        StringProperty(PropertyKeyStorage::Constant(string_ref))
    }
}

impl<T> PropertyGetter<T> for StringProperty where T: From<String> {
    fn value_from(&self, object: &Object) -> Result<T, OSStatus> {
        let property_key = self.0.as_string_ref();
        let mut string_ref: CFStringRef = unsafe { 
            mem::uninitialized()
        };
        let status = unsafe {
            MIDIObjectGetStringProperty(object.0, property_key, &mut string_ref)
        };
        result_from_status(status, || {
            let string: CFString = unsafe {
                TCFType::wrap_under_create_rule(string_ref)
            };
            string.to_string().into()
        })
    }
}

impl<'a, T> PropertySetter<T> for StringProperty where T: Into<String> {
    fn set_value(&self, object: &Object, value: T) -> Result<(), OSStatus> {
        let property_key = self.0.as_string_ref();
        let value: String = value.into();
        let string = CFString::new(&value);
        let string_ref = string.as_concrete_TypeRef();
        let status = unsafe {
            MIDIObjectSetStringProperty(object.0, property_key, string_ref)
        };
        unit_result_from_status(status)
    }
}

/// A MIDI object property which value is an Integer
///
pub struct IntegerProperty(CFStringRef);

impl IntegerProperty {
    pub fn new(name: &str) -> Self {
        IntegerProperty(CFString::new(name).as_concrete_TypeRef())
    }
}

impl<T> PropertyGetter<T> for IntegerProperty where T: From<SInt32> {
    fn value_from(&self, object: &Object) -> Result<T, OSStatus> {
        unsafe {
            let mut value: SInt32 = mem::uninitialized();
            let status = MIDIObjectGetIntegerProperty(object.0, self.0, &mut value);
            if status == 0 { Ok(From::from(value)) } else { Err(status) }
        }
    }
}

impl <T> PropertySetter<T> for IntegerProperty where T: Into<SInt32> {
    fn set_value(&self, object: &Object, value: T) -> Result<(), OSStatus> {
        unsafe {
            let status = MIDIObjectSetIntegerProperty(object.0, self.0, value.into());
            if status == 0 { Ok(()) } else { Err(status) }
        }
    }
}

/// A MIDI object property which value is a Boolean
///
pub struct BooleanProperty(CFStringRef);

impl BooleanProperty {
    pub fn new(name: &str) -> Self {
        BooleanProperty(CFString::new(name).as_concrete_TypeRef())
    }
}

impl<T> PropertyGetter<T> for BooleanProperty where T: From<bool> {
    fn value_from(&self, object: &Object) -> Result<T, OSStatus> {
        unsafe {
            let mut value: SInt32 = mem::uninitialized();
            let status = MIDIObjectGetIntegerProperty(object.0, self.0, &mut value);
            if status == 0 { Ok(From::from(value == 1)) } else { Err(status) }
        }
    }
}

impl<T> PropertySetter<T> for BooleanProperty where T: Into<bool> {
    fn set_value(&self, object: &Object, value: T) -> Result<(), OSStatus> {
        unsafe {
            let value: SInt32 = if value.into() { 1 } else { 0 };
            let status = MIDIObjectSetIntegerProperty(object.0, self.0, value);
            if status == 0 { Ok(()) } else { Err(status) }
        }
    }
}

/// The set of properties that might be available for MIDI objects.
///
pub struct Properties;

impl Properties {
    /// See [kMIDIPropertyName](https://developer.apple.com/reference/coremidi/kmidipropertyname)
    pub fn name() -> StringProperty {
        StringProperty::from_constant_string_ref(unsafe { kMIDIPropertyName })
    }
    
    /// See [kMIDIPropertyManufacturer](https://developer.apple.com/reference/coremidi/kmidipropertymanufacturer)
    pub fn manufacturer() -> StringProperty {
        StringProperty::from_constant_string_ref(unsafe { kMIDIPropertyManufacturer }) 
    }
    
    /// See [kMIDIPropertyModel](https://developer.apple.com/reference/coremidi/kmidipropertymodel)
    pub fn model() -> StringProperty {
        StringProperty::from_constant_string_ref(unsafe { kMIDIPropertyModel })
    }
    
    /// See [kMIDIPropertyUniqueID](https://developer.apple.com/reference/coremidi/kmidipropertyuniqueid)
    pub fn unique_id()          -> IntegerProperty { unsafe { IntegerProperty(kMIDIPropertyUniqueID) } }
    /// See [kMIDIPropertyDeviceID](https://developer.apple.com/reference/coremidi/kmidipropertydeviceid)
    pub fn device_id()          -> IntegerProperty { unsafe { IntegerProperty(kMIDIPropertyDeviceID) } }
    /// See [kMIDIPropertyReceiveChannels](https://developer.apple.com/reference/coremidi/kmidipropertyreceivechannels)
    pub fn receive_channels()   -> IntegerProperty { unsafe { IntegerProperty(kMIDIPropertyReceiveChannels) } }
    /// See [kMIDIPropertyTransmitChannels](https://developer.apple.com/reference/coremidi/kmidipropertytransmitchannels)
    pub fn transmit_channels()  -> IntegerProperty { unsafe { IntegerProperty(kMIDIPropertyTransmitChannels) } }
    /// See [kMIDIPropertyMaxSysExSpeed](https://developer.apple.com/reference/coremidi/kmidipropertymaxsysexspeed)
    pub fn max_sysex_speed()    -> IntegerProperty { unsafe { IntegerProperty(kMIDIPropertyMaxSysExSpeed) } }
    /// See [kMIDIPropertyAdvanceScheduleTimeMuSec](https://developer.apple.com/reference/coremidi/kMIDIPropertyAdvanceScheduleTimeMuSec)
    pub fn advance_schedule_time_musec() -> IntegerProperty { unsafe { IntegerProperty(kMIDIPropertyAdvanceScheduleTimeMuSec) } }
    /// See [kMIDIPropertyIsEmbeddedEntity](https://developer.apple.com/reference/coremidi/kMIDIPropertyIsEmbeddedEntity)
    pub fn is_embedded_entity() -> BooleanProperty { unsafe { BooleanProperty(kMIDIPropertyIsEmbeddedEntity) } }
    /// See [kMIDIPropertyIsBroadcast](https://developer.apple.com/reference/coremidi/kMIDIPropertyIsBroadcast)
    pub fn is_broadcast()       -> BooleanProperty { unsafe { BooleanProperty(kMIDIPropertyIsBroadcast) } }
    /// See [kMIDIPropertySingleRealtimeEntity](https://developer.apple.com/reference/coremidi/kMIDIPropertySingleRealtimeEntity)
    pub fn single_realtime_entity() -> IntegerProperty { unsafe { IntegerProperty(kMIDIPropertySingleRealtimeEntity) } }
    /// See [kMIDIPropertyConnectionUniqueID](https://developer.apple.com/reference/coremidi/kMIDIPropertyConnectionUniqueID)
    pub fn connection_unique_id() -> IntegerProperty { unsafe { IntegerProperty(kMIDIPropertyConnectionUniqueID) } }
    /// See [kMIDIPropertyOffline](https://developer.apple.com/reference/coremidi/kMIDIPropertyOffline)
    pub fn offline()            -> BooleanProperty { unsafe { BooleanProperty(kMIDIPropertyOffline) } }
    /// See [kMIDIPropertyPrivate](https://developer.apple.com/reference/coremidi/kMIDIPropertyPrivate)
    pub fn private()            -> BooleanProperty { unsafe { BooleanProperty(kMIDIPropertyPrivate) } }
    /// See [kMIDIPropertyDriverOwner](https://developer.apple.com/reference/coremidi/kMIDIPropertyDriverOwner)
    pub fn driver_owner() -> StringProperty {
        StringProperty::from_constant_string_ref(unsafe { kMIDIPropertyDriverOwner })
    }
    
    // /// See [kMIDIPropertyNameConfiguration](https://developer.apple.com/reference/coremidi/kMIDIPropertyNameConfiguration)
    // pub fn name_configuration() -> Property { unsafe { Property(kMIDIPropertyNameConfiguration) } }
    // /// See [kMIDIPropertyImage](https://developer.apple.com/reference/coremidi/kMIDIPropertyImage)
    // pub fn image() -> Property { unsafe { Property(kMIDIPropertyImage) } }
    /// See [kMIDIPropertyDriverVersion](https://developer.apple.com/reference/coremidi/kMIDIPropertyDriverVersion)
    pub fn driver_version()     -> IntegerProperty { unsafe { IntegerProperty(kMIDIPropertyDriverVersion) } }
    /// See [kMIDIPropertySupportsGeneralMIDI](https://developer.apple.com/reference/coremidi/kMIDIPropertySupportsGeneralMIDI)
    pub fn supports_general_midi() -> BooleanProperty { unsafe { BooleanProperty(kMIDIPropertySupportsGeneralMIDI) } }
    /// See [kMIDIPropertySupportsMMC](https://developer.apple.com/reference/coremidi/kMIDIPropertySupportsMMC)
    pub fn supports_mmc()       -> BooleanProperty { unsafe { BooleanProperty(kMIDIPropertySupportsMMC) } }
    /// See [kMIDIPropertyCanRoute](https://developer.apple.com/reference/coremidi/kMIDIPropertyCanRoute)
    pub fn can_route()          -> BooleanProperty { unsafe { BooleanProperty(kMIDIPropertyCanRoute) } }
    /// See [kMIDIPropertyReceivesClock](https://developer.apple.com/reference/coremidi/kMIDIPropertyReceivesClock)
    pub fn receives_clock()     -> BooleanProperty { unsafe { BooleanProperty(kMIDIPropertyReceivesClock) } }
    /// See [kMIDIPropertyReceivesMTC](https://developer.apple.com/reference/coremidi/kMIDIPropertyReceivesMTC)
    pub fn receives_mtc()       -> BooleanProperty { unsafe { BooleanProperty(kMIDIPropertyReceivesMTC) } }
    /// See [kMIDIPropertyReceivesNotes](https://developer.apple.com/reference/coremidi/kMIDIPropertyReceivesNotes)
    pub fn receives_notes()     -> BooleanProperty { unsafe { BooleanProperty(kMIDIPropertyReceivesNotes) } }
    /// See [kMIDIPropertyReceivesProgramChanges](https://developer.apple.com/reference/coremidi/kMIDIPropertyReceivesProgramChanges)
    pub fn receives_program_changes() -> BooleanProperty { unsafe { BooleanProperty(kMIDIPropertyReceivesProgramChanges) } }
    /// See [kMIDIPropertyReceivesBankSelectMSB](https://developer.apple.com/reference/coremidi/kMIDIPropertyReceivesBankSelectMSB)
    pub fn receives_bank_select_msb() -> BooleanProperty { unsafe { BooleanProperty(kMIDIPropertyReceivesBankSelectMSB) } }
    /// See [kMIDIPropertyReceivesBankSelectLSB](https://developer.apple.com/reference/coremidi/kMIDIPropertyReceivesBankSelectLSB)
    pub fn receives_bank_select_lsb() -> BooleanProperty { unsafe { BooleanProperty(kMIDIPropertyReceivesBankSelectLSB) } }
    /// See [kMIDIPropertyTransmitsBankSelectMSB](https://developer.apple.com/reference/coremidi/kMIDIPropertyTransmitsBankSelectMSB)
    pub fn transmits_bank_select_msb() -> BooleanProperty { unsafe { BooleanProperty(kMIDIPropertyTransmitsBankSelectMSB) } }
    /// See [kMIDIPropertyTransmitsBankSelectLSB](https://developer.apple.com/reference/coremidi/kMIDIPropertyTransmitsBankSelectLSB)
    pub fn transmits_bank_select_lsb() -> BooleanProperty { unsafe { BooleanProperty(kMIDIPropertyTransmitsBankSelectLSB) } }
    /// See [kMIDIPropertyTransmitsClock](https://developer.apple.com/reference/coremidi/kMIDIPropertyTransmitsClock)
    pub fn transmits_clock()    -> BooleanProperty { unsafe { BooleanProperty(kMIDIPropertyTransmitsClock) } }
    /// See [kMIDIPropertyTransmitsMTC](https://developer.apple.com/reference/coremidi/kMIDIPropertyTransmitsMTC)
    pub fn transmits_mtc()      -> BooleanProperty { unsafe { BooleanProperty(kMIDIPropertyTransmitsMTC) } }
    /// See [kMIDIPropertyTransmitsNotes](https://developer.apple.com/reference/coremidi/kMIDIPropertyTransmitsNotes)
    pub fn transmits_notes()    -> BooleanProperty { unsafe { BooleanProperty(kMIDIPropertyTransmitsNotes) } }
    /// See [kMIDIPropertyTransmitsProgramChanges](https://developer.apple.com/reference/coremidi/kMIDIPropertyTransmitsProgramChanges)
    pub fn transmits_program_changes() -> BooleanProperty { unsafe { BooleanProperty(kMIDIPropertyTransmitsProgramChanges) } }
    /// See [kMIDIPropertyPanDisruptsStereo](https://developer.apple.com/reference/coremidi/kMIDIPropertyPanDisruptsStereo)
    pub fn pan_disrupts_stereo() -> BooleanProperty { unsafe { BooleanProperty(kMIDIPropertyPanDisruptsStereo) } }
    /// See [kMIDIPropertyIsSampler](https://developer.apple.com/reference/coremidi/kMIDIPropertyIsSampler)
    pub fn is_sampler()          -> BooleanProperty { unsafe { BooleanProperty(kMIDIPropertyIsSampler) } }
    /// See [kMIDIPropertyIsDrumMachine](https://developer.apple.com/reference/coremidi/kMIDIPropertyIsDrumMachine)
    pub fn is_drum_machine()     -> BooleanProperty { unsafe { BooleanProperty(kMIDIPropertyIsDrumMachine) } }
    /// See [kMIDIPropertyIsMixer](https://developer.apple.com/reference/coremidi/kMIDIPropertyIsMixer)
    pub fn is_mixer()            -> BooleanProperty { unsafe { BooleanProperty(kMIDIPropertyIsMixer) } }
    /// See [kMIDIPropertyIsEffectUnit](https://developer.apple.com/reference/coremidi/kMIDIPropertyIsEffectUnit)
    pub fn is_effect_unit()      -> BooleanProperty { unsafe { BooleanProperty(kMIDIPropertyIsEffectUnit) } }
    /// See [kMIDIPropertyMaxReceiveChannels](https://developer.apple.com/reference/coremidi/kMIDIPropertyMaxReceiveChannels)
    pub fn max_receive_channels() -> IntegerProperty { unsafe { IntegerProperty(kMIDIPropertyMaxReceiveChannels) } }
    /// See [kMIDIPropertyMaxTransmitChannels](https://developer.apple.com/reference/coremidi/kMIDIPropertyMaxTransmitChannels)
    pub fn max_transmit_channels() -> IntegerProperty { unsafe { IntegerProperty(kMIDIPropertyMaxTransmitChannels) } }
    /// See [kMIDIPropertyDriverDeviceEditorApp](https://developer.apple.com/reference/coremidi/kMIDIPropertyDriverDeviceEditorApp)
    pub fn driver_device_editor_app() -> StringProperty {
        StringProperty::from_constant_string_ref(unsafe { kMIDIPropertyDriverDeviceEditorApp })
    }

    /// See [kMIDIPropertySupportsShowControl](https://developer.apple.com/reference/coremidi/kMIDIPropertySupportsShowControl)
    pub fn supports_show_control() -> BooleanProperty { unsafe { BooleanProperty(kMIDIPropertySupportsShowControl) } }
    /// See [kMIDIPropertyDisplayName](https://developer.apple.com/reference/coremidi/kMIDIPropertyDisplayName)
    pub fn display_name() -> StringProperty {
        StringProperty::from_constant_string_ref(unsafe { kMIDIPropertyDisplayName })
    }
}
