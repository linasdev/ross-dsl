use regex_lexer::{Lexer, LexerBuilder};
use parse_int::parse;

#[derive(Debug, Clone)]
pub enum Token {
    Keyword(KeywordToken),
    Symbol(SymbolToken),
    Data(DataToken),
    Text(String),
}

#[derive(Debug, Clone)]
pub enum KeywordToken {
    Store,
    Let,
    Send,
    From,
    To,
    Do,
    Match,
    Event,
    Producer,
    Fire,
}

#[derive(Debug, Clone)]
pub enum DataToken {
    Integer(i64),
    Boolean(bool),
}

#[derive(Debug, Clone)]
pub enum SymbolToken {
    Semicolon,
    Colon,
    Comma,
    OpenParenthesis,
    CloseParenthesis,
    OpenBrace,
    CloseBrace,
    EqualSign,
}

pub struct Tokenizer {}

#[derive(Debug)]
pub enum TokenizerError {
    LexerError(regex::Error),
}

impl Tokenizer {
    pub fn tokenize(text: &str) -> Result<Vec<Token>, TokenizerError> {
        let lexer = Self::build_lexer()?;
        let mut token_iterator = lexer.tokens(text);
        let mut tokens = vec![];

        while let Some(token) = token_iterator.next() {
            tokens.push(token);
        }

        Ok(tokens)
    }

    fn build_lexer<'a>() -> Result<Lexer<'a, Token>, TokenizerError> {
        Ok(LexerBuilder::new()
            .token(r"\s+", |_| None)
            .token(r"//.*\n", |_| None)
            .token(r"(_|[a-zA-Z])[a-zA-Z_0-9]*", |token| {
                Some(Token::Text(String::from(token)))
            })
            .token(r"store", |_| Some(Token::Keyword(KeywordToken::Store)))
            .token(r"let", |_| Some(Token::Keyword(KeywordToken::Let)))
            .token(r"send", |_| Some(Token::Keyword(KeywordToken::Send)))
            .token(r"from", |_| Some(Token::Keyword(KeywordToken::From)))
            .token(r"to", |_| Some(Token::Keyword(KeywordToken::To)))
            .token(r"do", |_| Some(Token::Keyword(KeywordToken::Do)))
            .token(r"match", |_| Some(Token::Keyword(KeywordToken::Match)))
            .token(r"event", |_| Some(Token::Keyword(KeywordToken::Event)))
            .token(r"producer", |_| Some(Token::Keyword(KeywordToken::Producer)))
            .token(r"fire", |_| Some(Token::Keyword(KeywordToken::Fire)))
            .token(r";", |_| Some(Token::Symbol(SymbolToken::Semicolon)))
            .token(r":", |_| Some(Token::Symbol(SymbolToken::Colon)))
            .token(r",", |_| Some(Token::Symbol(SymbolToken::Comma)))
            .token(r"\(", |_| Some(Token::Symbol(SymbolToken::OpenParenthesis)))
            .token(r"\)", |_| {
                Some(Token::Symbol(SymbolToken::CloseParenthesis))
            })
            .token(r"\{", |_| Some(Token::Symbol(SymbolToken::OpenBrace)))
            .token(r"\}", |_| Some(Token::Symbol(SymbolToken::CloseBrace)))
            .token(r"=", |_| Some(Token::Symbol(SymbolToken::EqualSign)))
            .token(r"-?[0-9]+", |token| {
                Some(Token::Data(DataToken::Integer(parse(token).unwrap())))
            })
            .token(r"0x[0-9a-f]+", |token| {
                Some(Token::Data(DataToken::Integer(parse(token).unwrap())))
            })
            .token(r"(true|false)", |token| {
                Some(Token::Data(DataToken::Boolean(token.parse().unwrap())))
            })
            .build()?)
    }
}

impl From<regex::Error> for TokenizerError {
    fn from(err: regex::Error) -> TokenizerError {
        TokenizerError::LexerError(err)
    }
}
