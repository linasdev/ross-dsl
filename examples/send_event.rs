use ross_dsl::Parser;

fn main() {
    let text = "
        const device_address = 0x0003~u16;
        const receiver_address = 0x000a~u16;

        send BUTTON_PRESSED_EVENT_CODE from device_address to receiver_address;
    ";

    println!("{:?}", Parser::parse(text))
}
