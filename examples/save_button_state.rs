use ross_dsl::Parser;

fn main() {
    let text = "
        let button = false;

        const device_address = 0x0002~u16;
        
        do {
            match event BUTTON_PRESSED_EVENT_CODE;
            match producer device_address;
            match { BoolSetStateFilter(button, true); }
        }
        
        do {
            match event BUTTON_RELEASED_EVENT_CODE;
            match producer device_address;
            match { BoolSetStateFilter(button, false); }
        }
    ";

    println!("{:?}", Parser::parse(text))
}
