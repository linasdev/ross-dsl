use nom::character::complete::{multispace0, multispace1};
use nom::error::ErrorKind as NomErrorKind;
use nom::multi::{separated_list0};
use nom::Err as NomErr;
use nom::sequence::{delimited, preceded, terminated};
use nom::InputTakeAtPosition;
use nom::{AsChar, IResult};
use std::collections::BTreeMap;
use std::convert::TryInto;

use ross_config::config::Config;
use ross_protocol::event::event_code::*;

use crate::error::{ErrorKind, Expectation, ParserError};
use crate::literal::{literal_or_constant, Literal};
use crate::statement::const_statement::const_statement;
use crate::statement::do_statement::do_statement;
use crate::statement::let_statement::let_statement;
use crate::statement::send_statement::send_statement;
use crate::symbol::{close_parenthesis, comma, open_parenthesis};

macro_rules! prepare_constant {
    ($name:expr, $constants:expr, $constant_type:path) => {
        $constants.insert(stringify!($name), $constant_type($name));
    };
}

pub struct Parser {}

impl Parser {
    pub fn parse(mut text: &str) -> Result<Config, ParserError<&str>> {
        let mut initial_state = BTreeMap::new();
        let mut constants = BTreeMap::new();
        let mut event_processors = vec![];

        Self::prepare_constants(&mut constants);

        while text.len() != 0 {
            let mut errors = vec![];

            println!("start\n{:?}\n{:?}\n{:?}\nend\n\n", initial_state, constants, event_processors);

            match preceded(multispace0, let_statement)(text) {
                Ok((input, (name, value))) => {
                    let initial_state_index = initial_state.len() as u32;
                    initial_state.insert(initial_state_index, value.try_into()?);
                    constants.insert(name, Literal::U32(initial_state_index));
                    text = input;
    
                    continue;
                },
                Err(NomErr::Error(err)) => errors.push(err),
                Err(NomErr::Failure(err)) => return Err(err),
                _ => {},
            }

            match preceded(multispace0, const_statement)(text) {
                Ok((input, (name, value))) => {
                    constants.insert(name, value);
                    text = input;

                    continue;
                },
                Err(NomErr::Error(err)) => errors.push(err),
                Err(NomErr::Failure(err)) => return Err(err),
                _ => {},
            }

            match preceded(multispace0, send_statement(&constants))(text) {
                Ok((input, event_processor)) => {
                    event_processors.push(event_processor);
                    text = input;

                    continue;
                },
                Err(NomErr::Error(err)) => errors.push(err),
                Err(NomErr::Failure(err)) => return Err(err),
                _ => {},
            }

            match preceded(multispace0, do_statement(&constants))(text) {
                Ok((input, event_processor)) => {
                    event_processors.push(event_processor);
                    text = input;

                    continue;
                },
                Err(NomErr::Error(err)) => errors.push(err),
                Err(NomErr::Failure(err)) => return Err(err),
                _ => {},
            }

            if let Ok((input, _)) = multispace1::<_, ParserError<&str>>(text) {
                text = input;
                continue;
            }

            return Err(ParserError::Base {
                location: text,
                kind: ErrorKind::Expected(Expectation::Something),
                child: None,
            });
        }
        
        Ok(Config {
            initial_state,
            event_processors,
        })
    }

    fn prepare_constants(constants: &mut BTreeMap<&str, Literal>) {
        prepare_constant!(BOOTLOADER_HELLO_EVENT_CODE, constants, Literal::U16);
        prepare_constant!(PROGRAMMER_HELLO_EVENT_CODE, constants, Literal::U16);
        prepare_constant!(PROGRAMMER_START_FIRMWARE_UPGRADE_EVENT_CODE, constants, Literal::U16);
        prepare_constant!(ACK_EVENT_CODE, constants, Literal::U16);
        prepare_constant!(DATA_EVENT_CODE, constants, Literal::U16);
        prepare_constant!(CONFIGURATOR_HELLO_EVENT_CODE, constants, Literal::U16);
        prepare_constant!(BCM_CHANGE_BRIGHTNESS_EVENT_CODE, constants, Literal::U16);
        prepare_constant!(BUTTON_PRESSED_EVENT_CODE, constants, Literal::U16);
        prepare_constant!(BUTTON_RELEASED_EVENT_CODE, constants, Literal::U16);
        prepare_constant!(INTERNAL_SYSTEM_TICK_EVENT_CODE, constants, Literal::U16);
        prepare_constant!(PROGRAMMER_START_CONFIG_UPGRADE_EVENT_CODE, constants, Literal::U16);
        prepare_constant!(PROGRAMMER_SET_DEVICE_ADDRESS_EVENT_CODE, constants, Literal::U16);
        prepare_constant!(MESSAGE_EVENT_CODE, constants, Literal::U16);
    }
}

pub fn alpha_or_underscore1(text: &str) -> IResult<&str, &str, ParserError<&str>> {
    text.split_at_position1_complete(|item| !item.is_alpha() && item != '_', NomErrorKind::Alpha)
}

pub fn hex1(text: &str) -> IResult<&str, &str, ParserError<&str>> {
    text.split_at_position1_complete(
        |item| !item.is_digit(16) && item != 'x',
        NomErrorKind::HexDigit,
    )
}

pub fn dec1(text: &str) -> IResult<&str, &str, ParserError<&str>> {
    text.split_at_position1_complete(
        |item| !item.is_digit(10) && item != '-',
        NomErrorKind::Digit,
    )
}

pub fn argument0<'a>(constants: &'a BTreeMap<&str, Literal>) -> impl FnMut(&str) -> IResult<&str, Vec<Literal>, ParserError<&str>> + 'a {
    move |text| {
        delimited(
            terminated(open_parenthesis, multispace0),
            separated_list0(comma, delimited(multispace0, literal_or_constant(constants), multispace0)),
            close_parenthesis,
        )(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use cool_asserts::assert_matches;

    use crate::error::{ErrorKind, Expectation, ParserError};

    #[test]
    fn alpha_or_underscore1_test() {
        assert_matches!(alpha_or_underscore1("while123123"), Ok(("123123", "while")));
    }

    #[test]
    fn alpha_or_underscore1_underscore_test() {
        assert_matches!(
            alpha_or_underscore1("while_true123123"),
            Ok(("123123", "while_true"))
        );
    }

    #[test]
    fn alpha_or_underscore1_non_alpha_test() {
        assert_matches!(
            alpha_or_underscore1("123123"),
            Err(NomErr::Error(ParserError::Base {
                location,
                kind,
                child,
            })) => {
                assert_matches!(location, "123123");
                assert_matches!(kind, ErrorKind::Expected(Expectation::Alpha));
                assert_matches!(child, None);
            }
        );
    }

    #[test]
    fn alpha_or_underscore1_empty_test() {
        assert_matches!(
            alpha_or_underscore1(""),
            Err(NomErr::Error(ParserError::Base {
                location,
                kind,
                child,
            })) => {
                assert_matches!(location, "");
                assert_matches!(kind, ErrorKind::Expected(Expectation::Alpha));
                assert_matches!(child, None);
            }
        );
    }

    #[test]
    fn hex1_test() {
        assert_matches!(hex1("0x01ab"), Ok(("", "0x01ab")),);
    }

    #[test]
    fn hex1_non_hex_test() {
        assert_matches!(
            hex1("ghjklp"),
            Err(NomErr::Error(ParserError::Base {
                location,
                kind,
                child,
            })) => {
                assert_matches!(location, "ghjklp");
                assert_matches!(kind, ErrorKind::Expected(Expectation::HexDigit));
                assert_matches!(child, None);
            }
        );
    }

    #[test]
    fn hex1_empty_test() {
        assert_matches!(
            hex1(""),
            Err(NomErr::Error(ParserError::Base {
                location,
                kind,
                child,
            })) => {
                assert_matches!(location, "");
                assert_matches!(kind, ErrorKind::Expected(Expectation::HexDigit));
                assert_matches!(child, None);
            }
        );
    }

    #[test]
    fn dec1_test() {
        assert_matches!(dec1("1234"), Ok(("", "1234")));
    }

    #[test]
    fn dec1_negative_test() {
        assert_matches!(dec1("-1234"), Ok(("", "-1234")));
    }

    #[test]
    fn dec1_non_dec_test() {
        assert_matches!(
            dec1("asdasd"),
            Err(NomErr::Error(ParserError::Base {
                location,
                kind,
                child,
            })) => {
                assert_matches!(location, "asdasd");
                assert_matches!(kind, ErrorKind::Expected(Expectation::Digit));
                assert_matches!(child, None);
            }
        );
    }

    #[test]
    fn dec1_empty_test() {
        assert_matches!(
            dec1(""),
            Err(NomErr::Error(ParserError::Base {
                location,
                kind,
                child,
            })) => {
                assert_matches!(location, "");
                assert_matches!(kind, ErrorKind::Expected(Expectation::Digit));
                assert_matches!(child, None);
            }
        );
    }

    #[test]
    fn argument0_two_arguments_test() {
        let constants = BTreeMap::new();
        assert_matches!(
            argument0(&constants)("(0xab~u16, false)"),
            Ok((input, arguments)) => {
                assert_eq!(input, "");
                assert_eq!(arguments, vec![Literal::U16(0xab), Literal::Bool(false)]);
            }
        );
    }

    #[test]
    fn argument0_one_argument_test() {
        let constants = BTreeMap::new();
        assert_matches!(
            argument0(&constants)("(0xab~u16)"),
            Ok((input, arguments)) => {
                assert_eq!(input, "");
                assert_eq!(arguments, vec![Literal::U16(0xab)]);
            }
        );
    }

    #[test]
    fn argument0_no_arguments_test() {
        let constants = BTreeMap::new();
        assert_matches!(
            argument0(&constants)("(  )"),
            Ok((input, arguments)) => {
                assert_eq!(input, "");
                assert_eq!(arguments, vec![]);
            }
        );
    }

    #[test]
    fn argument0_empty_test() {
        let constants = BTreeMap::new();
        assert_matches!(
            argument0(&constants)(""),
            Err(NomErr::Error(ParserError::Base {
                location,
                kind,
                child,
            })) => {
                assert_matches!(location, "");
                assert_matches!(kind, ErrorKind::Expected(Expectation::Symbol('(')));
                assert_matches!(child, None);
            }
        );
    }
}
