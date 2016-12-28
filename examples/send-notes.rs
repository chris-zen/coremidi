extern crate coremidi;

use std::time::Duration;
use std::thread;
use std::env;

fn main() {
    let destination_index = get_destination_index();
    println!("Destination index: {}", destination_index);

    let destination = coremidi::destinations::from_index(destination_index);

    let client = coremidi::Client::new("example-client").unwrap();
    let output_port = client.create_output_port("example-port").unwrap();

    let note_on = create_note_on(0, 64, 127);
    let note_off = create_note_off(0, 64, 127);

    for i in 0..10 {
        println!("[{}] Sending note ...", i);

        output_port.send(&destination, &note_on).unwrap();

        thread::sleep(Duration::from_millis(1000));

        output_port.send(&destination, &note_off).unwrap();
    }
}

fn get_destination_index() -> usize {
    let mut args_iter = env::args();
    args_iter.next();
    match args_iter.next() {
        Some(arg) => match arg.parse::<usize>() {
            Ok(index) => {
                if index >= coremidi::destinations::count() {
                    println!("Destination index out of range: {}", index);
                    std::process::exit(-1);
                }
                index
            },
            Err(_) => {
                println!("Wrong destination index: {}", arg);
                std::process::exit(-1);
            }
        },
        None => {
            println!("Usage: send <destination-index>");
            println!("");
            println!("Available Destinations:");
            print_destinations();
            std::process::exit(-1);
        }
    }
}

fn print_destinations() {
    let num_dest = coremidi::destinations::count();
    for i in 0..num_dest {
        let dest = coremidi::destinations::from_index(i);
        match dest.get_display_name() {
            Some(display_name) => println!("[{}] {}", i, display_name),
            None => ()
        }
    }
}

fn create_note_on(channel: u8, note: u8, velocity: u8) -> coremidi::PacketList {
    let data = vec![
        0x90 | (channel & 0x0f),
        note & 0x7f,
        velocity & 0x7f];
    coremidi::PacketList::from_data(0, data)
}

fn create_note_off(channel: u8, note: u8, velocity: u8) -> coremidi::PacketList {
    let data = vec![
        0x80 | (channel & 0x0f),
        note & 0x7f,
        velocity & 0x7f];
    coremidi::PacketList::from_data(0, data)
}
