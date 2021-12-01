use nom::branch::alt;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{cut, map};
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated};
use nom::IResult;
use std::collections::BTreeMap;

use ross_config::creator::Creator;
use ross_config::extractor::{Extractor, NoneExtractor};
use ross_config::producer::Producer;

use crate::error::ParserError;
use crate::extractor::extractor;
use crate::keyword::{fire_keyword, if_keyword};
use crate::literal::Literal;
use crate::producer::producer;
use crate::statement::match_statement::match_statement;
use crate::symbol::{close_brace, open_brace};

pub fn fire_statement<'a>(
    constants: &'a BTreeMap<&str, Literal>,
) -> impl FnMut(&str) -> IResult<&str, Creator, ParserError<&str>> + 'a {
    move |text| {
        let if_match_parser = {
            let additional_matcher_parser = cut(preceded(multispace1, match_statement(constants)));
            let pair_parser = separated_pair(
                terminated(base_syntax_parser(constants), multispace0),
                if_keyword,
                additional_matcher_parser,
            );

            map(pair_parser, |((extractor, producer), matcher)| Creator {
                extractor,
                producer,
                matcher: Some(matcher),
            })
        };

        let normal_syntax_parser = {
            map(base_syntax_parser(constants), |(extractor, producer)| {
                Creator {
                    extractor,
                    producer,
                    matcher: None,
                }
            })
        };

        alt((if_match_parser, normal_syntax_parser))(text)
    }
}

fn base_syntax_parser<'a>(
    constants: &'a BTreeMap<&str, Literal>,
) -> impl FnMut(&str) -> IResult<&str, (Box<dyn Extractor>, Box<dyn Producer>), ParserError<&str>> + 'a
{
    move |text| {
        let extractor_parser = alt((
            delimited(multispace0, extractor(constants), multispace0),
            |input| Ok((input, Box::new(NoneExtractor::new()) as Box<dyn Extractor>)),
        ));

        let producer_parser = delimited(multispace0, producer(constants), multispace0);
        let content_parser = preceded(open_brace, pair(extractor_parser, producer_parser));
        let keyword_parser = preceded(fire_keyword, cut(preceded(multispace1, content_parser)));
        let mut close_brace_parser = terminated(keyword_parser, preceded(multispace0, close_brace));
        close_brace_parser(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use cool_asserts::assert_matches;
    use nom::Err as NomErr;
    use nom::error::ErrorKind as NomErrorKind;

    use ross_config::extractor::{EventCodeExtractor, PacketExtractor};
    use ross_config::filter::ValueEqualToConstFilter;
    use ross_config::producer::PacketProducer;
    use ross_config::Value;

    use crate::error::ErrorKind;

    #[test]
    fn provided_extractor_test() {
        let constants = BTreeMap::new();
        let (input, creator) = fire_statement(&constants)(
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
        assert_matches!(creator.matcher, None,);
    }

    #[test]
    fn missing_close_brace_test() {
        let constants = BTreeMap::new();
        assert_matches!(
            fire_statement(&constants)(
                "fire {
                    PacketExtractor();
                    PacketProducer(0xabab~u16);",
            ),
            Err(NomErr::Error(ParserError::Base {
                location: _,
                kind: ErrorKind::Nom(NomErrorKind::Alt),
                child: Some(_),
            }))
        );
    }

    #[test]
    fn invalid_literal_test() {
        let constants = BTreeMap::new();
        assert_matches!(
            fire_statement(&constants)(
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

    #[test]
    fn if_match_test() {
        let constants = BTreeMap::new();
        let (input, creator) = fire_statement(&constants)(
            "fire {
                PacketExtractor();
                PacketProducer(0xabab~u16);
            } if match {
                EventCodeExtractor();
                ValueEqualToConstFilter(0x0123~u16);
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
        assert_matches!(
            creator.matcher,
            Some(matcher) => {
                assert_eq!(
                    format!("{:?}", matcher.extractor),
                    format!("{:?}", EventCodeExtractor::new())
                );
                assert_eq!(
                    format!("{:?}", matcher.filter),
                    format!("{:?}", ValueEqualToConstFilter::new(Value::U16(0x0123)))
                );
            },
        );
    }
}
