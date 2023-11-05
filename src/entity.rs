use coremidi_sys::MIDIObjectRef;
use std::ops::Deref;

use crate::object::Object;

/// A [MIDI object](https://developer.apple.com/documentation/coremidi/midientityref).
///
/// An entity that a device owns and that contains endpoints.
///
#[derive(Debug, Clone, PartialEq)]
pub struct Entity {
    pub(crate) object: Object,
}

impl Entity {
    pub(crate) fn new(object_ref: MIDIObjectRef) -> Self {
        Self {
            object: Object(object_ref),
        }
    }
}

impl Deref for Entity {
    type Target = Object;

    fn deref(&self) -> &Object {
        &self.object
    }
}

impl From<Object> for Entity {
    fn from(object: Object) -> Self {
        Self::new(object.0)
    }
}

impl From<Entity> for Object {
    fn from(entity: Entity) -> Self {
        entity.object
    }
}
