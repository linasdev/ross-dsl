use nom::character::complete::char;
use nom::combinator::cut;
use nom::{Err, IResult};

use crate::parser::ParserError;

macro_rules! implement_symbol_parser {
    ($symbol_name:ident, $symbol:expr) => {
        pub fn $symbol_name(text: &str) -> IResult<&str, char, ParserError> {
            match cut(char($symbol))(text) {
                Ok((input, _)) => Ok((input, $symbol)),
                Err(Err::Failure((value, _))) => Err(Err::Failure(ParserError::ExpectedSymbolFound(
                    text.to_string(),
                    $symbol.to_string(),
                    if let Some(value) = value.chars().nth(0) {
                        value.to_string()
                    } else {
                        "".to_string()
                    },
                ))),
                Err(err) => Err(Err::convert(err)),
            }
        }

        #[cfg(test)]
        mod $symbol_name {
            use super::*;

            #[test]
            fn $symbol_name() {
                assert_eq!(super::$symbol_name(concat!($symbol, ";input")), Ok((";input", $symbol)));
            }

            mod unexpected_token {
                use super::*;

                #[test]
                fn $symbol_name() {
                    assert_eq!(
                        super::super::$symbol_name("while123123"),
                        Err(Err::Failure(ParserError::ExpectedSymbolFound(
                            "while123123".to_string(),
                            $symbol.to_string(),
                            "w".to_string()
                        )))
                    );
                }
            }

            mod empty {
                use super::*;

                #[test]
                fn $symbol_name() {
                    assert_eq!(
                        super::super::$symbol_name(""),
                        Err(Err::Failure(ParserError::ExpectedSymbolFound(
                            "".to_string(),
                            $symbol.to_string(),
                            "".to_string()
                        )))
                    );
                }
            }
        }
    };
}

implement_symbol_parser!(semicolon, ';');
implement_symbol_parser!(tilde, '~');
implement_symbol_parser!(comma, ',');
implement_symbol_parser!(open_parenthesis, '(');
implement_symbol_parser!(close_parenthesis, ')');
implement_symbol_parser!(open_brace, '{');
implement_symbol_parser!(close_brace, '}');
implement_symbol_parser!(equal_sign, '=');
