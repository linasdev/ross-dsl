use nom::character::complete::multispace0;
use nom::error::ErrorKind as NomErrorKind;
use nom::multi::{separated_list0};
use nom::Err as NomErr;
use nom::sequence::{delimited, preceded, terminated};
use nom::InputTakeAtPosition;
use nom::{AsChar, IResult};
use std::collections::BTreeMap;
use std::convert::TryInto;

use ross_config::config::Config;

use crate::error::{ErrorKind, Expectation, ParserError};
use crate::literal::{literal, Literal};
use crate::statement::const_statement::const_statement;
use crate::statement::do_statement::do_statement;
use crate::statement::let_statement::let_statement;
use crate::statement::send_statement::send_statement;
use crate::symbol::{close_parenthesis, comma, open_parenthesis};

pub struct Parser {}

impl Parser {
    pub fn parse(mut text: &str) -> Result<Config, ParserError<&str>> {
        let mut initial_state = BTreeMap::new();
        let mut constants = BTreeMap::new();
        let mut event_processors = vec![];

        while text.len() != 0 {
            let mut errors = vec![];
            
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

            match preceded(multispace0, send_statement)(text) {
                Ok((input, event_processor)) => {
                    event_processors.push(event_processor);
                    text = input;

                    continue;
                },
                Err(NomErr::Error(err)) => errors.push(err),
                Err(NomErr::Failure(err)) => return Err(err),
                _ => {},
            }

            match preceded(multispace0, do_statement)(text) {
                Ok((input, event_processor)) => {
                    event_processors.push(event_processor);
                    text = input;

                    continue;
                },
                Err(NomErr::Error(err)) => errors.push(err),
                Err(NomErr::Failure(err)) => return Err(err),
                _ => {},
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
