use ross_dsl::Parser;

fn main() {
    let text = "
        store button: u32 = 0x00;
        store brightness: u8 = 0x00;
        
        let event_producer_address = 0xffff;
        let receiver_address = 0x0003;
        let led_channel = 0;

        // sends all button_pressed events from device with address 4 to device with adress 3
        send BUTTON_PRESSED_EVENT_CODE from event_producer_address to receiver_address;

        // maybe unneeded?!?
        // sends all button_pressed events from any device to device address 3
        // send BUTTON_PRESSED_EVENT_CODE from any to receiver_address;

        do {
            match {
                EventCodeExtractor();
                U16IsEqualFilter(BUTTON_PRESSED_EVENT_CODE);
            }
        
            match { U32SetStateFilter(button, 0x01); }
        }
        
        do {
            match {
                EventCodeExtractor();
                U16IsEqualFilter(BUTTON_RELEASED_EVENT_CODE);
            }
        
            match { U32SetStateFilter(button, 0x00); }
        }
        
        do {
            match {
                EventCodeExtractor();
                U16IsEqualFilter(INTERNAL_SYSTEM_TICK_EVENT_CODE);
            }
        
            match { CountFilter(0, 10); }
            match { U32IsEqualStateFilter(button, 0x00); }
            match { U8IncrementStateFilter(brightness); }
        
            fire { BcmChangeBrightnessStateProducer(receiver_address, led_channel, brightness); }
        }
    ";

    println!("{:?}", Parser::parse(text))
}
