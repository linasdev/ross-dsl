use nom::character::complete::multispace0;
use nom::combinator::{all_consuming, complete};
use nom::error::ErrorKind as NomErrorKind;
use nom::multi::{many0, separated_list0};
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom::InputTakeAtPosition;
use nom::{AsChar, Err as NomErr, IResult};
use std::collections::BTreeMap;
use std::convert::TryInto;

use ross_config::config::Config;

use crate::error::ParserError;
use crate::literal::{literal, Literal};
use crate::statement::const_statement::const_statement;
use crate::statement::do_statement::do_statement;
use crate::statement::let_statement::let_statement;
use crate::statement::send_statement::send_statement;
use crate::symbol::{close_parenthesis, comma, open_parenthesis};

pub struct Parser {}

impl Parser {
    pub fn parse(text: &str) -> Result<Config, ParserError<&str>> {
        let content_parser = terminated(
            tuple((
                many0(preceded(multispace0, let_statement)),
                many0(preceded(multispace0, const_statement)),
                many0(preceded(multispace0, send_statement)),
                many0(preceded(multispace0, do_statement)),
            )),
            multispace0,
        );

        match all_consuming(complete(content_parser))(text) {
            Ok((_, (initial_state, _constants, mut send_processors, mut do_processors))) => {
                let mut initial_state_map = BTreeMap::new();

                for (i, state) in initial_state.iter().enumerate() {
                    initial_state_map.insert(i as u32, state.1.clone().try_into()?);
                }

                send_processors.append(&mut do_processors);

                Ok(Config {
                    initial_state: initial_state_map,
                    event_processors: send_processors,
                })
            }
            Err(NomErr::Error(err)) => Err(err),
            Err(NomErr::Failure(err)) => Err(err),
            Err(NomErr::Incomplete(_)) => panic!("Unreachable code"),
        }
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
