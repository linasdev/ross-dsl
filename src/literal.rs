use nom::branch::alt;
use nom::character::complete::alphanumeric1;
use nom::combinator::success;
use nom::sequence::{separated_pair, tuple};
use nom::{Err as NomErr, IResult};
use parse_int::parse;
use std::collections::BTreeMap;
use std::convert::TryFrom;

use ross_config::Value;
use ross_protocol::event::message::MessageValue;

use crate::error::{ErrorKind, Expectation, ParserError};
use crate::keyword::{false_keyword, true_keyword};
use crate::parser::{dec1, hex1, name_parser};
use crate::symbol::tilde;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Literal {
    U8(u8),
    U16(u16),
    U32(u32),
    Bool(bool),
}

pub fn literal_or_constant<'a>(
    constants: &'a BTreeMap<&str, Literal>,
) -> impl FnMut(&str) -> IResult<&str, Literal, ParserError<&str>> + 'a {
    move |text| {
        if let Ok((input, name)) = name_parser(text) {
            if let Some(constant) = constants.get(name) {
                return Ok((input, constant.clone()));
            }
        }

        literal(text)
    }
}

pub fn literal(text: &str) -> IResult<&str, Literal, ParserError<&str>> {
    let boolean_parser = tuple((alt((false_keyword, true_keyword)), success("bool")));
    let hex_parser = separated_pair(hex1, tilde, alphanumeric1);
    let decimal_parser = separated_pair(dec1, tilde, alphanumeric1);

    match alt((boolean_parser, hex_parser, decimal_parser))(text) {
        Ok((input, (value, "u8"))) => {
            if let Ok(value) = parse(value) {
                Ok((input, Literal::U8(value)))
            } else {
                Err(NomErr::Error(ParserError::Base {
                    location: value,
                    kind: ErrorKind::Expected(Expectation::Value),
                    child: None,
                }))
            }
        }
        Ok((input, (value, "u16"))) => {
            if let Ok(value) = parse(value) {
                Ok((input, Literal::U16(value)))
            } else {
                Err(NomErr::Error(ParserError::Base {
                    location: value,
                    kind: ErrorKind::Expected(Expectation::Value),
                    child: None,
                }))
            }
        }
        Ok((input, (value, "u32"))) => {
            if let Ok(value) = parse::<u32>(value) {
                Ok((input, Literal::U32(value)))
            } else {
                Err(NomErr::Error(ParserError::Base {
                    location: value,
                    kind: ErrorKind::Expected(Expectation::Value),
                    child: None,
                }))
            }
        }
        Ok((input, (value, "bool"))) => match value {
            "true" => Ok((input, Literal::Bool(true))),
            "false" => Ok((input, Literal::Bool(false))),
            _ => Err(NomErr::Error(ParserError::Base {
                location: value,
                kind: ErrorKind::Expected(Expectation::Value),
                child: None,
            })),
        },
        Ok((_, (_, literal_type))) => Err(NomErr::Error(ParserError::Base {
            location: literal_type,
            kind: ErrorKind::Expected(Expectation::Type),
            child: None,
        })),
        Err(NomErr::Error(err)) => Err(NomErr::Error(ParserError::Base {
            location: text,
            kind: ErrorKind::Expected(Expectation::Literal),
            child: Some(Box::new(err)),
        })),
        Err(err) => Err(NomErr::convert(err)),
    }
}

impl TryFrom<Literal> for u8 {
    type Error = ParserError<&'static str>;

    fn try_from(literal: Literal) -> Result<Self, Self::Error> {
        match literal {
            Literal::U8(value) => Ok(value),
            Literal::U16(_) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("u16", "u8"),
                child: None,
            }),
            Literal::U32(_) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("u32", "u8"),
                child: None,
            }),
            Literal::Bool(_) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("bool", "u8"),
                child: None,
            }),
        }
    }
}

impl TryFrom<Literal> for u16 {
    type Error = ParserError<&'static str>;

    fn try_from(literal: Literal) -> Result<Self, Self::Error> {
        match literal {
            Literal::U8(_) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("u8", "u16"),
                child: None,
            }),
            Literal::U16(value) => Ok(value),
            Literal::U32(_) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("u32", "u16"),
                child: None,
            }),
            Literal::Bool(_) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("bool", "u16"),
                child: None,
            }),
        }
    }
}

impl TryFrom<Literal> for u32 {
    type Error = ParserError<&'static str>;

    fn try_from(literal: Literal) -> Result<Self, Self::Error> {
        match literal {
            Literal::U8(_) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("u8", "u32"),
                child: None,
            }),
            Literal::U16(_) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("u16", "u32"),
                child: None,
            }),
            Literal::U32(value) => Ok(value),
            Literal::Bool(_) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("bool", "u32"),
                child: None,
            }),
        }
    }
}

impl TryFrom<Literal> for Value {
    type Error = ParserError<&'static str>;

    fn try_from(literal: Literal) -> Result<Self, Self::Error> {
        match literal {
            Literal::U8(value) => Ok(Value::U8(value)),
            Literal::U16(value) => Ok(Value::U16(value)),
            Literal::U32(value) => Ok(Value::U32(value)),
            Literal::Bool(value) => Ok(Value::Bool(value)),
        }
    }
}

impl TryFrom<Literal> for MessageValue {
    type Error = ParserError<&'static str>;

    fn try_from(literal: Literal) -> Result<Self, Self::Error> {
        match literal {
            Literal::U8(value) => Ok(MessageValue::U8(value)),
            Literal::U16(value) => Ok(MessageValue::U16(value)),
            Literal::U32(value) => Ok(MessageValue::U32(value)),
            Literal::Bool(value) => Ok(MessageValue::Bool(value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use cool_asserts::assert_matches;

    #[test]
    fn hex_u8_test() {
        assert_matches!(literal("0xab~u8;input"), Ok((";input", Literal::U8(0xab))));
    }

    #[test]
    fn hex_u8_invalid_value_test() {
        assert_matches!(
            literal("0xabab~u8;input"),
            Err(NomErr::Error(ParserError::Base {
                location,
                kind,
                child,
            })) => {
                assert_matches!(location, "0xabab");
                assert_matches!(kind, ErrorKind::Expected(Expectation::Value));
                assert_matches!(child, None);
            }
        );
    }

    #[test]
    fn hex_u16_test() {
        assert_matches!(
            literal("0xabab~u16;input"),
            Ok((";input", Literal::U16(0xabab)))
        );
    }

    #[test]
    fn hex_u16_invalid_value_test() {
        assert_matches!(
            literal("0xababab~u16;input"),
            Err(NomErr::Error(ParserError::Base {
                location,
                kind,
                child,
            })) => {
                assert_matches!(location, "0xababab");
                assert_matches!(kind, ErrorKind::Expected(Expectation::Value));
                assert_matches!(child, None);
            }
        );
    }

    #[test]
    fn hex_u32_test() {
        assert_matches!(
            literal("0xabababab~u32;input"),
            Ok((";input", Literal::U32(0xabab_abab)))
        );
    }

    #[test]
    fn hex_u32_invalid_value_test() {
        assert_matches!(
            literal("0xababababab~u32;input"),
            Err(NomErr::Error(ParserError::Base {
                location,
                kind,
                child,
            })) => {
                assert_matches!(location, "0xababababab");
                assert_matches!(kind, ErrorKind::Expected(Expectation::Value));
                assert_matches!(child, None);
            }
        );
    }

    #[test]
    fn decimal_u8_test() {
        assert_matches!(literal("123~u8;input"), Ok((";input", Literal::U8(123))));
    }

    #[test]
    fn decimal_u8_negative_test() {
        assert_matches!(
            literal("-123~u8;input"),
            Err(NomErr::Error(ParserError::Base {
                location,
                kind,
                child,
            })) => {
                assert_matches!(location, "-123");
                assert_matches!(kind, ErrorKind::Expected(Expectation::Value));
                assert_matches!(child, None);
            }
        );
    }

    #[test]
    fn decimal_u16_test() {
        assert_matches!(literal("123~u16;input"), Ok((";input", Literal::U16(123))));
    }

    #[test]
    fn decimal_u16_negative_test() {
        assert_matches!(
            literal("-123~u16;input"),
            Err(NomErr::Error(ParserError::Base {
                location,
                kind,
                child,
            })) => {
                assert_matches!(location, "-123");
                assert_matches!(kind, ErrorKind::Expected(Expectation::Value));
                assert_matches!(child, None);
            }
        );
    }

    #[test]
    fn decimal_u32_test() {
        assert_matches!(literal("123~u32;input"), Ok((";input", Literal::U32(123))));
    }

    #[test]
    fn decimal_u32_negative_test() {
        assert_matches!(
            literal("-123~u32;input"),
            Err(NomErr::Error(ParserError::Base {
                location,
                kind,
                child,
            })) => {
                assert_matches!(location, "-123");
                assert_matches!(kind, ErrorKind::Expected(Expectation::Value));
                assert_matches!(child, None);
            }
        );
    }

    #[test]
    fn bool_true_test() {
        assert_matches!(literal("true;input"), Ok((";input", Literal::Bool(true))));
    }

    #[test]
    fn bool_false_test() {
        assert_matches!(literal("false;input"), Ok((";input", Literal::Bool(false))));
    }

    #[test]
    fn no_tilde_test() {
        assert_matches!(
            literal("0xababu16;input"),
            Err(NomErr::Error(ParserError::Base {
                location,
                kind,
                child,
            })) => {
                assert_matches!(location, "0xababu16;input");
                assert_matches!(kind, ErrorKind::Expected(Expectation::Literal));
                assert_matches!(child, Some(_));
            }
        );
    }

    #[test]
    fn double_tilde_test() {
        assert_matches!(
            literal("0xabab~~u16;input"),
            Err(NomErr::Error(ParserError::Base {
                location,
                kind,
                child,
            })) => {
                assert_matches!(location, "0xabab~~u16;input");
                assert_matches!(kind, ErrorKind::Expected(Expectation::Literal));
                assert_matches!(child, Some(_));
            }
        );
    }

    #[test]
    fn no_type_test() {
        assert_matches!(
            literal("0xabab~"),
            Err(NomErr::Error(ParserError::Base {
                location,
                kind,
                child,
            })) => {
                assert_matches!(location, "0xabab~");
                assert_matches!(kind, ErrorKind::Expected(Expectation::Literal));
                assert_matches!(child, Some(_));
            }
        );
    }

    #[test]
    fn invalid_type1_test() {
        assert_matches!(
            literal("0xabab~u15"),
            Err(NomErr::Error(ParserError::Base {
                location,
                kind,
                child,
            })) => {
                assert_matches!(location, "u15");
                assert_matches!(kind, ErrorKind::Expected(Expectation::Type));
                assert_matches!(child, None);
            }
        );
    }

    #[test]
    fn invalid_type2_test() {
        assert_matches!(
            literal("0xabab~u"),
            Err(NomErr::Error(ParserError::Base {
                location,
                kind,
                child,
            })) => {
                assert_matches!(location, "u");
                assert_matches!(kind, ErrorKind::Expected(Expectation::Type));
                assert_matches!(child, None);
            }
        );
    }

    #[test]
    fn unexpected_token_test() {
        assert_matches!(
            literal("asdasd"),
            Err(NomErr::Error(ParserError::Base {
                location,
                kind,
                child,
            })) => {
                assert_matches!(location, "asdasd");
                assert_matches!(kind, ErrorKind::Expected(Expectation::Literal));
                assert_matches!(child, Some(_));
            }
        );
    }

    #[test]
    fn empty_test() {
        assert_matches!(
            literal(""),
            Err(NomErr::Error(ParserError::Base {
                location,
                kind,
                child,
            })) => {
                assert_matches!(location, "");
                assert_matches!(kind, ErrorKind::Expected(Expectation::Literal));
                assert_matches!(child, Some(_));
            }
        );
    }
}
