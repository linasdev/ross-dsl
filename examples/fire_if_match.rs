use ross_dsl::Parser;

fn main() {
    let text = "
        let active = false;

        const device_address = 0x0002~u16;
        const receiver_address = 0x0003~u16;
        
        const active_value = 0xff~u8;
        const rest_value = 0x00~u8;

        const led_channel = 0~u8;

        do {
            match event BUTTON_PRESSED_EVENT_CODE;
            match producer device_address;
            match {
                FlipStateFilter(active);
            }
            fire { BcmChangeBrightnessProducer(receiver_address, led_channel, active_value); } if match {
                StateEqualToConstFilter(active, true);
            }
            fire { BcmChangeBrightnessProducer(receiver_address, led_channel, rest_value); } if match {
                StateEqualToConstFilter(active, false);
            }
        }
    ";

    match Parser::parse(text) {
        Ok(event_processors) => println!("{:?}", event_processors),
        Err(err) => println!("{}", err),
    }
}
