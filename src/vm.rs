use crate::chunk::Chunk;

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

    pub fn interpret(&self, chunk: &Chunk) -> InterpretResult {
        todo!()
    }
}
