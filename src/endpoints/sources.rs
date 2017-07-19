use core_foundation_sys::base::OSStatus;

use coremidi_sys::{
    MIDIGetNumberOfSources, MIDIGetSource, MIDIReceived, MIDIEndpointDispose, ItemCount
};

use std::ops::Deref;

use Object;
use Endpoint;
use Source;
use VirtualSource;
use PacketList;

impl Source {
    /// Create a source endpoint from its index.
    /// See [MIDIGetSource](https://developer.apple.com/reference/coremidi/1495168-midigetsource)
    ///
    pub fn from_index(index: usize) -> Option<Source> {
        let endpoint_ref = unsafe { MIDIGetSource(index as ItemCount) };
        match endpoint_ref {
            0 => None,
            _ => Some(Source { endpoint: Endpoint { object: Object(endpoint_ref) } })
        }
    }
}

impl Deref for Source {
    type Target = Endpoint;

    fn deref(&self) -> &Endpoint {
        &self.endpoint
    }
}

/// Source endpoints available in the system.
///
/// The number of sources available in the system can be retrieved with:
///
/// ```rust,no_run
/// let number_of_sources = coremidi::Sources::count();
/// ```
///
/// The sources in the system can be iterated as:
///
/// ```rust,no_run
/// for source in coremidi::Sources {
///   println!("{}", source.display_name().unwrap());
/// }
/// ```
///
pub struct Sources;

impl Sources {
    /// Get the number of sources available in the system for receiving MIDI messages.
    /// See [MIDIGetNumberOfSources](https://developer.apple.com/reference/coremidi/1495116-midigetnumberofsources).
    ///
    pub fn count() -> usize {
        unsafe { MIDIGetNumberOfSources() as usize }
    }
}

impl IntoIterator for Sources {
    type Item = Source;
    type IntoIter = SourcesIterator;

    fn into_iter(self) -> Self::IntoIter {
        SourcesIterator { index: 0, count: Self::count() }
    }
}

pub struct SourcesIterator {
    index: usize,
    count: usize
}

impl Iterator for SourcesIterator {
    type Item = Source;

    fn next(&mut self) -> Option<Source> {
        if self.index < self.count {
            let source = Source::from_index(self.index);
            self.index += 1;
            source
        }
        else {
            None
        }
    }
}

impl VirtualSource {
    /// Distributes incoming MIDI from a source to the client input ports which are connected to that source.
    /// See [MIDIReceived](https://developer.apple.com/reference/coremidi/1495276-midireceived)
    ///
    pub fn received(&self, packet_list: &PacketList) -> Result<(), OSStatus> {
        let status = unsafe { MIDIReceived(
            self.endpoint.object.0,
            packet_list.as_ptr())
        };
        if status == 0 { Ok(()) } else { Err(status) }
    }
}

impl Deref for VirtualSource {
    type Target = Endpoint;

    fn deref(&self) -> &Endpoint {
        &self.endpoint
    }
}

impl Drop for VirtualSource {
    fn drop(&mut self) {
        unsafe { MIDIEndpointDispose(self.endpoint.object.0) };
    }
}
