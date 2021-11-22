pub struct Scanner<'src> {
    source: &'src str,
    start: usize,
    current: usize,
    line: usize,
}

#[derive(Debug, Clone)]
pub struct Token<'src> {
    pub typ: TokenType,
    pub name: &'src str,
    pub line: usize,
}

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    pub fn advance(&mut self) -> char {
        self.current += 1;
        self.source.as_bytes()[self.current - 1] as char
    }

    fn matches(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.as_bytes()[self.current] as char != expected {
            return false;
        }
        self.current += 1;
        true
    }

    fn peek(&self) -> Option<char> {
        if self.current >= self.source.len() {
            None
        } else {
            Some(self.source.as_bytes()[self.current] as char)
        }
    }

    fn peek_next(&self) -> Option<char> {
        if self.is_at_end() {
            None
        } else {
            Some(self.source.as_bytes()[self.current + 1] as char)
        }
    }

    #[allow(dead_code)]
    fn skip_whitespace(&mut self) {
        loop {
            if let Some(c) = self.peek() {
                match c {
                    ' ' | '\r' | '\t' => {
                        self.advance();
                    }
                    '\n' => {
                        self.line += 1;
                        self.advance();
                    }
                    '/' => {
                        if let Some('/') = self.peek_next() {
                            while self.peek().filter(|c| *c != '\n').is_some() {
                                self.advance();
                            }
                        } else {
                            return;
                        }
                    }
                    _ => return,
                }
            } else {
                return;
            }
        }
    }

    pub fn scan_token(&mut self) -> Token<'src> {
        self.start = self.current;
        if self.is_at_end() {
            return self.make_token(TokenType::Eof);
        }

        let c = self.advance();
        if c.is_ascii_alphabetic() || c == '_' {
            return self.identifier();
        }
        if c.is_digit(10) {
            return self.number();
        }
        match c {
            '(' => return self.make_token(TokenType::LeftParen),
            ')' => return self.make_token(TokenType::RightParen),
            '{' => return self.make_token(TokenType::LeftBrace),
            '}' => return self.make_token(TokenType::RightBrace),
            ';' => return self.make_token(TokenType::Semicolon),
            ',' => return self.make_token(TokenType::Comma),
            '.' => return self.make_token(TokenType::Dot),
            '-' => return self.make_token(TokenType::Minus),
            '+' => return self.make_token(TokenType::Plus),
            '/' => return self.make_token(TokenType::Slash),
            '*' => return self.make_token(TokenType::Star),
            '!' => {
                let token = if self.matches('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                return self.make_token(token);
            }
            '=' => {
                let token = if self.matches('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                return self.make_token(token);
            }
            '<' => {
                let token = if self.matches('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                return self.make_token(token);
            }
            '>' => {
                let token = if self.matches('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                return self.make_token(token);
            }
            '"' => return self.string(),
            _ => {}
        }

        self.error_token("Unexpected character.")
    }

    fn string(&mut self) -> Token<'src> {
        while let Some(c) = self.peek().filter(|c| *c != '"') {
            if c == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return self.error_token("Unterminated string.");
        }

        // The closing quote.
        self.advance();
        self.make_token(TokenType::String)
    }

    fn number(&mut self) -> Token<'src> {
        while self.peek().filter(|c| c.is_digit(10)).is_some() {
            self.advance();
        }
        if self.peek() == Some('.') && self.peek_next().filter(|c| c.is_digit(10)).is_some() {
            // Consume the ".".
            self.advance();

            while self.peek().filter(|c| c.is_digit(10)).is_some() {
                self.advance();
            }
        }
        self.make_token(TokenType::Number)
    }

    fn identifier(&mut self) -> Token<'src> {
        while dbg!(self.peek())
            .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
            .is_some()
        {
            self.advance();
        }
        self.make_token(self.identifier_type())
    }

    fn identifier_type(&self) -> TokenType {
        match self.source.as_bytes()[self.start] as char {
            'a' => self.check_keyword(1, 2, "nd", TokenType::And),
            'c' => self.check_keyword(1, 4, "lass", TokenType::Class),
            'e' => self.check_keyword(1, 3, "lse", TokenType::Else),
            'f' => {
                if self.current - self.start > 1 {
                    match self.source.as_bytes()[self.start + 1] as char {
                        'a' => self.check_keyword(2, 3, "lse", TokenType::False),
                        'o' => self.check_keyword(2, 1, "r", TokenType::For),
                        'u' => self.check_keyword(2, 1, "n", TokenType::Fun),
                        _ => TokenType::Identifier,
                    }
                } else {
                    TokenType::Identifier
                }
            }
            'i' => self.check_keyword(1, 1, "f", TokenType::If),
            'n' => self.check_keyword(1, 2, "il", TokenType::Nil),
            'o' => self.check_keyword(1, 1, "r", TokenType::Or),
            'p' => self.check_keyword(1, 4, "rint", TokenType::Print),
            'r' => self.check_keyword(1, 5, "eturn", TokenType::Return),
            's' => self.check_keyword(1, 4, "uper", TokenType::Super),
            't' => {
                if self.current - self.start > 1 {
                    match self.source.as_bytes()[self.start + 1] as char {
                        'h' => self.check_keyword(2, 2, "is", TokenType::This),
                        'r' => self.check_keyword(2, 2, "ue", TokenType::True),
                        _ => TokenType::Identifier,
                    }
                } else {
                    TokenType::Identifier
                }
            }
            'v' => self.check_keyword(1, 2, "ar", TokenType::Var),
            'w' => self.check_keyword(1, 4, "hile", TokenType::While),
            _ => TokenType::Identifier,
        }
    }

    fn check_keyword(&self, start: usize, length: usize, rest: &str, typ: TokenType) -> TokenType {
        let start = self.start + start;
        let end = start + length;
        if self.current - self.start == start + length
            && &self.source.as_bytes()[start..end] == rest.as_bytes()
        {
            typ
        } else {
            TokenType::Identifier
        }
    }

    fn make_token(&self, typ: TokenType) -> Token<'src> {
        Token {
            typ,
            name: &self.source[self.start..self.current],
            line: self.line,
        }
    }

    fn error_token(&self, message: &'static str) -> Token<'src> {
        Token {
            typ: TokenType::Error,
            name: message,
            line: self.line,
        }
    }

    fn is_at_end(&self) -> bool {
        self.source.is_empty() || self.current == self.source.len() - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_empty_source() {
        let mut s = Scanner::new("");
        let tok = s.scan_token();
        assert_eq!(tok.typ, TokenType::Eof);
    }

    #[test]
    fn scan_random_identifier() {
        let mut s = Scanner::new("bar");
        let tok = s.scan_token();

        assert_eq!(tok.typ, TokenType::Identifier);
        assert_eq!(tok.name, "bar");
    }

    #[test]
    fn scan_keywords() {
        let test_cases = vec![
            ("if", TokenType::If),
            ("class", TokenType::Class),
            ("false", TokenType::False),
            ("true", TokenType::True),
            ("true_", TokenType::Identifier),
        ];

        for (source, expected_token_type) in test_cases {
            let mut s = Scanner::new(source);
            let tok = s.scan_token();

            assert_eq!(tok.typ, expected_token_type);
            assert_eq!(tok.name, source);
        }
    }
}
