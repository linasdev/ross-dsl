use regex_lexer::{Lexer, LexerBuilder};

#[derive(Debug)]
pub enum Token<'a> {
    Keyword(KeywordToken),
    Symbol(SymbolToken),
    Data(DataToken),
    Text(&'a str),
}

#[derive(Debug)]
pub enum KeywordToken {
    Let,
    Do,
    Match,
    Fire,
}

#[derive(Debug)]
pub enum DataToken {
    Integer(i64),
}

#[derive(Debug)]
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
    pub fn tokenize_and_iterate(text: &str, mut closure: Box<dyn FnMut(Token)>) -> Result<(), TokenizerError> {
        let lexer = Self::build_lexer()?;
        let mut tokens = lexer.tokens(text);

        while let Some(token) = tokens.next() {
            closure(token);
        }

        Ok(())
    }

    fn build_lexer<'a>() -> Result<Lexer<'a, Token<'a>>, TokenizerError> {
        Ok(LexerBuilder::new()
            .token(r"\s+", |_| None)
            .token(r"(_|[a-zA-Z])[a-zA-Z_0-9]*", |token| Some(Token::Text(token)))
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
            .build()?)
    }
}

impl From<regex::Error> for TokenizerError {
    fn from(err: regex::Error) -> TokenizerError {
        TokenizerError::LexerError(err)
    }
}
