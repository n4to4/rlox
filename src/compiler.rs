use crate::chunk::Chunk;
use crate::scanner::{Scanner, Token, TokenType};

pub struct Compiler<'src> {
    parser: Parser<'src>,
    scanner: Scanner<'src>,
}

struct Parser<'src> {
    current: Option<Token<'src>>,
    previous: Option<Token<'src>>,
    had_error: bool,
    panic_mode: bool,
}

impl<'src> Compiler<'src> {
    pub fn new(source: &'src str, _chunk: &mut Chunk) -> Self {
        let mut _scanner = Scanner::new(source);
        let parser = Parser::new();
        let scanner = Scanner::new(source);
        Compiler { parser, scanner }
    }

    pub fn compile(&mut self) -> anyhow::Result<()> {
        self.advance();
        self.expression();
        self.consume(TokenType::Eof, "Expect end of expression.");

        if self.parser.had_error {
            anyhow::bail!("parse error");
        }
        Ok(())
    }

    fn advance(&mut self) {
        self.parser.previous = self.parser.current.take();
        loop {
            let token = self.scanner.scan_token();
            let typ = token.typ;
            let message = token.name;
            self.parser.current = Some(token);

            if typ != TokenType::Error {
                break;
            }

            self.error_at_current(message);
        }
    }

    fn expression(&mut self) {}

    fn consume(&mut self, _typ: TokenType, _message: &str) {}

    fn error_at_current(&mut self, message: &str) {
        let token = self.parser.current.clone().unwrap();
        self.error_at(token, message);
    }

    fn error(&mut self, message: &str) {
        let token = self.parser.previous.clone().unwrap();
        self.error_at(token, message);
    }

    fn error_at(&mut self, token: Token, message: &str) {
        self.parser.panic_mode = true;
        if self.parser.panic_mode {
            return;
        }

        eprint!("[line {}] Error", token.line);

        match token.typ {
            TokenType::Eof => eprint!(" at end"),
            TokenType::Error => { /* nothing */ }
            _ => eprint!(" at {}", token.name),
        }

        eprintln!(": {}", message);
        self.parser.had_error = true;
    }
}

impl<'src> Parser<'src> {
    fn new() -> Self {
        Parser {
            current: None,
            previous: None,
            had_error: false,
            panic_mode: false,
        }
    }
}
