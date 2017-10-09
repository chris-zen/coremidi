use coremidi_sys::{
    MIDIEntityGetNumberOfDestinations, MIDIEntityGetDestination,
    MIDIEntityGetNumberOfSources, MIDIEntityGetSource,
    MIDIEntityGetDevice
};

use Object;
use Entity;
use Device;
use Destination;
use Source;
use Endpoint;

impl Entity {
    pub fn destination_count(&self) -> usize {
        unsafe { MIDIEntityGetNumberOfDestinations(self.object.0) as usize }
    }

    pub fn destinations(&self) -> EntityDestinationIterator {
        EntityDestinationIterator { entity: &self, index: 0, count: self.destination_count() }
    }

    pub fn source_count(&self) -> usize {
        unsafe { MIDIEntityGetNumberOfSources(self.object.0) as usize }
    }

    pub fn sources(&self) -> EntitySourceIterator {
        EntitySourceIterator { entity: &self, index: 0, count: self.source_count() }
    }

    pub fn get_device(&self) -> Option<Device> {
        let device_ref = 0 as *mut u32;
        let res = unsafe { MIDIEntityGetDevice(self.object.0, device_ref) };
        if res == 0 {
            Some(Device { object: Object(unsafe { *device_ref }) })
        }
        else {
            None
        }
    }
}

pub struct EntityDestinationIterator<'a> {
    entity: &'a Entity,
    index: usize,
    count: usize
}

impl<'a> Iterator for EntityDestinationIterator<'a> {
    type Item = Destination;

    fn next(&mut self) -> Option<Destination> {
        if self.index < self.count {
            let destination_ref =
                unsafe { MIDIEntityGetDestination(self.entity.object.0, self.index as u64) };
            self.index += 1;
            if destination_ref == 0 {
                None
            }
            else {
                Some(Destination {endpoint: Endpoint { object: Object(destination_ref) }})
            }
        }
        else {
            None
        }
    }
}

pub struct EntitySourceIterator<'a> {
    entity: &'a Entity,
    index: usize,
    count: usize
}

impl<'a> Iterator for EntitySourceIterator<'a> {
    type Item = Source;

    fn next(&mut self) -> Option<Source> {
        if self.index < self.count {
            let source_ref =
                unsafe { MIDIEntityGetSource(self.entity.object.0, self.index as u64) };
            self.index += 1;
            if source_ref == 0 {
                None
            }
            else {
                Some(Source {endpoint: Endpoint { object: Object(source_ref) }})
            }
        }
        else {
            None
        }
    }
}
