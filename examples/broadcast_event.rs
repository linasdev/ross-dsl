use ross_dsl::Parser;

fn main() {
    let text = "
        let device_address = 0x0003;
        let receiver_address = 0xffff;

        send BUTTON_PRESSED_EVENT_CODE from device_address to receiver_address;
    ";

    println!("{:?}", Parser::parse(text))
}
