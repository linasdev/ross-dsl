#[macro_export]
macro_rules! impl_item_arg0 {
    ($input:expr, $name:expr, $arguments:expr, $item_type:ty) => {
        if $name == stringify!($item_type) {
            use nom::Err as NomErr;

            use crate::error::{ErrorKind, Expectation, ParserError};

            return if $arguments.len() == 0 {
                Ok(($input, Box::new(<$item_type>::new())))
            } else {
                Err(NomErr::Error(ParserError::Base {
                    location: $input,
                    kind: ErrorKind::Expected(Expectation::ArgumentCount(0, $arguments.len())),
                    child: None,
                }))
            };
        }
    };
}

#[macro_export]
macro_rules! impl_item_arg1 {
    ($input:expr, $name:expr, $arguments:expr, $item_type:ty) => {
        if $name == stringify!($item_type) {
            use nom::Err as NomErr;

            use crate::error::{ErrorKind, Expectation, ParserError};

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
                Err(NomErr::Error(ParserError::Base {
                    location: $input,
                    kind: ErrorKind::Expected(Expectation::ArgumentCount(1, $arguments.len())),
                    child: None,
                }))
            };
        }
    };
}

#[macro_export]
macro_rules! impl_item_arg2 {
    ($input:expr, $name:expr, $arguments:expr, $item_type:ty) => {
        if $name == stringify!($item_type) {
            use nom::Err as NomErr;

            use crate::error::{ErrorKind, Expectation, ParserError};

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
                Err(NomErr::Error(ParserError::Base {
                    location: $input,
                    kind: ErrorKind::Expected(Expectation::ArgumentCount(2, $arguments.len())),
                    child: None,
                }))
            };
        }
    };
}

#[macro_export]
macro_rules! impl_item_arg3 {
    ($input:expr, $name:expr, $arguments:expr, $item_type:ty) => {
        if $name == stringify!($item_type) {
            use nom::Err as NomErr;

            use crate::error::{ErrorKind, Expectation, ParserError};

            return if $arguments.len() == 3 {
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
                        $arguments[2]
                            .clone()
                            .try_into()
                            .map_err(|err| Err::Error(err))?,
                    )),
                ))
            } else {
                Err(NomErr::Error(ParserError::Base {
                    location: $input,
                    kind: ErrorKind::Expected(Expectation::ArgumentCount(3, $arguments.len())),
                    child: None,
                }))
            };
        }
    };
}

#[macro_export]
macro_rules! impl_item_arg4 {
    ($input:expr, $name:expr, $arguments:expr, $item_type:ty) => {
        if $name == stringify!($item_type) {
            use nom::Err as NomErr;

            use crate::error::{ErrorKind, Expectation, ParserError};

            return if $arguments.len() == 4 {
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
                        $arguments[2]
                            .clone()
                            .try_into()
                            .map_err(|err| Err::Error(err))?,
                        $arguments[3]
                            .clone()
                            .try_into()
                            .map_err(|err| Err::Error(err))?,
                    )),
                ))
            } else {
                Err(NomErr::Error(ParserError::Base {
                    location: $input,
                    kind: ErrorKind::Expected(Expectation::ArgumentCount(4, $arguments.len())),
                    child: None,
                }))
            };
        }
    };
}

#[macro_export]
macro_rules! impl_tests_for_item_arg0 {
    ($test_module_name:ident, $item:ident, $item_type:ty) => {
        mod $test_module_name {
            use super::*;

            use cool_asserts::assert_matches;
            use nom::Err as NomErr;

            use crate::error::{ParserError, Expectation, ErrorKind};

            #[test]
            fn test() {
                let constants = BTreeMap::new();
                let (input, item) = $item(&constants)(concat!(stringify!($item_type), "( );input")).unwrap();

                assert_eq!(input, "input");
                assert_eq!(format!("{:?}", item), format!("{:?}", <$item_type>::new()));
            }

            #[test]
            fn missing_semicolon_test() {
                let constants = BTreeMap::new();
                assert_matches!(
                    $item(&constants)(concat!(stringify!($item_type), "( )input")),
                    Err(NomErr::Error(ParserError::Base {
                        location,
                        kind,
                        child,
                    })) => {
                        assert_matches!(location, "input");
                        assert_matches!(kind, ErrorKind::Expected(Expectation::Symbol(';')));
                        assert_matches!(child, None);
                    }
                );
            }

            #[test]

            fn too_many_arguments_test() {
                let constants = BTreeMap::new();
                assert_matches!(
                    $item(&constants)(concat!(stringify!($item_type), "(false);input")),
                    Err(NomErr::Error(ParserError::Base {
                        location,
                        kind,
                        child,
                    })) => {
                        assert_matches!(location, "input");
                        assert_matches!(kind, ErrorKind::Expected(Expectation::ArgumentCount(0, 1)));
                        assert_matches!(child, None);
                    }
                );
            }
        }
    };
}

#[macro_export]
macro_rules! impl_tests_for_item_arg1 {
    ($test_module_name:ident, $item:ident, $item_type:ty, ($argument_or_constant0:expr, $argument_or_constant0_value:expr)) => {
        mod $test_module_name {
            use super::*;

            use cool_asserts::assert_matches;
            use nom::Err as NomErr;

            use crate::error::{ParserError, Expectation, ErrorKind};

            #[test]
            fn test() {
                let constants = BTreeMap::new();
                let (input, item) = $item(&constants)(concat!(
                    stringify!($item_type),
                    "(",
                    $argument_or_constant0,
                    ");input"
                ))
                .unwrap();

                assert_eq!(input, "input");
                assert_eq!(
                    format!("{:?}", item),
                    format!("{:?}", <$item_type>::new($argument_or_constant0_value))
                );
            }

            #[test]
            fn missing_semicolon_test() {
                let constants = BTreeMap::new();
                assert_matches!(
                    $item(&constants)(concat!(stringify!($item_type), "(", $argument_or_constant0, ")input")),
                    Err(NomErr::Error(ParserError::Base {
                        location,
                        kind,
                        child,
                    })) => {
                        assert_matches!(location, "input");
                        assert_matches!(kind, ErrorKind::Expected(Expectation::Symbol(';')));
                        assert_matches!(child, None);
                    }
                );
            }

            #[test]

            fn too_few_arguments_test() {
                let constants = BTreeMap::new();
                assert_matches!(
                    $item(&constants)(concat!(stringify!($item_type), "();input")),
                    Err(NomErr::Error(ParserError::Base {
                        location,
                        kind,
                        child,
                    })) => {
                        assert_matches!(location, "input");
                        assert_matches!(kind, ErrorKind::Expected(Expectation::ArgumentCount(1, 0)));
                        assert_matches!(child, None);
                    }
                );
            }

            #[test]

            fn too_many_arguments_test() {
                let constants = BTreeMap::new();
                assert_matches!(
                    $item(&constants)(concat!(stringify!($item_type), "(false, false);input")),
                    Err(NomErr::Error(ParserError::Base {
                        location,
                        kind,
                        child,
                    })) => {
                        assert_matches!(location, "input");
                        assert_matches!(kind, ErrorKind::Expected(Expectation::ArgumentCount(1, 2)));
                        assert_matches!(child, None);
                    }
                );
            }
        }
    };
}

#[macro_export]
macro_rules! impl_tests_for_item_arg2 {
    ($test_module_name:ident, $item:ident, $item_type:ty, ($argument_or_constant0:expr, $argument_or_constant0_value:expr), ($argument1:expr, $argument1_value:expr)) => {
        mod $test_module_name {
            use super::*;

            use cool_asserts::assert_matches;
            use nom::Err as NomErr;

            use crate::error::{ParserError, Expectation, ErrorKind};

            #[test]
            fn test() {
                let constants = BTreeMap::new();
                let (input, item) = $item(&constants)(concat!(
                    stringify!($item_type),
                    "(",
                    $argument_or_constant0,
                    ", ",
                    $argument1,
                    ");input"
                ))
                .unwrap();

                assert_eq!(input, "input");
                assert_eq!(
                    format!("{:?}", item),
                    format!(
                        "{:?}",
                        <$item_type>::new($argument_or_constant0_value, $argument1_value)
                    )
                );
            }

            #[test]
            fn missing_semicolon_test() {
                let constants = BTreeMap::new();
                assert_matches!(
                    $item(&constants)(concat!(
                        stringify!($item_type),
                        "( ",
                        $argument_or_constant0,
                        " , ",
                        $argument1,
                        " )input"
                    )),
                    Err(NomErr::Error(ParserError::Base {
                        location,
                        kind,
                        child,
                    })) => {
                        assert_matches!(location, "input");
                        assert_matches!(kind, ErrorKind::Expected(Expectation::Symbol(';')));
                        assert_matches!(child, None);
                    }
                );
            }

            #[test]

            fn too_few_arguments_test() {
                let constants = BTreeMap::new();
                assert_matches!(
                    $item(&constants)(concat!(stringify!($item_type), "(false);input")),
                    Err(NomErr::Error(ParserError::Base {
                        location,
                        kind,
                        child,
                    })) => {
                        assert_matches!(location, "input");
                        assert_matches!(kind, ErrorKind::Expected(Expectation::ArgumentCount(2, 1)));
                        assert_matches!(child, None);
                    }
                );
            }

            #[test]

            fn too_many_arguments_test() {
                let constants = BTreeMap::new();
                assert_matches!(
                    $item(&constants)(concat!(stringify!($item_type), "(false, false, false);input")),
                    Err(NomErr::Error(ParserError::Base {
                        location,
                        kind,
                        child,
                    })) => {
                        assert_matches!(location, "input");
                        assert_matches!(kind, ErrorKind::Expected(Expectation::ArgumentCount(2, 3)));
                        assert_matches!(child, None);
                    }
                );
            }
        }
    };
}

#[macro_export]
macro_rules! impl_tests_for_item_arg3 {
    ($test_module_name:ident, $item:ident, $item_type:ty, ($argument_or_constant0:expr, $argument_or_constant0_value:expr), ($argument1:expr, $argument1_value:expr), ($argument2:expr, $argument2_value:expr)) => {
        mod $test_module_name {
            use super::*;

            use cool_asserts::assert_matches;
            use nom::Err as NomErr;

            use crate::error::{ParserError, Expectation, ErrorKind};

            #[test]
            fn test() {
                let constants = BTreeMap::new();
                let (input, item) = $item(&constants)(concat!(
                    stringify!($item_type),
                    "( ",
                    $argument_or_constant0,
                    " , ",
                    $argument1,
                    " , ",
                    $argument2,
                    " );input"
                ))
                .unwrap();

                assert_eq!(input, "input");
                assert_eq!(
                    format!("{:?}", item),
                    format!(
                        "{:?}",
                        <$item_type>::new($argument_or_constant0_value, $argument1_value, $argument2_value)
                    )
                );
            }

            #[test]
            fn missing_semicolon_test() {
                let constants = BTreeMap::new();
                assert_matches!(
                    $item(&constants)(concat!(
                        stringify!($item_type),
                        "( ",
                        $argument_or_constant0,
                        " , ",
                        $argument1,
                        " , ",
                        $argument2,
                        " )input"
                    )),
                    Err(NomErr::Error(ParserError::Base {
                        location,
                        kind,
                        child,
                    })) => {
                        assert_matches!(location, "input");
                        assert_matches!(kind, ErrorKind::Expected(Expectation::Symbol(';')));
                        assert_matches!(child, None);
                    }
                );
            }

            #[test]

            fn too_few_arguments_test() {
                let constants = BTreeMap::new();
                assert_matches!(
                    $item(&constants)(concat!(stringify!($item_type), "(false, false);input")),
                    Err(NomErr::Error(ParserError::Base {
                        location,
                        kind,
                        child,
                    })) => {
                        assert_matches!(location, "input");
                        assert_matches!(kind, ErrorKind::Expected(Expectation::ArgumentCount(3, 2)));
                        assert_matches!(child, None);
                    }
                );
            }

            #[test]

            fn too_many_arguments_test() {
                let constants = BTreeMap::new();
                assert_matches!(
                    $item(&constants)(concat!(stringify!($item_type), "(false, false, false, false);input")),
                    Err(NomErr::Error(ParserError::Base {
                        location,
                        kind,
                        child,
                    })) => {
                        assert_matches!(location, "input");
                        assert_matches!(kind, ErrorKind::Expected(Expectation::ArgumentCount(3, 4)));
                        assert_matches!(child, None);
                    }
                );
            }
        }
    };
}

#[macro_export]
macro_rules! impl_tests_for_item_arg4 {
    ($test_module_name:ident, $item:ident, $item_type:ty, ($argument_or_constant0:expr, $argument_or_constant0_value:expr), ($argument1:expr, $argument1_value:expr), ($argument2:expr, $argument2_value:expr), ($argument3:expr, $argument3_value:expr)) => {
        mod $test_module_name {
            use super::*;

            use cool_asserts::assert_matches;
            use nom::Err as NomErr;

            use crate::error::{ParserError, Expectation, ErrorKind};

            #[test]
            fn test() {
                let constants = BTreeMap::new();
                let (input, item) = $item(&constants)(concat!(
                    stringify!($item_type),
                    "( ",
                    $argument_or_constant0,
                    " , ",
                    $argument1,
                    " , ",
                    $argument2,
                    " , ",
                    $argument3,
                    " );input"
                ))
                .unwrap();

                assert_eq!(input, "input");
                assert_eq!(
                    format!("{:?}", item),
                    format!(
                        "{:?}",
                        <$item_type>::new($argument_or_constant0_value, $argument1_value, $argument2_value, $argument3_value)
                    )
                );
            }

            #[test]
            fn missing_semicolon_test() {
                let constants = BTreeMap::new();
                assert_matches!(
                    $item(&constants)(concat!(
                        stringify!($item_type),
                        "( ",
                        $argument_or_constant0,
                        " , ",
                        $argument1,
                        " , ",
                        $argument2,
                        " , ",
                        $argument3,
                        " )input"
                    )),
                    Err(NomErr::Error(ParserError::Base {
                        location,
                        kind,
                        child,
                    })) => {
                        assert_matches!(location, "input");
                        assert_matches!(kind, ErrorKind::Expected(Expectation::Symbol(';')));
                        assert_matches!(child, None);
                    }
                );
            }

            #[test]

            fn too_few_arguments_test() {
                let constants = BTreeMap::new();
                assert_matches!(
                    $item(&constants)(concat!(stringify!($item_type), "(false, false, false);input")),
                    Err(NomErr::Error(ParserError::Base {
                        location,
                        kind,
                        child,
                    })) => {
                        assert_matches!(location, "input");
                        assert_matches!(kind, ErrorKind::Expected(Expectation::ArgumentCount(4, 3)));
                        assert_matches!(child, None);
                    }
                );
            }

            #[test]

            fn too_many_arguments_test() {
                let constants = BTreeMap::new();
                assert_matches!(
                    $item(&constants)(concat!(stringify!($item_type), "(false, false, false, false, false);input")),
                    Err(NomErr::Error(ParserError::Base {
                        location,
                        kind,
                        child,
                    })) => {
                        assert_matches!(location, "input");
                        assert_matches!(kind, ErrorKind::Expected(Expectation::ArgumentCount(4, 5)));
                        assert_matches!(child, None);
                    }
                );
            }
        }
    };
}
