use nom::character::complete::alpha0;
use nom::{Err, IResult};

use crate::parser::ParserError;

macro_rules! implement_keyword_parser {
    ($parser_name:ident, $keyword:expr) => {
        pub fn $parser_name(text: &str) -> IResult<&str, &str, ParserError> {
            match alpha0(text)? {
                (input, $keyword) => Ok((input, $keyword)),
                (_, value) => Err(Err::Error(ParserError::ExpectedKeywordFound(
                    text.to_string(),
                    $keyword.to_string(),
                    value.to_string(),
                ))),
            }
        }

        #[cfg(test)]
        mod $parser_name {
            use super::*;

            #[test]
            fn $parser_name() {
                assert_eq!(
                    super::$parser_name(concat!($keyword, ";input")),
                    Ok((";input", $keyword))
                );
            }

            mod unexpected_token {
                use super::*;

                #[test]
                fn $parser_name() {
                    assert_eq!(
                        super::super::$parser_name("while123123"),
                        Err(Err::Error(ParserError::ExpectedKeywordFound(
                            "while123123".to_string(),
                            $keyword.to_string(),
                            "while".to_string()
                        )))
                    );
                }
            }

            mod non_alpha {
                use super::*;

                #[test]
                fn $parser_name() {
                    assert_eq!(
                        super::super::$parser_name("123123"),
                        Err(Err::Error(ParserError::ExpectedKeywordFound(
                            "123123".to_string(),
                            $keyword.to_string(),
                            "".to_string()
                        )))
                    );
                }
            }

            mod empty {
                use super::*;

                #[test]
                fn $parser_name() {
                    assert_eq!(
                        super::super::$parser_name(""),
                        Err(Err::Error(ParserError::ExpectedKeywordFound(
                            "".to_string(),
                            $keyword.to_string(),
                            "".to_string()
                        )))
                    );
                }
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
implement_keyword_parser!(true_keyword, "true");
implement_keyword_parser!(false_keyword, "false");
