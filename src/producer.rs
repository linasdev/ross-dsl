use nom::sequence::{pair, terminated};
use nom::{Err, IResult};
use std::convert::TryInto;

use ross_config::producer::*;

use crate::parser::{alpha1, argument0, ParserError};
use crate::symbol::semicolon;
use crate::{impl_item_arg0, impl_item_arg1, impl_item_arg3};

pub fn producer(text: &str) -> IResult<&str, Box<dyn Producer>, ParserError> {
    let (input, (name, arguments)) = terminated(pair(alpha1, argument0), semicolon)(text)?;

    impl_item_arg0!(input, name, arguments, NoneProducer);
    impl_item_arg1!(input, name, arguments, PacketProducer);
    impl_item_arg3!(input, name, arguments, MessageProducer);
    impl_item_arg3!(input, name, arguments, BcmChangeBrightnessProducer);
    impl_item_arg3!(input, name, arguments, BcmChangeBrightnessStateProducer);

    Err(Err::Error(ParserError::UnknownProducer(
        text.to_string(),
        name.to_string(),
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    use ross_protocol::event::message::MessageValue;

    use crate::{impl_tests_for_item_arg0, impl_tests_for_item_arg1, impl_tests_for_item_arg3};

    impl_tests_for_item_arg0!(none_producer, producer, NoneProducer);
    impl_tests_for_item_arg1!(
        packet_producer,
        producer,
        PacketProducer,
        ("0xabab~u16", 0xabab)
    );
    impl_tests_for_item_arg3!(
        message_producer,
        producer,
        MessageProducer,
        ("0xabab~u16", 0xabab),
        ("0x0123~u16", 0x0123),
        ("false", MessageValue::Bool(false))
    );
    impl_tests_for_item_arg3!(
        bcm_change_brightness_producer,
        producer,
        BcmChangeBrightnessProducer,
        ("0xabab~u16", 0xabab),
        ("0x01~u8", 0x01),
        ("0xff~u8", 0xff)
    );
    impl_tests_for_item_arg3!(
        bcm_change_brightness_state_producer,
        producer,
        BcmChangeBrightnessStateProducer,
        ("0xabab~u16", 0xabab),
        ("0x01~u8", 0x01),
        ("0xabababab~u32", 0xabab_abab)
    );
}
