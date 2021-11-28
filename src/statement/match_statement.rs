use nom::branch::alt;
use nom::sequence::{delimited, pair, preceded, terminated};
use nom::{Err, IResult};

use ross_config::extractor::Extractor;
use ross_config::extractor::NoneExtractor;
use ross_config::matcher::Matcher;

use crate::extractor::extractor;
use crate::filter::filter;
use crate::keyword::match_keyword;
use crate::parser::{multispace0, multispace1, ParserError};
use crate::symbol::{close_brace, open_brace};

pub fn match_statement(text: &str) -> IResult<&str, Matcher, ParserError> {
    let mut block_match_parser = {
        let extractor_parser = alt((delimited(multispace0, extractor, multispace0), |input| {
            Ok((input, Box::new(NoneExtractor::new()) as Box<dyn Extractor>))
        }));
        let filter_parser = delimited(multispace0, filter, multispace0);
        let content_parser = preceded(open_brace, pair(extractor_parser, filter_parser));
        let keyword_parser = preceded(match_keyword, preceded(multispace1, content_parser));
        terminated(keyword_parser, preceded(multispace0, close_brace))
    };

    match block_match_parser(text) {
        Ok((input, (extractor, filter))) => Ok((input, Matcher { extractor, filter })),
        Err(err) => Err(Err::convert(err)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ross_config::extractor::EventCodeExtractor;
    use ross_config::filter::ValueEqualToConstFilter;
    use ross_config::Value;

    #[test]
    fn block_provided_extractor_test() {
        let (input, matcher) = match_statement(
            "match {
                EventCodeExtractor();
                ValueEqualToConstFilter(0xabab~u16);
            }input",
        )
        .unwrap();

        assert_eq!(input, "input");
        assert_eq!(
            format!("{:?}", matcher.extractor),
            format!("{:?}", EventCodeExtractor::new())
        );
        assert_eq!(
            format!("{:?}", matcher.filter),
            format!("{:?}", ValueEqualToConstFilter::new(Value::U16(0xabab)))
        );
    }

    #[test]
    fn block_no_extractor_test() {
        let (input, matcher) = match_statement(
            "match {
                ValueEqualToConstFilter(0xabab~u16);
            }input",
        )
        .unwrap();

        assert_eq!(input, "input");
        assert_eq!(
            format!("{:?}", matcher.extractor),
            format!("{:?}", NoneExtractor::new())
        );
        assert_eq!(
            format!("{:?}", matcher.filter),
            format!("{:?}", ValueEqualToConstFilter::new(Value::U16(0xabab)))
        );
    }

    #[test]
    fn block_two_extractors_test() {
        assert_eq!(
            match_statement(
                "match {
                    EventCodeExtractor();
                    NoneExtractor();
                }input",
            )
            .unwrap_err(),
            Err::Error(ParserError::UnknownFilter(
                "NoneExtractor();\n                }input".to_string(),
                "NoneExtractor".to_string(),
            ))
        );
    }
}
