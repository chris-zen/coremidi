use core_foundation::string::{CFString, CFStringRef};
use core_foundation::base::{TCFType, OSStatus};

use coremidi_sys::*;

use std::mem;

use Object;

pub trait PropertyGetter<T> {
    fn value_from(&self, object: &Object) -> Result<T, OSStatus>;
}

pub trait PropertySetter<T> {
    fn set_value(&self, object: &Object, value: T) -> Result<(), OSStatus>;
}

pub struct StringProperty(CFStringRef);

impl StringProperty {
    pub fn new(name: &str) -> Self {
        StringProperty(CFString::new(name).as_concrete_TypeRef())
    }
}

impl<T> PropertyGetter<T> for StringProperty where T: From<String> {
    fn value_from(&self, object: &Object) -> Result<T, OSStatus> {
        unsafe {
            let mut string_ref: CFStringRef = mem::uninitialized();
            let status = MIDIObjectGetStringProperty(object.0, self.0, &mut string_ref);
            if status == 0 {
                let string: CFString = TCFType::wrap_under_create_rule(string_ref);
                Ok(From::<String>::from(format!("{}", string)))
            }
            else { Err(status) }
        }
    }
}

impl<T> PropertySetter<T> for StringProperty where T: Into<String> {
    fn set_value(&self, object: &Object, value: T) -> Result<(), OSStatus> {
        unsafe {
            let string_ref = CFString::new(value.into().as_ref()).as_concrete_TypeRef();
            let status = MIDIObjectSetStringProperty(object.0, self.0, string_ref);
            if status == 0 { Ok(()) } else { Err(status) }
        }
    }
}

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
    pub fn name()               -> StringProperty { unsafe { StringProperty(kMIDIPropertyName) } }
    /// See [kMIDIPropertyManufacturer](https://developer.apple.com/reference/coremidi/kmidipropertymanufacturer)
    pub fn manufacturer()       -> StringProperty { unsafe { StringProperty(kMIDIPropertyManufacturer) } }
    /// See [kMIDIPropertyModel](https://developer.apple.com/reference/coremidi/kmidipropertymodel)
    pub fn model()              -> StringProperty { unsafe { StringProperty(kMIDIPropertyModel) } }
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
    // /// See [kMIDIPropertySingleRealtimeEntity](https://developer.apple.com/reference/coremidi/kMIDIPropertySingleRealtimeEntity)
    // pub fn SingleRealtimeEntity() -> Property { unsafe { Property(kMIDIPropertySingleRealtimeEntity) } }
    // /// See [kMIDIPropertyConnectionUniqueID](https://developer.apple.com/reference/coremidi/kMIDIPropertyConnectionUniqueID)
    // pub fn ConnectionUniqueID() -> Property { unsafe { Property(kMIDIPropertyConnectionUniqueID) } }
    // /// See [kMIDIPropertyOffline](https://developer.apple.com/reference/coremidi/kMIDIPropertyOffline)
    // pub fn Offline() -> Property { unsafe { Property(kMIDIPropertyOffline) } }
    // /// See [kMIDIPropertyPrivate](https://developer.apple.com/reference/coremidi/kMIDIPropertyPrivate)
    // pub fn Private() -> Property { unsafe { Property(kMIDIPropertyPrivate) } }
    // /// See [kMIDIPropertyDriverOwner](https://developer.apple.com/reference/coremidi/kMIDIPropertyDriverOwner)
    // pub fn DriverOwner() -> Property { unsafe { Property(kMIDIPropertyDriverOwner) } }
    // /// See [kMIDIPropertyFactoryPatchNameFile](https://developer.apple.com/reference/coremidi/kMIDIPropertyFactoryPatchNameFile)
    // pub fn FactoryPatchNameFile() -> Property { unsafe { Property(kMIDIPropertyFactoryPatchNameFile) } }
    // /// See [kMIDIPropertyUserPatchNameFile](https://developer.apple.com/reference/coremidi/kMIDIPropertyUserPatchNameFile)
    // pub fn UserPatchNameFile() -> Property { unsafe { Property(kMIDIPropertyUserPatchNameFile) } }
    // /// See [kMIDIPropertyNameConfiguration](https://developer.apple.com/reference/coremidi/kMIDIPropertyNameConfiguration)
    // pub fn NameConfiguration() -> Property { unsafe { Property(kMIDIPropertyNameConfiguration) } }
    // /// See [kMIDIPropertyImage](https://developer.apple.com/reference/coremidi/kMIDIPropertyImage)
    // pub fn Image() -> Property { unsafe { Property(kMIDIPropertyImage) } }
    // /// See [kMIDIPropertyDriverVersion](https://developer.apple.com/reference/coremidi/kMIDIPropertyDriverVersion)
    // pub fn DriverVersion() -> Property { unsafe { Property(kMIDIPropertyDriverVersion) } }
    // /// See [kMIDIPropertySupportsGeneralMIDI](https://developer.apple.com/reference/coremidi/kMIDIPropertySupportsGeneralMIDI)
    // pub fn SupportsGeneralMIDI() -> Property { unsafe { Property(kMIDIPropertySupportsGeneralMIDI) } }
    // /// See [kMIDIPropertySupportsMMC](https://developer.apple.com/reference/coremidi/kMIDIPropertySupportsMMC)
    // pub fn SupportsMMC() -> Property { unsafe { Property(kMIDIPropertySupportsMMC) } }
    // /// See [kMIDIPropertyCanRoute](https://developer.apple.com/reference/coremidi/kMIDIPropertyCanRoute)
    // pub fn CanRoute() -> Property { unsafe { Property(kMIDIPropertyCanRoute) } }
    // /// See [kMIDIPropertyReceivesClock](https://developer.apple.com/reference/coremidi/kMIDIPropertyReceivesClock)
    // pub fn ReceivesClock() -> Property { unsafe { Property(kMIDIPropertyReceivesClock) } }
    // /// See [kMIDIPropertyReceivesMTC](https://developer.apple.com/reference/coremidi/kMIDIPropertyReceivesMTC)
    // pub fn ReceivesMTC() -> Property { unsafe { Property(kMIDIPropertyReceivesMTC) } }
    // /// See [kMIDIPropertyReceivesNotes](https://developer.apple.com/reference/coremidi/kMIDIPropertyReceivesNotes)
    // pub fn ReceivesNotes() -> Property { unsafe { Property(kMIDIPropertyReceivesNotes) } }
    // /// See [kMIDIPropertyReceivesProgramChanges](https://developer.apple.com/reference/coremidi/kMIDIPropertyReceivesProgramChanges)
    // pub fn ReceivesProgramChanges() -> Property { unsafe { Property(kMIDIPropertyReceivesProgramChanges) } }
    // /// See [kMIDIPropertyReceivesBankSelectMSB](https://developer.apple.com/reference/coremidi/kMIDIPropertyReceivesBankSelectMSB)
    // pub fn ReceivesBankSelectMSB() -> Property { unsafe { Property(kMIDIPropertyReceivesBankSelectMSB) } }
    // /// See [kMIDIPropertyReceivesBankSelectLSB](https://developer.apple.com/reference/coremidi/kMIDIPropertyReceivesBankSelectLSB)
    // pub fn ReceivesBankSelectLSB() -> Property { unsafe { Property(kMIDIPropertyReceivesBankSelectLSB) } }
    // /// See [kMIDIPropertyTransmitsClock](https://developer.apple.com/reference/coremidi/kMIDIPropertyTransmitsClock)
    // pub fn TransmitsClock() -> Property { unsafe { Property(kMIDIPropertyTransmitsClock) } }
    // /// See [kMIDIPropertyTransmitsMTC](https://developer.apple.com/reference/coremidi/kMIDIPropertyTransmitsMTC)
    // pub fn TransmitsMTC() -> Property { unsafe { Property(kMIDIPropertyTransmitsMTC) } }
    // /// See [kMIDIPropertyTransmitsNotes](https://developer.apple.com/reference/coremidi/kMIDIPropertyTransmitsNotes)
    // pub fn TransmitsNotes() -> Property { unsafe { Property(kMIDIPropertyTransmitsNotes) } }
    // /// See [kMIDIPropertyTransmitsProgramChanges](https://developer.apple.com/reference/coremidi/kMIDIPropertyTransmitsProgramChanges)
    // pub fn TransmitsProgramChanges() -> Property { unsafe { Property(kMIDIPropertyTransmitsProgramChanges) } }
    // /// See [kMIDIPropertyTransmitsBankSelectMSB](https://developer.apple.com/reference/coremidi/kMIDIPropertyTransmitsBankSelectMSB)
    // pub fn TransmitsBankSelectMSB() -> Property { unsafe { Property(kMIDIPropertyTransmitsBankSelectMSB) } }
    // /// See [kMIDIPropertyTransmitsBankSelectLSB](https://developer.apple.com/reference/coremidi/kMIDIPropertyTransmitsBankSelectLSB)
    // pub fn TransmitsBankSelectLSB() -> Property { unsafe { Property(kMIDIPropertyTransmitsBankSelectLSB) } }
    // /// See [kMIDIPropertyPanDisruptsStereo](https://developer.apple.com/reference/coremidi/kMIDIPropertyPanDisruptsStereo)
    // pub fn PanDisruptsStereo() -> Property { unsafe { Property(kMIDIPropertyPanDisruptsStereo) } }
    // /// See [kMIDIPropertyIsSampler](https://developer.apple.com/reference/coremidi/kMIDIPropertyIsSampler)
    // pub fn IsSampler() -> Property { unsafe { Property(kMIDIPropertyIsSampler) } }
    // /// See [kMIDIPropertyIsDrumMachine](https://developer.apple.com/reference/coremidi/kMIDIPropertyIsDrumMachine)
    // pub fn IsDrumMachine() -> Property { unsafe { Property(kMIDIPropertyIsDrumMachine) } }
    // /// See [kMIDIPropertyIsMixer](https://developer.apple.com/reference/coremidi/kMIDIPropertyIsMixer)
    // pub fn IsMixer() -> Property { unsafe { Property(kMIDIPropertyIsMixer) } }
    // /// See [kMIDIPropertyIsEffectUnit](https://developer.apple.com/reference/coremidi/kMIDIPropertyIsEffectUnit)
    // pub fn IsEffectUnit() -> Property { unsafe { Property(kMIDIPropertyIsEffectUnit) } }
    // /// See [kMIDIPropertyMaxReceiveChannels](https://developer.apple.com/reference/coremidi/kMIDIPropertyMaxReceiveChannels)
    // pub fn MaxReceiveChannels() -> Property { unsafe { Property(kMIDIPropertyMaxReceiveChannels) } }
    // /// See [kMIDIPropertyMaxTransmitChannels](https://developer.apple.com/reference/coremidi/kMIDIPropertyMaxTransmitChannels)
    // pub fn MaxTransmitChannels() -> Property { unsafe { Property(kMIDIPropertyMaxTransmitChannels) } }
    // /// See [kMIDIPropertyDriverDeviceEditorApp](https://developer.apple.com/reference/coremidi/kMIDIPropertyDriverDeviceEditorApp)
    // pub fn DriverDeviceEditorApp() -> Property { unsafe { Property(kMIDIPropertyDriverDeviceEditorApp) } }
    // /// See [kMIDIPropertySupportsShowControl](https://developer.apple.com/reference/coremidi/kMIDIPropertySupportsShowControl)
    // pub fn SupportsShowControl() -> Property { unsafe { Property(kMIDIPropertySupportsShowControl) } }
    /// See [kMIDIPropertyDisplayName](https://developer.apple.com/reference/coremidi/kMIDIPropertyDisplayName)
    pub fn display_name() -> StringProperty { unsafe { StringProperty(kMIDIPropertyDisplayName) } }
}
