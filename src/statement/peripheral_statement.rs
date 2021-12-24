use nom::branch::alt;
use nom::character::complete::multispace1;
use nom::combinator::cut;
use nom::sequence::{preceded, terminated, tuple};
use nom::Err as NomErr;
use nom::IResult;
use std::convert::TryInto;

use ross_config::peripheral::{BcmPeripheral, Peripheral, RelayPeripheral};

use crate::error::{ErrorKind, Expectation, ParserError};
use crate::keyword::{
    bcm_keyword, double_exclusive_keyword, peripheral_keyword, relay_keyword, rgb_keyword,
    rgbw_keyword, single_keyword,
};
use crate::literal::literal;
use crate::parser::argument0;
use crate::symbol::semicolon;

pub fn peripheral_statement(text: &str) -> IResult<&str, (u32, Peripheral), ParserError<&str>> {
    let tuple_parser = tuple((
        literal,
        preceded(multispace1, alt((bcm_keyword, relay_keyword))),
        preceded(
            multispace1,
            alt((
                single_keyword,
                rgb_keyword,
                rgbw_keyword,
                double_exclusive_keyword,
            )),
        ),
        argument0,
    ));
    let peripheral_keyword_parser =
        preceded(peripheral_keyword, cut(preceded(multispace1, tuple_parser)));

    match terminated(peripheral_keyword_parser, semicolon)(text) {
        Ok((input, (peripheral_index, "bcm", "single", mut arguments))) => {
            if arguments.len() == 1 {
                let peripheral_index = peripheral_index
                    .try_into()
                    .map_err(|err| NomErr::Error(err))?;
                let channel = arguments
                    .pop()
                    .unwrap()
                    .try_into()
                    .map_err(|err| NomErr::Error(err))?;

                Ok((
                    input,
                    (
                        peripheral_index,
                        Peripheral::Bcm(BcmPeripheral::Single(channel), vec![]),
                    ),
                ))
            } else {
                Err(NomErr::Error(ParserError::Base {
                    location: text,
                    kind: ErrorKind::Expected(Expectation::ArgumentCount(1, arguments.len())),
                    child: None,
                }))
            }
        }
        Ok((input, (peripheral_index, "bcm", "rgb", mut arguments))) => {
            if arguments.len() == 3 {
                let peripheral_index = peripheral_index
                    .try_into()
                    .map_err(|err| NomErr::Error(err))?;
                let b = arguments
                    .pop()
                    .unwrap()
                    .try_into()
                    .map_err(|err| NomErr::Error(err))?;
                let g = arguments
                    .pop()
                    .unwrap()
                    .try_into()
                    .map_err(|err| NomErr::Error(err))?;
                let r = arguments
                    .pop()
                    .unwrap()
                    .try_into()
                    .map_err(|err| NomErr::Error(err))?;

                Ok((
                    input,
                    (
                        peripheral_index,
                        Peripheral::Bcm(BcmPeripheral::Rgb(r, g, b), vec![]),
                    ),
                ))
            } else {
                Err(NomErr::Error(ParserError::Base {
                    location: text,
                    kind: ErrorKind::Expected(Expectation::ArgumentCount(3, arguments.len())),
                    child: None,
                }))
            }
        }
        Ok((input, (peripheral_index, "bcm", "rgbw", mut arguments))) => {
            if arguments.len() == 4 {
                let peripheral_index = peripheral_index
                    .try_into()
                    .map_err(|err| NomErr::Error(err))?;
                let w = arguments
                    .pop()
                    .unwrap()
                    .try_into()
                    .map_err(|err| NomErr::Error(err))?;
                let b = arguments
                    .pop()
                    .unwrap()
                    .try_into()
                    .map_err(|err| NomErr::Error(err))?;
                let g = arguments
                    .pop()
                    .unwrap()
                    .try_into()
                    .map_err(|err| NomErr::Error(err))?;
                let r = arguments
                    .pop()
                    .unwrap()
                    .try_into()
                    .map_err(|err| NomErr::Error(err))?;

                Ok((
                    input,
                    (
                        peripheral_index,
                        Peripheral::Bcm(BcmPeripheral::Rgbw(r, g, b, w), vec![]),
                    ),
                ))
            } else {
                Err(NomErr::Error(ParserError::Base {
                    location: text,
                    kind: ErrorKind::Expected(Expectation::ArgumentCount(4, arguments.len())),
                    child: None,
                }))
            }
        }
        Ok((input, (peripheral_index, "relay", "single", mut arguments))) => {
            if arguments.len() == 1 {
                let peripheral_index = peripheral_index
                    .try_into()
                    .map_err(|err| NomErr::Error(err))?;
                let channel = arguments
                    .pop()
                    .unwrap()
                    .try_into()
                    .map_err(|err| NomErr::Error(err))?;

                Ok((
                    input,
                    (
                        peripheral_index,
                        Peripheral::Relay(RelayPeripheral::Single(channel), vec![]),
                    ),
                ))
            } else {
                Err(NomErr::Error(ParserError::Base {
                    location: text,
                    kind: ErrorKind::Expected(Expectation::ArgumentCount(1, arguments.len())),
                    child: None,
                }))
            }
        }
        Ok((input, (peripheral_index, "relay", "double_exclusive", mut arguments))) => {
            if arguments.len() == 2 {
                let peripheral_index = peripheral_index
                    .try_into()
                    .map_err(|err| NomErr::Error(err))?;
                let channel2 = arguments
                    .pop()
                    .unwrap()
                    .try_into()
                    .map_err(|err| NomErr::Error(err))?;
                let channel1 = arguments
                    .pop()
                    .unwrap()
                    .try_into()
                    .map_err(|err| NomErr::Error(err))?;

                Ok((
                    input,
                    (
                        peripheral_index,
                        Peripheral::Relay(RelayPeripheral::DoubleExclusive(channel1, channel2), vec![]),
                    ),
                ))
            } else {
                Err(NomErr::Error(ParserError::Base {
                    location: text,
                    kind: ErrorKind::Expected(Expectation::ArgumentCount(2, arguments.len())),
                    child: None,
                }))
            }
        }
        Ok((_, _)) => Err(NomErr::Error(ParserError::Base {
            location: text,
            kind: ErrorKind::Expected(Expectation::Something),
            child: None,
        })),
        Err(err) => Err(NomErr::convert(err)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bcm_single_test() {
        let (input, (index, peripheral)) =
            peripheral_statement("peripheral 0x00~u32 bcm single(0x01~u8);input").unwrap();

        assert_eq!(input, "input");
        assert_eq!(index, 0x00);
        assert_eq!(peripheral, Peripheral::Bcm(BcmPeripheral::Single(0x01), vec![]));
    }

    #[test]
    fn bcm_rgb_test() {
        let (input, (index, peripheral)) =
            peripheral_statement("peripheral 0x00~u32 bcm rgb(0x01~u8, 0x23~u8, 0x45~u8);input")
                .unwrap();

                assert_eq!(input, "input");
        assert_eq!(index, 0x00);
        assert_eq!(
            peripheral,
            Peripheral::Bcm(BcmPeripheral::Rgb(0x01, 0x23, 0x45), vec![])
        );
    }

    #[test]
    fn bcm_rgbw_test() {
        let (input, (index, peripheral)) = peripheral_statement(
            "peripheral 0x00~u32 bcm rgbw(0x01~u8, 0x23~u8, 0x45~u8, 0x67~u8);input",
        )
        .unwrap();

        assert_eq!(input, "input");
        assert_eq!(index, 0x00);
        assert_eq!(
            peripheral,
            Peripheral::Bcm(BcmPeripheral::Rgbw(0x01, 0x23, 0x45, 0x67), vec![])
        );
    }

    #[test]
    fn relay_single_test() {
        let (input, (index, peripheral)) =
            peripheral_statement("peripheral 0x00~u32 relay single(0x01~u8);input").unwrap();

        assert_eq!(input, "input");
        assert_eq!(index, 0x00);
        assert_eq!(peripheral, Peripheral::Relay(RelayPeripheral::Single(0x01), vec![]));
    }

    #[test]
    fn relay_double_exclusive_test() {
        let (input, (index, peripheral)) = peripheral_statement(
            "peripheral 0x00~u32 relay double_exclusive(0x01~u8, 0x23~u8);input",
        )
        .unwrap();

        assert_eq!(input, "input");
        assert_eq!(index, 0x00);
        assert_eq!(
            peripheral,
            Peripheral::Relay(RelayPeripheral::DoubleExclusive(0x01, 0x23), vec![])
        );
    }
}
