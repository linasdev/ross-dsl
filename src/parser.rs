use nom::error::{ErrorKind, FromExternalError, ParseError};
use nom::multi::separated_list0;
use nom::sequence::{delimited, terminated};
use nom::InputTakeAtPosition;
use nom::{AsChar, Err, IResult};

use crate::literal::{literal, Literal};
use crate::symbol::{close_parenthesis, comma, open_parenthesis};

#[derive(Debug, PartialEq)]
pub enum ParserError {
    ExpectedKeywordFound(String, String, String),
    ExpectedSymbolFound(String, String, String),
    ExpectedValueFound(String, String),
    ExpectedTypeFound(String, String),
    ExpectedNameFound(String, String),

    ExpectedAlphaFound(String, String),
    ExpectedAlphanumericFound(String, String),
    ExpectedNumberFound(String, String),

    ExpectedArgumentsButGot(String, usize, usize),

    UnknownExtractor(String, String),
    UnknownFilter(String, String),
    UnknownProducer(String, String),

    Nom(String, ErrorKind),

    CastFromToNotAllowed(String, String),
}

impl ParseError<&str> for ParserError {
    fn from_error_kind(input: &str, kind: ErrorKind) -> Self {
        ParserError::Nom(input.to_string(), kind)
    }

    fn append(_: &str, _: ErrorKind, other: Self) -> Self {
        other
    }
}

impl FromExternalError<&str, ParserError> for ParserError {
    fn from_external_error(_input: &str, _kind: ErrorKind, e: ParserError) -> Self {
        e
    }
}

impl From<(&str, ErrorKind)> for ParserError {
    fn from(err: (&str, ErrorKind)) -> ParserError {
        ParserError::Nom(err.0.to_string(), err.1)
    }
}

pub fn alpha1(text: &str) -> IResult<&str, &str, ParserError> {
    match text
        .split_at_position1_complete::<_, ParserError>(|item| !item.is_alpha(), ErrorKind::Alpha)
    {
        Ok((input, value)) => Ok((input, value)),
        Err(Err::Error(ParserError::Nom(input, kind))) if matches!(kind, ErrorKind::Alpha) => {
            Err(Err::Error(ParserError::ExpectedAlphaFound(
                text.to_string(),
                input.to_string(),
            )))
        }
        Err(err) => Err(Err::convert(err)),
    }
}

pub fn alphanumeric1(text: &str) -> IResult<&str, &str, ParserError> {
    match text.split_at_position1_complete::<_, ParserError>(
        |item| !item.is_alphanumeric(),
        ErrorKind::AlphaNumeric,
    ) {
        Ok((input, value)) => Ok((input, value)),
        Err(Err::Error(ParserError::Nom(input, kind)))
            if matches!(kind, ErrorKind::AlphaNumeric) =>
        {
            Err(Err::Error(ParserError::ExpectedAlphanumericFound(
                text.to_string(),
                input.to_string(),
            )))
        }
        Err(err) => Err(Err::convert(err)),
    }
}

pub fn hex1(text: &str) -> IResult<&str, &str, ParserError> {
    match text.split_at_position1_complete::<_, ParserError>(
        |item| !item.is_digit(16) && item != 'x',
        ErrorKind::HexDigit,
    ) {
        Ok((input, value)) => Ok((input, value)),
        Err(Err::Error(ParserError::Nom(input, kind))) if matches!(kind, ErrorKind::HexDigit) => {
            Err(Err::Error(ParserError::ExpectedNumberFound(
                text.to_string(),
                input.to_string(),
            )))
        }
        Err(err) => Err(Err::convert(err)),
    }
}

pub fn dec1(text: &str) -> IResult<&str, &str, ParserError> {
    match text.split_at_position1_complete::<_, ParserError>(
        |item| !item.is_digit(10) && item != '-',
        ErrorKind::Digit,
    ) {
        Ok((input, value)) => Ok((input, value)),
        Err(Err::Error(ParserError::Nom(input, kind))) if matches!(kind, ErrorKind::Digit) => {
            Err(Err::Error(ParserError::ExpectedNumberFound(
                text.to_string(),
                input.to_string(),
            )))
        }
        Err(err) => Err(Err::convert(err)),
    }
}

pub fn argument0(text: &str) -> IResult<&str, Vec<Literal>, ParserError> {
    delimited(
        terminated(open_parenthesis, multispace0),
        separated_list0(comma, delimited(multispace0, literal, multispace0)),
        close_parenthesis,
    )(text)
}

pub fn multispace0(text: &str) -> IResult<&str, &str, ParserError> {
    nom::character::complete::multispace0(text)
}

pub fn multispace1(text: &str) -> IResult<&str, &str, ParserError> {
    match nom::character::complete::multispace1(text) {
        Ok((input, value)) => Ok((input, value)),
        Err(Err::Error(ParserError::Nom(input, kind))) if matches!(kind, ErrorKind::MultiSpace) => {
            Err(Err::Error(ParserError::ExpectedSymbolFound(
                input.clone(),
                " ".to_string(),
                input,
            )))
        }
        Err(err) => Err(Err::convert(err)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alpha1_test() {
        assert_eq!(alpha1("while123123"), Ok(("123123", "while")));
    }

    #[test]
    fn alpha1_non_alpha_test() {
        assert_eq!(
            alpha1("123123"),
            Err(Err::Error(ParserError::ExpectedAlphaFound(
                "123123".to_string(),
                "123123".to_string()
            )))
        );
    }

    #[test]
    fn alpha1_empty_test() {
        assert_eq!(
            alpha1(""),
            Err(Err::Error(ParserError::ExpectedAlphaFound(
                "".to_string(),
                "".to_string()
            )))
        );
    }

    #[test]
    fn alphanumeric1_test() {
        assert_eq!(
            alphanumeric1("while123123;input"),
            Ok((";input", "while123123"))
        );
    }

    #[test]
    fn alphanumeric1_non_alpha_test() {
        assert_eq!(
            alphanumeric1(";123123"),
            Err(Err::Error(ParserError::ExpectedAlphanumericFound(
                ";123123".to_string(),
                ";123123".to_string()
            )))
        );
    }

    #[test]
    fn alphanumeric1_empty_test() {
        assert_eq!(
            alphanumeric1(""),
            Err(Err::Error(ParserError::ExpectedAlphanumericFound(
                "".to_string(),
                "".to_string()
            )))
        );
    }

    #[test]
    fn hex1_test() {
        assert_eq!(hex1("0x01ab"), Ok(("", "0x01ab")));
    }

    #[test]
    fn hex1_non_hex_test() {
        assert_eq!(
            hex1("ghjklp"),
            Err(Err::Error(ParserError::ExpectedNumberFound(
                "ghjklp".to_string(),
                "ghjklp".to_string()
            )))
        );
    }

    #[test]
    fn hex1_empty_test() {
        assert_eq!(
            hex1(""),
            Err(Err::Error(ParserError::ExpectedNumberFound(
                "".to_string(),
                "".to_string()
            )))
        );
    }

    #[test]
    fn dec1_test() {
        assert_eq!(dec1("1234"), Ok(("", "1234")));
    }

    #[test]
    fn dec1_negative_test() {
        assert_eq!(dec1("-1234"), Ok(("", "-1234")));
    }

    #[test]
    fn dec1_non_dec_test() {
        assert_eq!(
            dec1("abcabc"),
            Err(Err::Error(ParserError::ExpectedNumberFound(
                "abcabc".to_string(),
                "abcabc".to_string()
            )))
        );
    }

    #[test]
    fn dec1_empty_test() {
        assert_eq!(
            dec1(""),
            Err(Err::Error(ParserError::ExpectedNumberFound(
                "".to_string(),
                "".to_string()
            )))
        );
    }

    #[test]
    fn argument0_two_arguments_test() {
        assert_eq!(
            argument0("(0xab~u16  ,  false )"),
            Ok(("", vec![Literal::U16(0x00ab), Literal::Bool(false)]))
        );
    }

    #[test]
    fn argument0_one_argument_test() {
        assert_eq!(
            argument0("(  0xab~u16)"),
            Ok(("", vec![Literal::U16(0x00ab)]))
        );
    }

    #[test]
    fn argument0_no_arguments_test() {
        assert_eq!(argument0("(   )"), Ok(("", vec![])));
    }

    #[test]
    fn argument0_empty_test() {
        assert_eq!(
            argument0(""),
            Err(Err::Error(ParserError::ExpectedSymbolFound(
                "".to_string(),
                "(".to_string(),
                "".to_string(),
            )))
        );
    }

    #[test]
    fn multispace_test() {
        assert_eq!(multispace0(" asd"), Ok(("asd", " ")),);
    }

    #[test]
    fn multispace1_two_spaces_test() {
        assert_eq!(multispace1("  asd"), Ok(("asd", "  ")),);
    }

    #[test]
    fn multispace1_zero_spaces_test() {
        assert_eq!(
            multispace1("asd"),
            Err(Err::Error(ParserError::ExpectedSymbolFound(
                "asd".to_string(),
                " ".to_string(),
                "asd".to_string()
            )))
        );
    }

    #[test]
    fn multispace1_empty_test() {
        assert_eq!(
            multispace1(""),
            Err(Err::Error(ParserError::ExpectedSymbolFound(
                "".to_string(),
                " ".to_string(),
                "".to_string()
            )))
        );
    }
}
