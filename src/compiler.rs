use crate::chunk::Chunk;
use crate::scanner::{Scanner, TokenType};

pub struct Compiler;

impl Compiler {
    pub fn new() -> Self {
        Compiler
    }

    pub fn compile(&mut self, source: &str, _chunk: &mut Chunk) -> anyhow::Result<()> {
        let mut _scanner = Scanner::new(source);
        self.advance();
        self.expression();
        self.consume(TokenType::Eof, "Expect end of expression.");
        Ok(())
    }

    fn advance(&mut self) {}

    fn expression(&mut self) {}

    fn consume(&mut self, _typ: TokenType, _message: &str) {}
}
