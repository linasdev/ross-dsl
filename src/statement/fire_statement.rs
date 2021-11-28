use nom::branch::alt;
use nom::sequence::{delimited, pair, preceded, terminated};
use nom::{Err, IResult};

use ross_config::creator::Creator;
use ross_config::extractor::{Extractor, NoneExtractor};

use crate::extractor::extractor;
use crate::keyword::fire_keyword;
use crate::parser::{multispace0, multispace1, ParserError};
use crate::producer::producer;
use crate::symbol::{close_brace, open_brace};

pub fn fire_statement(text: &str) -> IResult<&str, Creator, ParserError> {
    let extractor_parser = alt((delimited(multispace0, extractor, multispace0), |input| {
        Ok((input, Box::new(NoneExtractor::new()) as Box<dyn Extractor>))
    }));

    let producer_parser = delimited(multispace0, producer, multispace0);
    let content_parser = preceded(open_brace, pair(extractor_parser, producer_parser));
    let keyword_parser = preceded(fire_keyword, preceded(multispace1, content_parser));
    let mut close_brace_parser = terminated(keyword_parser, preceded(multispace0, close_brace));

    match close_brace_parser(text) {
        Ok((input, (extractor, producer))) => {
            return Ok((
                input,
                Creator {
                    extractor,
                    producer,
                },
            ))
        }
        Err(err) => return Err(Err::convert(err)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ross_config::extractor::PacketExtractor;
    use ross_config::producer::PacketProducer;

    #[test]
    fn provided_extractor_test() {
        let (input, matcher) = fire_statement(
            "fire {
                PacketExtractor();
                PacketProducer(0xabab~u16);
            }input",
        )
        .unwrap();

        assert_eq!(input, "input");
        assert_eq!(
            format!("{:?}", matcher.extractor),
            format!("{:?}", PacketExtractor::new())
        );
        assert_eq!(
            format!("{:?}", matcher.producer),
            format!("{:?}", PacketProducer::new(0xabab))
        );
    }

    #[test]
    fn missing_close_brace_test() {
        assert_eq!(
            fire_statement(
                "fire {
                    PacketExtractor();
                    PacketProducer(0xabab~u16);"
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
            fire_statement(
                "fire {
                    PacketExtractor();
                    PacketProducer(0xabababab~u32);
                }"
            )
            .unwrap_err(),
            Err::Error(ParserError::CastFromToNotAllowed(
                "u32".to_string(),
                "u16".to_string(),
            ))
        );
    }
}
