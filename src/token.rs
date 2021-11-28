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

#[derive(Debug, PartialEq)]
pub enum DataToken {
    Integer(i64),
    Boolean(bool),
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
