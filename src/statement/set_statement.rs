use nom::branch::alt;
use nom::character::complete::{multispace0, multispace1};
use nom::combinator::{cut, map, map_res};
use nom::sequence::{preceded, separated_pair, terminated, tuple};
use nom::IResult;
use std::collections::BTreeMap;
use std::convert::TryInto;

use ross_config::creator::Creator;
use ross_config::event_processor::EventProcessor;
use ross_config::extractor::{EventCodeExtractor, EventProducerAddressExtractor, NoneExtractor};
use ross_config::filter::{SetStateToConstFilter, ValueEqualToConstFilter};
use ross_config::matcher::Matcher;
use ross_config::Value;

use crate::error::ParserError;
use crate::keyword::{from_keyword, if_keyword, on_keyword, set_keyword, to_keyword};
use crate::literal::{literal_or_constant, state_variable, Literal};
use crate::statement::match_statement::match_statement;
use crate::symbol::semicolon;

pub fn set_statement<'a>(
    constants: &'a BTreeMap<&str, Literal>,
    state_variables: &'a BTreeMap<&str, u32>,
) -> impl FnMut(&str) -> IResult<&str, EventProcessor, ParserError<&str>> + 'a {
    move |text| {
        let if_match_parser = {
            let additional_matcher_parser = cut(preceded(multispace1, match_statement(constants)));
            let pair_parser = separated_pair(
                terminated(base_syntax_parser(constants, state_variables), multispace1),
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
            base_syntax_parser(constants, state_variables),
            preceded(multispace0, semicolon),
        );

        let (input, (matcher, creators)) = alt((if_match_parser, normal_syntax_parser))(text)?;

        Ok((input, EventProcessor { matcher, creators }))
    }
}

fn base_syntax_parser<'a>(
    constants: &'a BTreeMap<&str, Literal>,
    state_variables: &'a BTreeMap<&str, u32>,
) -> impl FnMut(&str) -> IResult<&str, (Matcher, Vec<Creator>), ParserError<&str>> + 'a {
    move |text| {
        let tuple_parser = tuple((
            state_variable(state_variables),
            multispace1,
            to_keyword,
            multispace1,
            literal_or_constant(constants),
            multispace1,
            on_keyword,
            multispace1,
            literal_or_constant(constants),
            multispace1,
            from_keyword,
            multispace1,
            literal_or_constant(constants),
        ));

        let content_parser = map_res::<_, _, _, _, ParserError<&str>, _, _>(
            tuple_parser,
            |(state_index, _, _, _, target_value, _, _, _, event_code, _, _, _, from_address)| {
                let target_value = target_value.try_into()?;
                let event_code = event_code.try_into()?;
                let from_address = from_address.try_into()?;

                let event_matcher = Matcher::Single {
                    extractor: Box::new(EventCodeExtractor::new()),
                    filter: Box::new(ValueEqualToConstFilter::new(Value::U16(event_code))),
                };

                let producer_matcher = Matcher::Single {
                    extractor: Box::new(EventProducerAddressExtractor::new()),
                    filter: Box::new(ValueEqualToConstFilter::new(Value::U16(from_address))),
                };

                let set_matcher = Matcher::Single {
                    extractor: Box::new(NoneExtractor::new()),
                    filter: Box::new(SetStateToConstFilter::new(state_index, target_value)),
                };

                let combined_matcher = Matcher::And(
                    Box::new(event_matcher),
                    Box::new(Matcher::And(
                        Box::new(producer_matcher),
                        Box::new(set_matcher),
                    )),
                );

                Ok((combined_matcher, vec![]))
            },
        );

        preceded(set_keyword, cut(preceded(multispace1, content_parser)))(text)
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
        let mut state_variables = BTreeMap::new();
        state_variables.insert("button_pressed", 0);

        let (input, event_processor) = set_statement(&constants, &state_variables)(
            "set button_pressed to true on 0xabab~u16 from 0x0123~u16;input",
        )
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

            assert_matches!(*matcher2, Matcher::And(matcher1, matcher2) => {
                assert_matches!(*matcher1, Matcher::Single {extractor, filter} => {
                    assert_eq!(
                        format!("{:?}", extractor),
                        format!("{:?}", EventProducerAddressExtractor::new()),
                    );
                    assert_eq!(
                        format!("{:?}", filter),
                        format!("{:?}", ValueEqualToConstFilter::new(Value::U16(0x0123))),
                    );
                });

                assert_matches!(*matcher2, Matcher::Single {extractor, filter} => {
                    assert_eq!(
                        format!("{:?}", extractor),
                        format!("{:?}", NoneExtractor::new()),
                    );
                    assert_eq!(
                        format!("{:?}", filter),
                        format!("{:?}", SetStateToConstFilter::new(0, Value::Bool(true))),
                    );
                });
            });
        });

        assert_eq!(event_processor.creators.len(), 0);
    }

    #[test]
    fn normal_syntax_missing_semicolon_test() {
        let constants = BTreeMap::new();
        let mut state_variables = BTreeMap::new();
        state_variables.insert("button_pressed", 0);

        assert_matches!(
            set_statement(&constants, &state_variables)(
                "set button_pressed to true on 0xabab~u16 from 0x0123~u16"
            ),
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
        let mut state_variables = BTreeMap::new();
        state_variables.insert("button_pressed", 0);

        assert_matches!(
            set_statement(&constants, &state_variables)(
                "set button_pressed to true on 0xabab~u16 0x0123~u16;"
            ),
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
        let mut state_variables = BTreeMap::new();
        state_variables.insert("button_pressed", 0);

        assert_matches!(
            set_statement(&constants, &state_variables)(
                "set button_pressed true on 0xabab~u16 from 0x0123~u16;"
            ),
            Err(NomErr::Failure(ParserError::Base {
                location: _,
                kind: ErrorKind::Expected(Expectation::Keyword("to")),
                child: None,
            }))
        );
    }

    #[test]
    fn normal_syntax_missing_on_keyword_test() {
        let constants = BTreeMap::new();
        let mut state_variables = BTreeMap::new();
        state_variables.insert("button_pressed", 0);

        assert_matches!(
            set_statement(&constants, &state_variables)(
                "set button_pressed to true 0xabab~u16 from 0x0123~u16;"
            ),
            Err(NomErr::Failure(ParserError::Base {
                location: _,
                kind: ErrorKind::Expected(Expectation::Keyword("on")),
                child: Some(_),
            }))
        );
    }

    #[test]
    fn normal_syntax_empty_test() {
        let constants = BTreeMap::new();
        let state_variables = BTreeMap::new();

        assert_matches!(
            set_statement(&constants, &state_variables)(""),
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
        let mut state_variables = BTreeMap::new();
        state_variables.insert("button_pressed", 0);

        let (input, event_processor) = set_statement(&constants, &state_variables)(
            "set button_pressed to true on 0xabab~u16 from 0x0123~u16 if match event 0xbaba~u16;input",
        )
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
                    format!("{:?}", ValueEqualToConstFilter::new(Value::U16(0xbaba))),
                );
            });

            assert_matches!(*matcher2, Matcher::And(matcher1, matcher2) => {
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

                assert_matches!(*matcher2, Matcher::And(matcher1, matcher2) => {
                    assert_matches!(*matcher1, Matcher::Single {extractor, filter} => {
                        assert_eq!(
                            format!("{:?}", extractor),
                            format!("{:?}", EventProducerAddressExtractor::new()),
                        );
                        assert_eq!(
                            format!("{:?}", filter),
                            format!("{:?}", ValueEqualToConstFilter::new(Value::U16(0x0123))),
                        );
                    });

                    assert_matches!(*matcher2, Matcher::Single {extractor, filter} => {
                        assert_eq!(
                            format!("{:?}", extractor),
                            format!("{:?}", NoneExtractor::new()),
                        );
                        assert_eq!(
                            format!("{:?}", filter),
                            format!("{:?}", SetStateToConstFilter::new(0, Value::Bool(true))),
                        );
                    });
                });
            });
        });

        assert_eq!(event_processor.creators.len(), 0);
    }

    #[test]
    fn if_match_event_missing_semicolon_test() {
        let constants = BTreeMap::new();
        let mut state_variables = BTreeMap::new();
        state_variables.insert("button_pressed", 0);

        assert_matches!(
            set_statement(&constants, &state_variables)(
                "set button_pressed to true on 0xabab~u16 from 0x0123~u16 if match event 0xbaba~u16",
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
        let mut state_variables = BTreeMap::new();
        state_variables.insert("button_pressed", 0);

        assert_matches!(
            set_statement(&constants, &state_variables)(
                "set button_pressed to true on 0xabab~u16 0x0123~u16 if match event 0xbaba~u16;"
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
        let mut state_variables = BTreeMap::new();
        state_variables.insert("button_pressed", 0);

        assert_matches!(
            set_statement(&constants, &state_variables)(
                "set button_pressed true on 0xabab~u16 from 0x0123~u16 if match event 0xbaba~u16;",
            ),
            Err(NomErr::Failure(ParserError::Base {
                location: _,
                kind: ErrorKind::Expected(Expectation::Keyword("to")),
                child: None,
            }))
        );
    }

    #[test]
    fn if_match_event_missing_on_keyword_test() {
        let constants = BTreeMap::new();
        let mut state_variables = BTreeMap::new();
        state_variables.insert("button_pressed", 0);

        assert_matches!(
            set_statement(&constants, &state_variables)(
                "set button_pressed to true  0xabab~u16 from 0x0123~u16 if match event 0xbaba~u16;",
            ),
            Err(NomErr::Failure(ParserError::Base {
                location: _,
                kind: ErrorKind::Expected(Expectation::Keyword("on")),
                child: Some(_),
            }))
        );
    }
}
