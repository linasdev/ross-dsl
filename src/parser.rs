use nom::error::{ErrorKind, ParseError};

#[derive(Debug, PartialEq)]
pub enum ParserError {
    ExpectedKeywordFound(String, String, String),
    ExpectedSymbolFound(String, String, String),
    ExpectedValueFound(String, String),
    ExpectedTypeFound(String, String),
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

impl From<(&str, ErrorKind)> for ParserError {
    fn from(err: (&str, ErrorKind)) -> ParserError {
        ParserError::Nom(err.0.to_string(), err.1)
    }
}
