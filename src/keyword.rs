use nom::character::complete::alpha1;
use nom::{Err, IResult,};

use crate::parser::ParserError;
use crate::token::{KeywordToken, Token};

pub fn parse_keyword(text: &str) -> IResult<&str, Token, ParserError> {
    match alpha1(text)? {
        (input, "let") => Ok((input, KeywordToken::Let.into())),
        (input, "const") => Ok((input, KeywordToken::Const.into())),
        (input, "send") => Ok((input, KeywordToken::Send.into())),
        (input, "from") => Ok((input, KeywordToken::From.into())),
        (input, "to") => Ok((input, KeywordToken::To.into())),
        (input, "if") => Ok((input, KeywordToken::If.into())),
        (input, "do") => Ok((input, KeywordToken::Do.into())),
        (input, "match") => Ok((input, KeywordToken::Match.into())),
        (input, "event") => Ok((input, KeywordToken::Event.into())),
        (input, "producer") => Ok((input, KeywordToken::Producer.into())),
        (input, "tick") => Ok((input, KeywordToken::Tick.into())),
        (input, "fire") => Ok((input, KeywordToken::Fire.into())),
        (input, token) => Err(Err::Failure(ParserError::UnexpectedToken(
            input.to_string(),
            token.to_string(),
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn let_keyword_test() {
        assert_eq!(
            parse_keyword("let"),
            Ok(("", Token::Keyword(KeywordToken::Let)))
        );
    }

    #[test]
    fn const_keyword_test() {
        assert_eq!(
            parse_keyword("const"),
            Ok(("", Token::Keyword(KeywordToken::Const)))
        );
    }

    #[test]
    fn send_keyword_test() {
        assert_eq!(
            parse_keyword("send"),
            Ok(("", Token::Keyword(KeywordToken::Send)))
        );
    }

    #[test]
    fn from_keyword_test() {
        assert_eq!(
            parse_keyword("from"),
            Ok(("", Token::Keyword(KeywordToken::From)))
        );
    }

    #[test]
    fn to_keyword_test() {
        assert_eq!(
            parse_keyword("to"),
            Ok(("", Token::Keyword(KeywordToken::To)))
        );
    }

    #[test]
    fn if_test() {
        assert_eq!(
            parse_keyword("if"),
            Ok(("", Token::Keyword(KeywordToken::If)))
        );
    }

    #[test]
    fn do_test() {
        assert_eq!(
            parse_keyword("do"),
            Ok(("", Token::Keyword(KeywordToken::Do)))
        );
    }

    #[test]
    fn match_test() {
        assert_eq!(
            parse_keyword("match"),
            Ok(("", Token::Keyword(KeywordToken::Match)))
        );
    }

    #[test]
    fn event_test() {
        assert_eq!(
            parse_keyword("event"),
            Ok(("", Token::Keyword(KeywordToken::Event)))
        );
    }

    #[test]
    fn producer_test() {
        assert_eq!(
            parse_keyword("producer"),
            Ok(("", Token::Keyword(KeywordToken::Producer)))
        );
    }

    #[test]
    fn tick_test() {
        assert_eq!(
            parse_keyword("tick"),
            Ok(("", Token::Keyword(KeywordToken::Tick)))
        );
    }

    #[test]
    fn fire_test() {
        assert_eq!(
            parse_keyword("fire"),
            Ok(("", Token::Keyword(KeywordToken::Fire)))
        );
    }

    #[test]
    fn unexpected_token_test() {
        assert_eq!(
            parse_keyword("while123123"),
            Err(Err::Failure(ParserError::UnexpectedToken(
                "123123".to_string(),
                "while".to_string()
            )))
        );
    }
}
