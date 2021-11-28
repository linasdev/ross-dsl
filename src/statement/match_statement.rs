use nom::branch::alt;
use nom::combinator::map_res;
use nom::sequence::{delimited, pair, preceded, terminated};
use nom::{Err, IResult};
use std::convert::TryInto;

use ross_config::extractor::Extractor;
use ross_config::extractor::{EventCodeExtractor, EventProducerAddressExtractor, NoneExtractor};
use ross_config::filter::Filter;
use ross_config::filter::ValueEqualToConstFilter;
use ross_config::matcher::Matcher;
use ross_config::Value;

use crate::extractor::extractor;
use crate::filter::filter;
use crate::keyword::{event_keyword, match_keyword, producer_keyword};
use crate::literal::literal;
use crate::parser::{multispace0, multispace1, ParserError};
use crate::symbol::{close_brace, open_brace, semicolon};

pub fn match_statement(text: &str) -> IResult<&str, Matcher, ParserError> {
    let mut event_match_parser = {
        let content_parser = map_res(literal, |event_code| {
            let event_code = event_code.try_into()?;
            let extractor = Box::new(EventCodeExtractor::new()) as Box<dyn Extractor>;
            let filter =
                Box::new(ValueEqualToConstFilter::new(Value::U16(event_code))) as Box<dyn Filter>;

            Ok((extractor, filter))
        });

        let event_keyword_parser = preceded(event_keyword, preceded(multispace1, content_parser));
        let match_keyword_parser =
            preceded(match_keyword, preceded(multispace1, event_keyword_parser));
        terminated(match_keyword_parser, semicolon)
    };

    match event_match_parser(text) {
        Ok((input, (extractor, filter))) => return Ok((input, Matcher { extractor, filter })),
        Err(Err::Error(ParserError::ExpectedKeywordFound(_, expected, _)))
            if expected == "event" => {}
        Err(err) => return Err(Err::convert(err)),
    }

    let mut producer_match_parser = {
        let content_parser = map_res(literal, |producer_address| {
            let producer_address = producer_address.try_into()?;
            let extractor = Box::new(EventProducerAddressExtractor::new()) as Box<dyn Extractor>;
            let filter = Box::new(ValueEqualToConstFilter::new(Value::U16(producer_address)))
                as Box<dyn Filter>;

            Ok((extractor, filter))
        });

        let producer_keyword_parser =
            preceded(producer_keyword, preceded(multispace1, content_parser));
        let match_keyword_parser = preceded(
            match_keyword,
            preceded(multispace1, producer_keyword_parser),
        );
        terminated(match_keyword_parser, semicolon)
    };

    match producer_match_parser(text) {
        Ok((input, (extractor, filter))) => return Ok((input, Matcher { extractor, filter })),
        Err(Err::Error(ParserError::ExpectedKeywordFound(_, expected, _)))
            if expected == "producer" => {}
        Err(err) => return Err(Err::convert(err)),
    }

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
        Ok((input, (extractor, filter))) => return Ok((input, Matcher { extractor, filter })),
        Err(err) => return Err(Err::convert(err)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn event_test() {
        let (input, matcher) = match_statement("match  event  0xabab~u16;input").unwrap();

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
    fn event_invalid_literal_test() {
        assert_eq!(
            match_statement("match  event  0xabababab~u32;input").unwrap_err(),
            Err::Error(ParserError::CastFromToNotAllowed(
                "u32".to_string(),
                "u16".to_string(),
            ))
        );
    }

    #[test]
    fn event_missing_semicolon_test() {
        assert_eq!(
            match_statement("match  event  0xabab~u16").unwrap_err(),
            Err::Error(ParserError::ExpectedSymbolFound(
                "".to_string(),
                ";".to_string(),
                "".to_string(),
            ))
        );
    }

    #[test]
    fn producer_test() {
        let (input, matcher) = match_statement("match  producer  0xabab~u16;input").unwrap();

        assert_eq!(input, "input");
        assert_eq!(
            format!("{:?}", matcher.extractor),
            format!("{:?}", EventProducerAddressExtractor::new())
        );
        assert_eq!(
            format!("{:?}", matcher.filter),
            format!("{:?}", ValueEqualToConstFilter::new(Value::U16(0xabab)))
        );
    }

    #[test]
    fn producer_invalid_literal_test() {
        assert_eq!(
            match_statement("match  producer  0xabababab~u32;input").unwrap_err(),
            Err::Error(ParserError::CastFromToNotAllowed(
                "u32".to_string(),
                "u16".to_string(),
            ))
        );
    }

    #[test]
    fn producer_missing_semicolon_test() {
        assert_eq!(
            match_statement("match  producer  0xabab~u16").unwrap_err(),
            Err::Error(ParserError::ExpectedSymbolFound(
                "".to_string(),
                ";".to_string(),
                "".to_string(),
            ))
        );
    }
}
