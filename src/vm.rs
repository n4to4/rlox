use crate::chunk::{disassemble_instruction, Chunk, OpCode};
use crate::value::Value;

const DEBUG_TRACE_EXECUTION: bool = true;

pub struct VM {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
}

pub enum InterpretResult {
    InterpretOk,
    InterpretCompileError,
    InterpretRuntimeError,
}

impl VM {
    pub fn new(chunk: Chunk) -> Self {
        VM {
            chunk,
            ip: 0,
            stack: Vec::new(),
        }
    }

    pub fn run(&mut self) -> InterpretResult {
        loop {
            if DEBUG_TRACE_EXECUTION {
                disassemble_instruction(&self.chunk, self.ip);
                dbg!(&self.stack);
            }

            match self.chunk.code[self.ip] {
                OpCode::Constant(idx) => {
                    let constant = self.read_const(idx as usize);
                    self.push(constant);
                }
                OpCode::Return => {
                    if let Some(value) = self.pop() {
                        println!("{}", value);
                    }
                    return InterpretResult::InterpretOk;
                }
            }
            self.ip += 1
        }
    }

    fn read_const(&self, idx: usize) -> Value {
        self.chunk.constants.values[idx]
    }

    // Stack

    fn reset_stack(&mut self) {
        self.stack.clear();
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Option<Value> {
        self.stack.pop()
    }
}
