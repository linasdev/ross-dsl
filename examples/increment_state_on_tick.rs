use ross_dsl::Parser;

fn main() {
    let text = "
        let time: u32 = 0;

        const device_address = 0x0002;

        do {
            match tick;
            match producer device_address;
            match { U32IncrementStateFilter(time); }
        }
    ";

    println!("{:?}", Parser::parse(text))
}
