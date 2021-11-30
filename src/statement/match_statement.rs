use nom::branch::alt;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::map_res;
use nom::sequence::{delimited, pair, preceded, terminated};
use nom::IResult;
use std::convert::TryInto;

use ross_config::extractor::Extractor;
use ross_config::extractor::{EventCodeExtractor, EventProducerAddressExtractor, NoneExtractor};
use ross_config::filter::Filter;
use ross_config::filter::ValueEqualToConstFilter;
use ross_config::matcher::Matcher;
use ross_config::Value;

use crate::error::ParserError;
use crate::extractor::extractor;
use crate::filter::filter;
use crate::keyword::{event_keyword, match_keyword, producer_keyword};
use crate::literal::literal;
use crate::symbol::{close_brace, open_brace, semicolon};

pub fn match_statement(text: &str) -> IResult<&str, Matcher, ParserError<&str>> {
    let event_match_parser = {
        let content_parser =
            map_res::<_, _, _, _, ParserError<&str>, _, _>(literal, |event_code| {
                let event_code = event_code.try_into()?;

                let extractor = Box::new(EventCodeExtractor::new()) as Box<dyn Extractor>;
                let filter = Box::new(ValueEqualToConstFilter::new(Value::U16(event_code)))
                    as Box<dyn Filter>;

                Ok((extractor, filter))
            });

        let event_keyword_parser = preceded(event_keyword, preceded(multispace1, content_parser));

        let match_keyword_parser =
            preceded(match_keyword, preceded(multispace1, event_keyword_parser));

        terminated(match_keyword_parser, semicolon)
    };

    let producer_match_parser = {
        let content_parser =
            map_res::<_, _, _, _, ParserError<&str>, _, _>(literal, |producer_address| {
                let producer_address = producer_address.try_into()?;

                let extractor =
                    Box::new(EventProducerAddressExtractor::new()) as Box<dyn Extractor>;
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

    let block_match_parser = {
        let extractor_parser = alt((delimited(multispace0, extractor, multispace0), |input| {
            Ok((input, Box::new(NoneExtractor::new()) as Box<dyn Extractor>))
        }));

        let filter_parser = delimited(multispace0, filter, multispace0);
        let content_parser = preceded(open_brace, pair(extractor_parser, filter_parser));
        let keyword_parser = preceded(match_keyword, preceded(multispace1, content_parser));

        terminated(keyword_parser, preceded(multispace0, close_brace))
    };

    let (input, (extractor, filter)) = alt((
        event_match_parser,
        producer_match_parser,
        block_match_parser,
    ))(text)?;

    Ok((input, Matcher { extractor, filter }))
}

#[cfg(test)]
mod tests {
    use super::*;

    use cool_asserts::assert_matches;
    use nom::error::ErrorKind as NomErrorKind;
    use nom::Err as NomErr;

    use crate::error::ErrorKind;

    #[test]
    fn block_extractor_test() {
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
        assert_matches!(
            match_statement(
                "match {
                    EventCodeExtractor();
                    NoneExtractor();
                }input",
            ),
            Err(NomErr::Error(ParserError::Base {
                location: _,
                kind: ErrorKind::Nom(NomErrorKind::Alt),
                child: Some(_),
            }))
        );
    }

    #[test]
    fn event_test() {
        let (input, matcher) = match_statement("match event 0xabab~u16;input").unwrap();

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
        assert_matches!(
            match_statement("match event 0xabababab~u32;input"),
            Err(NomErr::Error(ParserError::Base {
                location: _,
                kind: ErrorKind::Nom(NomErrorKind::Alt),
                child: Some(_),
            }))
        );
    }

    #[test]
    fn event_missing_semicolon_test() {
        assert_matches!(
            match_statement("match event 0xabab~u16"),
            Err(NomErr::Error(ParserError::Base {
                location: _,
                kind: ErrorKind::Nom(NomErrorKind::Alt),
                child: Some(_),
            }))
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
        assert_matches!(
            match_statement("match producer 0xabababab~u32;input"),
            Err(NomErr::Error(ParserError::Base {
                location: _,
                kind: ErrorKind::Nom(NomErrorKind::Alt),
                child: Some(_),
            }))
        );
    }

    #[test]
    fn producer_missing_semicolon_test() {
        assert_matches!(
            match_statement("match producer 0xabab~u16"),
            Err(NomErr::Error(ParserError::Base {
                location: _,
                kind: ErrorKind::Nom(NomErrorKind::Alt),
                child: Some(_),
            }))
        );
    }
}
