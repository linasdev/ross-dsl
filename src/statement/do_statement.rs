use nom::character::complete::multispace0;
use nom::combinator::{cut, map};
use nom::multi::{many0, many1};
use nom::sequence::{pair, preceded, terminated};
use nom::IResult;
use std::collections::BTreeMap;

use ross_config::event_processor::EventProcessor;
use ross_config::matcher::Matcher;

use crate::error::ParserError;
use crate::keyword::do_keyword;
use crate::literal::Literal;
use crate::statement::fire_statement::fire_statement;
use crate::statement::match_statement::match_statement;
use crate::symbol::{close_brace, open_brace};

pub fn do_statement<'a>(
    constants: &'a BTreeMap<&str, Literal>,
) -> impl FnMut(&str) -> IResult<&str, EventProcessor, ParserError<&str>> + 'a {
    move |text| {
        let content_parser = preceded(
            open_brace,
            pair(
                map(many1(preceded(multispace0, match_statement(constants))), map_matchers_to_matcher),
                many0(preceded(multispace0, fire_statement(constants))),
            ),
        );
        let keyword_parser = preceded(do_keyword, cut(preceded(multispace0, content_parser)));
        let mut close_brace_parser = terminated(keyword_parser, preceded(multispace0, close_brace));

        let (input, (matcher, creators)) = close_brace_parser(text)?;

        Ok((input, EventProcessor { matcher, creators }))
    }
}

fn map_matchers_to_matcher(mut matchers: Vec<Matcher>) -> Matcher {
    if matchers.len() < 2 {
        panic!("This should never be reached because we're using many1");
    } else if matchers.len() == 2 {
        let matcher2 = matchers.pop().unwrap();
        let matcher1 = matchers.pop().unwrap();
        Matcher::And(Box::new(matcher1), Box::new(matcher2))
    } else {
        let matcher2 = matchers.pop().unwrap();
        Matcher::And(
            Box::new(map_matchers_to_matcher(matchers)),
            Box::new(matcher2),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use cool_asserts::assert_matches;
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

        assert_matches!(
            event_producer.matcher,
            Matcher::And(matcher1, matcher2) => {
                assert_matches!(*matcher1, Matcher::Single{extractor, filter} => {
                        assert_eq!(
                            format!("{:?}", extractor),
                            format!("{:?}", EventCodeExtractor::new())
                        );
                        assert_eq!(
                            format!("{:?}", filter),
                            format!("{:?}", ValueEqualToConstFilter::new(Value::U16(0xabab)))
                        );
                    },
                );

                assert_matches!(*matcher2, Matcher::Single{extractor, filter} => {
                        assert_eq!(
                            format!("{:?}", extractor),
                            format!("{:?}", EventProducerAddressExtractor::new())
                        );
                        assert_eq!(
                            format!("{:?}", filter),
                            format!("{:?}", ValueEqualToConstFilter::new(Value::U16(0x0123)))
                        );
                    },
                );
            },
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
}
