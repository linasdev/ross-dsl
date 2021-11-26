use ross_dsl::Parser;

fn main() {
    let text = "
        const device_address = 0x0003;
        const receiver_address = 0x000a;

        send BUTTON_PRESSED_EVENT_CODE from device_address to receiver_address;
    ";

    println!("{:?}", Parser::parse(text))
}
