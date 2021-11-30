use nom::sequence::{pair, terminated};
use nom::{Err, IResult};
use std::collections::BTreeMap;

use ross_config::extractor::*;

use crate::error::{ErrorKind, ParserError};
use crate::impl_item_arg0;
use crate::literal::Literal;
use crate::parser::{alpha_or_underscore1, argument0};
use crate::symbol::semicolon;

pub fn extractor<'a>(
    constants: &'a BTreeMap<&str, Literal>,
) -> impl FnMut(&str) -> IResult<&str, Box<dyn Extractor>, ParserError<&str>> + 'a {
    move |text| {
        let (input, (name, arguments)) =
            terminated(pair(alpha_or_underscore1, argument0(constants)), semicolon)(text)?;

        impl_item_arg0!(input, name, arguments, NoneExtractor);
        impl_item_arg0!(input, name, arguments, PacketExtractor);
        impl_item_arg0!(input, name, arguments, EventCodeExtractor);
        impl_item_arg0!(input, name, arguments, EventProducerAddressExtractor);
        impl_item_arg0!(input, name, arguments, MessageCodeExtractor);
        impl_item_arg0!(input, name, arguments, MessageValueExtractor);
        impl_item_arg0!(input, name, arguments, ButtonIndexExtractor);

        Err(Err::Error(ParserError::Base {
            location: name,
            kind: ErrorKind::UnknownExtractor,
            child: None,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::impl_tests_for_item_arg0;

    impl_tests_for_item_arg0!(none_extractor, extractor, NoneExtractor);
    impl_tests_for_item_arg0!(packet_extractor, extractor, PacketExtractor);
    impl_tests_for_item_arg0!(event_code_extractor, extractor, EventCodeExtractor);
    impl_tests_for_item_arg0!(
        event_producer_address_extractor,
        extractor,
        EventProducerAddressExtractor
    );
    impl_tests_for_item_arg0!(message_code_extractor, extractor, MessageCodeExtractor);
    impl_tests_for_item_arg0!(message_value_extractor, extractor, MessageValueExtractor);
    impl_tests_for_item_arg0!(button_index_extractor, extractor, ButtonIndexExtractor);
}
