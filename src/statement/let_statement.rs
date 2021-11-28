use nom::multi::{many0, many1};
use nom::sequence::{terminated, preceded, separated_pair, delimited};
use nom::{Err, IResult};

use crate::keyword::let_keyword;
use crate::literal::{literal, Literal};
use crate::parser::{alpha1, ParserError};
use crate::symbol::{semicolon, equal_sign, space};

pub fn parse_let_statement(text: &str) -> IResult<&str, (String, Literal), ParserError> {
    let name_parser = delimited(many1(space), alpha1, many0(space));
    let equal_sign_parser = terminated(equal_sign, many0(space));
    let name_value_pair_parser = separated_pair(name_parser, equal_sign_parser, literal);
    let keyword_parser = preceded(let_keyword, name_value_pair_parser);
    let mut semicolon_parser = terminated(keyword_parser, semicolon);

    match semicolon_parser(text) {
        Ok((input, (name, value))) => {
            Ok((input, (name.to_string(), value)))
        },
        Err(Err::Error(ParserError::ExpectedAlphaFound(input, value))) => Err(
            Err::Error(ParserError::ExpectedNameFound(input, value))
        ),
        Err(err) => Err(Err::convert(err)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_u32_test() {
        assert_eq!(
            parse_let_statement("let state = 0xabababab~u32;input"),
            Ok(("input", ("state".to_string(), Literal::U32(0xabab_abab))))
        );
    }

    #[test]
    fn bool_test() {
        assert_eq!(
            parse_let_statement("let state = false;input"),
            Ok(("input", ("state".to_string(), Literal::Bool(false))))
        );
    }

    #[test]
    fn weird_spacing1_test() {
        assert_eq!(
            parse_let_statement("let state=0xabababab~u32;input"),
            Ok(("input", ("state".to_string(), Literal::U32(0xabab_abab))))
        );
    }

    #[test]
    fn weird_spacing2_test() {
        assert_eq!(
            parse_let_statement("let state  =  0xabababab~u32;input"),
            Ok(("input", ("state".to_string(), Literal::U32(0xabab_abab))))
        );
    }

    #[test]
    fn only_keyword_test() {
        assert_eq!(
            parse_let_statement("let;input"),
            Err(Err::Error(ParserError::ExpectedSymbolFound(
                ";input".to_string(),
                " ".to_string(),
                ";input".to_string(),
            )))
        );
    }

    #[test]
    fn invalid_name_test() {
        assert_eq!(
            parse_let_statement("let 1state = 0xabababab~u32;input"),
            Err(Err::Error(ParserError::ExpectedNameFound(
                "1state = 0xabababab~u32;input".to_string(),
                "1state = 0xabababab~u32;input".to_string(),
            )))
        );
    }

    #[test]
    fn wrong_keyword_test() {
        assert_eq!(
            parse_let_statement("const state = 0xabababab~u32;input"),
            Err(Err::Error(ParserError::ExpectedKeywordFound(
                "const state = 0xabababab~u32;input".to_string(),
                "let".to_string(),
                "const".to_string(),
            )))
        );
    }
}
