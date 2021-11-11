use crate::chunk::{Chunk, OpCode};
use crate::value::Value;

pub struct VM {
    chunk: Chunk,
    ip: usize,
}

pub enum InterpretResult {
    InterpretOk,
    InterpretCompileError,
    InterpretRuntimeError,
}

impl VM {
    pub fn new(chunk: Chunk) -> Self {
        VM { chunk, ip: 0 }
    }

    pub fn run(&mut self) -> InterpretResult {
        loop {
            match self.chunk.code[self.ip] {
                OpCode::Constant(idx) => {
                    let constant = self.read_const(idx as usize);
                    println!("{}", &constant);
                }
                OpCode::Return => {
                    return InterpretResult::InterpretOk;
                }
            }
            self.ip += 1
        }
    }

    fn read_const(&self, idx: usize) -> Value {
        self.chunk.constants.values[idx]
    }
}
