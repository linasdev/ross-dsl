use nom::character::complete::alpha0;
use nom::{Err, IResult};

use crate::parser::ParserError;
use crate::token::KeywordToken;

pub fn parse_keyword(input: &str) -> IResult<&str, KeywordToken, ParserError> {
    match alpha0(input)? {
        (input, "let") => Ok((input, KeywordToken::Let)),
        (input, "const") => Ok((input, KeywordToken::Const)),
        (input, "send") => Ok((input, KeywordToken::Send)),
        (input, "from") => Ok((input, KeywordToken::From)),
        (input, "to") => Ok((input, KeywordToken::To)),
        (input, "if") => Ok((input, KeywordToken::If)),
        (input, "do") => Ok((input, KeywordToken::Do)),
        (input, "match") => Ok((input, KeywordToken::Match)),
        (input, "event") => Ok((input, KeywordToken::Event)),
        (input, "producer") => Ok((input, KeywordToken::Producer)),
        (input, "tick") => Ok((input, KeywordToken::Tick)),
        (input, "fire") => Ok((input, KeywordToken::Fire)),
        (input, token) => Err(Err::Failure(ParserError::ExpectedKeywordFound(
            token.to_string() + input,
            if token.is_empty() {
                input.to_string()
            } else {
                token.to_string()
            },
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn let_keyword_test() {
        assert_eq!(parse_keyword("let"), Ok(("", KeywordToken::Let)));
    }

    #[test]
    fn const_keyword_test() {
        assert_eq!(parse_keyword("const"), Ok(("", KeywordToken::Const)));
    }

    #[test]
    fn send_keyword_test() {
        assert_eq!(parse_keyword("send"), Ok(("", KeywordToken::Send)));
    }

    #[test]
    fn from_keyword_test() {
        assert_eq!(parse_keyword("from"), Ok(("", KeywordToken::From)));
    }

    #[test]
    fn to_keyword_test() {
        assert_eq!(parse_keyword("to"), Ok(("", KeywordToken::To)));
    }

    #[test]
    fn if_test() {
        assert_eq!(parse_keyword("if"), Ok(("", KeywordToken::If)));
    }

    #[test]
    fn do_test() {
        assert_eq!(parse_keyword("do"), Ok(("", KeywordToken::Do)));
    }

    #[test]
    fn match_test() {
        assert_eq!(parse_keyword("match"), Ok(("", KeywordToken::Match)));
    }

    #[test]
    fn event_test() {
        assert_eq!(parse_keyword("event"), Ok(("", KeywordToken::Event)));
    }

    #[test]
    fn producer_test() {
        assert_eq!(parse_keyword("producer"), Ok(("", KeywordToken::Producer)));
    }

    #[test]
    fn tick_test() {
        assert_eq!(parse_keyword("tick"), Ok(("", KeywordToken::Tick)));
    }

    #[test]
    fn fire_test() {
        assert_eq!(parse_keyword("fire"), Ok(("", KeywordToken::Fire)));
    }

    #[test]
    fn unexpected_token_test() {
        assert_eq!(
            parse_keyword("while123123"),
            Err(Err::Failure(ParserError::ExpectedKeywordFound(
                "while123123".to_string(),
                "while".to_string()
            )))
        );
    }

    #[test]
    fn non_alpha_test() {
        assert_eq!(
            parse_keyword("123123"),
            Err(Err::Failure(ParserError::ExpectedKeywordFound(
                "123123".to_string(),
                "123123".to_string()
            )))
        );
    }
}
