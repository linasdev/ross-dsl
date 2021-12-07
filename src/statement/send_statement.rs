use nom::branch::alt;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{cut, map, map_res};
use nom::sequence::{preceded, separated_pair, terminated, tuple};
use nom::IResult;
use std::collections::BTreeMap;
use std::convert::TryInto;

use ross_config::creator::Creator;
use ross_config::event_processor::EventProcessor;
use ross_config::extractor::{EventCodeExtractor, EventProducerAddressExtractor, PacketExtractor};
use ross_config::filter::ValueEqualToConstFilter;
use ross_config::matcher::Matcher;
use ross_config::producer::PacketProducer;
use ross_config::Value;

use crate::error::ParserError;
use crate::keyword::{from_keyword, if_keyword, send_keyword, to_keyword};
use crate::literal::{literal_or_constant, Literal};
use crate::statement::match_statement::match_statement;
use crate::symbol::semicolon;

pub fn send_statement<'a>(
    constants: &'a BTreeMap<&str, Literal>,
) -> impl FnMut(&str) -> IResult<&str, EventProcessor, ParserError<&str>> + 'a {
    move |text| {
        let if_match_parser = {
            let additional_matcher_parser = cut(preceded(multispace1, match_statement(constants)));
            let pair_parser = separated_pair(
                terminated(base_syntax_parser(constants), multispace1),
                if_keyword,
                additional_matcher_parser,
            );

            map(pair_parser, |((matcher, creators), additional_matcher)| {
                (
                    Matcher::And(Box::new(additional_matcher), Box::new(matcher)),
                    creators,
                )
            })
        };

        let normal_syntax_parser = terminated(
            base_syntax_parser(constants),
            preceded(multispace0, semicolon),
        );

        let (input, (matcher, creators)) = alt((if_match_parser, normal_syntax_parser))(text)?;

        Ok((input, EventProcessor { matcher, creators }))
    }
}

fn base_syntax_parser<'a>(
    constants: &'a BTreeMap<&str, Literal>,
) -> impl FnMut(&str) -> IResult<&str, (Matcher, Vec<Creator>), ParserError<&str>> + 'a {
    move |text| {
        let tuple_parser = tuple((
            literal_or_constant(constants),
            multispace1,
            from_keyword,
            multispace1,
            literal_or_constant(constants),
            multispace1,
            to_keyword,
            multispace1,
            literal_or_constant(constants),
        ));

        let content_parser = map_res::<_, _, _, _, ParserError<&str>, _, _>(
            tuple_parser,
            |(event_code, _, _, _, from_address, _, _, _, to_address)| {
                let event_code = event_code.try_into()?;
                let from_address = from_address.try_into()?;
                let to_address = to_address.try_into()?;

                let event_matcher = Matcher::Single {
                    extractor: Box::new(EventCodeExtractor::new()),
                    filter: Box::new(ValueEqualToConstFilter::new(Value::U16(event_code))),
                };

                let producer_matcher = Matcher::Single {
                    extractor: Box::new(EventProducerAddressExtractor::new()),
                    filter: Box::new(ValueEqualToConstFilter::new(Value::U16(from_address))),
                };

                let combined_matcher =
                    Matcher::And(Box::new(event_matcher), Box::new(producer_matcher));

                let packet_creator = Creator {
                    extractor: Box::new(PacketExtractor::new()),
                    producer: Box::new(PacketProducer::new(to_address)),
                    matcher: None,
                };

                Ok((combined_matcher, vec![packet_creator]))
            },
        );

        preceded(send_keyword, cut(preceded(multispace1, content_parser)))(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use cool_asserts::assert_matches;
    use nom::error::ErrorKind as NomErrorKind;
    use nom::Err as NomErr;

    use crate::error::{ErrorKind, Expectation};

    #[test]
    fn normal_syntax_test() {
        let constants = BTreeMap::new();
        let (input, event_processor) =
            send_statement(&constants)("send 0xabab~u16 from 0x0123~u16 to 0xffff~u16;input")
                .unwrap();

        assert_eq!(input, "input");

        assert_matches!(event_processor.matcher, Matcher::And(matcher1, matcher2) => {
            assert_matches!(*matcher1, Matcher::Single {extractor, filter} => {
                assert_eq!(
                    format!("{:?}", extractor),
                    format!("{:?}", EventCodeExtractor::new()),
                );
                assert_eq!(
                    format!("{:?}", filter),
                    format!("{:?}", ValueEqualToConstFilter::new(Value::U16(0xabab))),
                );
            });

            assert_matches!(*matcher2, Matcher::Single {extractor, filter} => {
                assert_eq!(
                    format!("{:?}", extractor),
                    format!("{:?}", EventProducerAddressExtractor::new()),
                );
                assert_eq!(
                    format!("{:?}", filter),
                    format!("{:?}", ValueEqualToConstFilter::new(Value::U16(0x0123))),
                );
            });
        });

        let creators = event_processor.creators;

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
    fn normal_syntax_missing_semicolon_test() {
        let constants = BTreeMap::new();
        assert_matches!(
            send_statement(&constants)("send 0xabab~u16 from 0x0123~u16 to 0xffff~u16"),
            Err(NomErr::Error(ParserError::Base {
                location: _,
                kind: ErrorKind::Nom(NomErrorKind::Alt),
                child: Some(_),
            }))
        );
    }

    #[test]
    fn normal_syntax_missing_from_keyword_test() {
        let constants = BTreeMap::new();
        assert_matches!(
            send_statement(&constants)("send 0xabab~u16 0x0123~u16 to 0xffff~u16;"),
            Err(NomErr::Failure(ParserError::Base {
                location: _,
                kind: ErrorKind::Expected(Expectation::Keyword("from")),
                child: Some(_),
            }))
        );
    }

    #[test]
    fn normal_syntax_missing_to_keyword_test() {
        let constants = BTreeMap::new();
        assert_matches!(
            send_statement(&constants)("send 0xabab~u16 from 0x0123~u16 0xffff~u16;"),
            Err(NomErr::Failure(ParserError::Base {
                location: _,
                kind: ErrorKind::Expected(Expectation::Keyword("to")),
                child: Some(_),
            }))
        );
    }

    #[test]
    fn normal_syntax_empty_test() {
        let constants = BTreeMap::new();
        assert_matches!(
            send_statement(&constants)(""),
            Err(NomErr::Error(ParserError::Base {
                location: _,
                kind: ErrorKind::Nom(NomErrorKind::Alt),
                child: Some(_),
            }))
        );
    }

    #[test]
    fn if_match_event_test() {
        let constants = BTreeMap::new();
        let (input, event_processor) = send_statement(&constants)(
            "send 0xabab~u16 from 0x0123~u16 to 0xffff~u16 if match event 0xbaba~u16;input",
        )
        .unwrap();

        assert_eq!(input, "input");

        assert_matches!(event_processor.matcher, Matcher::And(matcher1, matcher2) => {
            assert_matches!(*matcher1, Matcher::And(matcher1, matcher2) => {
                assert_matches!(*matcher1, Matcher::Single {extractor, filter} => {
                    assert_eq!(
                        format!("{:?}", extractor),
                        format!("{:?}", EventCodeExtractor::new()),
                    );
                    assert_eq!(
                        format!("{:?}", filter),
                        format!("{:?}", ValueEqualToConstFilter::new(Value::U16(0xabab))),
                    );
                });

                assert_matches!(*matcher2, Matcher::Single {extractor, filter} => {
                    assert_eq!(
                        format!("{:?}", extractor),
                        format!("{:?}", EventProducerAddressExtractor::new()),
                    );
                    assert_eq!(
                        format!("{:?}", filter),
                        format!("{:?}", ValueEqualToConstFilter::new(Value::U16(0x0123))),
                    );
                });
            });

            assert_matches!(*matcher2, Matcher::Single {extractor, filter} => {
                assert_eq!(
                    format!("{:?}", extractor),
                    format!("{:?}", EventCodeExtractor::new()),
                );
                assert_eq!(
                    format!("{:?}", filter),
                    format!("{:?}", ValueEqualToConstFilter::new(Value::U16(0xbaba))),
                );
            });
        });

        let creators = event_processor.creators;

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
    fn if_match_event_missing_semicolon_test() {
        let constants = BTreeMap::new();
        assert_matches!(
            send_statement(&constants)(
                "send 0xabab~u16 from 0x0123~u16 to 0xffff~u16 if match event 0xbaba~u16",
            ),
            Err(NomErr::Failure(ParserError::Base {
                location: _,
                kind: ErrorKind::Nom(NomErrorKind::Alt),
                child: Some(_),
            }))
        );
    }

    #[test]
    fn if_match_event_missing_from_keyword_test() {
        let constants = BTreeMap::new();
        assert_matches!(
            send_statement(&constants)(
                "send 0xabab~u16 0x0123~u16 to 0xffff~u16 if match event 0xbaba~u16;"
            ),
            Err(NomErr::Failure(ParserError::Base {
                location: _,
                kind: ErrorKind::Expected(Expectation::Keyword("from")),
                child: Some(_),
            }))
        );
    }

    #[test]
    fn if_match_event_missing_to_keyword_test() {
        let constants = BTreeMap::new();
        assert_matches!(
            send_statement(&constants)(
                "send 0xabab~u16 from 0x0123~u16 0xffff~u16 if match event 0xbaba~u16;",
            ),
            Err(NomErr::Failure(ParserError::Base {
                location: _,
                kind: ErrorKind::Expected(Expectation::Keyword("to")),
                child: Some(_),
            }))
        );
    }
}
