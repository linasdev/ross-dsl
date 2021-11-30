use ross_dsl::Parser;

fn main() {
    let text = "
        const device_address = 0x0003~u16;
        let button_index = 0x00~u8;
        const receiver_address = 0x000a~u16;


        send device_address from device_address to receiver_address if match {
            ButtonIndexExtractor();
            ValueEqualToConstFilter(button_index);
        }
    ";

    match Parser::parse(text) {
        Ok(event_processors) => println!("{:?}", event_processors),
        Err(err) => println!("{}", err),
    }
}
