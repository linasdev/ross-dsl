use nom::branch::alt;
use nom::combinator::success;
use nom::sequence::{separated_pair, tuple};
use nom::{Err, IResult};
use parse_int::parse;
use std::convert::TryInto;

use ross_config::Value;

use crate::keyword::{false_keyword, true_keyword};
use crate::parser::{alphanumeric1, dec1, hex1, ParserError};
use crate::symbol::tilde;

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    U8(u8),
    U16(u16),
    U32(u32),
    Bool(bool),
}

pub fn literal(text: &str) -> IResult<&str, Literal, ParserError> {
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
        }
        Err(Err::Error(ParserError::ExpectedSymbolFound(input, _, value))) => {
            Err(Err::Error(ParserError::ExpectedTypeFound(input, value)))
        }
        Err(err) => Err(Err::convert(err)),
    }
}

impl TryInto<u8> for Literal {
    type Error = ParserError;

    fn try_into(self) -> Result<u8, Self::Error> {
        match self {
            Literal::U8(value) => Ok(value),
            Literal::U16(_) => Err(ParserError::CastFromToNotAllowed(
                "u16".to_string(),
                "u8".to_string(),
            )),
            Literal::U32(_) => Err(ParserError::CastFromToNotAllowed(
                "u32".to_string(),
                "u8".to_string(),
            )),
            Literal::Bool(_) => Err(ParserError::CastFromToNotAllowed(
                "bool".to_string(),
                "u8".to_string(),
            )),
        }
    }
}

impl TryInto<u16> for Literal {
    type Error = ParserError;

    fn try_into(self) -> Result<u16, Self::Error> {
        match self {
            Literal::U8(_) => Err(ParserError::CastFromToNotAllowed(
                "u8".to_string(),
                "u16".to_string(),
            )),
            Literal::U16(value) => Ok(value),
            Literal::U32(_) => Err(ParserError::CastFromToNotAllowed(
                "u32".to_string(),
                "u16".to_string(),
            )),
            Literal::Bool(_) => Err(ParserError::CastFromToNotAllowed(
                "bool".to_string(),
                "u16".to_string(),
            )),
        }
    }
}

impl TryInto<u32> for Literal {
    type Error = ParserError;

    fn try_into(self) -> Result<u32, Self::Error> {
        match self {
            Literal::U8(_) => Err(ParserError::CastFromToNotAllowed(
                "u8".to_string(),
                "u32".to_string(),
            )),
            Literal::U16(_) => Err(ParserError::CastFromToNotAllowed(
                "u16".to_string(),
                "u32".to_string(),
            )),
            Literal::U32(value) => Ok(value),
            Literal::Bool(_) => Err(ParserError::CastFromToNotAllowed(
                "bool".to_string(),
                "u32".to_string(),
            )),
        }
    }
}

impl TryInto<bool> for Literal {
    type Error = ParserError;

    fn try_into(self) -> Result<bool, Self::Error> {
        match self {
            Literal::U8(_) => Err(ParserError::CastFromToNotAllowed(
                "u8".to_string(),
                "bool".to_string(),
            )),
            Literal::U16(_) => Err(ParserError::CastFromToNotAllowed(
                "u16".to_string(),
                "bool".to_string(),
            )),
            Literal::U32(_) => Err(ParserError::CastFromToNotAllowed(
                "u32".to_string(),
                "bool".to_string(),
            )),
            Literal::Bool(value) => Ok(value),
        }
    }
}

impl TryInto<Value> for Literal {
    type Error = ParserError;

    fn try_into(self) -> Result<Value, Self::Error> {
        match self {
            Literal::U8(value) => Ok(Value::U8(value)),
            Literal::U16(value) => Ok(Value::U16(value)),
            Literal::U32(value) => Ok(Value::U32(value)),
            Literal::Bool(value) => Ok(Value::Bool(value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_u8_test() {
        assert_eq!(literal("0xab~u8;input"), Ok((";input", Literal::U8(0xab))));
    }

    #[test]
    fn hex_u8_invalid_value_test() {
        assert_eq!(
            literal("0xabab~u8;input"),
            Err(Err::Error(ParserError::ExpectedValueFound(
                "0xabab~u8;input".to_string(),
                "0xabab".to_string()
            )))
        );
    }

    #[test]
    fn hex_u16_test() {
        assert_eq!(
            literal("0xabab~u16;input"),
            Ok((";input", Literal::U16(0xabab)))
        );
    }

    #[test]
    fn hex_u16_invalid_value_test() {
        assert_eq!(
            literal("0xababab~u16;input"),
            Err(Err::Error(ParserError::ExpectedValueFound(
                "0xababab~u16;input".to_string(),
                "0xababab".to_string()
            )))
        );
    }

    #[test]
    fn hex_u32_test() {
        assert_eq!(
            literal("0xabababab~u32;input"),
            Ok((";input", Literal::U32(0xabab_abab)))
        );
    }

    #[test]
    fn hex_u32_invalid_value_test() {
        assert_eq!(
            literal("0xababababab~u32;input"),
            Err(Err::Error(ParserError::ExpectedValueFound(
                "0xababababab~u32;input".to_string(),
                "0xababababab".to_string()
            )))
        );
    }

    #[test]
    fn decimal_u8_test() {
        assert_eq!(literal("123~u8;input"), Ok((";input", Literal::U8(123))));
    }

    #[test]
    fn decimal_u8_negative_test() {
        assert_eq!(
            literal("-123~u8;input"),
            Err(Err::Error(ParserError::ExpectedValueFound(
                "-123~u8;input".to_string(),
                "-123".to_string()
            )))
        );
    }

    #[test]
    fn decimal_u16_test() {
        assert_eq!(literal("123~u16;input"), Ok((";input", Literal::U16(123))));
    }

    #[test]
    fn decimal_u16_negative_test() {
        assert_eq!(
            literal("-123~u16;input"),
            Err(Err::Error(ParserError::ExpectedValueFound(
                "-123~u16;input".to_string(),
                "-123".to_string()
            )))
        );
    }

    #[test]
    fn decimal_u32_test() {
        assert_eq!(literal("123~u32;input"), Ok((";input", Literal::U32(123))));
    }

    #[test]
    fn decimal_u32_negative_test() {
        assert_eq!(
            literal("-123~u32;input"),
            Err(Err::Error(ParserError::ExpectedValueFound(
                "-123~u32;input".to_string(),
                "-123".to_string()
            )))
        );
    }

    #[test]
    fn bool_true_test() {
        assert_eq!(literal("true;input"), Ok((";input", Literal::Bool(true))));
    }

    #[test]
    fn bool_false_test() {
        assert_eq!(literal("false;input"), Ok((";input", Literal::Bool(false))));
    }

    #[test]
    fn bool_invalid_value_test() {
        assert_eq!(
            literal("falsy;input"),
            Err(Err::Error(ParserError::ExpectedValueFound(
                "falsy;input".to_string(),
                "falsy;input".to_string(),
            ))),
        )
    }

    #[test]
    fn no_tilde_test() {
        assert_eq!(
            literal("0xab;input"),
            Err(Err::Error(ParserError::ExpectedTypeFound(
                "xab;input".to_string(),
                "xab;input".to_string(),
            ))),
        );
    }

    #[test]
    fn double_tilde_test() {
        assert_eq!(
            literal("0xab~~u8;input"),
            Err(Err::Error(ParserError::ExpectedTypeFound(
                "xab~~u8;input".to_string(),
                "xab~~u8;input".to_string()
            )))
        );
    }

    #[test]
    fn no_type_test() {
        assert_eq!(
            literal("0xab~"),
            Err(Err::Error(ParserError::ExpectedTypeFound(
                "xab~".to_string(),
                "xab~".to_string(),
            ))),
        );
    }

    #[test]
    fn invalid_type1_test() {
        assert_eq!(
            literal("0xab~u15;input"),
            Err(Err::Error(ParserError::ExpectedTypeFound(
                "0xab~u15;input".to_string(),
                "u15".to_string()
            )))
        );
    }

    #[test]
    fn invalid_type2_test() {
        assert_eq!(
            literal("0xab~u"),
            Err(Err::Error(ParserError::ExpectedTypeFound(
                "0xab~u".to_string(),
                "u".to_string(),
            ))),
        );
    }
}
