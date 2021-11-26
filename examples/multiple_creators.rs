use ross_dsl::Parser;

fn main() {
    let text = "
        const device_address = 0x0002;

        const receiver_address1 = 0x0003;
        const led_channel1 = 0;

        const receiver_address2 = 0x0004;
        const led_channel2 = 1;

        const pressed_value = 0xff;
        const released_value = 0x00;

        do {
            match event BUTTON_PRESSED_EVENT_CODE;
            match producer device_address;
            fire { BcmChangeBrightnessStateProducer(receiver_address1, led_channel1, pressed_value); }
            fire { BcmChangeBrightnessStateProducer(receiver_address2, led_channel2, pressed_value); }
        }

        do {
            match event BUTTON_RELEASED_EVENT_CODE;
            match producer device_address;
            fire { BcmChangeBrightnessStateProducer(receiver_address1, led_channel1, released_value); }
            fire { BcmChangeBrightnessStateProducer(receiver_address2, led_channel2, released_value); }
        }
    ";

    println!("{:?}", Parser::parse(text))
}
