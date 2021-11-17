pub struct Scanner<'src> {
    source: &'src str,
    start: usize,
    current: usize,
    line: usize,
}

pub struct Token<'src> {
    typ: TokenType,
    name: &'src str,
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
    UnexpectedCharacter,
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

    pub fn scan_token(&mut self) -> Token {
        self.start = self.current;
        if self.is_at_end() {
            return self.make_token(TokenType::Eof);
        }
        self.error_token("Unexpected character.")
    }

    fn make_token(&self, typ: TokenType) -> Token {
        Token {
            typ,
            name: &self.source[self.start..self.current],
            line: self.line,
        }
    }

    fn error_token(&self, message: &'static str) -> Token {
        Token {
            typ: TokenType::Error,
            name: message,
            line: self.line,
        }
    }

    fn is_at_end(&self) -> bool {
        self.current == self.source.len() - 1
    }
}
