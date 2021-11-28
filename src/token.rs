use core::convert::Into;

#[derive(Debug, PartialEq)]
pub enum Token {
    Keyword(KeywordToken),
    Symbol(SymbolToken),
    Data(DataToken),
    Text(String),
}

#[derive(Debug, PartialEq)]
pub enum KeywordToken {
    Let,
    Const,
    Send,
    From,
    To,
    If,
    Do,
    Match,
    Event,
    Producer,
    Tick,
    Fire,
}

impl Into<Token> for KeywordToken {
    fn into(self) -> Token {
        Token::Keyword(self)
    }
}

#[derive(Debug, PartialEq)]
pub enum DataToken {
    Integer(i64),
    Boolean(bool),
}

impl Into<Token> for DataToken {
    fn into(self) -> Token {
        Token::Data(self)
    }
}

#[derive(Debug, PartialEq)]
pub enum SymbolToken {
    Semicolon,
    Tilde,
    Comma,
    OpenParenthesis,
    CloseParenthesis,
    OpenBrace,
    CloseBrace,
    EqualSign,
}

impl Into<Token> for SymbolToken {
    fn into(self) -> Token {
        Token::Symbol(self)
    }
}
