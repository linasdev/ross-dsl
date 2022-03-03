use cron_parser::parse_field;
use nom::branch::alt;
use nom::bytes::complete::take_until;
use nom::character::complete::alphanumeric1;
use nom::combinator::success;
use nom::sequence::{delimited, preceded, separated_pair, tuple};
use nom::{Err as NomErr, IResult};
use parse_int::parse;
use std::collections::BTreeMap;
use std::convert::TryFrom;

use ross_config::cron::{CronExpression, CronField};
use ross_config::Value;
use ross_protocol::event::bcm::BcmValue;
use ross_protocol::event::message::MessageValue;
use ross_protocol::event::relay::{RelayDoubleExclusiveValue, RelayValue};

use crate::error::{ErrorKind, Expectation, ParserError};
use crate::keyword::{false_keyword, true_keyword};
use crate::parser::{dec1, hex1, name_parser};
use crate::symbol::{double_quote, hashtag, tilde};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Literal {
    U8(u8),
    U16(u16),
    U32(u32),
    Bool(bool),
    String(String),
    Rgb(u8, u8, u8, u8),
    Rgbw(u8, u8, u8, u8, u8),
}

pub fn state_variable<'a>(
    state_variables: &'a BTreeMap<&str, u32>,
) -> impl FnMut(&str) -> IResult<&str, u32, ParserError<&str>> + 'a {
    move |text| {
        let (input, name) = name_parser(text)?;

        if let Some(state_index) = state_variables.get(name) {
            Ok((input, *state_index))
        } else {
            Err(NomErr::Error(ParserError::Base {
                location: text,
                kind: ErrorKind::Expected(Expectation::StateVariable),
                child: None,
            }))
        }
    }
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
    let string_parser = tuple((
        delimited(double_quote, take_until("\""), double_quote),
        success("string"),
    ));
    let color_parser = tuple((preceded(hashtag, hex1), success("color")));

    match alt((
        boolean_parser,
        hex_parser,
        decimal_parser,
        string_parser,
        color_parser,
    ))(text)
    {
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
        Ok((input, (value, "string"))) => Ok((input, Literal::String(value.to_string()))),
        Ok((input, (value, "color"))) => {
            if value.len() == 8 {
                let r = u8::from_str_radix(&value[0..2], 16);
                let g = u8::from_str_radix(&value[2..4], 16);
                let b = u8::from_str_radix(&value[4..6], 16);
                let brightness = u8::from_str_radix(&value[6..8], 16);

                if let (Ok(r), Ok(g), Ok(b), Ok(brightness)) = (r, g, b, brightness) {
                    Ok((input, Literal::Rgb(r, g, b, brightness)))
                } else {
                    Err(NomErr::Error(ParserError::Base {
                        location: text,
                        kind: ErrorKind::Expected(Expectation::Value),
                        child: None,
                    }))
                }
            } else if value.len() == 10 {
                let r = u8::from_str_radix(&value[0..2], 16);
                let g = u8::from_str_radix(&value[2..4], 16);
                let b = u8::from_str_radix(&value[4..6], 16);
                let w = u8::from_str_radix(&value[6..8], 16);
                let brightness = u8::from_str_radix(&value[8..10], 16);

                if let (Ok(r), Ok(g), Ok(b), Ok(w), Ok(brightness)) = (r, g, b, w, brightness) {
                    Ok((input, Literal::Rgbw(r, g, b, w, brightness)))
                } else {
                    Err(NomErr::Error(ParserError::Base {
                        location: text,
                        kind: ErrorKind::Expected(Expectation::Value),
                        child: None,
                    }))
                }
            } else {
                Err(NomErr::Error(ParserError::Base {
                    location: text,
                    kind: ErrorKind::Expected(Expectation::Value),
                    child: None,
                }))
            }
        }
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
            Literal::String(_) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("string", "u8"),
                child: None,
            }),
            Literal::Rgb(_, _, _, _) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("rgb", "u8"),
                child: None,
            }),
            Literal::Rgbw(_, _, _, _, _) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("rgbw", "u8"),
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
            Literal::String(_) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("string", "u16"),
                child: None,
            }),
            Literal::Rgb(_, _, _, _) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("rgb", "u16"),
                child: None,
            }),
            Literal::Rgbw(_, _, _, _, _) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("rgbw", "u16"),
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
            Literal::String(_) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("string", "u32"),
                child: None,
            }),
            Literal::Rgb(_, _, _, _) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("rgb", "u32"),
                child: None,
            }),
            Literal::Rgbw(_, _, _, _, _) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("rgbw", "u32"),
                child: None,
            }),
        }
    }
}

impl TryFrom<Literal> for bool {
    type Error = ParserError<&'static str>;

    fn try_from(literal: Literal) -> Result<Self, Self::Error> {
        match literal {
            Literal::U8(_) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("u8", "bool"),
                child: None,
            }),
            Literal::U16(_) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("u16", "bool"),
                child: None,
            }),
            Literal::U32(_) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("u32", "bool"),
                child: None,
            }),
            Literal::Bool(value) => Ok(value),
            Literal::String(_) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("string", "bool"),
                child: None,
            }),
            Literal::Rgb(_, _, _, _) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("rgb", "bool"),
                child: None,
            }),
            Literal::Rgbw(_, _, _, _, _) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("rgbw", "bool"),
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
            Literal::Rgb(r, g, b, brightness) => Ok(Value::Rgb(r, g, b, brightness)),
            Literal::Rgbw(r, g, b, w, brightness) => Ok(Value::Rgbw(r, g, b, w, brightness)),
            Literal::String(_) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("string", "value"),
                child: None,
            }),
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
            Literal::String(_) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("string", "message value"),
                child: None,
            }),
            Literal::Rgb(_, _, _, _) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("rgb", "message value"),
                child: None,
            }),
            Literal::Rgbw(_, _, _, _, _) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("rgbw", "message value"),
                child: None,
            }),
        }
    }
}

impl TryFrom<Literal> for BcmValue {
    type Error = ParserError<&'static str>;

    fn try_from(literal: Literal) -> Result<Self, Self::Error> {
        match literal {
            Literal::U8(value) => Ok(BcmValue::Single(value)),
            Literal::Rgb(r, g, b, brightness) => Ok(BcmValue::Rgb(r, g, b, brightness)),
            Literal::Rgbw(r, g, b, w, brightness) => Ok(BcmValue::Rgbw(r, g, b, w, brightness)),
            Literal::U16(_) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("u16", "bcm value"),
                child: None,
            }),
            Literal::U32(_) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("u32", "bcm value"),
                child: None,
            }),
            Literal::Bool(_) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("bool", "bcm value"),
                child: None,
            }),
            Literal::String(_) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("string", "bcm value"),
                child: None,
            }),
        }
    }
}

impl TryFrom<Literal> for RelayValue {
    type Error = ParserError<&'static str>;

    fn try_from(literal: Literal) -> Result<Self, Self::Error> {
        match literal {
            Literal::U8(_) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("u8", "relay value"),
                child: None,
            }),
            Literal::U16(_) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("u16", "relay value"),
                child: None,
            }),
            Literal::U32(_) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("u32", "relay value"),
                child: None,
            }),
            Literal::Rgb(_, _, _, _) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("rgb", "relay value"),
                child: None,
            }),
            Literal::Rgbw(_, _, _, _, _) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("rgbw", "relay value"),
                child: None,
            }),
            Literal::Bool(value) => Ok(RelayValue::Single(value)),
            Literal::String(value) => match value.as_str() {
                "first" => Ok(RelayValue::DoubleExclusive(
                    RelayDoubleExclusiveValue::FirstChannelOn,
                )),
                "second" => Ok(RelayValue::DoubleExclusive(
                    RelayDoubleExclusiveValue::SecondChannelOn,
                )),
                "none" => Ok(RelayValue::DoubleExclusive(
                    RelayDoubleExclusiveValue::NoChannelOn,
                )),
                _ => Err(ParserError::Base {
                    location: "",
                    kind: ErrorKind::Expected(Expectation::Value),
                    child: None,
                }),
            },
        }
    }
}

impl TryFrom<Literal> for CronExpression {
    type Error = ParserError<&'static str>;

    fn try_from(literal: Literal) -> Result<Self, Self::Error> {
        match literal {
            Literal::U8(_) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("u8", "cron expression"),
                child: None,
            }),
            Literal::U16(_) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("u16", "cron expression"),
                child: None,
            }),
            Literal::U32(_) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("u32", "cron expression"),
                child: None,
            }),
            Literal::Bool(_) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("bool", "cron expression"),
                child: None,
            }),
            Literal::Rgb(_, _, _, _) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("rgb", "cron expression"),
                child: None,
            }),
            Literal::Rgbw(_, _, _, _, _) => Err(ParserError::Base {
                location: "",
                kind: ErrorKind::CastFromToNotAllowed("rgbw", "cron expression"),
                child: None,
            }),
            Literal::String(string) => {
                let mut string_split = string.split_whitespace();

                let second = if let Some(field_string) = string_split.next() {
                    if let Ok(field_set) = parse_field(field_string, 0, 59) {
                        CronField::<u8>::Including(
                            field_set.iter().map(|value| *value as u8).collect(),
                        )
                    } else {
                        return Err(ParserError::Base {
                            location: "",
                            kind: ErrorKind::Expected(Expectation::Value),
                            child: None,
                        });
                    }
                } else {
                    return Err(ParserError::Base {
                        location: "",
                        kind: ErrorKind::Expected(Expectation::Value),
                        child: None,
                    });
                };

                let minute = if let Some(field_string) = string_split.next() {
                    if let Ok(field_set) = parse_field(field_string, 0, 59) {
                        CronField::<u8>::Including(
                            field_set.iter().map(|value| *value as u8).collect(),
                        )
                    } else {
                        return Err(ParserError::Base {
                            location: "",
                            kind: ErrorKind::Expected(Expectation::Value),
                            child: None,
                        });
                    }
                } else {
                    return Err(ParserError::Base {
                        location: "",
                        kind: ErrorKind::Expected(Expectation::Value),
                        child: None,
                    });
                };

                let hour = if let Some(field_string) = string_split.next() {
                    if let Ok(field_set) = parse_field(field_string, 0, 23) {
                        CronField::<u8>::Including(
                            field_set.iter().map(|value| *value as u8).collect(),
                        )
                    } else {
                        return Err(ParserError::Base {
                            location: "",
                            kind: ErrorKind::Expected(Expectation::Value),
                            child: None,
                        });
                    }
                } else {
                    return Err(ParserError::Base {
                        location: "",
                        kind: ErrorKind::Expected(Expectation::Value),
                        child: None,
                    });
                };

                let day_month = if let Some(field_string) = string_split.next() {
                    if let Ok(field_set) = parse_field(field_string, 1, 31) {
                        CronField::<u8>::Including(
                            field_set.iter().map(|value| *value as u8).collect(),
                        )
                    } else {
                        return Err(ParserError::Base {
                            location: "",
                            kind: ErrorKind::Expected(Expectation::Value),
                            child: None,
                        });
                    }
                } else {
                    return Err(ParserError::Base {
                        location: "",
                        kind: ErrorKind::Expected(Expectation::Value),
                        child: None,
                    });
                };

                let month = if let Some(field_string) = string_split.next() {
                    if let Ok(field_set) = parse_field(field_string, 1, 12) {
                        CronField::<u8>::Including(
                            field_set.iter().map(|value| *value as u8).collect(),
                        )
                    } else {
                        return Err(ParserError::Base {
                            location: "",
                            kind: ErrorKind::Expected(Expectation::Value),
                            child: None,
                        });
                    }
                } else {
                    return Err(ParserError::Base {
                        location: "",
                        kind: ErrorKind::Expected(Expectation::Value),
                        child: None,
                    });
                };

                let day_week = if let Some(field_string) = string_split.next() {
                    if let Ok(field_set) = parse_field(field_string, 1, 7) {
                        CronField::<u8>::Including(
                            field_set.iter().map(|value| *value as u8).collect(),
                        )
                    } else {
                        return Err(ParserError::Base {
                            location: "",
                            kind: ErrorKind::Expected(Expectation::Value),
                            child: None,
                        });
                    }
                } else {
                    return Err(ParserError::Base {
                        location: "",
                        kind: ErrorKind::Expected(Expectation::Value),
                        child: None,
                    });
                };

                // TODO: Requires custom parser to not produce big sets of year numbers
                let year = CronField::Any;

                Ok(CronExpression {
                    second,
                    minute,
                    hour,
                    day_month,
                    month,
                    day_week,
                    year,
                })
            }
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
    fn string_test() {
        assert_matches!(literal("\"This is a string\";input"), Ok((";input", Literal::String(string))) => {
            assert_eq!(string, "This is a string");
        });
    }

    #[test]
    fn rgb_test() {
        assert_matches!(literal("#01234567;input"), Ok((";input", Literal::Rgb(r, g, b, brightness))) => {
            assert_eq!(r, 0x01);
            assert_eq!(g, 0x23);
            assert_eq!(b, 0x45);
            assert_eq!(brightness, 0x67);
        });
    }

    #[test]
    fn rgbw_test() {
        assert_matches!(literal("#0123456789;input"), Ok((";input", Literal::Rgbw(r, g, b, w, brightness))) => {
            assert_eq!(r, 0x01);
            assert_eq!(g, 0x23);
            assert_eq!(b, 0x45);
            assert_eq!(w, 0x67);
            assert_eq!(brightness, 0x89);
        });
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
