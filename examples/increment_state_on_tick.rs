use ross_dsl::Parser;

fn main() {
    let text = "
        let time = 0~u32;

        const device_address = 0x0002~u16;

        do {
            match tick;
            match producer device_address;
            match { IncrementStateByConstFilter(time, 1~u32); }
        }
    ";

    println!("{:?}", Parser::parse(text))
}
