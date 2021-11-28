use nom::sequence::{delimited, preceded, separated_pair, terminated};
use nom::{Err, IResult};

use crate::keyword::const_keyword;
use crate::literal::{literal, Literal};
use crate::parser::{alpha1, multispace0, multispace1, ParserError};
use crate::symbol::{equal_sign, semicolon};

pub fn parse_const_statement(text: &str) -> IResult<&str, (String, Literal), ParserError> {
    let name_parser = delimited(multispace1, alpha1, multispace0);
    let equal_sign_parser = terminated(equal_sign, multispace0);
    let name_value_pair_parser = separated_pair(name_parser, equal_sign_parser, literal);
    let keyword_parser = preceded(const_keyword, name_value_pair_parser);
    let mut semicolon_parser = terminated(keyword_parser, semicolon);

    match semicolon_parser(text) {
        Ok((input, (name, value))) => Ok((input, (name.to_string(), value))),
        Err(Err::Error(ParserError::ExpectedAlphaFound(input, value))) => {
            Err(Err::Error(ParserError::ExpectedNameFound(input, value)))
        }
        Err(err) => Err(Err::convert(err)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_u32_test() {
        assert_eq!(
            parse_const_statement("const state = 0xabababab~u32;input"),
            Ok(("input", ("state".to_string(), Literal::U32(0xabab_abab))))
        );
    }

    #[test]
    fn bool_test() {
        assert_eq!(
            parse_const_statement("const state = false;input"),
            Ok(("input", ("state".to_string(), Literal::Bool(false))))
        );
    }

    #[test]
    fn weird_spacing1_test() {
        assert_eq!(
            parse_const_statement("const state=0xabababab~u32;input"),
            Ok(("input", ("state".to_string(), Literal::U32(0xabab_abab))))
        );
    }

    #[test]
    fn weird_spacing2_test() {
        assert_eq!(
            parse_const_statement("const state  =  0xabababab~u32;input"),
            Ok(("input", ("state".to_string(), Literal::U32(0xabab_abab))))
        );
    }

    #[test]
    fn only_keyword_test() {
        assert_eq!(
            parse_const_statement("const;input"),
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
            parse_const_statement("const 1state = 0xabababab~u32;input"),
            Err(Err::Error(ParserError::ExpectedNameFound(
                "1state = 0xabababab~u32;input".to_string(),
                "1state = 0xabababab~u32;input".to_string(),
            )))
        );
    }

    #[test]
    fn wrong_keyword_test() {
        assert_eq!(
            parse_const_statement("let state = 0xabababab~u32;input"),
            Err(Err::Error(ParserError::ExpectedKeywordFound(
                "let state = 0xabababab~u32;input".to_string(),
                "const".to_string(),
                "let".to_string(),
            )))
        );
    }
}
