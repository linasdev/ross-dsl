use nom::sequence::{pair, terminated};
use nom::{Err, IResult};

use ross_config::extractor::*;

use crate::parser::{alpha1, argument0, ParserError};
use crate::symbol::semicolon;

macro_rules! impl_extractor_arg0 {
    ($input:expr, $name:expr, $arguments:expr, $extractor_type:ty) => {
        if $name == stringify!($extractor_type) {
            return if $arguments.len() == 0 {
                Ok(($input, Box::new(<$extractor_type>::new())))
            } else {
                Err(Err::Error(ParserError::ExpectedArgumentsButGot(
                    $input.to_string(),
                    0,
                    $arguments.len(),
                )))
            };
        }
    };
}

pub fn extractor(text: &str) -> IResult<&str, Box<dyn Extractor>, ParserError> {
    let (input, (name, arguments)) = terminated(pair(alpha1, argument0), semicolon)(text)?;

    impl_extractor_arg0!(input, name, arguments, NoneExtractor);
    impl_extractor_arg0!(input, name, arguments, PacketExtractor);
    impl_extractor_arg0!(input, name, arguments, EventCodeExtractor);
    impl_extractor_arg0!(input, name, arguments, EventProducerAddressExtractor);
    impl_extractor_arg0!(input, name, arguments, MessageCodeExtractor);
    impl_extractor_arg0!(input, name, arguments, MessageValueExtractor);
    impl_extractor_arg0!(input, name, arguments, ButtonIndexExtractor);

    Err(Err::Error(ParserError::UnknownExtractor(
        text.to_string(),
        name.to_string(),
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! impl_tests_extractor_arg0 {
        ($test_module_name:ident, $extractor_type:ty) => {
            mod $test_module_name {
                use super::*;

                #[test]
                fn extractor_test() {
                    let (input, extractor) =
                    extractor(concat!(stringify!($extractor_type), "( );input")).unwrap();

                    assert_eq!(input, "input");
                    assert_eq!(
                        format!("{:?}", extractor),
                        format!("{:?}", <$extractor_type>::new())
                    );
                }

                #[test]
                fn missing_semicolon_test() {
                    assert_eq!(
                        extractor(concat!(stringify!($extractor_type), "( )input"))
                            .unwrap_err(),
                        Err::Error(ParserError::ExpectedSymbolFound(
                            "input".to_string(),
                            ";".to_string(),
                            "input".to_string()
                        ))
                    );
                }

                #[test]

                fn too_many_arguments_test() {
                    assert_eq!(
                        extractor(concat!(stringify!($extractor_type), "( false );input"))
                            .unwrap_err(),
                        Err::Error(ParserError::ExpectedArgumentsButGot(
                            "input".to_string(),
                            0,
                            1,
                        ))
                    );
                }
            }
        };
    }

    impl_tests_extractor_arg0!(none_extractor, NoneExtractor);
    impl_tests_extractor_arg0!(packet_extractor, PacketExtractor);
    impl_tests_extractor_arg0!(event_code_extractor, EventCodeExtractor);
    impl_tests_extractor_arg0!(
        event_producer_address_extractor,
        EventProducerAddressExtractor
    );
    impl_tests_extractor_arg0!(message_code_extractor, MessageCodeExtractor);
    impl_tests_extractor_arg0!(message_value_extractor, MessageValueExtractor);
    impl_tests_extractor_arg0!(button_index_extractor, ButtonIndexExtractor);
}
