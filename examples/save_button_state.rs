use ross_dsl::Parser;

fn main() {
    let text = "
        store button: bool = false;

        do {
            match {
                EventCodeExtractor();
                U16IsEqualFilter(BUTTON_PRESSED_EVENT_CODE);
            }
        
            match { BoolSetStateFilter(button, true); }
        }
        
        do {
            match {
                EventCodeExtractor();
                U16IsEqualFilter(BUTTON_RELEASED_EVENT_CODE);
            }
        
            match { BoolSetStateFilter(button, false); }
        }
    ";

    println!("{:?}", Parser::parse(text))
}
