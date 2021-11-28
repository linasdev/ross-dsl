use nom::sequence::{pair, terminated};
use nom::{Err, IResult};
use std::convert::TryInto;

use ross_config::filter::*;

use crate::parser::{alpha1, argument0, ParserError};
use crate::symbol::semicolon;

macro_rules! impl_filter_arg1 {
    ($input:expr, $name:expr, $arguments:expr, $filter_type:ty) => {
        if $name == stringify!($filter_type) {
            return if $arguments.len() == 1 {
                Ok((
                    $input,
                    Box::new(<$filter_type>::new(
                        $arguments[0]
                            .clone()
                            .try_into()
                            .map_err(|err| Err::Error(err))?,
                    )),
                ))
            } else {
                Err(Err::Error(ParserError::ExpectedArgumentsButGot(
                    $input.to_string(),
                    1,
                    $arguments.len(),
                )))
            };
        }
    };
}

macro_rules! impl_filter_arg2 {
    ($input:expr, $name:expr, $arguments:expr, $filter_type:ty) => {
        if $name == stringify!($filter_type) {
            return if $arguments.len() == 2 {
                Ok((
                    $input,
                    Box::new(<$filter_type>::new(
                        $arguments[0]
                            .clone()
                            .try_into()
                            .map_err(|err| Err::Error(err))?,
                        $arguments[1]
                            .clone()
                            .try_into()
                            .map_err(|err| Err::Error(err))?,
                    )),
                ))
            } else {
                Err(Err::Error(ParserError::ExpectedArgumentsButGot(
                    $input.to_string(),
                    2,
                    $arguments.len(),
                )))
            };
        }
    };
}

pub fn filter(text: &str) -> IResult<&str, Box<dyn Filter>, ParserError> {
    let (input, (name, arguments)) = terminated(pair(alpha1, argument0), semicolon)(text)?;

    impl_filter_arg1!(input, name, arguments, ValueEqualToConstFilter);
    impl_filter_arg2!(input, name, arguments, StateEqualToConstFilter);
    impl_filter_arg1!(input, name, arguments, StateEqualToValueFilter);
    impl_filter_arg2!(input, name, arguments, IncrementStateByConstFilter);
    impl_filter_arg1!(input, name, arguments, IncrementStateByValueFilter);
    impl_filter_arg2!(input, name, arguments, DecrementStateByConstFilter);
    impl_filter_arg1!(input, name, arguments, DecrementStateByValueFilter);
    impl_filter_arg2!(input, name, arguments, SetStateToConstFilter);
    impl_filter_arg1!(input, name, arguments, SetStateToValueFilter);
    impl_filter_arg1!(input, name, arguments, FlipStateFilter);

    Err(Err::Error(ParserError::UnknownFilter(
        text.to_string(),
        name.to_string(),
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    use ross_config::Value;

    macro_rules! impl_tests_extractor_arg1 {
        ($test_module_name:ident, $filter_type:ty, ($argument0:expr, $argument0_value:expr)) => {
            mod $test_module_name {
                use super::*;

                #[test]
                fn extractor_test() {
                    let (input, extractor) = filter(concat!(
                        stringify!($filter_type),
                        "( ",
                        $argument0,
                        " );input"
                    ))
                    .unwrap();

                    assert_eq!(input, "input");
                    assert_eq!(
                        format!("{:?}", extractor),
                        format!("{:?}", <$filter_type>::new($argument0_value))
                    );
                }

                #[test]
                fn missing_semicolon_test() {
                    assert_eq!(
                        filter(concat!(
                            stringify!($filter_type),
                            "( ",
                            $argument0,
                            " )input"
                        ))
                        .unwrap_err(),
                        Err::Error(ParserError::ExpectedSymbolFound(
                            "input".to_string(),
                            ";".to_string(),
                            "input".to_string()
                        ))
                    );
                }

                #[test]

                fn too_few_arguments_test() {
                    assert_eq!(
                        filter(concat!(stringify!($filter_type), "( );input")).unwrap_err(),
                        Err::Error(ParserError::ExpectedArgumentsButGot(
                            "input".to_string(),
                            1,
                            0,
                        ))
                    );
                }

                #[test]

                fn too_many_arguments_test() {
                    assert_eq!(
                        filter(concat!(stringify!($filter_type), "( false, false );input"))
                            .unwrap_err(),
                        Err::Error(ParserError::ExpectedArgumentsButGot(
                            "input".to_string(),
                            1,
                            2,
                        ))
                    );
                }
            }
        };
    }

    macro_rules! impl_tests_extractor_arg2 {
        ($test_module_name:ident, $filter_type:ty, ($argument0:expr, $argument0_value:expr), ($argument1:expr, $argument1_value:expr)) => {
            mod $test_module_name {
                use super::*;

                #[test]
                fn extractor_test() {
                    let (input, extractor) = filter(concat!(
                        stringify!($filter_type),
                        "( ",
                        $argument0,
                        " , ",
                        $argument1,
                        " );input"
                    ))
                    .unwrap();

                    assert_eq!(input, "input");
                    assert_eq!(
                        format!("{:?}", extractor),
                        format!(
                            "{:?}",
                            <$filter_type>::new($argument0_value, $argument1_value)
                        )
                    );
                }

                #[test]
                fn missing_semicolon_test() {
                    assert_eq!(
                        filter(concat!(
                            stringify!($filter_type),
                            "( ",
                            $argument0,
                            " , ",
                            $argument1,
                            " )input"
                        ))
                        .unwrap_err(),
                        Err::Error(ParserError::ExpectedSymbolFound(
                            "input".to_string(),
                            ";".to_string(),
                            "input".to_string()
                        ))
                    );
                }

                #[test]

                fn too_few_arguments_test() {
                    assert_eq!(
                        filter(concat!(stringify!($filter_type), "( false );input"))
                            .unwrap_err(),
                        Err::Error(ParserError::ExpectedArgumentsButGot(
                            "input".to_string(),
                            2,
                            1,
                        ))
                    );
                }

                #[test]

                fn too_many_arguments_test() {
                    assert_eq!(
                        filter(concat!(
                            stringify!($filter_type),
                            "( false, false, false );input"
                        ))
                        .unwrap_err(),
                        Err::Error(ParserError::ExpectedArgumentsButGot(
                            "input".to_string(),
                            2,
                            3,
                        ))
                    );
                }
            }
        };
    }

    impl_tests_extractor_arg1!(
        value_equal_to_const_filter,
        ValueEqualToConstFilter,
        ("0xabababab~u32", Value::U32(0xabab_abab))
    );
    impl_tests_extractor_arg2!(
        state_equal_to_const_filter,
        StateEqualToConstFilter,
        ("0xabababab~u32", 0xabab_abab),
        ("0xbabababa~u32", Value::U32(0xbaba_baba))
    );
    impl_tests_extractor_arg1!(
        state_equal_to_value_filter,
        StateEqualToValueFilter,
        ("0xabababab~u32", 0xabab_abab)
    );
    impl_tests_extractor_arg2!(
        increment_state_by_const_filter,
        IncrementStateByConstFilter,
        ("0xabababab~u32", 0xabab_abab),
        ("0xbabababa~u32", Value::U32(0xbaba_baba))
    );
    impl_tests_extractor_arg1!(
        increment_state_by_value_filter,
        IncrementStateByValueFilter,
        ("0xabababab~u32", 0xabab_abab)
    );
    impl_tests_extractor_arg2!(
        decrement_state_by_const_filter,
        DecrementStateByConstFilter,
        ("0xabababab~u32", 0xabab_abab),
        ("0xbabababa~u32", Value::U32(0xbaba_baba))
    );
    impl_tests_extractor_arg1!(
        decrement_state_by_value_filter,
        DecrementStateByValueFilter,
        ("0xabababab~u32", 0xabab_abab)
    );
    impl_tests_extractor_arg2!(
        set_state_to_const_filter,
        SetStateToConstFilter,
        ("0xabababab~u32", 0xabab_abab),
        ("0xbabababa~u32", Value::U32(0xbaba_baba))
    );
    impl_tests_extractor_arg1!(
        set_state_to_value_filter,
        SetStateToValueFilter,
        ("0xabababab~u32", 0xabab_abab)
    );
    impl_tests_extractor_arg1!(
        flip_state_filter,
        FlipStateFilter,
        ("0xabababab~u32", 0xabab_abab)
    );
}
