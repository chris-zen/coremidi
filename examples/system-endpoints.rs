extern crate coremidi;

fn main() {
    println!("System destinations:");

    for (i, destination) in coremidi::Destinations.into_iter().enumerate() {
        let display_name = get_display_name(&destination);
        println!("[{}] {}", i, display_name);
    }

    println!("\nSystem sources:");

    for (i, source) in coremidi::Sources.into_iter().enumerate() {
        let display_name = get_display_name(&source);
        println!("[{}] {}", i, display_name);
    }
}

fn get_display_name(endpoint: &coremidi::Endpoint) -> String {
    match endpoint.display_name() {
        Some(name) => name,
        None => "[Unknown Display Name]".to_string()
    }
}
