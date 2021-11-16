use crate::scanner::Scanner;

pub struct Compiler;

impl Compiler {
    pub fn compile(source: &str) {
        let scanner = Scanner::new(source);
    }
}
