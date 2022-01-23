use crate::chunk::{Chunk, OpCode};
use crate::common::DEBUG_PRINT_CODE;
use crate::scanner::{Scanner, Token, TokenType};
use crate::value::Value;
use crate::vm::VM;

pub struct Compiler<'src> {
    vm: &'src mut VM,
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
    prefix_name: &'static str,
    prefix: Option<ParseFn<'src>>,
    infix_name: &'static str,
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
            prefix_name: "",
            prefix: None,
            infix_name: "",
            infix: None,
            precedence: Precedence::None,
        }
    }
}

impl<'src> std::fmt::Debug for ParseRule<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //f.debug_struct("ParseRule")
        //    .field("prefix", self.prefix_name)
        //    .field("precedence", &self.precedence)
        //    .finish()
        write!(
            f,
            "<ParseRule prefix:{}, infix:{}, precedence:{:?}>",
            self.prefix_name, self.infix_name, &self.precedence
        )
    }
}

impl<'src> ParseRuleTable<'src> {
    fn new() -> Self {
        let mut rules = vec![ParseRule::default(); TokenType::Eof as usize + 1];

        macro_rules! rules {
            ($({ $tok:ident, { $prefix:expr, $infix:expr, $precedence:expr } }),* $(,)?) => {
                $(
                    rules[$tok as usize] = ParseRule {
                        prefix_name: stringify!($prefix),
                        prefix: $prefix,
                        infix_name: stringify!($infix),
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
            { Bang,         { Some(Compiler::unary), None, Precedence::None } },
            { BangEqual,    { None, Some(Compiler::binary), Precedence::Equality } },
            { Equal,        { None, None, Precedence::None } },
            { EqualEqual,   { None, Some(Compiler::binary), Precedence::Equality } },
            { Greater,      { None, Some(Compiler::binary), Precedence::Comparison } },
            { GreaterEqual, { None, Some(Compiler::binary), Precedence::Comparison } },
            { Less,         { None, Some(Compiler::binary), Precedence::Comparison } },
            { LessEqual,    { None, Some(Compiler::binary), Precedence::Comparison } },
            { Identifier,   { None, None, Precedence::None } },
            { String,       { Some(Compiler::string), None, Precedence::None } },
            { Number,       { Some(Compiler::number), None, Precedence::None } },
            { And,          { None, None, Precedence::None } },
            { Class,        { None, None, Precedence::None } },
            { Else,         { None, None, Precedence::None } },
            { False,        { Some(Compiler::literal), None, Precedence::None } },
            { For,          { None, None, Precedence::None } },
            { Fun,          { None, None, Precedence::None } },
            { If,           { None, None, Precedence::None } },
            { Nil,          { Some(Compiler::literal), None, Precedence::None } },
            { Or,           { None, None, Precedence::None } },
            { Print,        { None, None, Precedence::None } },
            { Return,       { None, None, Precedence::None } },
            { Super,        { None, None, Precedence::None } },
            { This,         { None, None, Precedence::None } },
            { True,         { Some(Compiler::literal), None, Precedence::None } },
            { Var,          { None, None, Precedence::None } },
            { While,        { None, None, Precedence::None } },
            { Error,        { None, None, Precedence::None } },
            { Eof,          { None, None, Precedence::None } },
        };

        ParseRuleTable { rules }
    }
}

impl<'src> Compiler<'src> {
    pub fn new(vm: &'src mut VM, source: &'src str, chunk: &'src mut Chunk) -> Self {
        Compiler {
            vm,
            parser: Parser::new(),
            scanner: Scanner::new(source),
            compiling_chunk: chunk,
            parse_rule_table: ParseRuleTable::new(),
        }
    }

    pub fn compile(&mut self) -> anyhow::Result<()> {
        self.advance();

        while !self.matches(TokenType::Eof) {
            self.statement();
        }

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
            let token = dbg!(self.scanner.scan_token());
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

    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.");

        if self.matches(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_byte(OpCode::Nil);
        }
        self.consume(
            TokenType::Semicolon,
            "Expect ';' after variable declaration",
        );

        self.define_variable(global);
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after expression.");
        self.emit_byte(OpCode::Pop);
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::Semicolon, "Expect ';' after value.");
        self.emit_byte(OpCode::Print);
    }

    fn synchronize(&mut self) {
        self.parser.panic_mode = false;

        while self.parser.current.clone().unwrap().typ != TokenType::Eof {
            if self.parser.previous.clone().unwrap().typ == TokenType::Semicolon {
                return;
            }
            match self.parser.current.clone().unwrap().typ {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => {
                    return;
                }
                _ => {
                    // do nothing.
                }
            }

            self.advance();
        }
    }

    #[allow(dead_code)]
    fn declaration(&mut self) {
        if self.matches(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if self.parser.panic_mode {
            self.synchronize();
        }
    }

    fn statement(&mut self) {
        if self.matches(TokenType::Print) {
            self.print_statement();
        } else {
            self.expression_statement();
        }
    }

    fn number(&mut self) {
        let tok = self.parser.previous.clone().expect("number");
        let value: f64 = tok.name.parse().expect("number");
        self.emit_constant(Value::Number(value));
    }

    fn string(&mut self) {
        let tok = self.parser.previous.clone().expect("string");
        let len = tok.name.len();
        //self.emit_constant(Value::new_string(&tok.name[1..len - 1]));
        let string = self.vm.new_string(&tok.name[1..len - 1]);
        self.emit_constant(string);
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
            TokenType::Bang => self.emit_byte(OpCode::Not),
            TokenType::Minus => self.emit_byte(OpCode::Negate),
            _ => unreachable!(),
        }
    }

    fn binary(&mut self) {
        let tok = self.parser.previous.clone().unwrap();
        let rule = self.get_rule(tok.typ);
        self.parse_precedence(rule.precedence.next());

        match tok.typ {
            TokenType::BangEqual => self.emit_bytes(&[OpCode::Equal, OpCode::Not]),
            TokenType::EqualEqual => self.emit_byte(OpCode::Equal),
            TokenType::Greater => self.emit_byte(OpCode::Greater),
            TokenType::GreaterEqual => self.emit_bytes(&[OpCode::Less, OpCode::Not]),
            TokenType::Less => self.emit_byte(OpCode::Less),
            TokenType::LessEqual => self.emit_bytes(&[OpCode::Greater, OpCode::Not]),
            TokenType::Plus => self.emit_byte(OpCode::Add),
            TokenType::Minus => self.emit_byte(OpCode::Subtract),
            TokenType::Star => self.emit_byte(OpCode::Multiply),
            TokenType::Slash => self.emit_byte(OpCode::Divide),
            _ => unreachable!(),
        }
    }

    fn literal(&mut self) {
        match self.parser.previous.clone().unwrap().typ {
            TokenType::False => self.emit_byte(OpCode::False),
            TokenType::Nil => self.emit_byte(OpCode::Nil),
            TokenType::True => self.emit_byte(OpCode::True),
            _ => unreachable!(),
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();

        let rule = self.get_rule(self.parser.previous.clone().unwrap().typ);
        dbg!(&rule);
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

    fn parse_variable(&mut self, error_message: &str) -> u8 {
        self.consume(TokenType::Identifier, error_message);
        let p = self.parser.previous.clone().unwrap();
        self.identifier_constant(&p)
    }

    fn identifier_constant(&mut self, name: &Token) -> u8 {
        let v = self.vm.new_string(name.name);
        self.make_constant(v)
    }

    fn define_variable(&mut self, global: u8) {
        self.emit_byte(OpCode::DefineGlobal(global));
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

    fn matches(&mut self, typ: TokenType) -> bool {
        if !self.check(typ) {
            return false;
        }
        self.advance();
        true
    }

    fn check(&self, typ: TokenType) -> bool {
        let tok = self.parser.current.clone().unwrap();
        tok.typ == typ
    }

    fn emit_byte(&mut self, byte: OpCode) {
        let line = self.parser.previous.clone().unwrap().line as i32;
        let chunk = self.current_chunk_mut();
        chunk.write_chunk(byte, line);
    }

    fn emit_bytes(&mut self, bytes: &[OpCode]) {
        let line = self.parser.previous.clone().unwrap().line as i32;
        let chunk = self.current_chunk_mut();
        bytes.iter().for_each(|&b| {
            chunk.write_chunk(b, line);
        })
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
        self.compiling_chunk
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
