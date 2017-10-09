use coremidi_sys::{
    MIDIGetNumberOfDevices, MIDIGetDevice, MIDIDeviceGetNumberOfEntities, MIDIDeviceGetEntity,
    ItemCount
};

use Object;
use Device;
use Entity;

use std::ops::Deref;

impl Device {
    /// Get a device from its index.
    /// See [MIDIGetDevice](https://developer.apple.com/documentation/coremidi/1495368-midigetdevice)
    ///
    pub fn from_index(index: usize) -> Option<Device> {
        let device_ref = unsafe { MIDIGetDevice(index as ItemCount) };
        match device_ref {
            0 => None,
            _ => Some(Device { object: Object(device_ref) })
        }
    }

    pub fn entity_count(&self) -> usize {
        unsafe { MIDIDeviceGetNumberOfEntities(self.object.0) as usize }
    }

    pub fn entities(&self) -> DeviceEntitiesIterator {
        DeviceEntitiesIterator { device: &self, index: 0, count: self.entity_count() }
    }
}

impl Deref for Device {
    type Target = Object;

    fn deref(&self) -> &Object {
        &self.object
    }
}

/// Devices available in the system.
///
/// The number of devices available in the system can be retrieved with:
///
/// ```
/// let number_of_devices = coremidi::Devices::count();
/// ```
///
/// The devices in the system can be iterated as:
///
/// ```rust,no_run
/// for device in coremidi::Devices {
///   println!("{}", device.display_name().unwrap());
/// }
/// ```
///
pub struct Devices;

impl Devices {
    /// Get the number of devices available in the system for sending MIDI messages.
    /// See [MIDIGetNumberOfDevices](https://developer.apple.com/documentation/coremidi/1495164-midigetnumberofdevices).
    ///
    pub fn count() -> usize {
        unsafe { MIDIGetNumberOfDevices() as usize }
    }
}

impl IntoIterator for Devices {
    type Item = Device;
    type IntoIter = DevicesIterator;

    fn into_iter(self) -> Self::IntoIter {
        DevicesIterator { index: 0, count: Self::count() }
    }
}

pub struct DevicesIterator {
    index: usize,
    count: usize
}

impl Iterator for DevicesIterator {
    type Item = Device;

    fn next(&mut self) -> Option<Device> {
        if self.index < self.count {
            let device = Device::from_index(self.index);
            self.index += 1;
            device
        }
        else {
            None
        }
    }
}

pub struct DeviceEntitiesIterator<'a> {
    device: &'a Device,
    index: usize,
    count: usize
}

impl<'a> Iterator for DeviceEntitiesIterator<'a> {
    type Item = Entity;

    fn next(&mut self) -> Option<Entity> {
        if self.index < self.count {
            let entity_ref =
                unsafe { MIDIDeviceGetEntity(self.device.object.0, self.index as u64) };
            self.index += 1;
            if entity_ref == 0 {
                Some(Entity { object: Object(entity_ref) })
            }
            else {
                None
            }
        }
        else {
            None
        }
    }
}
