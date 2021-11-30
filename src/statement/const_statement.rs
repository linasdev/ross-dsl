use nom::character::complete::{multispace0, multispace1};
use nom::sequence::{delimited, preceded, separated_pair, terminated};
use nom::{Err as NomErr, IResult};

use crate::error::{ErrorKind, Expectation, ParserError};
use crate::keyword::const_keyword;
use crate::literal::{literal, Literal};
use crate::parser::alpha_or_underscore1;
use crate::symbol::{equal_sign, semicolon};

pub fn const_statement(text: &str) -> IResult<&str, (&str, Literal), ParserError<&str>> {
    let name_parser = delimited(multispace1, alpha_or_underscore1, multispace0);
    let equal_sign_parser = terminated(equal_sign, multispace0);
    let name_value_pair_parser = separated_pair(name_parser, equal_sign_parser, literal);
    let keyword_parser = preceded(const_keyword, name_value_pair_parser);
    let mut semicolon_parser = terminated(keyword_parser, semicolon);

    match semicolon_parser(text) {
        Ok((input, (name, value))) => Ok((input, (name, value))),
        Err(NomErr::Error(ParserError::Base {
            location,
            kind: ErrorKind::Expected(Expectation::Alpha),
            child,
        })) => Err(NomErr::Error(ParserError::Base {
            location,
            kind: ErrorKind::Expected(Expectation::Name),
            child: Some(Box::new(ParserError::Base {
                location,
                kind: ErrorKind::Expected(Expectation::Alpha),
                child,
            })),
        })),
        err => err,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use cool_asserts::assert_matches;

    #[test]
    fn hex_u32_test() {
        assert_matches!(
            const_statement("const state = 0xabababab~u32;input"),
            Ok(("input", ("state", Literal::U32(0xabab_abab)))),
        );
    }

    #[test]
    fn bool_test() {
        assert_matches!(
            const_statement("const state = false;input"),
            Ok(("input", ("state", Literal::Bool(false)))),
        );
    }

    #[test]
    fn weird_spacing1_test() {
        assert_matches!(
            const_statement("const state=0xabababab~u32;input"),
            Ok(("input", ("state", Literal::U32(0xabab_abab)))),
        );
    }

    #[test]
    fn weird_spacing2_test() {
        assert_matches!(
            const_statement("const  state  =  0xabababab~u32;input"),
            Ok(("input", ("state", Literal::U32(0xabab_abab)))),
        );
    }

    #[test]
    fn only_keyword_test() {
        assert_matches!(
            const_statement("const;input"),
            Err(NomErr::Error(ParserError::Base {
                location,
                kind,
                child,
            })) => {
                assert_matches!(location, ";input");
                assert_matches!(kind, ErrorKind::Expected(Expectation::MultiSpace));
                assert_matches!(child, None);
            },
        );
    }

    #[test]
    fn invalid_name_test() {
        assert_matches!(
            const_statement("const 1state = 0xabababab~u32;input"),
            Err(NomErr::Error(ParserError::Base {
                location,
                kind,
                child,
            })) => {
                assert_matches!(location, "1state = 0xabababab~u32;input");
                assert_matches!(kind, ErrorKind::Expected(Expectation::Name));
                assert_matches!(child, Some(_));
            },
        );
    }

    #[test]
    fn wrong_keyword_test() {
        assert_matches!(
            const_statement("let state = 0xabababab~u32;input"),
            Err(NomErr::Error(ParserError::Base {
                location,
                kind,
                child,
            })) => {
                assert_matches!(location, "let");
                assert_matches!(kind, ErrorKind::Expected(Expectation::Keyword("const")));
                assert_matches!(child, None);
            },
        );
    }
}
