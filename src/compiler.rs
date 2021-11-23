use crate::chunk::{Chunk, OpCode};
use crate::scanner::{Scanner, Token, TokenType};
use crate::value::Value;

pub struct Compiler<'src> {
    parser: Parser<'src>,
    scanner: Scanner<'src>,
    compiling_chunk: Chunk,
}

struct Parser<'src> {
    current: Option<Token<'src>>,
    previous: Option<Token<'src>>,
    had_error: bool,
    panic_mode: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    None = 0,
    Assignment, // =
    Or,         // or
    And,        // and
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Unary,      // ! -
    Call,       // . ()
    Primary,
}

struct ParseRule {
    prefix: ParseFn,
    infix: ParseFn,
    precedence: Precedence,
}

type ParseFn = fn();

impl<'src> Compiler<'src> {
    pub fn new(source: &'src str, _chunk: &mut Chunk) -> Self {
        Compiler {
            parser: Parser::new(),
            scanner: Scanner::new(source),
            compiling_chunk: Chunk::new(),
        }
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

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn number(&mut self) {
        let tok = self.parser.previous.clone().expect("number");
        let value: f64 = tok.name.parse().expect("number");
        self.emit_constant(Value(value));
    }

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn unary(&mut self) {
        let tok = self
            .parser
            .previous
            .clone()
            .expect("parser previous is None");

        // Compile the operand.
        self.parse_precedence(Precedence::Unary);

        // Emit the operator instruction.
        match tok.typ {
            TokenType::Minus => self.emit_byte(OpCode::Negate),
            _ => unreachable!(),
        }
    }

    fn binary(&mut self) {
        let tok = self.parser.previous.clone().unwrap();
        //let rule = self.get_rule(tok.typ);
        //self.parse_precedence(rule.precedence + 1);

        match tok.typ {
            TokenType::Plus => self.emit_byte(OpCode::Add),
            TokenType::Minus => self.emit_byte(OpCode::Subtract),
            TokenType::Star => self.emit_byte(OpCode::Multiply),
            TokenType::Slash => self.emit_byte(OpCode::Divide),
            _ => unreachable!(),
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        todo!()
    }

    fn consume(&mut self, typ: TokenType, message: &str) {
        if let Some(tok) = &self.parser.current {
            if tok.typ == typ {
                self.advance();
                return;
            }
        }
        self.error_at_current(message);
    }

    fn emit_byte(&mut self, byte: OpCode) {
        let line = self.parser.previous.clone().unwrap().line as i32;
        let chunk = self.current_chunk_mut();
        chunk.write_chunk(byte, line);
    }

    fn emit_bytes(&mut self, bytes: &[OpCode]) {
        for byte in bytes {
            self.emit_byte(*byte);
        }
    }

    fn emit_constant(&mut self, value: Value) {
        let constant = self.make_constant(value);
        self.emit_byte(OpCode::Constant(constant));
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let chunk = self.current_chunk_mut();
        let constant = chunk.add_constant(value);
        if constant > u8::MAX as usize {
            self.error("Too many constants in one chunk.");
            return 0;
        }
        constant as u8
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::Return);
    }

    fn current_chunk(&self) -> &Chunk {
        &self.compiling_chunk
    }

    fn current_chunk_mut(&mut self) -> &mut Chunk {
        &mut self.compiling_chunk
    }

    fn error_at_current(&mut self, message: &str) {
        let token = self.parser.current.clone().expect("parser.current is None");
        self.error_at(token, message);
    }

    fn error(&mut self, message: &str) {
        let token = self
            .parser
            .previous
            .clone()
            .expect("parser.previous is None");
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

impl<'src> Drop for Compiler<'src> {
    fn drop(&mut self) {
        self.emit_return();
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
