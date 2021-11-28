use nom::character::complete::alpha0;
use nom::{Err, IResult};

use crate::parser::ParserError;

#[derive(Debug, PartialEq)]
pub enum Keyword {
    Let,
    Const,
    Send,
    From,
    To,
    If,
    Do,
    Match,
    Event,
    Producer,
    Tick,
    Fire,
}

pub fn parse_keyword(text: &str) -> IResult<&str, Keyword, ParserError> {
    match alpha0(text)? {
        (input, "let") => Ok((input, Keyword::Let)),
        (input, "const") => Ok((input, Keyword::Const)),
        (input, "send") => Ok((input, Keyword::Send)),
        (input, "from") => Ok((input, Keyword::From)),
        (input, "to") => Ok((input, Keyword::To)),
        (input, "if") => Ok((input, Keyword::If)),
        (input, "do") => Ok((input, Keyword::Do)),
        (input, "match") => Ok((input, Keyword::Match)),
        (input, "event") => Ok((input, Keyword::Event)),
        (input, "producer") => Ok((input, Keyword::Producer)),
        (input, "tick") => Ok((input, Keyword::Tick)),
        (input, "fire") => Ok((input, Keyword::Fire)),
        (_, token) => Err(Err::Failure(ParserError::ExpectedKeywordFound(
            text.to_string(),
            token.to_string(),
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn let_keyword_test() {
        assert_eq!(parse_keyword("let"), Ok(("", Keyword::Let)));
    }

    #[test]
    fn const_keyword_test() {
        assert_eq!(parse_keyword("const"), Ok(("", Keyword::Const)));
    }

    #[test]
    fn send_keyword_test() {
        assert_eq!(parse_keyword("send"), Ok(("", Keyword::Send)));
    }

    #[test]
    fn from_keyword_test() {
        assert_eq!(parse_keyword("from"), Ok(("", Keyword::From)));
    }

    #[test]
    fn to_keyword_test() {
        assert_eq!(parse_keyword("to"), Ok(("", Keyword::To)));
    }

    #[test]
    fn if_test() {
        assert_eq!(parse_keyword("if"), Ok(("", Keyword::If)));
    }

    #[test]
    fn do_test() {
        assert_eq!(parse_keyword("do"), Ok(("", Keyword::Do)));
    }

    #[test]
    fn match_test() {
        assert_eq!(parse_keyword("match"), Ok(("", Keyword::Match)));
    }

    #[test]
    fn event_test() {
        assert_eq!(parse_keyword("event"), Ok(("", Keyword::Event)));
    }

    #[test]
    fn producer_test() {
        assert_eq!(parse_keyword("producer"), Ok(("", Keyword::Producer)));
    }

    #[test]
    fn tick_test() {
        assert_eq!(parse_keyword("tick"), Ok(("", Keyword::Tick)));
    }

    #[test]
    fn fire_test() {
        assert_eq!(parse_keyword("fire"), Ok(("", Keyword::Fire)));
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
                "".to_string()
            )))
        );
    }
}
