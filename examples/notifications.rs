extern crate coremidi;

fn main() {
    println!("Logging Client Notifications");
    println!("Press Enter to Finish");
    println!("");

    let _client = coremidi::Client::new_with_notifications("example-client", print_notification).unwrap();

    let mut input_line = String::new();
    std::io::stdin().read_line(&mut input_line).ok().expect("Failed to read line");
}

fn print_notification(notification: &coremidi::Notification) {
    println!("Received Notification: {:?} \r", notification);
}