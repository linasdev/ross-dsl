use nom::error::{ErrorKind, ParseError};

#[derive(Debug, PartialEq)]
pub enum ParserError {
    ExpectedKeywordFound(String, String),
    Nom(String, ErrorKind),
}

impl ParseError<&str> for ParserError {
    fn from_error_kind(input: &str, kind: ErrorKind) -> Self {
        ParserError::Nom(input.to_string(), kind)
    }

    fn append(_: &str, _: ErrorKind, other: Self) -> Self {
        other
    }
}
