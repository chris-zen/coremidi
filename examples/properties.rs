extern crate coremidi;

use coremidi::{
    PropertyGetter,
    PropertySetter, 
};

fn main() {
    let client = coremidi::Client::new("Example Client").unwrap();

    let callback = |packet_list: &coremidi::PacketList| {
        println!("{}", packet_list);
    };

    // Creates a virtual destination, then gets its properties
    let destination = client.virtual_destination("Example Destination", callback).unwrap();

    println!("Created Virtual Destination...");
    let name: String = coremidi::Properties::name().value_from(&destination).unwrap();
    println!("With Name: {}", name);
}
