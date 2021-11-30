use ross_dsl::Parser;

fn main() {
    let text = "
        const device_address = 0x0003~u16;
        const receiver_address = 0xffff~u16;

        send BUTTON_PRESSED_EVENT_CODE from device_address to receiver_address;
    ";

    match Parser::parse(text) {
        Ok(event_processors) => println!("{:?}", event_processors),
        Err(err) => println!("{}", err),
    }
}
