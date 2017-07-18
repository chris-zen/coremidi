#![allow(non_upper_case_globals)]

use core_foundation::string::{CFString, CFStringRef};
use core_foundation::base::{TCFType, OSStatus};

use coremidi_sys::{
    MIDINotification,
    MIDIObjectAddRemoveNotification,
    MIDIObjectPropertyChangeNotification,
    MIDIIOErrorNotification,
    kMIDIMsgSetupChanged,
    kMIDIMsgObjectAdded,
    kMIDIMsgObjectRemoved,
    kMIDIMsgPropertyChanged,
    kMIDIMsgThruConnectionsChanged,
    kMIDIMsgSerialPortOwnerChanged,
    kMIDIMsgIOError
};

use Object;
use object::ObjectType;
use Device;

#[derive(Debug)]
#[derive(PartialEq)]
pub struct AddedRemovedInfo {
    pub parent: Object,
    pub parent_type: ObjectType,
    pub child: Object,
    pub child_type: ObjectType
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct PropertyChangedInfo {
    pub object: Object,
    pub object_type: ObjectType,
    pub property_name: String
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct IOErrorInfo {
    pub driver_device: Device,
    pub error_code: OSStatus
}

/// A message describing a system state change.
/// See [MIDINotification](https://developer.apple.com/reference/coremidi/midinotification).
///
#[derive(Debug)]
#[derive(PartialEq)]
pub enum Notification {
    SetupChanged,
    ObjectAdded(AddedRemovedInfo),
    ObjectRemoved(AddedRemovedInfo),
    PropertyChanged(PropertyChangedInfo),
    ThruConnectionsChanged,
    SerialPortOwnerChanged,
    IOError(IOErrorInfo)
}

impl Notification {
    pub fn from(notification: &MIDINotification) -> Result<Notification, i32> {
        match notification.messageID as ::std::os::raw::c_uint {
            kMIDIMsgSetupChanged => Ok(Notification::SetupChanged),
            kMIDIMsgObjectAdded | kMIDIMsgObjectRemoved => Self::from_object_added_removed(notification),
            kMIDIMsgPropertyChanged => Self::from_property_changed(notification),
            kMIDIMsgThruConnectionsChanged => Ok(Notification::ThruConnectionsChanged),
            kMIDIMsgSerialPortOwnerChanged => Ok(Notification::SerialPortOwnerChanged),
            kMIDIMsgIOError => Self::from_io_error(notification),
            unknown => Err(unknown as i32)
        }
    }

    fn from_object_added_removed(notification: &MIDINotification) -> Result<Notification, i32> {
        let add_remove_notification = unsafe { &*(notification as *const _ as *const MIDIObjectAddRemoveNotification) };
        let parent_type = ObjectType::from(add_remove_notification.parentType);
        let child_type = ObjectType::from(add_remove_notification.childType);
        if parent_type.is_ok() && child_type.is_ok() {
            let add_remove_info = AddedRemovedInfo {
                parent: Object(add_remove_notification.parent),
                parent_type: parent_type.unwrap(),
                child: Object(add_remove_notification.child),
                child_type: child_type.unwrap()
            };
            match notification.messageID as ::std::os::raw::c_uint {
                kMIDIMsgObjectAdded => Ok(Notification::ObjectAdded(add_remove_info)),
                kMIDIMsgObjectRemoved => Ok(Notification::ObjectRemoved(add_remove_info)),
                _ => Err(0) // Never reached
            }
        }
        else { Err(notification.messageID as i32) }
    }

    fn from_property_changed(notification: &MIDINotification) -> Result<Notification, i32> {
        let property_changed_notification = unsafe { &*(notification as *const _ as *const MIDIObjectPropertyChangeNotification) };
        match ObjectType::from(property_changed_notification.objectType) {
            Ok(object_type) => {
                let property_name = {
                    let name_ref: CFStringRef = property_changed_notification.propertyName;
                    let name: CFString = unsafe { TCFType::wrap_under_create_rule(name_ref) };
                    format!("{}", name)
                };
                let property_changed_info = PropertyChangedInfo {
                    object: Object(property_changed_notification.object),
                    object_type: object_type,
                    property_name: property_name
                };
                Ok(Notification::PropertyChanged(property_changed_info))
            },
            Err(_) => Err(notification.messageID as i32)
        }

    }

    fn from_io_error(notification: &MIDINotification) -> Result<Notification, i32> {
        let io_error_notification = unsafe { &*(notification as *const _ as *const MIDIIOErrorNotification) };
        let io_error_info = IOErrorInfo {
            driver_device: Device { object: Object(io_error_notification.driverDevice) },
            error_code: io_error_notification.errorCode
        };
        Ok(Notification::IOError(io_error_info))
    }
}

#[cfg(test)]
mod tests {

    use core_foundation::string::CFString;
    use core_foundation::base::{TCFType, OSStatus};

    use coremidi_sys::{
        MIDIObjectRef,
        MIDINotification,
        MIDINotificationMessageID,
        MIDIObjectAddRemoveNotification,
        MIDIObjectPropertyChangeNotification,
        MIDIIOErrorNotification,
        kMIDIMsgSetupChanged,
        kMIDIMsgObjectAdded,
        kMIDIMsgObjectRemoved,
        kMIDIMsgPropertyChanged,
        kMIDIMsgThruConnectionsChanged,
        kMIDIMsgSerialPortOwnerChanged,
        kMIDIMsgIOError,
        kMIDIObjectType_Device, kMIDIObjectType_Other
    };

    use Object;
    use Device;
    use object::ObjectType;
    use notifications::{Notification, AddedRemovedInfo, PropertyChangedInfo, IOErrorInfo};

    #[test]
    fn notification_from_error() {
        let notification_raw = MIDINotification {
            messageID: 0xffff as MIDINotificationMessageID,
            messageSize: 8
        };
        let notification = Notification::from(&notification_raw);
        assert!(notification.is_err());
        assert_eq!(notification.err().unwrap(), 0xffff as i32);
    }

    #[test]
    fn notification_from_setup_changed() {
        let notification_raw = MIDINotification {
            messageID: kMIDIMsgSetupChanged as MIDINotificationMessageID,
            messageSize: 8
        };
        let notification = Notification::from(&notification_raw);
        assert!(notification.is_ok());
        assert_eq!(notification.unwrap(), Notification::SetupChanged);
    }

    #[test]
    fn notification_from_object_added() {
        let notification_raw = MIDIObjectAddRemoveNotification {
            messageID: kMIDIMsgObjectAdded as MIDINotificationMessageID,
            messageSize: 24,
            parent: 1 as MIDIObjectRef,
            parentType: kMIDIObjectType_Device,
            child: 2 as MIDIObjectRef,
            childType: kMIDIObjectType_Other
        };

        let notification = Notification::from(
            unsafe { &*(&notification_raw as *const _ as *const MIDINotification) });

        assert!(notification.is_ok());

        let info = AddedRemovedInfo {
            parent: Object(1),
            parent_type: ObjectType::Device,
            child: Object(2),
            child_type: ObjectType::Other
        };

        assert_eq!(notification.unwrap(), Notification::ObjectAdded(info));
    }

    #[test]
    fn notification_from_object_removed() {
        let notification_raw = MIDIObjectAddRemoveNotification {
            messageID: kMIDIMsgObjectRemoved as MIDINotificationMessageID,
            messageSize: 24,
            parent: 1 as MIDIObjectRef,
            parentType: kMIDIObjectType_Device,
            child: 2 as MIDIObjectRef,
            childType: kMIDIObjectType_Other
        };

        let notification = Notification::from(
            unsafe { &*(&notification_raw as *const _ as *const MIDINotification) });

        assert!(notification.is_ok());

        let info = AddedRemovedInfo {
            parent: Object(1),
            parent_type: ObjectType::Device,
            child: Object(2),
            child_type: ObjectType::Other
        };

        assert_eq!(notification.unwrap(), Notification::ObjectRemoved(info));
    }

    #[test]
    fn notification_from_object_added_removed_err() {
        let notification_raw = MIDIObjectAddRemoveNotification {
            messageID: kMIDIMsgObjectAdded as MIDINotificationMessageID,
            messageSize: 24,
            parent: 1 as MIDIObjectRef,
            parentType: kMIDIObjectType_Device,
            child: 2 as MIDIObjectRef,
            childType: 0xffff
        };

        let notification = Notification::from(
            unsafe { &*(&notification_raw as *const _ as *const MIDINotification) });

        assert!(notification.is_err());
        assert_eq!(notification.err().unwrap(), kMIDIMsgObjectAdded as i32);

        let notification_raw = MIDIObjectAddRemoveNotification {
            messageID: kMIDIMsgObjectRemoved as MIDINotificationMessageID,
            messageSize: 24,
            parent: 1 as MIDIObjectRef,
            parentType: 0xffff,
            child: 2 as MIDIObjectRef,
            childType: kMIDIObjectType_Device
        };

        let notification = Notification::from(
            unsafe { &*(&notification_raw as *const _ as *const MIDINotification) });

        assert!(notification.is_err());
        assert_eq!(notification.err().unwrap(), kMIDIMsgObjectRemoved as i32);
    }

    #[test]
    fn notification_from_property_changed() {
        let notification_raw = MIDIObjectPropertyChangeNotification {
            messageID: kMIDIMsgPropertyChanged as MIDINotificationMessageID,
            messageSize: 24,
            object: 1 as MIDIObjectRef,
            objectType: kMIDIObjectType_Device,
            propertyName: CFString::new("name").as_concrete_TypeRef()
        };

        let notification = Notification::from(
            unsafe { &*(&notification_raw as *const _ as *const MIDINotification) });

        assert!(notification.is_ok());

        let info = PropertyChangedInfo {
            object: Object(1),
            object_type: ObjectType::Device,
            property_name: "name".to_string()
        };

        assert_eq!(notification.unwrap(), Notification::PropertyChanged(info));
    }

    #[test]
    fn notification_from_property_changed_error() {
        let notification_raw = MIDIObjectPropertyChangeNotification {
            messageID: kMIDIMsgPropertyChanged as MIDINotificationMessageID,
            messageSize: 24,
            object: 1 as MIDIObjectRef,
            objectType: 0xffff,
            propertyName: CFString::new("name").as_concrete_TypeRef()
        };

        let notification = Notification::from(
            unsafe { &*(&notification_raw as *const _ as *const MIDINotification) });

        assert!(notification.is_err());
        assert_eq!(notification.err().unwrap(), kMIDIMsgPropertyChanged as i32);
    }

    #[test]
    fn notification_from_thru_connections_changed() {
        let notification_raw = MIDINotification {
            messageID: kMIDIMsgThruConnectionsChanged as MIDINotificationMessageID,
            messageSize: 8
        };
        let notification = Notification::from(&notification_raw);
        assert!(notification.is_ok());
        assert_eq!(notification.unwrap(), Notification::ThruConnectionsChanged);
    }

    #[test]
    fn notification_from_serial_port_owner_changed() {
        let notification_raw = MIDINotification {
            messageID: kMIDIMsgSerialPortOwnerChanged as MIDINotificationMessageID,
            messageSize: 8
        };
        let notification = Notification::from(&notification_raw);
        assert!(notification.is_ok());
        assert_eq!(notification.unwrap(), Notification::SerialPortOwnerChanged);
    }

    #[test]
    fn notification_from_io_error() {
        let notification_raw = MIDIIOErrorNotification {
            messageID: kMIDIMsgIOError as MIDINotificationMessageID,
            messageSize: 16,
            driverDevice: 1 as MIDIObjectRef,
            errorCode: 123 as OSStatus
        };

        let notification = Notification::from(
            unsafe { &*(&notification_raw as *const _ as *const MIDINotification) });

        assert!(notification.is_ok());

        let info = IOErrorInfo {
            driver_device: Device { object: Object(1) },
            error_code: 123 as OSStatus
        };

        assert_eq!(notification.unwrap(), Notification::IOError(info));
    }
}
