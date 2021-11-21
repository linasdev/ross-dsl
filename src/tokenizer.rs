use regex_lexer::{Lexer, LexerBuilder};

#[derive(Debug, Clone)]
pub enum Token {
    Keyword(KeywordToken),
    Symbol(SymbolToken),
    Data(DataToken),
    Text(String),
}

#[derive(Debug, Clone)]
pub enum KeywordToken {
    Let,
    Do,
    Match,
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
    LexerError(regex::Error)
}

impl Tokenizer {
    pub fn tokenize(text: &str) -> Result<Vec<Token>, TokenizerError> {
        let lexer = Self::build_lexer()?;
        let mut token_iterator = lexer.tokens(text);
        let mut tokens = vec!();

        while let Some(token) = token_iterator.next() {
            tokens.push(token);
        }

        Ok(tokens)
    }

    fn build_lexer<'a>() -> Result<Lexer<'a, Token>, TokenizerError> {
        Ok(LexerBuilder::new()
            .token(r"\s+", |_| None)
            .token(r"(_|[a-zA-Z])[a-zA-Z_0-9]*", |token| Some(Token::Text(String::from(token))))
            .token(r"let", |_| Some(Token::Keyword(KeywordToken::Let)))
            .token(r"do", |_| Some(Token::Keyword(KeywordToken::Do)))
            .token(r"match", |_| Some(Token::Keyword(KeywordToken::Match)))
            .token(r"fire", |_| Some(Token::Keyword(KeywordToken::Fire)))
            .token(r";", |_| Some(Token::Symbol(SymbolToken::Semicolon)))
            .token(r":", |_| Some(Token::Symbol(SymbolToken::Colon)))
            .token(r",", |_| Some(Token::Symbol(SymbolToken::Comma)))
            .token(r"\(", |_| Some(Token::Symbol(SymbolToken::OpenParenthesis)))
            .token(r"\)", |_| Some(Token::Symbol(SymbolToken::CloseParenthesis)))
            .token(r"\{", |_| Some(Token::Symbol(SymbolToken::OpenBrace)))
            .token(r"\}", |_| Some(Token::Symbol(SymbolToken::CloseBrace)))
            .token(r"=", |_| Some(Token::Symbol(SymbolToken::EqualSign)))
            .token(r"-?[0-9]+", |token| Some(Token::Data(DataToken::Integer(token.parse().unwrap()))))
            .token(r"(true|false)", |token| Some(Token::Data(DataToken::Boolean(token.parse().unwrap()))))
            .build()?)
    }
}

impl From<regex::Error> for TokenizerError {
    fn from(err: regex::Error) -> TokenizerError {
        TokenizerError::LexerError(err)
    }
}
