use nom::sequence::{pair, terminated};
use nom::{Err, IResult};
use std::collections::BTreeMap;
use std::convert::TryInto;

use ross_config::filter::*;

use crate::error::{ErrorKind, ParserError};
use crate::literal::Literal;
use crate::parser::{argument_or_constant0, name_parser};
use crate::symbol::semicolon;
use crate::{impl_item_arg1, impl_item_arg2};

pub fn filter<'a>(
    constants: &'a BTreeMap<&str, Literal>,
) -> impl FnMut(&str) -> IResult<&str, Box<dyn Filter>, ParserError<&str>> + 'a {
    move |text| {
        let (input, (name, arguments)) =
            terminated(pair(name_parser, argument_or_constant0(constants)), semicolon)(text)?;

        impl_item_arg1!(input, name, arguments, ValueEqualToConstFilter);
        impl_item_arg2!(input, name, arguments, StateEqualToConstFilter);
        impl_item_arg1!(input, name, arguments, StateEqualToValueFilter);
        impl_item_arg2!(input, name, arguments, IncrementStateByConstFilter);
        impl_item_arg1!(input, name, arguments, IncrementStateByValueFilter);
        impl_item_arg2!(input, name, arguments, DecrementStateByConstFilter);
        impl_item_arg1!(input, name, arguments, DecrementStateByValueFilter);
        impl_item_arg2!(input, name, arguments, SetStateToConstFilter);
        impl_item_arg1!(input, name, arguments, SetStateToValueFilter);
        impl_item_arg1!(input, name, arguments, FlipStateFilter);
        impl_item_arg1!(input, name, arguments, TimeMatchesCronExpressionFilter);
        impl_item_arg2!(input, name, arguments, StateMoreThanConstFilter);

        Err(Err::Error(ParserError::Base {
            location: name,
            kind: ErrorKind::UnknownFilter,
            child: None,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ross_config::cron::{CronExpression, CronField};
    use ross_config::Value;

    use crate::{impl_tests_for_item_arg1, impl_tests_for_item_arg2};

    impl_tests_for_item_arg1!(
        value_equal_to_const_filter,
        filter,
        ValueEqualToConstFilter,
        ("0xabababab~u32", Value::U32(0xabab_abab))
    );
    impl_tests_for_item_arg2!(
        state_equal_to_const_filter,
        filter,
        StateEqualToConstFilter,
        ("0xabababab~u32", 0xabab_abab),
        ("0xbabababa~u32", Value::U32(0xbaba_baba))
    );
    impl_tests_for_item_arg1!(
        state_equal_to_value_filter,
        filter,
        StateEqualToValueFilter,
        ("0xabababab~u32", 0xabab_abab)
    );
    impl_tests_for_item_arg2!(
        increment_state_by_const_filter,
        filter,
        IncrementStateByConstFilter,
        ("0xabababab~u32", 0xabab_abab),
        ("0xbabababa~u32", Value::U32(0xbaba_baba))
    );
    impl_tests_for_item_arg1!(
        increment_state_by_value_filter,
        filter,
        IncrementStateByValueFilter,
        ("0xabababab~u32", 0xabab_abab)
    );
    impl_tests_for_item_arg2!(
        decrement_state_by_const_filter,
        filter,
        DecrementStateByConstFilter,
        ("0xabababab~u32", 0xabab_abab),
        ("0xbabababa~u32", Value::U32(0xbaba_baba))
    );
    impl_tests_for_item_arg1!(
        decrement_state_by_value_filter,
        filter,
        DecrementStateByValueFilter,
        ("0xabababab~u32", 0xabab_abab)
    );
    impl_tests_for_item_arg2!(
        set_state_to_const_filter,
        filter,
        SetStateToConstFilter,
        ("0xabababab~u32", 0xabab_abab),
        ("0xbabababa~u32", Value::U32(0xbaba_baba))
    );
    impl_tests_for_item_arg1!(
        set_state_to_value_filter,
        filter,
        SetStateToValueFilter,
        ("0xabababab~u32", 0xabab_abab)
    );
    impl_tests_for_item_arg1!(
        flip_state_filter,
        filter,
        FlipStateFilter,
        ("0xabababab~u32", 0xabab_abab)
    );
    impl_tests_for_item_arg1!(
        time_matches_cron_expression_filter,
        filter,
        TimeMatchesCronExpressionFilter,
        (
            "\"1 1 1 1 1 1 *\"",
            CronExpression {
                second: CronField::Including([1].iter().map(|n| *n).collect()),
                minute: CronField::Including([1].iter().map(|n| *n).collect()),
                hour: CronField::Including([1].iter().map(|n| *n).collect()),
                day_month: CronField::Including([1].iter().map(|n| *n).collect()),
                month: CronField::Including([1].iter().map(|n| *n).collect()),
                day_week: CronField::Including([1].iter().map(|n| *n).collect()),
                year: CronField::Any,
            }
        )
    );
    impl_tests_for_item_arg2!(
        state_more_than_const_filter,
        filter,
        StateMoreThanConstFilter,
        ("0xabababab~u32", 0xabab_abab),
        ("0xbabababa~u32", Value::U32(0xbaba_baba))
    );
}
