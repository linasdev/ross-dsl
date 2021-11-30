use nom::branch::alt;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::cut;
use nom::sequence::{delimited, pair, preceded, terminated};
use nom::IResult;

use ross_config::creator::Creator;
use ross_config::extractor::{Extractor, NoneExtractor};

use crate::error::ParserError;
use crate::extractor::extractor;
use crate::keyword::fire_keyword;
use crate::producer::producer;
use crate::symbol::{close_brace, open_brace};

pub fn fire_statement(text: &str) -> IResult<&str, Creator, ParserError<&str>> {
    let extractor_parser = alt((delimited(multispace0, extractor, multispace0), |input| {
        Ok((input, Box::new(NoneExtractor::new()) as Box<dyn Extractor>))
    }));

    let producer_parser = delimited(multispace0, producer, multispace0);
    let content_parser = preceded(open_brace, pair(extractor_parser, producer_parser));
    let keyword_parser = preceded(fire_keyword, cut(preceded(multispace1, content_parser)));
    let mut close_brace_parser = terminated(keyword_parser, preceded(multispace0, close_brace));

    let (input, (extractor, producer)) = close_brace_parser(text)?;

    Ok((
        input,
        Creator {
            extractor,
            producer,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    use cool_asserts::assert_matches;
    use nom::Err as NomErr;

    use ross_config::extractor::PacketExtractor;
    use ross_config::producer::PacketProducer;

    use crate::error::{ErrorKind, Expectation};

    #[test]
    fn provided_extractor_test() {
        let (input, creator) = fire_statement(
            "fire {
                PacketExtractor();
                PacketProducer(0xabab~u16);
            }input",
        )
        .unwrap();

        assert_eq!(input, "input");
        assert_eq!(
            format!("{:?}", creator.extractor),
            format!("{:?}", PacketExtractor::new())
        );
        assert_eq!(
            format!("{:?}", creator.producer),
            format!("{:?}", PacketProducer::new(0xabab))
        );
    }

    #[test]
    fn missing_close_brace_test() {
        assert_matches!(
            fire_statement(
                "fire {
                    PacketExtractor();
                    PacketProducer(0xabab~u16);",
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
        assert_matches!(
            fire_statement(
                "fire {
                    PacketExtractor();
                    PacketProducer(0xabababab~u32);
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
