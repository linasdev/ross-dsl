use nom::character::complete::alpha1;
use nom::{Err as NomErr, IResult};

use crate::error::{ErrorKind, Expectation, ParserError};

macro_rules! implement_keyword_parser {
    ($parser_name:ident, $keyword:expr) => {
        pub fn $parser_name(text: &str) -> IResult<&str, &str, ParserError<&str>> {
            match alpha1(text) {
                Ok((input, $keyword)) => Ok((input, $keyword)),
                Ok((_, value)) => Err(NomErr::Error(ParserError::Base {
                    location: value,
                    kind: ErrorKind::Expected(Expectation::Keyword($keyword)),
                    child: None,
                })),
                Err(NomErr::Error(err)) => Err(NomErr::Error(ParserError::Base {
                    location: text,
                    kind: ErrorKind::Expected(Expectation::Keyword($keyword)),
                    child: Some(Box::new(err)),
                })),
                err => err,
            }
        }

        #[cfg(test)]
        mod $parser_name {
            use super::*;

            use cool_asserts::assert_matches;

            #[test]
            fn test() {
                assert_matches!(
                    $parser_name(concat!($keyword, ";input")),
                    Ok((";input", $keyword)),
                );
            }

            #[test]
            fn unexpected_token_test() {
                assert_matches!(
                    $parser_name("asdasd"),
                    Err(NomErr::Error(ParserError::Base {
                        location,
                        kind,
                        child,
                    })) => {
                        assert_matches!(location, "asdasd");
                        assert_matches!(kind, ErrorKind::Expected(Expectation::Keyword($keyword)));
                        assert_matches!(child, None);
                    }
                );
            }

            #[test]
            fn non_alpha_test() {
                assert_matches!(
                    $parser_name("123123"),
                    Err(NomErr::Error(ParserError::Base {
                        location,
                        kind,
                        child,
                    })) => {
                        assert_matches!(location, "123123");
                        assert_matches!(kind, ErrorKind::Expected(Expectation::Keyword($keyword)));
                        assert_matches!(child, Some(_));
                    }
                );
            }

            #[test]
            fn empty_test() {
                assert_matches!(
                    $parser_name(""),
                    Err(NomErr::Error(ParserError::Base {
                        location,
                        kind,
                        child,
                    })) => {
                        assert_matches!(location, "");
                        assert_matches!(kind, ErrorKind::Expected(Expectation::Keyword($keyword)));
                        assert_matches!(child, Some(_));
                    }
                );
            }
        }
    };
}

implement_keyword_parser!(let_keyword, "let");
implement_keyword_parser!(const_keyword, "const");
implement_keyword_parser!(send_keyword, "send");
implement_keyword_parser!(from_keyword, "from");
implement_keyword_parser!(to_keyword, "to");
implement_keyword_parser!(if_keyword, "if");
implement_keyword_parser!(do_keyword, "do");
implement_keyword_parser!(match_keyword, "match");
implement_keyword_parser!(event_keyword, "event");
implement_keyword_parser!(producer_keyword, "producer");
implement_keyword_parser!(tick_keyword, "tick");
implement_keyword_parser!(fire_keyword, "fire");
implement_keyword_parser!(set_keyword, "set");
implement_keyword_parser!(on_keyword, "on");
implement_keyword_parser!(true_keyword, "true");
implement_keyword_parser!(false_keyword, "false");
implement_keyword_parser!(not_keyword, "not");
implement_keyword_parser!(or_keyword, "or");
implement_keyword_parser!(and_keyword, "and");
implement_keyword_parser!(peripheral_keyword, "peripheral");
implement_keyword_parser!(bcm_keyword, "bcm");
implement_keyword_parser!(single_keyword, "single");
implement_keyword_parser!(rgb_keyword, "rgb");
implement_keyword_parser!(rgbw_keyword, "rgbw");
