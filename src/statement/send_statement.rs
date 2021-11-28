use nom::combinator::map_res;
use nom::sequence::{preceded, terminated, tuple};
use nom::{Err, IResult};
use std::convert::TryInto;

use ross_config::creator::Creator;
use ross_config::event_processor::EventProcessor;
use ross_config::extractor::{EventCodeExtractor, EventProducerAddressExtractor, PacketExtractor};
use ross_config::filter::ValueEqualToConstFilter;
use ross_config::matcher::Matcher;
use ross_config::producer::PacketProducer;
use ross_config::Value;

use crate::keyword::{from_keyword, send_keyword, to_keyword};
use crate::literal::literal;
use crate::parser::{multispace0, multispace1, ParserError};
use crate::symbol::semicolon;

pub fn send_statement(text: &str) -> IResult<&str, EventProcessor, ParserError> {
    // send literal from literal to literal;

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

    let content_parser = map_res(
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

    let keyword_parser = preceded(send_keyword, preceded(multispace1, content_parser));
    let mut semicolon_parser = terminated(keyword_parser, preceded(multispace0, semicolon));

    match semicolon_parser(text) {
        Ok((input, (matchers, creators))) => {
            return Ok((input, EventProcessor { matchers, creators }))
        }
        Err(err) => return Err(Err::convert(err)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
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
    fn missing_semicolon_test() {
        assert_eq!(
            send_statement("send 0xabab~u16 from 0x0123~u16 to 0xffff~u16").unwrap_err(),
            Err::Error(ParserError::ExpectedSymbolFound(
                "".to_string(),
                ";".to_string(),
                "".to_string(),
            ))
        );
    }

    #[test]
    fn missing_from_keyword_test() {
        assert_eq!(
            send_statement("send 0xabab~u16 0x0123~u16 to 0xffff~u16").unwrap_err(),
            Err::Error(ParserError::ExpectedKeywordFound(
                "0x0123~u16 to 0xffff~u16".to_string(),
                "from".to_string(),
                "0x0123~u16 to 0xffff~u16".to_string(),
            ))
        );
    }

    #[test]
    fn missing_to_keyword_test() {
        assert_eq!(
            send_statement("send 0xabab~u16 from 0x0123~u16 0xffff~u16").unwrap_err(),
            Err::Error(ParserError::ExpectedKeywordFound(
                "0xffff~u16".to_string(),
                "to".to_string(),
                "0xffff~u16".to_string(),
            ))
        );
    }

    #[test]
    fn empty_test() {
        assert_eq!(
            send_statement("").unwrap_err(),
            Err::Error(ParserError::ExpectedKeywordFound(
                "".to_string(),
                "send".to_string(),
                "".to_string(),
            ))
        );
    }
}
