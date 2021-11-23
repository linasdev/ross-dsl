use std::slice::Iter;
use std::collections::BTreeMap;
use std::convert::TryInto;

use ross_config::Value;
use ross_config::config::Config;
use ross_config::event_processor::EventProcessor;
use ross_config::matcher::Matcher;
use ross_config::extractor::*;
use ross_config::filter::*;
use ross_config::filter::state_filter::*;
use ross_config::producer::*;
use ross_config::producer::state_producer::*;
use ross_protocol::event::event_code::*;

use crate::tokenizer::{Token, KeywordToken, SymbolToken, DataToken, Tokenizer, TokenizerError};

macro_rules! prepare_variable {
    ($variable_map:expr, $variable_name:tt) => {
        $variable_map.insert(String::from(stringify!($variable_name)), Variable::Integer($variable_name.into()));
    };
}

macro_rules! match_text_token {
    ($token_iterator:expr) => {
        match $token_iterator.next() {
            Some(Token::Text(value)) => value.clone(),
            Some(token) => return Err(ParserError::UnexpectedToken(token.clone())),
            None => return Err(ParserError::UnexpectedEndOfFile),
        };
    };
}

macro_rules! match_symbol_token {
    ($token_iterator:expr, $symbol_token:path) => {
        match $token_iterator.next() {
            Some(Token::Symbol($symbol_token)) => {},
            Some(token) => return Err(ParserError::UnexpectedToken(token.clone())),
            None => return Err(ParserError::UnexpectedEndOfFile),
        };
    };
}

pub struct Parser {}

#[derive(Debug)]
pub enum ParserError {
    TokenizerError(TokenizerError),
    UnexpectedEndOfFile,
    UnexpectedToken(Token),
    TypeError,
    DataError,
    DuplicateVariable,
    UndefinedVariable(String),
    WrongArgumentCount,
    UnknownExtractor(String),
    UnknownFilter(String),
    UnknownProducer(String),
    TooManyItemsInStatement,
    TooFewItemsInStatement,
}

#[derive(Debug, Copy, Clone)]
enum Variable {
    Integer(i64),
    Boolean(bool),
}

impl<'a> Parser {
    pub fn parse(text: &str) -> Result<Config, ParserError> {
        let mut initial_state = BTreeMap::new();
        let mut variable_map = BTreeMap::new();
        let mut event_processors = vec!();

        Self::prepare_variable_map(&mut variable_map);

        let tokens = Tokenizer::tokenize(text)?;
        let mut token_iterator = tokens.iter();
        
        while let Some(token) = token_iterator.next() {
            match token {
                Token::Keyword(KeywordToken::Let) => {
                    let mut state_name = String::new();
                    let state = Self::parse_let_statement(&mut token_iterator, &mut state_name)?;

                    let mut state_index = 0;
                    
                    while let Some(_) = initial_state.get(&state_index) {
                        state_index += 1;
                    }

                    initial_state.insert(state_index, state);
                    if let Some(_) = variable_map.insert(state_name, Variable::Integer(state_index.into())) {
                        return Err(ParserError::DuplicateVariable);
                    }
                },
                Token::Keyword(KeywordToken::Do) => {
                    let event_processor = Self::parse_do_statement(&mut token_iterator, &variable_map)?;
                    event_processors.push(event_processor);
                },
                _ => return Err(ParserError::UnexpectedToken(token.clone())),
            }
        }

        Ok(Config {
            initial_state,
            event_processors,
        })
    }

    fn parse_let_statement(token_iterator: &mut Iter<Token>, state_name: &mut String) -> Result<Value, ParserError> {
        *state_name = match_text_token!(token_iterator);

        match_symbol_token!(token_iterator, SymbolToken::Colon);

        let state_type = match_text_token!(token_iterator);

        match_symbol_token!(token_iterator, SymbolToken::EqualSign);

        let state_value = match token_iterator.next() {
            Some(Token::Data(DataToken::Integer(value))) => Variable::Integer(*value),
            Some(Token::Data(DataToken::Boolean(value))) => Variable::Boolean(*value),
            Some(token) => return Err(ParserError::UnexpectedToken(token.clone())),
            None => return Err(ParserError::UnexpectedEndOfFile),
        };

        match_symbol_token!(token_iterator, SymbolToken::Semicolon);

        let state = match state_type.as_str() {
            "u8" => Value::U8(state_value.try_into()?),
            "u16" => Value::U16(state_value.try_into()?),
            "u32" => Value::U32(state_value.try_into()?),
            _ => return Err(ParserError::TypeError),
        };

        Ok(state)
    }

    fn parse_do_statement(token_iterator: &mut Iter<Token>, variable_map: &BTreeMap<String, Variable>) -> Result<EventProcessor, ParserError> {
        match_symbol_token!(token_iterator, SymbolToken::OpenBrace);

        let mut matchers = vec!();
        let mut extractor: Box<dyn Extractor> = Box::new(NoneExtractor::new());
        let mut producer: Box<dyn Producer> = Box::new(NoneProducer::new());

        while let Some(token) = token_iterator.next() {
            match token {
                Token::Keyword(KeywordToken::Match) => {
                    let matcher = Self::parse_match_statement(token_iterator, variable_map)?;
                    matchers.push(matcher);
                },
                Token::Keyword(KeywordToken::Fire) => {
                    let result = Self::parse_fire_statement(token_iterator, variable_map)?;
                    extractor = result.0;
                    producer = result.1;
                },
                Token::Symbol(SymbolToken::CloseBrace) => break,
                _ => return Err(ParserError::UnexpectedToken(token.clone())),
            }
        }

        Ok(EventProcessor {
            matchers,
            extractor,
            producer,
        })
    }

    fn parse_match_statement(token_iterator: &mut Iter<Token>, variable_map: &BTreeMap<String, Variable>) -> Result<Matcher, ParserError> {
        match_symbol_token!(token_iterator, SymbolToken::OpenBrace);

        let mut sub_token_iterator = token_iterator.clone();
        let mut item_count = 0;

        while let Some(token) = token_iterator.next() {
            match token {
                Token::Symbol(SymbolToken::Semicolon) => item_count += 1,
                Token::Symbol(SymbolToken::CloseBrace) => break,
                _ => {}
            }
        }

        if item_count > 2 {
            return Err(ParserError::TooManyItemsInStatement);
        }

        if item_count == 0 {
            return Err(ParserError::TooFewItemsInStatement);
        }

        let extractor = if item_count == 1 {
            Box::new(NoneExtractor::new())
        } else {
            Self::parse_extractor(&mut sub_token_iterator, variable_map)?
        };

        let filter = Self::parse_filter(&mut sub_token_iterator, variable_map)?;

        Ok(Matcher {
            extractor,
            filter,
        })
    }

    fn parse_fire_statement(token_iterator: &mut Iter<Token>, variable_map: &BTreeMap<String, Variable>) -> Result<(Box<dyn Extractor>, Box<dyn Producer>), ParserError> {
        match_symbol_token!(token_iterator, SymbolToken::OpenBrace);

        let mut sub_token_iterator = token_iterator.clone();
        let mut item_count = 0;

        while let Some(token) = token_iterator.next() {
            match token {
                Token::Symbol(SymbolToken::Semicolon) => item_count += 1,
                Token::Symbol(SymbolToken::CloseBrace) => break,
                _ => {}
            }
        }

        if item_count > 2 {
            return Err(ParserError::TooManyItemsInStatement);
        }

        if item_count == 0 {
            return Err(ParserError::TooFewItemsInStatement);
        }

        let extractor = if item_count == 1 {
            Box::new(NoneExtractor::new())
        } else {
            Self::parse_extractor(&mut sub_token_iterator, variable_map)?
        };

        let producer = Self::parse_producer(&mut sub_token_iterator, variable_map)?;

        Ok((extractor, producer))
    }

    fn parse_extractor(token_iterator: &mut Iter<Token>, variable_map: &BTreeMap<String, Variable>) -> Result<Box<dyn Extractor>, ParserError> {
        let extractor_type = match_text_token!(token_iterator);

        let arguments = Self::parse_arguments(token_iterator, variable_map)?;

        match_symbol_token!(token_iterator, SymbolToken::Semicolon);

        if extractor_type == "NoneExtractor" {
            if arguments.len() != 0 {
                return Err(ParserError::WrongArgumentCount);
            }

            return Ok(Box::new(NoneExtractor::new()));
        }

        if extractor_type == "EventCodeExtractor" {
            if arguments.len() != 0 {
                return Err(ParserError::WrongArgumentCount);
            }

            return Ok(Box::new(EventCodeExtractor::new()));
        }

        Err(ParserError::UnknownExtractor(extractor_type))
    }

    fn parse_filter(token_iterator: &mut Iter<Token>, variable_map: &BTreeMap<String, Variable>) -> Result<Box<dyn Filter>, ParserError> {
        let filter_type = match_text_token!(token_iterator);

        let arguments = Self::parse_arguments(token_iterator, variable_map)?;

        match_symbol_token!(token_iterator, SymbolToken::Semicolon);

        if filter_type == "U8IncrementStateFilter" {
            if arguments.len() != 1 {
                return Err(ParserError::WrongArgumentCount);
            }

            return Ok(Box::new(U8IncrementStateFilter::new(
                arguments[0].try_into().map_err(|_| ParserError::DataError)?,
            )));
        }

        if filter_type == "U16IsEqualFilter" {
            if arguments.len() != 1 {
                return Err(ParserError::WrongArgumentCount);
            }

            return Ok(Box::new(U16IsEqualFilter::new(
                arguments[0].try_into().map_err(|_| ParserError::DataError)?,
            )));
        }

        if filter_type == "U32IsEqualStateFilter" {
            if arguments.len() != 2 {
                return Err(ParserError::WrongArgumentCount);
            }

            return Ok(Box::new(U32IsEqualStateFilter::new(
                arguments[0].try_into().map_err(|_| ParserError::DataError)?,
                arguments[1].try_into().map_err(|_| ParserError::DataError)?,
            )));
        }

        if filter_type == "U32IncrementStateFilter" {
            if arguments.len() != 1 {
                return Err(ParserError::WrongArgumentCount);
            }

            return Ok(Box::new(U32IncrementStateFilter::new(
                arguments[0].try_into().map_err(|_| ParserError::DataError)?,
            )));
        }

        if filter_type == "U32SetStateFilter" {
            if arguments.len() != 2 {
                return Err(ParserError::WrongArgumentCount);
            }

            return Ok(Box::new(U32SetStateFilter::new(
                arguments[0].try_into().map_err(|_| ParserError::DataError)?,
                arguments[1].try_into().map_err(|_| ParserError::DataError)?,
            )));
        }

        if filter_type == "FlipFlopFilter" {
            if arguments.len() != 1 {
                return Err(ParserError::WrongArgumentCount);
            }

            return Ok(Box::new(FlipFlopFilter::new(
                arguments[0].try_into().map_err(|_| ParserError::DataError)?,
            )));
        }

        if filter_type == "CountFilter" {
            if arguments.len() != 2 {
                return Err(ParserError::WrongArgumentCount);
            }

            return Ok(Box::new(CountFilter::new(
                arguments[0].try_into().map_err(|_| ParserError::DataError)?,
                arguments[1].try_into().map_err(|_| ParserError::DataError)?,
            )));
        }

        if filter_type == "CountStateFilter" {
            if arguments.len() != 2 {
                return Err(ParserError::WrongArgumentCount);
            }

            return Ok(Box::new(CountStateFilter::new(
                arguments[0].try_into().map_err(|_| ParserError::DataError)?,
                arguments[1].try_into().map_err(|_| ParserError::DataError)?,
            )));
        }

        Err(ParserError::UnknownFilter(filter_type))
    }

    fn parse_producer(token_iterator: &mut Iter<Token>, variable_map: &BTreeMap<String, Variable>) -> Result<Box<dyn Producer>, ParserError> {
        let producer_type = match_text_token!(token_iterator);

        let arguments = Self::parse_arguments(token_iterator, variable_map)?;

        match_symbol_token!(token_iterator, SymbolToken::Semicolon);

        if producer_type == "NoneProducer" {
            if arguments.len() != 0 {
                return Err(ParserError::WrongArgumentCount);
            }

            return Ok(Box::new(NoneProducer::new()));
        }

        if producer_type == "BcmChangeBrightnessProducer" {
            if arguments.len() != 3 {
                return Err(ParserError::WrongArgumentCount);
            }

            return Ok(Box::new(BcmChangeBrightnessProducer::new(
                arguments[0].try_into().map_err(|_| ParserError::DataError)?,
                arguments[1].try_into().map_err(|_| ParserError::DataError)?,
                arguments[2].try_into().map_err(|_| ParserError::DataError)?,
            )));
        }

        if producer_type == "BcmChangeBrightnessStateProducer" {
            if arguments.len() != 3 {
                return Err(ParserError::WrongArgumentCount);
            }

            return Ok(Box::new(BcmChangeBrightnessStateProducer::new(
                arguments[0].try_into().map_err(|_| ParserError::DataError)?,
                arguments[1].try_into().map_err(|_| ParserError::DataError)?,
                arguments[2].try_into().map_err(|_| ParserError::DataError)?,
            )));
        }

        Err(ParserError::UnknownProducer(producer_type))
    }

    fn parse_arguments(token_iterator: &mut Iter<Token>, variable_map: &BTreeMap<String, Variable>) -> Result<Vec<Variable>, ParserError> {
        match_symbol_token!(token_iterator, SymbolToken::OpenParenthesis);

        let mut arguments = vec!();
        let mut comma_next = false;

        while let Some(token) = token_iterator.next() {
            if comma_next {
                match token {
                    Token::Symbol(SymbolToken::Comma) => {},
                    Token::Symbol(SymbolToken::CloseParenthesis) => break,
                    _ => return Err(ParserError::UnexpectedToken(token.clone())),
                }

                comma_next = false;
            } else {
                match token {
                    Token::Data(DataToken::Integer(value)) => {
                        arguments.push(Variable::Integer(*value));
                    },
                    Token::Data(DataToken::Boolean(value)) => {
                        arguments.push(Variable::Boolean(*value));
                    },
                    Token::Text(value) => {
                        if let Some(variable) = variable_map.get(value) {
                            arguments.push(*variable);
                        } else {
                            return Err(ParserError::UndefinedVariable(value.clone()));
                        }
                    },
                    Token::Symbol(SymbolToken::CloseParenthesis) => break,
                    _ => return Err(ParserError::UnexpectedToken(token.clone())),
                }

                comma_next = true;
            }
        }

        Ok(arguments)
    }

    fn prepare_variable_map(variable_map: &mut BTreeMap<String, Variable>) {
        prepare_variable!(variable_map, NONE_EXTRACTOR_CODE);
        prepare_variable!(variable_map, EVENT_CODE_EXTRACTOR_CODE);

        prepare_variable!(variable_map, U8_INCREMENT_STATE_FILTER);
        prepare_variable!(variable_map, U16_IS_EQUAL_FILTER_CODE);
        prepare_variable!(variable_map, U32_IS_EQUAL_STATE_FILTER_CODE);
        prepare_variable!(variable_map, U32_INCREMENT_STATE_FILTER_CODE);
        prepare_variable!(variable_map, U32_SET_STATE_FILTER_CODE);
        prepare_variable!(variable_map, FLIP_FLOP_FILTER_CODE);
        prepare_variable!(variable_map, COUNT_FILTER_CODE);
        prepare_variable!(variable_map, COUNT_STATE_FILTER_CODE);

        prepare_variable!(variable_map, NONE_PRODUCER_CODE);
        prepare_variable!(variable_map, BCM_CHANGE_BRIGHTNESS_PRODUCER_CODE);
        prepare_variable!(variable_map, BCM_CHANGE_BRIGHTNESS_STATE_PRODUCER_CODE);

        prepare_variable!(variable_map, BOOTLOADER_HELLO_EVENT_CODE);
        prepare_variable!(variable_map, PROGRAMMER_HELLO_EVENT_CODE);
        prepare_variable!(variable_map, PROGRAMMER_START_UPLOAD_EVENT_CODE);
        prepare_variable!(variable_map, ACK_EVENT_CODE);
        prepare_variable!(variable_map, DATA_EVENT_CODE);
        prepare_variable!(variable_map, CONFIGURATOR_HELLO_EVENT_CODE);
        prepare_variable!(variable_map, BCM_CHANGE_BRIGHTNESS_EVENT_CODE);
        prepare_variable!(variable_map, BUTTON_PRESSED_EVENT_CODE);
        prepare_variable!(variable_map, BUTTON_RELEASED_EVENT_CODE);
        prepare_variable!(variable_map, INTERNAL_SYSTEM_TICK_EVENT_CODE);
        prepare_variable!(variable_map, PROGRAMMER_START_EVENT_PROCESSOR_UPLOAD_EVENT_CODE);
    }
}

impl From<TokenizerError> for ParserError {
    fn from(err: TokenizerError) -> ParserError {
        ParserError::TokenizerError(err)
    }
}

impl TryInto<u32> for Variable {
    type Error = ParserError;

    fn try_into(self) -> Result<u32, Self::Error> {
        match self {
            Variable::Integer(value) => Ok(value.try_into().map_err(|_| ParserError::DataError)?),
            _ => Err(ParserError::DataError),
        }
    }
}

impl TryInto<u16> for Variable {
    type Error = ParserError;

    fn try_into(self) -> Result<u16, Self::Error> {
        match self {
            Variable::Integer(value) => Ok(value.try_into().map_err(|_| ParserError::DataError)?),
            _ => Err(ParserError::DataError),
        }
    }
}

impl TryInto<u8> for Variable {
    type Error = ParserError;

    fn try_into(self) -> Result<u8, Self::Error> {
        match self {
            Variable::Integer(value) => Ok(value.try_into().map_err(|_| ParserError::DataError)?),
            _ => Err(ParserError::DataError),
        }
    }
}

impl TryInto<bool> for Variable {
    type Error = ParserError;

    fn try_into(self) -> Result<bool, Self::Error> {
        match self {
            Variable::Boolean(value) => Ok(value),
            _ => Err(ParserError::DataError),
        }
    }
}
