use crate::scanner::Scanner;

pub struct Compiler;

impl Compiler {
    pub fn compile(source: &str) {
        #[allow(unused_variables)]
        let scanner = Scanner::new(source);
    }
}
