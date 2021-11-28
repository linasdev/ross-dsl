use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::alphanumeric0;
use nom::combinator::{cut, success};
use nom::error::ErrorKind;
use nom::sequence::{separated_pair, tuple};
use nom::{AsChar, Err, IResult, InputTakeAtPosition};
use parse_int::parse;

use crate::parser::ParserError;

#[derive(Debug, PartialEq)]
pub enum Literal {
    U8(u8),
    U16(u16),
    U32(u32),
    Bool(bool),
}

pub fn parse_literal(text: &str) -> IResult<&str, Literal, ParserError> {
    let mut alphanumeric_parser = cut(separated_pair::<_, _, _, _, ParserError, _, _, _>(
        alphanumeric_or_dash1,
        tag("~"),
        alphanumeric0,
    ));
    let mut boolean_parser = cut(tuple((alt((tag("true"), tag("false"))), success("bool"))));

    match alphanumeric_parser(text).or(boolean_parser(text))? {
        (input, (value, "u8")) => {
            eprintln!("{} {}", input, value);
            if let Ok(value) = parse(value) {
                Ok((input, Literal::U8(value)))
            } else {
                Err(Err::Failure(ParserError::ExpectedValueFound(
                    text.to_string(),
                    value.to_string(),
                )))
            }
        }
        (input, (value, "u16")) => {
            if let Ok(value) = parse(value) {
                Ok((input, Literal::U16(value)))
            } else {
                Err(Err::Failure(ParserError::ExpectedValueFound(
                    text.to_string(),
                    value.to_string(),
                )))
            }
        }
        (input, (value, "u32")) => {
            if let Ok(value) = parse::<u32>(value) {
                Ok((input, Literal::U32(value)))
            } else {
                Err(Err::Failure(ParserError::ExpectedValueFound(
                    text.to_string(),
                    value.to_string(),
                )))
            }
        }
        (input, (value, "bool")) => match value {
            "true" => Ok((input, Literal::Bool(true))),
            "false" => Ok((input, Literal::Bool(false))),
            _ => Err(Err::Failure(ParserError::ExpectedValueFound(
                text.to_string(),
                value.to_string(),
            ))),
        },
        (_, (_, literal_type)) => Err(Err::Failure(ParserError::ExpectedTypeFound(
            text.to_string(),
            literal_type.to_string(),
        ))),
    }
}

fn alphanumeric_or_dash1(input: &str) -> IResult<&str, &str, ParserError> {
    input.split_at_position1_complete(
        |item| !item.is_alphanum() && item != '-',
        ErrorKind::AlphaNumeric,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_u8_test() {
        assert_eq!(
            parse_literal("0xab~u8;input"),
            Ok((";input", Literal::U8(0xab)))
        );
    }

    #[test]
    fn hex_u8_invalid_value_test() {
        assert_eq!(
            parse_literal("0xabab~u8;input"),
            Err(Err::Failure(ParserError::ExpectedValueFound(
                "0xabab~u8;input".to_string(),
                "0xabab".to_string()
            )))
        );
    }

    #[test]
    fn hex_u16_test() {
        assert_eq!(
            parse_literal("0xabab~u16;input"),
            Ok((";input", Literal::U16(0xabab)))
        );
    }

    #[test]
    fn hex_u16_invalid_value_test() {
        assert_eq!(
            parse_literal("0xababab~u16;input"),
            Err(Err::Failure(ParserError::ExpectedValueFound(
                "0xababab~u16;input".to_string(),
                "0xababab".to_string()
            )))
        );
    }

    #[test]
    fn hex_u32_test() {
        assert_eq!(
            parse_literal("0xabababab~u32;input"),
            Ok((";input", Literal::U32(0xabab_abab)))
        );
    }

    #[test]
    fn hex_u32_invalid_value_test() {
        assert_eq!(
            parse_literal("0xababababab~u32;input"),
            Err(Err::Failure(ParserError::ExpectedValueFound(
                "0xababababab~u32;input".to_string(),
                "0xababababab".to_string()
            )))
        );
    }

    #[test]
    fn decimal_u8_test() {
        assert_eq!(
            parse_literal("123~u8;input"),
            Ok((";input", Literal::U8(123)))
        );
    }

    #[test]
    fn decimal_u8_negative_test() {
        assert_eq!(
            parse_literal("-123~u8;input"),
            Err(Err::Failure(ParserError::ExpectedValueFound(
                "-123~u8;input".to_string(),
                "-123".to_string()
            )))
        );
    }

    #[test]
    fn decimal_u16_test() {
        assert_eq!(
            parse_literal("123~u16;input"),
            Ok((";input", Literal::U16(123)))
        );
    }

    #[test]
    fn decimal_u16_negative_test() {
        assert_eq!(
            parse_literal("-123~u16;input"),
            Err(Err::Failure(ParserError::ExpectedValueFound(
                "-123~u16;input".to_string(),
                "-123".to_string()
            )))
        );
    }

    #[test]
    fn decimal_u32_test() {
        assert_eq!(
            parse_literal("123~u32;input"),
            Ok((";input", Literal::U32(123)))
        );
    }

    #[test]
    fn decimal_u32_negative_test() {
        assert_eq!(
            parse_literal("-123~u32;input"),
            Err(Err::Failure(ParserError::ExpectedValueFound(
                "-123~u32;input".to_string(),
                "-123".to_string()
            )))
        );
    }

    #[test]
    fn bool_true_test() {
        assert_eq!(
            parse_literal("true;input"),
            Ok((";input", Literal::Bool(true)))
        );
    }

    #[test]
    fn explicit_bool_true_test() {
        assert_eq!(
            parse_literal("true~bool;input"),
            Ok((";input", Literal::Bool(true)))
        );
    }

    #[test]
    fn bool_false_test() {
        assert_eq!(
            parse_literal("false;input"),
            Ok((";input", Literal::Bool(false)))
        );
    }

    #[test]
    fn explicit_bool_false_test() {
        assert_eq!(
            parse_literal("false~bool;input"),
            Ok((";input", Literal::Bool(false)))
        );
    }

    #[test]
    fn bool_invalid_value_test() {
        assert_eq!(
            parse_literal("falsy;input"),
            Err(Err::Failure(ParserError::Nom(
                "falsy;input".to_string(),
                ErrorKind::Tag
            ))),
        );
    }

    #[test]
    fn explicit_bool_invalid_value_test() {
        assert_eq!(
            parse_literal("falsy~bool;input"),
            Err(Err::Failure(ParserError::ExpectedValueFound(
                "falsy~bool;input".to_string(),
                "falsy".to_string()
            )))
        );
    }

    #[test]
    fn no_tilde_test() {
        assert_eq!(
            parse_literal("0xab;input"),
            Err(Err::Failure(ParserError::Nom(
                "0xab;input".to_string(),
                ErrorKind::Tag
            ))),
        );
    }

    #[test]
    fn double_tilde_test() {
        assert_eq!(
            parse_literal("0xab~~u8;input"),
            Err(Err::Failure(ParserError::ExpectedTypeFound(
                "0xab~~u8;input".to_string(),
                "".to_string()
            )))
        );
    }

    #[test]
    fn invalid_type_test() {
        assert_eq!(
            parse_literal("0xab~u15;input"),
            Err(Err::Failure(ParserError::ExpectedTypeFound(
                "0xab~u15;input".to_string(),
                "u15".to_string()
            )))
        );
    }
}
