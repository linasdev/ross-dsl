use nom::character::complete::{multispace0, multispace1};
use nom::error::ErrorKind as NomErrorKind;
use nom::multi::separated_list0;
use nom::sequence::{delimited, preceded, terminated};
use nom::Err as NomErr;
use nom::InputTakeAtPosition;
use nom::{AsChar, IResult};
use std::collections::BTreeMap;
use std::convert::TryInto;

use ross_config::config::Config;
use ross_protocol::event::event_code::*;

use crate::error::{ErrorKind, Expectation, ParserError};
use crate::literal::{literal, literal_or_constant, Literal};
use crate::statement::const_statement::const_statement;
use crate::statement::do_statement::do_statement;
use crate::statement::let_statement::let_statement;
use crate::statement::peripheral_statement::peripheral_statement;
use crate::statement::send_statement::send_statement;
use crate::statement::set_statement::set_statement;
use crate::symbol::{close_parenthesis, comma, open_parenthesis};

macro_rules! prepare_constant {
    ($name:expr, $constants:expr, $constant_type:path) => {
        $constants.insert(stringify!($name), $constant_type($name));
    };
}

pub struct Parser {}

impl Parser {
    pub fn parse<'a, 'b>(text: &'a str) -> Result<Config, ParserError<String>> {
        let mut peripherals = BTreeMap::new();
        let mut initial_state = BTreeMap::new();
        let mut state_variables = BTreeMap::new();
        let mut constants = BTreeMap::new();
        let mut event_processors = vec![];

        Self::prepare_constants(&mut constants);

        let commentless_text_string = Self::remove_comments(text.to_string());
        let mut commentless_text = commentless_text_string.as_str();

        while commentless_text.len() != 0 {
            let mut errors = vec![];

            match preceded(multispace0, peripheral_statement(&constants))(commentless_text) {
                Ok((input, (index, peripheral))) => {
                    peripherals.insert(index, peripheral);
                    commentless_text = input;
                    continue;
                }
                Err(NomErr::Error(err)) => errors.push(err),
                Err(NomErr::Failure(err)) => return Err(err.into()),
                _ => {}
            }

            match preceded(multispace0, let_statement)(commentless_text) {
                Ok((input, (name, value))) => {
                    let initial_state_index = initial_state.len() as u32;
                    initial_state.insert(initial_state_index, value.try_into()?);
                    state_variables.insert(name, initial_state_index);
                    constants.insert(name, Literal::U32(initial_state_index));
                    commentless_text = input;

                    continue;
                }
                Err(NomErr::Error(err)) => errors.push(err),
                Err(NomErr::Failure(err)) => return Err(err.into()),
                _ => {}
            }

            match preceded(multispace0, const_statement)(commentless_text) {
                Ok((input, (name, value))) => {
                    constants.insert(name, value);
                    commentless_text = input;

                    continue;
                }
                Err(NomErr::Error(err)) => errors.push(err),
                Err(NomErr::Failure(err)) => return Err(err.into()),
                _ => {}
            }

            match preceded(multispace0, send_statement(&constants))(commentless_text) {
                Ok((input, event_processor)) => {
                    event_processors.push(event_processor);
                    commentless_text = input;

                    continue;
                }
                Err(NomErr::Error(err)) => errors.push(err),
                Err(NomErr::Failure(err)) => return Err(err.into()),
                _ => {}
            }

            match preceded(multispace0, do_statement(&constants))(commentless_text) {
                Ok((input, event_processor)) => {
                    event_processors.push(event_processor);
                    commentless_text = input;

                    continue;
                }
                Err(NomErr::Error(err)) => errors.push(err),
                Err(NomErr::Failure(err)) => return Err(err.into()),
                _ => {}
            }

            match preceded(multispace0, set_statement(&constants, &state_variables))(commentless_text) {
                Ok((input, event_processor)) => {
                    event_processors.push(event_processor);
                    commentless_text = input;

                    continue;
                }
                Err(NomErr::Error(err)) => errors.push(err),
                Err(NomErr::Failure(err)) => return Err(err.into()),
                _ => {}
            }

            if let Ok((input, _)) = multispace1::<_, ParserError<&str>>(commentless_text) {
                commentless_text = input;
                continue;
            }

            return Err(ParserError::Base {
                location: commentless_text.to_string(),
                kind: ErrorKind::Expected(Expectation::Something),
                child: None,
            });
        }

        Ok(Config {
            peripherals,
            initial_state,
            event_processors,
        })
    }

    fn remove_comments(text: String) -> String {
        let mut result = "".to_string();

        for line in text.lines() {
            if let Some(code_portion) = line.split("//").nth(0) {
                result += code_portion;
            }

            result += "\n";
        }

        result
    }

    fn prepare_constants(constants: &mut BTreeMap<&str, Literal>) {
        prepare_constant!(BOOTLOADER_HELLO_EVENT_CODE, constants, Literal::U16);
        prepare_constant!(PROGRAMMER_HELLO_EVENT_CODE, constants, Literal::U16);
        prepare_constant!(
            PROGRAMMER_START_FIRMWARE_UPGRADE_EVENT_CODE,
            constants,
            Literal::U16
        );
        prepare_constant!(ACK_EVENT_CODE, constants, Literal::U16);
        prepare_constant!(DATA_EVENT_CODE, constants, Literal::U16);
        prepare_constant!(CONFIGURATOR_HELLO_EVENT_CODE, constants, Literal::U16);
        prepare_constant!(BCM_CHANGE_BRIGHTNESS_EVENT_CODE, constants, Literal::U16);
        prepare_constant!(BUTTON_PRESSED_EVENT_CODE, constants, Literal::U16);
        prepare_constant!(BUTTON_RELEASED_EVENT_CODE, constants, Literal::U16);
        prepare_constant!(INTERNAL_SYSTEM_TICK_EVENT_CODE, constants, Literal::U16);
        prepare_constant!(
            PROGRAMMER_START_CONFIG_UPGRADE_EVENT_CODE,
            constants,
            Literal::U16
        );
        prepare_constant!(
            PROGRAMMER_SET_DEVICE_ADDRESS_EVENT_CODE,
            constants,
            Literal::U16
        );
        prepare_constant!(MESSAGE_EVENT_CODE, constants, Literal::U16);
    }
}

pub fn name_parser(text: &str) -> IResult<&str, &str, ParserError<&str>> {
    if let Some(character) = text.chars().nth(0) {
        if character.is_digit(10) || character == '_' {
            return Err(NomErr::Error(ParserError::Base {
                location: text,
                kind: ErrorKind::Expected(Expectation::Name),
                child: Some(Box::new(ParserError::Base {
                    location: text,
                    kind: ErrorKind::Expected(Expectation::Alpha),
                    child: None,
                })),
            }));
        }
    }

    match text.split_at_position1_complete(
        |item| !item.is_alpha() && !item.is_digit(10) && item != '_',
        NomErrorKind::Alpha,
    ) {
        Ok((input, name)) => Ok((input, name)),
        Err(NomErr::Error(ParserError::Base {
            location,
            kind: ErrorKind::Expected(Expectation::Alpha),
            child,
        })) => Err(NomErr::Error(ParserError::Base {
            location,
            kind: ErrorKind::Expected(Expectation::Name),
            child: Some(Box::new(ParserError::Base {
                location,
                kind: ErrorKind::Expected(Expectation::Alpha),
                child,
            })),
        })),
        err => err,
    }
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

pub fn argument_or_constant0<'a>(
    constants: &'a BTreeMap<&str, Literal>,
) -> impl FnMut(&str) -> IResult<&str, Vec<Literal>, ParserError<&str>> + 'a {
    move |text| {
        delimited(
            terminated(open_parenthesis, multispace0),
            separated_list0(
                comma,
                delimited(multispace0, literal_or_constant(constants), multispace0),
            ),
            close_parenthesis,
        )(text)
    }
}

pub fn argument0(text: &str) -> IResult<&str, Vec<Literal>, ParserError<&str>> {
    delimited(
        terminated(open_parenthesis, multispace0),
        separated_list0(comma, delimited(multispace0, literal, multispace0)),
        close_parenthesis,
    )(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    use cool_asserts::assert_matches;

    use crate::error::{ErrorKind, Expectation, ParserError};

    #[test]
    fn name_parser_test() {
        assert_matches!(name_parser("while;input"), Ok((";input", "while")));
    }

    #[test]
    fn name_parser_underscore_test() {
        assert_matches!(
            name_parser("while_true;input"),
            Ok((";input", "while_true")),
        );
    }

    #[test]
    fn name_parser_first_character_digit_test() {
        assert_matches!(
            name_parser("123123"),
            Err(NomErr::Error(ParserError::Base {
                location,
                kind,
                child,
            })) => {
                assert_matches!(location, "123123");
                assert_matches!(kind, ErrorKind::Expected(Expectation::Name));
                assert_matches!(child, Some(_));
            },
        );
    }

    #[test]
    fn name_parser_first_character_underscore_test() {
        assert_matches!(
            name_parser("_while;input"),
            Err(NomErr::Error(ParserError::Base {
                location,
                kind,
                child,
            })) => {
                assert_matches!(location, "_while;input");
                assert_matches!(kind, ErrorKind::Expected(Expectation::Name));
                assert_matches!(child, Some(_));
            },
        );
    }

    #[test]
    fn name_parser_empty_test() {
        assert_matches!(
            name_parser(""),
            Err(NomErr::Error(ParserError::Base {
                location,
                kind,
                child,
            })) => {
                assert_matches!(location, "");
                assert_matches!(kind, ErrorKind::Expected(Expectation::Name));
                assert_matches!(child, Some(_));
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
    fn argument_or_constant0_two_arguments_test() {
        let constants = BTreeMap::new();
        assert_matches!(
            argument_or_constant0(&constants)("(0xab~u16, false)"),
            Ok((input, arguments)) => {
                assert_eq!(input, "");
                assert_eq!(arguments, vec![Literal::U16(0xab), Literal::Bool(false)]);
            }
        );
    }

    #[test]
    fn argument_or_constant0_one_argument_test() {
        let constants = BTreeMap::new();
        assert_matches!(
            argument_or_constant0(&constants)("(0xab~u16)"),
            Ok((input, arguments)) => {
                assert_eq!(input, "");
                assert_eq!(arguments, vec![Literal::U16(0xab)]);
            }
        );
    }

    #[test]
    fn argument_or_constant0_no_arguments_test() {
        let constants = BTreeMap::new();
        assert_matches!(
            argument_or_constant0(&constants)("(  )"),
            Ok((input, arguments)) => {
                assert_eq!(input, "");
                assert_eq!(arguments, vec![]);
            }
        );
    }

    #[test]
    fn argument_or_constant0_empty_test() {
        let constants = BTreeMap::new();
        assert_matches!(
            argument_or_constant0(&constants)(""),
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

    #[test]
    fn argument0_two_arguments_test() {
        assert_matches!(
            argument0("(0xab~u16, false)"),
            Ok((input, arguments)) => {
                assert_eq!(input, "");
                assert_eq!(arguments, vec![Literal::U16(0xab), Literal::Bool(false)]);
            }
        );
    }

    #[test]
    fn argument0_one_argument_test() {
        assert_matches!(
            argument0("(0xab~u16)"),
            Ok((input, arguments)) => {
                assert_eq!(input, "");
                assert_eq!(arguments, vec![Literal::U16(0xab)]);
            }
        );
    }

    #[test]
    fn argument0_no_arguments_test() {
        assert_matches!(
            argument0("(  )"),
            Ok((input, arguments)) => {
                assert_eq!(input, "");
                assert_eq!(arguments, vec![]);
            }
        );
    }

    #[test]
    fn argument0_empty_test() {
        assert_matches!(
            argument0(""),
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
