use nom::branch::alt;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{cut, map, map_res};
use nom::sequence::{delimited, pair, preceded, separated_pair, terminated};
use nom::IResult;
use std::collections::BTreeMap;
use std::convert::TryInto;

use ross_config::extractor::Extractor;
use ross_config::extractor::{EventCodeExtractor, EventProducerAddressExtractor, NoneExtractor};
use ross_config::filter::Filter;
use ross_config::filter::ValueEqualToConstFilter;
use ross_config::matcher::Matcher;
use ross_config::Value;
use ross_protocol::event::event_code::INTERNAL_SYSTEM_TICK_EVENT_CODE;

use crate::error::ParserError;
use crate::extractor::extractor;
use crate::filter::filter;
use crate::keyword::{
    and_keyword, event_keyword, match_keyword, not_keyword, or_keyword, producer_keyword,
    tick_keyword,
};
use crate::literal::{literal_or_constant, Literal};
use crate::symbol::{close_brace, comma, open_brace, semicolon};

pub fn match_statement<'a>(
    constants: &'a BTreeMap<&str, Literal>,
) -> impl FnMut(&str) -> IResult<&str, Matcher, ParserError<&str>> + 'a {
    move |text| {
        let event_match_parser = {
            let content_parser = map_res::<_, _, _, _, ParserError<&str>, _, _>(
                literal_or_constant(constants),
                |event_code| {
                    let event_code = event_code.try_into()?;

                    let extractor = Box::new(EventCodeExtractor::new()) as Box<dyn Extractor>;
                    let filter = Box::new(ValueEqualToConstFilter::new(Value::U16(event_code)))
                        as Box<dyn Filter>;

                    Ok(Matcher::Single { extractor, filter })
                },
            );

            let event_keyword_parser =
                preceded(event_keyword, cut(preceded(multispace1, content_parser)));

            let match_keyword_parser =
                preceded(match_keyword, preceded(multispace1, event_keyword_parser));

            terminated(match_keyword_parser, semicolon)
        };

        let producer_match_parser = {
            let content_parser = map_res::<_, _, _, _, ParserError<&str>, _, _>(
                literal_or_constant(constants),
                |producer_address| {
                    let producer_address = producer_address.try_into()?;

                    let extractor =
                        Box::new(EventProducerAddressExtractor::new()) as Box<dyn Extractor>;
                    let filter =
                        Box::new(ValueEqualToConstFilter::new(Value::U16(producer_address)))
                            as Box<dyn Filter>;

                    Ok(Matcher::Single { extractor, filter })
                },
            );

            let producer_keyword_parser =
                preceded(producer_keyword, cut(preceded(multispace1, content_parser)));

            let match_keyword_parser = preceded(
                match_keyword,
                preceded(multispace1, producer_keyword_parser),
            );

            terminated(match_keyword_parser, semicolon)
        };

        let tick_match_parser = {
            let content_parser = |text| {
                let extractor = Box::new(EventCodeExtractor::new()) as Box<dyn Extractor>;
                let filter = Box::new(ValueEqualToConstFilter::new(Value::U16(
                    INTERNAL_SYSTEM_TICK_EVENT_CODE,
                ))) as Box<dyn Filter>;

                Ok((text, Matcher::Single { extractor, filter }))
            };

            let tick_keyword_parser = preceded(tick_keyword, content_parser);

            let match_keyword_parser =
                preceded(match_keyword, preceded(multispace1, tick_keyword_parser));

            terminated(match_keyword_parser, semicolon)
        };

        let normal_syntax_parser = preceded(
            match_keyword,
            preceded(multispace1, block_match_parser(constants)),
        );

        alt((
            event_match_parser,
            producer_match_parser,
            tick_match_parser,
            normal_syntax_parser,
        ))(text)
    }
}

fn block_match_parser<'a>(
    constants: &'a BTreeMap<&str, Literal>,
) -> impl FnMut(&str) -> IResult<&str, Matcher, ParserError<&str>> + 'a {
    move |text| {
        let not_parser = {
            let extractor_parser = alt((
                delimited(multispace0, extractor(constants), multispace0),
                |input| Ok((input, Box::new(NoneExtractor::new()) as Box<dyn Extractor>)),
            ));
            let filter_parser = delimited(multispace0, filter(constants), multispace0);
            let content_parser = preceded(open_brace, cut(pair(extractor_parser, filter_parser)));
            let map_parser = map(content_parser, |(extractor, filter)| {
                Matcher::Not(Box::new(Matcher::Single { extractor, filter }))
            });
            let keyword_parser = preceded(not_keyword, cut(preceded(multispace0, map_parser)));
            terminated(keyword_parser, preceded(multispace0, close_brace))
        };

        let or_parser = {
            let content_parser = preceded(
                open_brace,
                separated_pair(
                    preceded(multispace0, block_match_parser(constants)),
                    preceded(multispace0, comma),
                    preceded(multispace0, block_match_parser(constants)),
                ),
            );
            let map_parser = map(content_parser, |(matcher1, matcher2)| {
                Matcher::Or(Box::new(matcher1), Box::new(matcher2))
            });
            let keyword_parser = preceded(or_keyword, cut(preceded(multispace0, map_parser)));
            terminated(keyword_parser, preceded(multispace0, close_brace))
        };

        let and_parser = {
            let content_parser = preceded(
                open_brace,
                separated_pair(
                    preceded(multispace0, block_match_parser(constants)),
                    preceded(multispace0, comma),
                    preceded(multispace0, block_match_parser(constants)),
                ),
            );
            let map_parser = map(content_parser, |(matcher1, matcher2)| {
                Matcher::And(Box::new(matcher1), Box::new(matcher2))
            });
            let keyword_parser = preceded(and_keyword, cut(preceded(multispace0, map_parser)));
            terminated(keyword_parser, preceded(multispace0, close_brace))
        };

        let base_syntax_parser = {
            let extractor_parser = alt((
                delimited(multispace0, extractor(constants), multispace0),
                |input| Ok((input, Box::new(NoneExtractor::new()) as Box<dyn Extractor>)),
            ));
            let filter_parser = delimited(multispace0, filter(constants), multispace0);
            let content_parser = preceded(open_brace, cut(pair(extractor_parser, filter_parser)));
            let map_parser = map(content_parser, |(extractor, filter)| Matcher::Single {
                extractor,
                filter,
            });
            terminated(map_parser, preceded(multispace0, close_brace))
        };

        alt((not_parser, or_parser, and_parser, base_syntax_parser))(text)
    }
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
        let constants = BTreeMap::new();
        let (input, matcher) = match_statement(&constants)(
            "match {
                EventCodeExtractor();
                ValueEqualToConstFilter(0xabab~u16);
            }input",
        )
        .unwrap();

        assert_eq!(input, "input");
        assert_matches!(matcher, Matcher::Single {extractor, filter} => {
            assert_eq!(
                format!("{:?}", extractor),
                format!("{:?}", EventCodeExtractor::new())
            );
            assert_eq!(
                format!("{:?}", filter),
                format!("{:?}", ValueEqualToConstFilter::new(Value::U16(0xabab)))
            );
        });
    }

    #[test]
    fn block_no_extractor_test() {
        let constants = BTreeMap::new();
        let (input, matcher) = match_statement(&constants)(
            "match {
                ValueEqualToConstFilter(0xabab~u16);
            }input",
        )
        .unwrap();

        assert_eq!(input, "input");
        assert_matches!(matcher, Matcher::Single {extractor, filter} => {
            assert_eq!(
                format!("{:?}", extractor),
                format!("{:?}", NoneExtractor::new())
            );
            assert_eq!(
                format!("{:?}", filter),
                format!("{:?}", ValueEqualToConstFilter::new(Value::U16(0xabab)))
            );
        });
    }

    #[test]
    fn block_two_extractors_test() {
        let constants = BTreeMap::new();
        assert_matches!(
            match_statement(&constants)(
                "match {
                    EventCodeExtractor();
                    NoneExtractor();
                }input",
            ),
            Err(NomErr::Failure(ParserError::Base {
                location: _,
                kind: ErrorKind::UnknownFilter,
                child: None,
            }))
        );
    }

    #[test]
    fn block_not_test() {
        let constants = BTreeMap::new();
        let (input, matcher) = match_statement(&constants)(
            "match not {
                EventCodeExtractor();
                ValueEqualToConstFilter(0xabab~u16);
            }input",
        )
        .unwrap();

        assert_eq!(input, "input");
        assert_matches!(matcher, Matcher::Not(matcher) => {
            assert_matches!(*matcher, Matcher::Single {extractor, filter} => {
                assert_eq!(
                    format!("{:?}", extractor),
                    format!("{:?}", EventCodeExtractor::new())
                );
                assert_eq!(
                    format!("{:?}", filter),
                    format!("{:?}", ValueEqualToConstFilter::new(Value::U16(0xabab)))
                );
            });
        });
    }

    #[test]
    fn block_or_test() {
        let constants = BTreeMap::new();
        let (input, matcher) = match_statement(&constants)(
            "match or {
                {
                    EventCodeExtractor();
                    ValueEqualToConstFilter(0xabab~u16);
                },
                {
                    EventCodeExtractor();
                    ValueEqualToConstFilter(0x0123~u16);
                }
            }input",
        )
        .unwrap();

        assert_eq!(input, "input");
        assert_matches!(matcher, Matcher::Or(matcher1, matcher2) => {
            assert_matches!(*matcher1, Matcher::Single {extractor, filter} => {
                assert_eq!(
                    format!("{:?}", extractor),
                    format!("{:?}", EventCodeExtractor::new())
                );
                assert_eq!(
                    format!("{:?}", filter),
                    format!("{:?}", ValueEqualToConstFilter::new(Value::U16(0xabab)))
                );
            });

            assert_matches!(*matcher2, Matcher::Single {extractor, filter} => {
                assert_eq!(
                    format!("{:?}", extractor),
                    format!("{:?}", EventCodeExtractor::new())
                );
                assert_eq!(
                    format!("{:?}", filter),
                    format!("{:?}", ValueEqualToConstFilter::new(Value::U16(0x0123)))
                );
            });
        });
    }

    #[test]
    fn block_and_test() {
        let constants = BTreeMap::new();
        let (input, matcher) = match_statement(&constants)(
            "match and {
                {
                    EventCodeExtractor();
                    ValueEqualToConstFilter(0xabab~u16);
                },
                {
                    EventCodeExtractor();
                    ValueEqualToConstFilter(0x0123~u16);
                }
            }input",
        )
        .unwrap();

        assert_eq!(input, "input");
        assert_matches!(matcher, Matcher::And(matcher1, matcher2) => {
            assert_matches!(*matcher1, Matcher::Single {extractor, filter} => {
                assert_eq!(
                    format!("{:?}", extractor),
                    format!("{:?}", EventCodeExtractor::new())
                );
                assert_eq!(
                    format!("{:?}", filter),
                    format!("{:?}", ValueEqualToConstFilter::new(Value::U16(0xabab)))
                );
            });

            assert_matches!(*matcher2, Matcher::Single {extractor, filter} => {
                assert_eq!(
                    format!("{:?}", extractor),
                    format!("{:?}", EventCodeExtractor::new())
                );
                assert_eq!(
                    format!("{:?}", filter),
                    format!("{:?}", ValueEqualToConstFilter::new(Value::U16(0x0123)))
                );
            });
        });
    }

    #[test]
    fn block_nested_logic_test() {
        let constants = BTreeMap::new();
        let (input, matcher) = match_statement(&constants)(
            "match and {
                or {
                    {
                        EventCodeExtractor();
                        ValueEqualToConstFilter(0xabab~u16);
                    },
                    {
                        EventCodeExtractor();
                        ValueEqualToConstFilter(0x0123~u16);
                    }
                },
                {
                    EventProducerAddressExtractor();
                    ValueEqualToConstFilter(0xffff~u16);
                }
            }input",
        )
        .unwrap();

        assert_eq!(input, "input");
        assert_matches!(matcher, Matcher::And(matcher1, matcher2) => {
            assert_matches!(*matcher1, Matcher::Or(matcher1, matcher2) => {
                assert_matches!(*matcher1, Matcher::Single {extractor, filter} => {
                    assert_eq!(
                        format!("{:?}", extractor),
                        format!("{:?}", EventCodeExtractor::new())
                    );
                    assert_eq!(
                        format!("{:?}", filter),
                        format!("{:?}", ValueEqualToConstFilter::new(Value::U16(0xabab)))
                    );
                });

                assert_matches!(*matcher2, Matcher::Single {extractor, filter} => {
                    assert_eq!(
                        format!("{:?}", extractor),
                        format!("{:?}", EventCodeExtractor::new())
                    );
                    assert_eq!(
                        format!("{:?}", filter),
                        format!("{:?}", ValueEqualToConstFilter::new(Value::U16(0x0123)))
                    );
                });
            });

            assert_matches!(*matcher2, Matcher::Single {extractor, filter} => {
                assert_eq!(
                    format!("{:?}", extractor),
                    format!("{:?}", EventProducerAddressExtractor::new())
                );
                assert_eq!(
                    format!("{:?}", filter),
                    format!("{:?}", ValueEqualToConstFilter::new(Value::U16(0xffff)))
                );
            });
        });
    }

    #[test]
    fn event_test() {
        let constants = BTreeMap::new();
        let (input, matcher) = match_statement(&constants)("match event 0xabab~u16;input").unwrap();

        assert_eq!(input, "input");
        assert_matches!(matcher, Matcher::Single {extractor, filter} => {
            assert_eq!(
                format!("{:?}", extractor),
                format!("{:?}", EventCodeExtractor::new())
            );
            assert_eq!(
                format!("{:?}", filter),
                format!("{:?}", ValueEqualToConstFilter::new(Value::U16(0xabab)))
            );
        });
    }

    #[test]
    fn event_invalid_literal_test() {
        let constants = BTreeMap::new();
        assert_matches!(
            match_statement(&constants)("match event 0xabababab~u32;input"),
            Err(NomErr::Failure(ParserError::Base {
                location: _,
                kind: ErrorKind::CastFromToNotAllowed("u32", "u16"),
                child: None,
            }))
        );
    }

    #[test]
    fn event_missing_semicolon_test() {
        let constants = BTreeMap::new();
        assert_matches!(
            match_statement(&constants)("match event 0xabab~u16"),
            Err(NomErr::Error(ParserError::Base {
                location: _,
                kind: ErrorKind::Nom(NomErrorKind::Alt),
                child: Some(_),
            }))
        );
    }

    #[test]
    fn producer_test() {
        let constants = BTreeMap::new();
        let (input, matcher) =
            match_statement(&constants)("match producer 0xabab~u16;input").unwrap();

        assert_eq!(input, "input");

        assert_eq!(input, "input");
        assert_matches!(matcher, Matcher::Single {extractor, filter} => {
            assert_eq!(
                format!("{:?}", extractor),
                format!("{:?}", EventProducerAddressExtractor::new())
            );
            assert_eq!(
                format!("{:?}", filter),
                format!("{:?}", ValueEqualToConstFilter::new(Value::U16(0xabab)))
            );
        });
    }

    #[test]
    fn producer_invalid_literal_test() {
        let constants = BTreeMap::new();
        assert_matches!(
            match_statement(&constants)("match producer 0xabababab~u32;input"),
            Err(NomErr::Failure(ParserError::Base {
                location: _,
                kind: ErrorKind::CastFromToNotAllowed("u32", "u16"),
                child: None,
            }))
        );
    }

    #[test]
    fn producer_missing_semicolon_test() {
        let constants = BTreeMap::new();
        assert_matches!(
            match_statement(&constants)("match producer 0xabab~u16"),
            Err(NomErr::Error(ParserError::Base {
                location: _,
                kind: ErrorKind::Nom(NomErrorKind::Alt),
                child: Some(_),
            }))
        );
    }

    #[test]
    fn tick_test() {
        let constants = BTreeMap::new();
        let (input, matcher) = match_statement(&constants)("match tick;input").unwrap();

        assert_eq!(input, "input");

        assert_eq!(input, "input");
        assert_matches!(matcher, Matcher::Single {extractor, filter} => {
            assert_eq!(
                format!("{:?}", extractor),
                format!("{:?}", EventCodeExtractor::new())
            );
            assert_eq!(
                format!("{:?}", filter),
                format!("{:?}", ValueEqualToConstFilter::new(Value::U16(INTERNAL_SYSTEM_TICK_EVENT_CODE)))
            );
        });
    }

    #[test]
    fn tick_missing_semicolon_test() {
        let constants = BTreeMap::new();
        assert_matches!(
            match_statement(&constants)("match tick"),
            Err(NomErr::Error(ParserError::Base {
                location: _,
                kind: ErrorKind::Nom(NomErrorKind::Alt),
                child: Some(_),
            }))
        );
    }
}
