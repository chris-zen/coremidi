extern crate coremidi;

fn main() {
    let client = coremidi::Client::new("example-client").unwrap();

    let callback = |packet_list: coremidi::PacketList| {
        println!("{}", packet_list);
    };

    client.virtual_destination("example-destination", callback).unwrap();

    let mut input_line = String::new();
    println!("Press [Intro] to finish ...");
    std::io::stdin().read_line(&mut input_line).ok().expect("Failed to read line");
}
