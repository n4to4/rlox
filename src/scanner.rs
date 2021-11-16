pub struct Scanner<'src> {
    source: &'src str,
    start: usize,
    current: usize,
    line: usize,
}

pub struct Token {
    typ: TokenType,
    line: usize,
}

#[rustfmt::skip]
pub enum TokenType {
    // Single-character tokens.
    LeftParen, RightParen, LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus, Semicolon, Slash, Star,
    // One or two character tokens.
    Bang, BangEqual, Equal, EqualEqual,
    Greater, GreaterEqual, Less, LessEqual,
    // Literals.
    Identifier, String, Number,
    // Keywords.
    And, Class, Else, False, For, Fun, If, Nil,
    Or, Print, Return, Super, This, True, Var, While,

    Error, Eof,
}

pub enum ScannerError {
    Unknown,
}

impl<'src> Scanner<'src> {
    pub fn new(source: &'src str) -> Self {
        Scanner {
            source,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> Result<Token, ScannerError> {
        todo!()
    }
}
