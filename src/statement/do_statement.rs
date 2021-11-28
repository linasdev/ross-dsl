use nom::multi::{many0, many1};
use nom::sequence::{pair, preceded, terminated};
use nom::{Err, IResult};

use ross_config::event_processor::EventProcessor;

use crate::keyword::do_keyword;
use crate::parser::{multispace0, ParserError};
use crate::statement::fire_statement::fire_statement;
use crate::statement::match_statement::match_statement;
use crate::symbol::{close_brace, open_brace};

pub fn do_statement(text: &str) -> IResult<&str, EventProcessor, ParserError> {
    let content_parser = preceded(
        open_brace,
        pair(many0(preceded(multispace0, match_statement)), many1(preceded(multispace0, fire_statement))),
    );
    let keyword_parser = preceded(do_keyword, preceded(multispace0, content_parser));
    let mut close_brace_parser = terminated(keyword_parser, preceded(multispace0, close_brace));

    match close_brace_parser(text) {
        Ok((input, (matchers, creators))) => {
            return Ok((input, EventProcessor { matchers, creators }))
        }
        Err(err) => return Err(Err::convert(err)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ross_config::extractor::{EventCodeExtractor, PacketExtractor, EventProducerAddressExtractor};
    use ross_config::filter::ValueEqualToConstFilter;
    use ross_config::producer::PacketProducer;
    use ross_config::Value;

    #[test]
    fn provided_extractor_test() {
        let (input, event_producer) = do_statement(
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
        assert_eq!(
            do_statement(
                "do {
                    match event 0xabab~u16;
                    match producer 0x0123~u16;
                    fire {
                        PacketExtractor();
                        PacketProducer(0xffff~u16);
                    }"
            )
            .unwrap_err(),
            Err::Error(ParserError::ExpectedSymbolFound(
                "".to_string(),
                "}".to_string(),
                "".to_string(),
            ))
        );
    }

    #[test]
    fn invalid_literal_test() {
        assert_eq!(
            do_statement(
                "do {
                    match event 0xabab~u16;
                    match producer 0x0123~u16;
                    fire {
                        PacketExtractor();
                        PacketProducer(0xffffffff~u32);
                    }
                }"
            )
            .unwrap_err(),
            Err::Error(ParserError::CastFromToNotAllowed(
                "u32".to_string(),
                "u16".to_string(),
            ))
        );
    }

    #[test]
    fn no_fire_statement_test() {
        assert_eq!(
            do_statement(
                "do {
                    match event 0xabab~u16;
                    match producer 0x0123~u16;
                }"
            )
            .unwrap_err(),
            Err::Error(ParserError::ExpectedKeywordFound(
                "}".to_string(),
                "fire".to_string(),
                "}".to_string(),
            ))
        );
    }
}
