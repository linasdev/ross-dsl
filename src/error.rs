use indent_write::fmt::IndentWriter;
use nom::error::{ErrorKind as NomErrorKind, FromExternalError, ParseError};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Write};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Expectation {
    ArgumentCount(usize, usize),
    Keyword(&'static str),
    Symbol(char),
    Name,
    Literal,
    Value,
    Type,
    Alpha,
    AlphaNumeric,
    Digit,
    HexDigit,
    MultiSpace,
}

impl Display for Expectation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            Expectation::ArgumentCount(expected, found) => {
                write!(f, "{} arguments found {}", expected, found)
            }
            Expectation::Keyword(keyword) => write!(f, "{:?}", keyword),
            Expectation::Symbol(symbol) => write!(f, "{:?}", symbol),
            Expectation::Name => write!(f, "a name"),
            Expectation::Literal => write!(f, "a literal"),
            Expectation::Value => write!(f, "a value"),
            Expectation::Type => write!(f, "a type"),
            Expectation::Alpha => write!(f, "an ascii letter"),
            Expectation::AlphaNumeric => write!(f, "an ascii alphanumeric character"),
            Expectation::Digit => write!(f, "an ascii digit"),
            Expectation::HexDigit => write!(f, "a hexadecimal digit"),
            Expectation::MultiSpace => write!(f, "a space, tab or newline"),
        }
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    Expected(Expectation),
    Nom(NomErrorKind),
    UnknownExtractor,
    UnknownFilter,
    UnknownProducer,
    CastFromToNotAllowed(&'static str, &'static str),
    External(Box<dyn Error + Send + Sync + 'static>),
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::Expected(expectation) => write!(f, "expected {}", expectation),
            ErrorKind::Nom(kind) => write!(f, "error in {:?}", kind),
            ErrorKind::UnknownExtractor => write!(f, "unknown extractor"),
            ErrorKind::UnknownFilter => write!(f, "unknown filter"),
            ErrorKind::UnknownProducer => write!(f, "unknown producer"),
            ErrorKind::CastFromToNotAllowed(from, to) => {
                write!(f, "cast from {} to {} not allowed", from, to)
            }
            ErrorKind::External(ref err) => {
                writeln!(f, "external error:")?;
                let mut f = IndentWriter::new("  ", f);
                write!(f, "{}", err)
            }
        }
    }
}

#[derive(Debug)]
pub enum ParserError<I> {
    Base {
        location: I,
        kind: ErrorKind,
        child: Option<Box<Self>>,
    },
    Alt(Vec<Self>),
}

impl<I: Display> Display for ParserError<I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserError::Base {
                location,
                kind,
                child,
            } => {
                write!(f, "{} at {}", kind, location)?;

                if let Some(child) = child {
                    if let ParserError::Alt(ref siblings) = **child {
                        writeln!(f, " caused by one of:")?;

                        let mut f = IndentWriter::new("  ", f);

                        for (i, sibling) in siblings.iter().enumerate() {
                            write!(f, "{}", sibling)?;

                            if i != siblings.len() - 1 {
                                writeln!(f)?;
                            }
                        }
                    } else {
                        writeln!(f, " caused by:")?;
                        let mut f = IndentWriter::new("  ", f);
                        write!(f, "{}", child)?;
                    }
                }

                Ok(())
            }
            ParserError::Alt(siblings) => {
                writeln!(f, "one of:")?;

                let mut f = IndentWriter::new("  ", f);

                for (i, sibling) in siblings.iter().enumerate() {
                    write!(f, "{}", sibling)?;

                    if i != siblings.len() - 1 {
                        writeln!(f)?;
                    }
                }

                Ok(())
            }
        }
    }
}

impl<I: Debug + Display> Error for ParserError<I> {}

impl<I> ParseError<I> for ParserError<I> {
    fn from_error_kind(location: I, kind: NomErrorKind) -> Self {
        let kind = match kind {
            NomErrorKind::Alpha => ErrorKind::Expected(Expectation::Alpha),
            NomErrorKind::AlphaNumeric => ErrorKind::Expected(Expectation::AlphaNumeric),
            NomErrorKind::Digit => ErrorKind::Expected(Expectation::Digit),
            NomErrorKind::HexDigit => ErrorKind::Expected(Expectation::HexDigit),
            NomErrorKind::MultiSpace => ErrorKind::Expected(Expectation::MultiSpace),
            kind => ErrorKind::Nom(kind),
        };

        ParserError::Base {
            location,
            kind,
            child: None,
        }
    }

    fn append(location: I, kind: NomErrorKind, other: Self) -> Self {
        ParserError::Base {
            location,
            kind: ErrorKind::Nom(kind),
            child: Some(Box::new(other)),
        }
    }

    fn from_char(location: I, character: char) -> Self {
        ParserError::Base {
            location: location,
            kind: ErrorKind::Expected(Expectation::Symbol(character)),
            child: None,
        }
    }

    fn or(self, other: Self) -> Self {
        let siblings = match (self, other) {
            (ParserError::Alt(mut siblings1), ParserError::Alt(mut siblings2)) => {
                siblings2.append(&mut siblings1);
                siblings2
            }
            (ParserError::Alt(mut siblings), err) | (err, ParserError::Alt(mut siblings)) => {
                siblings.push(err);
                siblings
            }
            (err1, err2) => vec![err1, err2],
        };

        ParserError::Alt(siblings)
    }
}

impl<I, E: Error + Send + Sync + 'static> FromExternalError<I, E> for ParserError<I> {
    fn from_external_error(location: I, _kind: NomErrorKind, e: E) -> Self {
        ParserError::Base {
            location,
            kind: ErrorKind::External(Box::new(e)),
            child: None,
        }
    }
}
