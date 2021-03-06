use ross_dsl::Parser;

fn main() {
    let text = "
        let button = false;

        const device_address = 0x0002~u16;
        
        do {
            match event BUTTON_PRESSED_EVENT_CODE;
            match producer device_address;
            match { SetStateToConstFilter(button, true); }
        }
        
        do {
            match event BUTTON_RELEASED_EVENT_CODE;
            match producer device_address;
            match { SetStateToConstFilter(button, false); }
        }
    ";

    match Parser::parse(text) {
        Ok(event_processors) => println!("{:?}", event_processors),
        Err(err) => println!("{}", err),
    }
}
