use nom::branch::alt;
use nom::combinator::success;
use nom::sequence::{separated_pair, tuple};
use nom::{Err, IResult};
use parse_int::parse;

use crate::keyword::{false_keyword, true_keyword};
use crate::parser::{alphanumeric1, dec1, hex1, ParserError};
use crate::symbol::tilde;

#[derive(Debug, PartialEq)]
pub enum Literal {
    U8(u8),
    U16(u16),
    U32(u32),
    Bool(bool),
}

pub fn parse_literal(text: &str) -> IResult<&str, Literal, ParserError> {
    let decimal_parser = separated_pair(dec1, tilde, alphanumeric1);
    let hex_parser = separated_pair(hex1, tilde, alphanumeric1);
    let boolean_parser = tuple((alt((false_keyword, true_keyword)), success("bool")));

    match alt((boolean_parser, hex_parser, decimal_parser))(text) {
        Ok((input, (value, "u8"))) => {
            if let Ok(value) = parse(value) {
                Ok((input, Literal::U8(value)))
            } else {
                Err(Err::Error(ParserError::ExpectedValueFound(
                    text.to_string(),
                    value.to_string(),
                )))
            }
        }
        Ok((input, (value, "u16"))) => {
            if let Ok(value) = parse(value) {
                Ok((input, Literal::U16(value)))
            } else {
                Err(Err::Error(ParserError::ExpectedValueFound(
                    text.to_string(),
                    value.to_string(),
                )))
            }
        }
        Ok((input, (value, "u32"))) => {
            if let Ok(value) = parse::<u32>(value) {
                Ok((input, Literal::U32(value)))
            } else {
                Err(Err::Error(ParserError::ExpectedValueFound(
                    text.to_string(),
                    value.to_string(),
                )))
            }
        }
        Ok((input, (value, "bool"))) => match value {
            "true" => Ok((input, Literal::Bool(true))),
            "false" => Ok((input, Literal::Bool(false))),
            _ => Err(Err::Error(ParserError::ExpectedValueFound(
                text.to_string(),
                value.to_string(),
            ))),
        },
        Ok((_, (_, literal_type))) => Err(Err::Error(ParserError::ExpectedTypeFound(
            text.to_string(),
            literal_type.to_string(),
        ))),
        Err(Err::Error(ParserError::ExpectedNumberFound(input, value))) => {
            Err(Err::Error(ParserError::ExpectedValueFound(input, value)))
        },
        Err(Err::Error(ParserError::ExpectedSymbolFound(input, _, value))) => {
            Err(Err::Error(ParserError::ExpectedValueFound(input, value)))
        },
        Err(err) => Err(Err::convert(err)),
    }
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
            Err(Err::Error(ParserError::ExpectedValueFound(
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
            Err(Err::Error(ParserError::ExpectedValueFound(
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
            Err(Err::Error(ParserError::ExpectedValueFound(
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
            Err(Err::Error(ParserError::ExpectedValueFound(
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
            Err(Err::Error(ParserError::ExpectedValueFound(
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
            Err(Err::Error(ParserError::ExpectedValueFound(
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
    fn bool_false_test() {
        assert_eq!(
            parse_literal("false;input"),
            Ok((";input", Literal::Bool(false)))
        );
    }

    #[test]
    fn bool_invalid_value_test() {
        assert_eq!(
            parse_literal("falsy;input"),
            Err(Err::Error(ParserError::ExpectedValueFound(
                "falsy;input".to_string(),
                "falsy;input".to_string(),
            ))),
        )
    }

    #[test]
    fn no_tilde_test() {
        assert_eq!(
            parse_literal("0xab;input"),
            Err(Err::Error(ParserError::ExpectedValueFound(
                "xab;input".to_string(),
                "xab;input".to_string(),
            ))),
        );
    }

    #[test]
    fn double_tilde_test() {
        assert_eq!(
            parse_literal("0xab~~u8;input"),
            Err(Err::Error(ParserError::ExpectedValueFound(
                "xab~~u8;input".to_string(),
                "xab~~u8;input".to_string()
            )))
        );
    }

    #[test]
    fn invalid_type_test() {
        assert_eq!(
            parse_literal("0xab~u15;input"),
            Err(Err::Error(ParserError::ExpectedTypeFound(
                "0xab~u15;input".to_string(),
                "u15".to_string()
            )))
        );
    }
}
