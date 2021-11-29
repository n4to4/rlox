use crate::chunk::{Chunk, OpCode};
use crate::common::DEBUG_PRINT_CODE;
use crate::scanner::{Scanner, Token, TokenType};
use crate::value::Value;

pub struct Compiler<'src> {
    parser: Parser<'src>,
    scanner: Scanner<'src>,
    compiling_chunk: &'src mut Chunk,
    parse_rule_table: ParseRuleTable<'src>,
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

impl Precedence {
    fn next(&self) -> Self {
        match *self {
            Self::None => Self::Assignment,
            Self::Assignment => Self::Or,
            Self::Or => Self::And,
            Self::And => Self::Equality,
            Self::Equality => Self::Comparison,
            Self::Comparison => Self::Term,
            Self::Term => Self::Factor,
            Self::Factor => Self::Unary,
            Self::Unary => Self::Call,
            Self::Call => Self::Primary,
            Self::Primary => Self::Primary,
        }
    }
}

#[derive(Clone)]
struct ParseRule<'src> {
    prefix: Option<ParseFn<'src>>,
    infix: Option<ParseFn<'src>>,
    precedence: Precedence,
}

type ParseFn<'src> = fn(&mut Compiler<'src>);

struct ParseRuleTable<'src> {
    rules: Vec<ParseRule<'src>>,
}

impl<'src> Default for ParseRule<'src> {
    fn default() -> Self {
        ParseRule {
            prefix: None,
            infix: None,
            precedence: Precedence::None,
        }
    }
}

impl<'src> ParseRuleTable<'src> {
    fn new() -> Self {
        let mut rules = vec![ParseRule::default(); TokenType::Eof as usize + 1];

        macro_rules! rules {
            ($({ $tok:ident, { $prefix:expr, $infix:expr, $precedence:expr } }),* $(,)?) => {
                $(
                    rules[$tok as usize] = ParseRule {
                        prefix: $prefix,
                        infix: $infix,
                        precedence: $precedence,
                    };
                )*
            };
        }

        use TokenType::*;
        rules! {
            { LeftParen,    { Some(Compiler::grouping), None, Precedence::None } },
            { RightParen,   { None, None, Precedence::None } },
            { LeftBrace,    { None, None, Precedence::None } },
            { RightBrace,   { None, None, Precedence::None } },
            { Comma,        { None, None, Precedence::None } },
            { Dot,          { None, None, Precedence::None } },
            { Minus,        { Some(Compiler::unary), Some(Compiler::binary), Precedence::Term } },
            { Plus,         { None, Some(Compiler::binary), Precedence::Term } },
            { Semicolon,    { None, None, Precedence::None } },
            { Slash,        { None, Some(Compiler::binary), Precedence::Factor } },
            { Star,         { None, Some(Compiler::binary), Precedence::Factor } },
            { Bang,         { None, None, Precedence::None } },
            { BangEqual,    { None, None, Precedence::None } },
            { Equal,        { None, None, Precedence::None } },
            { EqualEqual,   { None, None, Precedence::None } },
            { Greater,      { None, None, Precedence::None } },
            { GreaterEqual, { None, None, Precedence::None } },
            { Less,         { None, None, Precedence::None } },
            { LessEqual,    { None, None, Precedence::None } },
            { Identifier,   { None, None, Precedence::None } },
            { String,       { None, None, Precedence::None } },
            { Number,       { Some(Compiler::number), None, Precedence::None } },
            { And,          { None, None, Precedence::None } },
            { Class,        { None, None, Precedence::None } },
            { Else,         { None, None, Precedence::None } },
            { False,        { None, None, Precedence::None } },
            { For,          { None, None, Precedence::None } },
            { Fun,          { None, None, Precedence::None } },
            { If,           { None, None, Precedence::None } },
            { Nil,          { None, None, Precedence::None } },
            { Or,           { None, None, Precedence::None } },
            { Print,        { None, None, Precedence::None } },
            { Return,       { None, None, Precedence::None } },
            { Super,        { None, None, Precedence::None } },
            { This,         { None, None, Precedence::None } },
            { True,         { None, None, Precedence::None } },
            { Var,          { None, None, Precedence::None } },
            { While,        { None, None, Precedence::None } },
            { Error,        { None, None, Precedence::None } },
            { Eof,          { None, None, Precedence::None } },
        };

        ParseRuleTable { rules }
    }
}

impl<'src> Compiler<'src> {
    pub fn new(source: &'src str, chunk: &'src mut Chunk) -> Self {
        Compiler {
            parser: Parser::new(),
            scanner: Scanner::new(source),
            compiling_chunk: chunk,
            parse_rule_table: ParseRuleTable::new(),
        }
    }

    pub fn compile(&mut self) -> anyhow::Result<()> {
        self.advance();
        self.expression();
        self.consume(TokenType::Eof, "Expect end of expression.");
        self.end_compiler();

        if self.parser.had_error {
            anyhow::bail!("parse error");
        }
        Ok(())
    }

    fn end_compiler(&mut self) {
        self.emit_return();
        if DEBUG_PRINT_CODE && !self.parser.had_error {
            let chunk = self.current_chunk();
            chunk.disassemble("code");
        }
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
        self.emit_constant(Value::Number(value));
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
        let rule = self.get_rule(tok.typ);
        self.parse_precedence(rule.precedence.next());

        match tok.typ {
            TokenType::Plus => self.emit_byte(OpCode::Add),
            TokenType::Minus => self.emit_byte(OpCode::Subtract),
            TokenType::Star => self.emit_byte(OpCode::Multiply),
            TokenType::Slash => self.emit_byte(OpCode::Divide),
            _ => unreachable!(),
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();

        let rule = self.get_rule(self.parser.previous.clone().unwrap().typ);
        if let Some(prefix_rule) = rule.prefix {
            prefix_rule(self);
        } else {
            self.error("Expect expression.");
            return;
        }

        loop {
            let rule = self.get_rule(self.parser.current.clone().unwrap().typ);
            if precedence > rule.precedence {
                break;
            }
            self.advance();
            let rule = self.get_rule(self.parser.previous.clone().unwrap().typ);
            if let Some(infix_rule) = rule.infix {
                infix_rule(self);
            }
        }
    }

    fn get_rule(&self, typ: TokenType) -> ParseRule<'src> {
        self.parse_rule_table
            .rules
            .get(typ as usize)
            .expect("get_rule")
            .clone()
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
        self.compiling_chunk
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
