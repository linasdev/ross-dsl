#[macro_export]
macro_rules! impl_item_arg0 {
    ($input:expr, $name:expr, $arguments:expr, $item_type:ty) => {
        if $name == stringify!($item_type) {
            return if $arguments.len() == 0 {
                Ok(($input, Box::new(<$item_type>::new())))
            } else {
                Err(Err::Error(ParserError::ExpectedArgumentsButGot(
                    $input.to_string(),
                    0,
                    $arguments.len(),
                )))
            };
        }
    };
}

#[macro_export]
macro_rules! impl_item_arg1 {
    ($input:expr, $name:expr, $arguments:expr, $item_type:ty) => {
        if $name == stringify!($item_type) {
            return if $arguments.len() == 1 {
                Ok((
                    $input,
                    Box::new(<$item_type>::new(
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

#[macro_export]
macro_rules! impl_item_arg2 {
    ($input:expr, $name:expr, $arguments:expr, $item_type:ty) => {
        if $name == stringify!($item_type) {
            return if $arguments.len() == 2 {
                Ok((
                    $input,
                    Box::new(<$item_type>::new(
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

#[macro_export]
macro_rules! impl_tests_for_item_arg0 {
    ($test_module_name:ident, $item:ident, $item_type:ty) => {
        mod $test_module_name {
            use super::*;

            #[test]
            fn test() {
                let (input, item) = $item(concat!(stringify!($item_type), "( );input")).unwrap();

                assert_eq!(input, "input");
                assert_eq!(format!("{:?}", item), format!("{:?}", <$item_type>::new()));
            }

            #[test]
            fn missing_semicolon_test() {
                assert_eq!(
                    $item(concat!(stringify!($item_type), "( )input")).unwrap_err(),
                    Err::Error(ParserError::ExpectedSymbolFound(
                        "input".to_string(),
                        ";".to_string(),
                        "input".to_string()
                    ))
                );
            }

            #[test]

            fn too_many_arguments_test() {
                assert_eq!(
                    $item(concat!(stringify!($item_type), "( false );input")).unwrap_err(),
                    Err::Error(ParserError::ExpectedArgumentsButGot(
                        "input".to_string(),
                        0,
                        1,
                    ))
                );
            }
        }
    };
}

#[macro_export]
macro_rules! impl_tests_for_item_arg1 {
    ($test_module_name:ident, $item:ident, $item_type:ty, ($argument0:expr, $argument0_value:expr)) => {
        mod $test_module_name {
            use super::*;

            #[test]
            fn test() {
                let (input, item) = $item(concat!(
                    stringify!($item_type),
                    "( ",
                    $argument0,
                    " );input"
                ))
                .unwrap();

                assert_eq!(input, "input");
                assert_eq!(
                    format!("{:?}", item),
                    format!("{:?}", <$item_type>::new($argument0_value))
                );
            }

            #[test]
            fn missing_semicolon_test() {
                assert_eq!(
                    $item(concat!(stringify!($item_type), "( ", $argument0, " )input"))
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
                    $item(concat!(stringify!($item_type), "( );input")).unwrap_err(),
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
                    $item(concat!(stringify!($item_type), "( false, false );input")).unwrap_err(),
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

#[macro_export]
macro_rules! impl_tests_for_item_arg2 {
    ($test_module_name:ident, $item:ident, $item_type:ty, ($argument0:expr, $argument0_value:expr), ($argument1:expr, $argument1_value:expr)) => {
        mod $test_module_name {
            use super::*;

            #[test]
            fn test() {
                let (input, item) = $item(concat!(
                    stringify!($item_type),
                    "( ",
                    $argument0,
                    " , ",
                    $argument1,
                    " );input"
                ))
                .unwrap();

                assert_eq!(input, "input");
                assert_eq!(
                    format!("{:?}", item),
                    format!(
                        "{:?}",
                        <$item_type>::new($argument0_value, $argument1_value)
                    )
                );
            }

            #[test]
            fn missing_semicolon_test() {
                assert_eq!(
                    $item(concat!(
                        stringify!($item_type),
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
                    $item(concat!(stringify!($item_type), "( false );input")).unwrap_err(),
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
                    $item(concat!(
                        stringify!($item_type),
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
