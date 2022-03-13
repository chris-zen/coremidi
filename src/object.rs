use core_foundation_sys::base::OSStatus;
use std::fmt;

use coremidi_sys::{MIDIObjectRef, SInt32};

use crate::properties::{
    BooleanProperty, IntegerProperty, Properties, PropertyGetter, PropertySetter, StringProperty,
};

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum ObjectType {
    Other,
    Device,
    Entity,
    Source,
    Destination,
    ExternalDevice,
    ExternalEntity,
    ExternalSource,
    ExternalDestination,
}

impl TryFrom<i32> for ObjectType {
    type Error = i32;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            coremidi_sys::kMIDIObjectType_Other => Ok(ObjectType::Other),
            coremidi_sys::kMIDIObjectType_Device => Ok(ObjectType::Device),
            coremidi_sys::kMIDIObjectType_Entity => Ok(ObjectType::Entity),
            coremidi_sys::kMIDIObjectType_Source => Ok(ObjectType::Source),
            coremidi_sys::kMIDIObjectType_Destination => Ok(ObjectType::Destination),
            coremidi_sys::kMIDIObjectType_ExternalDevice => Ok(ObjectType::ExternalDevice),
            coremidi_sys::kMIDIObjectType_ExternalEntity => Ok(ObjectType::ExternalEntity),
            coremidi_sys::kMIDIObjectType_ExternalSource => Ok(ObjectType::ExternalSource),
            coremidi_sys::kMIDIObjectType_ExternalDestination => {
                Ok(ObjectType::ExternalDestination)
            }
            unknown => Err(unknown),
        }
    }
}

/// A [MIDI Object](https://developer.apple.com/reference/coremidi/midiobjectref).
///
/// The base class of many CoreMIDI objects.
///
#[derive(Hash, Eq, PartialEq)]
pub struct Object(pub(crate) MIDIObjectRef);

impl Object {
    /// Get the name for the object.
    ///
    pub fn name(&self) -> Option<String> {
        Properties::name().value_from(self).ok()
    }

    /// Get the unique id for the object.
    ///
    pub fn unique_id(&self) -> Option<u32> {
        Properties::unique_id()
            .value_from(self)
            .ok()
            .map(|v: SInt32| v as u32)
    }

    /// Get the display name for the object.
    ///
    pub fn display_name(&self) -> Option<String> {
        Properties::display_name().value_from(self).ok()
    }

    /// Sets an object's string-type property.
    ///
    pub fn set_property_string(&self, name: &str, value: &str) -> Result<(), OSStatus> {
        StringProperty::new(name).set_value(self, value)
    }

    /// Gets an object's string-type property.
    ///
    pub fn get_property_string(&self, name: &str) -> Result<String, OSStatus> {
        StringProperty::new(name).value_from(self)
    }

    /// Sets an object's integer-type property.
    ///
    pub fn set_property_integer(&self, name: &str, value: i32) -> Result<(), OSStatus> {
        IntegerProperty::new(name).set_value(self, value)
    }

    /// Gets an object's integer-type property.
    ///
    pub fn get_property_integer(&self, name: &str) -> Result<i32, OSStatus> {
        IntegerProperty::new(name).value_from(self)
    }

    /// Sets an object's boolean-type property.
    ///
    /// CoreMIDI treats booleans as integers (0/1) but this API uses native bool types
    ///
    pub fn set_property_boolean(&self, name: &str, value: bool) -> Result<(), OSStatus> {
        BooleanProperty::new(name).set_value(self, value)
    }

    /// Gets an object's boolean-type property.
    ///
    /// CoreMIDI treats booleans as integers (0/1) but this API uses native bool types
    ///
    pub fn get_property_boolean(&self, name: &str) -> Result<bool, OSStatus> {
        BooleanProperty::new(name).value_from(self)
    }

    pub fn set_property<T>(
        &self,
        property: &dyn PropertySetter<T>,
        value: T,
    ) -> Result<(), OSStatus> {
        property.set_value(self, value)
    }

    pub fn get_property<T>(&self, property: &dyn PropertyGetter<T>) -> Result<T, OSStatus> {
        property.value_from(self)
    }
}

impl fmt::Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Object({:x})", self.0 as usize)
    }
}

#[cfg(test)]
mod tests {
    use crate::object::ObjectType;

    #[test]
    fn objecttype_try_from() {
        assert_eq!(
            ObjectType::try_from(coremidi_sys::kMIDIObjectType_Other),
            Ok(ObjectType::Other)
        );
        assert_eq!(
            ObjectType::try_from(coremidi_sys::kMIDIObjectType_Device),
            Ok(ObjectType::Device)
        );
        assert_eq!(
            ObjectType::try_from(coremidi_sys::kMIDIObjectType_Entity),
            Ok(ObjectType::Entity)
        );
        assert_eq!(
            ObjectType::try_from(coremidi_sys::kMIDIObjectType_Source),
            Ok(ObjectType::Source)
        );
        assert_eq!(
            ObjectType::try_from(coremidi_sys::kMIDIObjectType_Destination),
            Ok(ObjectType::Destination)
        );
        assert_eq!(
            ObjectType::try_from(coremidi_sys::kMIDIObjectType_ExternalDevice),
            Ok(ObjectType::ExternalDevice)
        );
        assert_eq!(
            ObjectType::try_from(coremidi_sys::kMIDIObjectType_ExternalEntity),
            Ok(ObjectType::ExternalEntity)
        );
        assert_eq!(
            ObjectType::try_from(coremidi_sys::kMIDIObjectType_ExternalSource),
            Ok(ObjectType::ExternalSource)
        );
        assert_eq!(
            ObjectType::try_from(coremidi_sys::kMIDIObjectType_ExternalDestination),
            Ok(ObjectType::ExternalDestination)
        );
    }

    #[test]
    fn objecttype_from_error() {
        assert_eq!(ObjectType::try_from(0xffff_i32), Err(0xffff));
    }
}
