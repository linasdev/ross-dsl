use nom::character::complete::char;
use nom::error::ErrorKind;
use nom::{Err, IResult};
use std::str::from_utf8_unchecked;

use crate::parser::ParserError;

macro_rules! implement_symbol_parser {
    ($symbol_name:ident, $symbol:expr) => {
        pub fn $symbol_name(text: &str) -> IResult<&str, &str, ParserError> {
            match char($symbol)(text) {
                Ok((input, _)) => unsafe { Ok((input, from_utf8_unchecked(&[$symbol as u8]))) },
                Err(Err::Error(ParserError::Nom(input, kind)))
                    if matches!(kind, ErrorKind::Char) =>
                {
                    Err(Err::Error(ParserError::ExpectedSymbolFound(
                        text.to_string(),
                        $symbol.to_string(),
                        input.to_string(),
                    )))
                }
                Err(err) => Err(Err::convert(err)),
            }
        }

        #[cfg(test)]
        mod $symbol_name {
            use super::*;

            #[test]
            fn $symbol_name() {
                assert_eq!(super::$symbol_name(concat!($symbol, ";input")), unsafe {
                    Ok((";input", from_utf8_unchecked(&[$symbol as u8])))
                },);
            }

            mod unexpected_token {
                use super::*;

                #[test]
                fn $symbol_name() {
                    assert_eq!(
                        super::super::$symbol_name("while123123"),
                        Err(Err::Error(ParserError::ExpectedSymbolFound(
                            "while123123".to_string(),
                            $symbol.to_string(),
                            "while123123".to_string()
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
                        Err(Err::Error(ParserError::ExpectedSymbolFound(
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
implement_symbol_parser!(space, ' ');
