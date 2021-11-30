use nom::branch::alt;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{cut, map_res, map,};
use nom::sequence::{preceded, terminated, tuple, separated_pair};
use nom::IResult;
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
use crate::literal::literal;
use crate::statement::match_statement::match_statement;
use crate::symbol::semicolon;

pub fn send_statement(text: &str) -> IResult<&str, EventProcessor, ParserError<&str>> {    
    let if_match_parser = {
        let additional_matcher_parser = cut(preceded(multispace1, match_statement));
        let pair_parser = separated_pair(terminated(base_syntax_parser, multispace1), if_keyword, additional_matcher_parser);
        
        map(pair_parser, |((mut matchers, creators), matcher)| {
            matchers.push(matcher);
            (matchers, creators)
        })
    };

    let normal_syntax_parser = terminated(base_syntax_parser, preceded(multispace0, semicolon));

    let (input, (matchers, creators)) = alt((if_match_parser, normal_syntax_parser))(text)?;

    Ok((input, EventProcessor { matchers, creators }))
}

fn base_syntax_parser(text: &str) -> IResult<&str, (Vec<Matcher>, Vec<Creator>), ParserError<&str>>{
    let tuple_parser = tuple((
        literal,
        multispace1,
        from_keyword,
        multispace1,
        literal,
        multispace1,
        to_keyword,
        multispace1,
        literal,
    ));

    let content_parser = map_res::<_, _, _, _, ParserError<&str>, _, _>(
        tuple_parser,
        |(event_code, _, _, _, from_address, _, _, _, to_address)| {
            let event_code = event_code.try_into()?;
            let from_address = from_address.try_into()?;
            let to_address = to_address.try_into()?;

            let event_matcher = Matcher {
                extractor: Box::new(EventCodeExtractor::new()),
                filter: Box::new(ValueEqualToConstFilter::new(Value::U16(event_code))),
            };

            let producer_matcher = Matcher {
                extractor: Box::new(EventProducerAddressExtractor::new()),
                filter: Box::new(ValueEqualToConstFilter::new(Value::U16(from_address))),
            };

            let packet_creator = Creator {
                extractor: Box::new(PacketExtractor::new()),
                producer: Box::new(PacketProducer::new(to_address)),
            };

            Ok((vec![event_matcher, producer_matcher], vec![packet_creator]))
        },
    );

    preceded(send_keyword, cut(preceded(multispace1, content_parser)))(text)
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
        let (input, event_processor) =
            send_statement("send 0xabab~u16 from 0x0123~u16 to 0xffff~u16;input").unwrap();

        assert_eq!(input, "input");

        let matchers = event_processor.matchers;

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
        assert_matches!(
            send_statement("send 0xabab~u16 from 0x0123~u16 to 0xffff~u16"),
            Err(NomErr::Error(ParserError::Base {
                location: _,
                kind: ErrorKind::Nom(NomErrorKind::Alt),
                child: Some(_),
            }))
        );
    }

    #[test]
    fn normal_syntax_missing_from_keyword_test() {
        assert_matches!(
            send_statement("send 0xabab~u16 0x0123~u16 to 0xffff~u16;"),
            Err(NomErr::Failure(ParserError::Base {
                location: _,
                kind: ErrorKind::Expected(Expectation::Keyword("from")),
                child: Some(_),
            }))
        );
    }

    #[test]
    fn normal_syntax_missing_to_keyword_test() {
        assert_matches!(
            send_statement("send 0xabab~u16 from 0x0123~u16 0xffff~u16;"),
            Err(NomErr::Failure(ParserError::Base {
                location: _,
                kind: ErrorKind::Expected(Expectation::Keyword("to")),
                child: Some(_),
            }))
        );
    }

    #[test]
    fn normal_syntax_empty_test() {
        assert_matches!(
            send_statement(""),
            Err(NomErr::Error(ParserError::Base {
                location: _,
                kind: ErrorKind::Nom(NomErrorKind::Alt),
                child: Some(_),
            }))
        );
    }

    #[test]
    fn if_match_event_test() {
        let (input, event_processor) = send_statement(
            "send 0xabab~u16 from 0x0123~u16 to 0xffff~u16 if match event 0xbaba~u16;input",
        )
        .unwrap();

        assert_eq!(input, "input");

        let matchers = event_processor.matchers;

        assert_eq!(matchers.len(), 3);
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
        assert_eq!(
            format!("{:?}", matchers[2].extractor),
            format!("{:?}", EventCodeExtractor::new()),
        );
        assert_eq!(
            format!("{:?}", matchers[2].filter),
            format!("{:?}", ValueEqualToConstFilter::new(Value::U16(0xbaba))),
        );

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
        assert_matches!(
            send_statement(
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
        assert_matches!(
            send_statement("send 0xabab~u16 0x0123~u16 to 0xffff~u16 if match event 0xbaba~u16;"),
            Err(NomErr::Failure(ParserError::Base {
                location: _,
                kind: ErrorKind::Expected(Expectation::Keyword("from")),
                child: Some(_),
            }))
        );
    }

    #[test]
    fn if_match_event_missing_to_keyword_test() {
        assert_matches!(
            send_statement(
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
