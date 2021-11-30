use std::collections::BTreeMap;
use nom::character::complete::multispace0;
use nom::combinator::cut;
use nom::multi::{many0, many1};
use nom::sequence::{pair, preceded, terminated};
use nom::IResult;

use ross_config::event_processor::EventProcessor;

use crate::literal::Literal;
use crate::error::ParserError;
use crate::keyword::do_keyword;
use crate::statement::fire_statement::fire_statement;
use crate::statement::match_statement::match_statement;
use crate::symbol::{close_brace, open_brace};

pub fn do_statement<'a>(constants: &'a BTreeMap<&str, Literal>) -> impl FnMut(&str) -> IResult<&str, EventProcessor, ParserError<&str>> + 'a{
    move |text| {
        let content_parser = preceded(
            open_brace,
            pair(
                many0(preceded(multispace0, match_statement(constants))),
                many1(preceded(multispace0, fire_statement(constants))),
            ),
        );
        let keyword_parser = preceded(do_keyword, cut(preceded(multispace0, content_parser)));
        let mut close_brace_parser = terminated(keyword_parser, preceded(multispace0, close_brace));
    
        let (input, (matchers, creators)) = close_brace_parser(text)?;
    
        Ok((input, EventProcessor { matchers, creators }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use cool_asserts::assert_matches;
    use nom::error::ErrorKind as NomErrorKind;
    use nom::Err as NomErr;

    use ross_config::extractor::{
        EventCodeExtractor, EventProducerAddressExtractor, PacketExtractor,
    };
    use ross_config::filter::ValueEqualToConstFilter;
    use ross_config::producer::PacketProducer;
    use ross_config::Value;

    use crate::error::{ErrorKind, Expectation};

    #[test]
    fn provided_extractor_test() {
        let constants = BTreeMap::new();
        let (input, event_producer) = do_statement(&constants)(
            "do {
                match event 0xabab~u16;
                match producer 0x0123~u16;
                fire {
                    PacketExtractor();
                    PacketProducer(0xffff~u16);
                }
            }input",
        )
        .unwrap();

        assert_eq!(input, "input");

        let matchers = event_producer.matchers;

        assert_eq!(matchers.len(), 2);
        assert_eq!(
            format!("{:?}", matchers[0].extractor),
            format!("{:?}", EventCodeExtractor::new()),
        );
        assert_eq!(
            format!("{:?}", matchers[0].filter),
            format!("{:?}", ValueEqualToConstFilter::new(Value::U16(0xabab))),
        );
        assert_eq!(
            format!("{:?}", matchers[1].extractor),
            format!("{:?}", EventProducerAddressExtractor::new()),
        );
        assert_eq!(
            format!("{:?}", matchers[1].filter),
            format!("{:?}", ValueEqualToConstFilter::new(Value::U16(0x0123))),
        );

        let creators = event_producer.creators;

        assert_eq!(creators.len(), 1);
        assert_eq!(
            format!("{:?}", creators[0].extractor),
            format!("{:?}", PacketExtractor::new()),
        );
        assert_eq!(
            format!("{:?}", creators[0].producer),
            format!("{:?}", PacketProducer::new(0xffff)),
        );
    }

    #[test]
    fn missing_close_brace_test() {
        let constants = BTreeMap::new();
        assert_matches!(
            do_statement(&constants)(
                "do {
                    match event 0xabab~u16;
                    match producer 0x0123~u16;
                    fire {
                        PacketExtractor();
                        PacketProducer(0xffff~u16);
                    }",
            ),
            Err(NomErr::Error(ParserError::Base {
                location: _,
                kind: ErrorKind::Expected(Expectation::Symbol('}')),
                child: None,
            }))
        );
    }

    #[test]
    fn invalid_literal_test() {
        let constants = BTreeMap::new();
        assert_matches!(
            do_statement(&constants)(
                "do {
                    match event 0xabab~u16;
                    match producer 0x0123~u16;
                    fire {
                        PacketExtractor();
                        PacketProducer(0xffffffff~u32);
                    }
                }",
            ),
            Err(NomErr::Failure(ParserError::Base {
                location: _,
                kind: ErrorKind::CastFromToNotAllowed("u32", "u16"),
                child: None,
            }))
        );
    }

    #[test]
    fn no_fire_statement_test() {
        let constants = BTreeMap::new();
        assert_matches!(
            do_statement(&constants)(
                "do {
                    match event 0xabab~u16;
                    match producer 0x0123~u16;",
            ),
            Err(NomErr::Failure(ParserError::Base {
                location: _,
                kind: ErrorKind::Nom(NomErrorKind::Many1),
                child: Some(_),
            }))
        );
    }
}
