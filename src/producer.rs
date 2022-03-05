use nom::sequence::{pair, terminated};
use nom::{Err, IResult};
use std::collections::BTreeMap;
use std::convert::TryInto;

use ross_config::producer::*;

use crate::error::{ErrorKind, ParserError};
use crate::literal::Literal;
use crate::parser::{argument_or_constant0, name_parser};
use crate::symbol::semicolon;
use crate::{impl_item_arg0, impl_item_arg1, impl_item_arg3, impl_item_arg4};

pub fn producer<'a>(
    constants: &'a BTreeMap<&str, Literal>,
) -> impl FnMut(&str) -> IResult<&str, Box<dyn Producer>, ParserError<&str>> + 'a {
    move |text| {
        let (input, (name, arguments)) = terminated(
            pair(name_parser, argument_or_constant0(constants)),
            semicolon,
        )(text)?;

        impl_item_arg0!(input, name, arguments, NoneProducer);
        impl_item_arg1!(input, name, arguments, PacketProducer);
        impl_item_arg3!(input, name, arguments, MessageProducer);
        impl_item_arg3!(input, name, arguments, BcmChangeBrightnessProducer);
        impl_item_arg3!(input, name, arguments, BcmChangeBrightnessStateProducer);
        impl_item_arg4!(input, name, arguments, BcmAnimateBrightnessProducer);
        impl_item_arg4!(input, name, arguments, BcmAnimateBrightnessStateProducer);
        impl_item_arg3!(input, name, arguments, RelaySetValueProducer);

        Err(Err::Error(ParserError::Base {
            location: name,
            kind: ErrorKind::UnknownProducer,
            child: None,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ross_protocol::event::bcm::BcmValue;
    use ross_protocol::event::message::MessageValue;
    use ross_protocol::event::relay::{RelayDoubleExclusiveValue, RelayValue};

    use crate::{
        impl_tests_for_item_arg0, impl_tests_for_item_arg1, impl_tests_for_item_arg3,
        impl_tests_for_item_arg4,
    };

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
        ("#234567", BcmValue::Rgb(0x23, 0x45, 0x67))
    );
    impl_tests_for_item_arg3!(
        bcm_change_brightness_state_producer,
        producer,
        BcmChangeBrightnessStateProducer,
        ("0xabab~u16", 0xabab),
        ("0x01~u8", 0x01),
        ("0xabababab~u32", 0xabab_abab)
    );
    impl_tests_for_item_arg4!(
        bcm_animate_brightness_producer,
        producer,
        BcmAnimateBrightnessProducer,
        ("0xabab~u16", 0xabab),
        ("0x01~u8", 0x01),
        ("0xabababab~u32", 0xabab_abab),
        ("#234567", BcmValue::Rgb(0x23, 0x45, 0x67))
    );
    impl_tests_for_item_arg4!(
        bcm_animate_brightness_state_producer,
        producer,
        BcmAnimateBrightnessStateProducer,
        ("0xabab~u16", 0xabab),
        ("0x01~u8", 0x01),
        ("0xabababab~u32", 0xabab_abab),
        ("0xffffffff~u32", 0xffff_ffff)
    );
    impl_tests_for_item_arg3!(
        relay_set_value_producer,
        producer,
        RelaySetValueProducer,
        ("0xabab~u16", 0xabab),
        ("0x01~u8", 0x01),
        (
            "\"first\"",
            RelayValue::DoubleExclusive(RelayDoubleExclusiveValue::FirstChannelOn)
        )
    );
}
