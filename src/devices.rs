use Object;
use Device;

use std::ops::Deref;

impl Deref for Device {
    type Target = Object;

    fn deref(&self) -> &Object {
        &self.object
    }
}
