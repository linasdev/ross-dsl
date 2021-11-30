use nom::character::complete::{multispace0, multispace1};
use nom::combinator::cut;
use nom::sequence::{delimited, preceded, separated_pair, terminated};
use nom::IResult;

use crate::error::ParserError;
use crate::keyword::let_keyword;
use crate::literal::{literal, Literal};
use crate::parser::name_parser;
use crate::symbol::{equal_sign, semicolon};

pub fn let_statement(text: &str) -> IResult<&str, (&str, Literal), ParserError<&str>> {
    let name_parser = delimited(multispace1, name_parser, multispace0);
    let equal_sign_parser = terminated(equal_sign, multispace0);
    let name_value_pair_parser = separated_pair(name_parser, equal_sign_parser, literal);
    let keyword_parser = preceded(let_keyword, cut(name_value_pair_parser));
    let mut semicolon_parser = terminated(keyword_parser, semicolon);

    semicolon_parser(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    use cool_asserts::assert_matches;
    use nom::Err as NomErr;

    use crate::error::{ErrorKind, Expectation};

    #[test]
    fn hex_u32_test() {
        assert_matches!(
            let_statement("let state = 0xabababab~u32;input"),
            Ok(("input", ("state", Literal::U32(0xabab_abab)))),
        );
    }

    #[test]
    fn bool_test() {
        assert_matches!(
            let_statement("let state = false;input"),
            Ok(("input", ("state", Literal::Bool(false)))),
        );
    }

    #[test]
    fn weird_spacing1_test() {
        assert_matches!(
            let_statement("let state=0xabababab~u32;input"),
            Ok(("input", ("state", Literal::U32(0xabab_abab)))),
        );
    }

    #[test]
    fn weird_spacing2_test() {
        assert_matches!(
            let_statement("let  state  =  0xabababab~u32;input"),
            Ok(("input", ("state", Literal::U32(0xabab_abab)))),
        );
    }

    #[test]
    fn only_keyword_test() {
        assert_matches!(
            let_statement("let;input"),
            Err(NomErr::Failure(ParserError::Base {
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
            let_statement("let 1state = 0xabababab~u32;input"),
            Err(NomErr::Failure(ParserError::Base {
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
            let_statement("const state = 0xabababab~u32;input"),
            Err(NomErr::Error(ParserError::Base {
                location,
                kind,
                child,
            })) => {
                assert_matches!(location, "const");
                assert_matches!(kind, ErrorKind::Expected(Expectation::Keyword("let")));
                assert_matches!(child, None);
            },
        );
    }
}
