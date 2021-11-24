use ross_dsl::Parser;

fn main() {
    let text = "
        store button: u32 = 0;
        store brightness: u8 = 0;
        
        let receiver_address = 3;
        let led_channel = 0;

        // sends all button_pressed events from device with address 4 to device with adress 3
        // send BUTTON_PRESSED_EVENT_CODE from event_producer_address to receiver_address;

        // maybe unneeded?!?
        // sends all button_pressed events from any device to device address 3
        // send BUTTON_PRESSED_EVENT_CODE from any to 3;

        do {
            match {
                EventCodeExtractor();
                U16IsEqualFilter(BUTTON_PRESSED_EVENT_CODE);
            }
        
            match { U32SetStateFilter(button, 1); }
        }
        
        do {
            match {
                EventCodeExtractor();
                U16IsEqualFilter(BUTTON_RELEASED_EVENT_CODE);
            }
        
            match { U32SetStateFilter(button, 0); }
        }
        
        do {
            match {
                EventCodeExtractor();
                U16IsEqualFilter(INTERNAL_SYSTEM_TICK_EVENT_CODE);
            }
        
            match { CountFilter(0, 10); }
            match { U32IsEqualStateFilter(button, 1); }
            match { U8IncrementStateFilter(brightness); }
        
            fire { BcmChangeBrightnessStateProducer(receiver_address, led_channel, brightness); }
        }
    ";

    println!("{:?}", Parser::parse(text))
}
