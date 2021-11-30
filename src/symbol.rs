use crate::error::ParserError;
use nom::character::complete::char;
use nom::IResult;

macro_rules! implement_symbol_parser {
    ($symbol_name:ident, $symbol:expr) => {
        pub fn $symbol_name(text: &str) -> IResult<&str, &str, ParserError<&str>> {
            let (input, _) = char($symbol)(text)?;

            Ok((input, stringify!($symbol)))
        }

        #[cfg(test)]
        mod $symbol_name {
            use super::*;

            use cool_asserts::assert_matches;
            use nom::Err as NomErr;

            use crate::error::{ErrorKind, Expectation};

            #[test]
            fn test() {
                assert_matches!(
                    $symbol_name(concat!($symbol, ";input")),
                    Ok((";input", stringify!($symbol)))
                );
            }

            #[test]
            fn unexpected_token_test() {
                assert_matches!(
                    $symbol_name("asdasd"),
                    Err(NomErr::Error(ParserError::Base {
                        location,
                        kind,
                        child,
                    })) => {
                        assert_matches!(location, "asdasd");
                        assert_matches!(kind, ErrorKind::Expected(Expectation::Symbol($symbol)));
                        assert_matches!(child, None);
                    }
                );
            }

            #[test]
            fn empty_test() {
                assert_matches!(
                    $symbol_name(""),
                    Err(NomErr::Error(ParserError::Base {
                        location,
                        kind,
                        child,
                    })) => {
                        assert_matches!(location, "");
                        assert_matches!(kind, ErrorKind::Expected(Expectation::Symbol($symbol)));
                        assert_matches!(child, None);
                    }
                );
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
