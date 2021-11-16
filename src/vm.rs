use crate::chunk::{disassemble_instruction, Chunk, OpCode};
use crate::compiler::Compiler;
use crate::value::Value;

const DEBUG_TRACE_EXECUTION: bool = true;

pub struct VM {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
}

#[derive(Debug, thiserror::Error)]
pub enum InterpretError {
    #[error("Compile error")]
    CompileError,
    #[error("Runtime error")]
    RuntimeError,
}

impl VM {
    pub fn new() -> Self {
        VM {
            chunk: Chunk::new(),
            ip: 0,
            stack: Vec::new(),
        }
    }

    pub fn interpret(&mut self, source: &str) -> anyhow::Result<(), InterpretError> {
        Compiler::compile(source);
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), InterpretError> {
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
                op @ (OpCode::Add | OpCode::Subtract | OpCode::Multiply | OpCode::Divide) => {
                    let b = self.pop().expect("empty stack");
                    let a = self.pop().expect("empty stack");
                    let ret = match op {
                        OpCode::Add => a.0 + b.0,
                        OpCode::Subtract => a.0 - b.0,
                        OpCode::Multiply => a.0 * b.0,
                        OpCode::Divide => a.0 / b.0,
                        _ => unreachable!(),
                    };
                    self.push(Value(ret));
                }
                OpCode::Negate => {
                    let v = self.pop().expect("empty stack");
                    self.push(Value(-v.0));
                }
                OpCode::Return => {
                    if let Some(value) = self.pop() {
                        println!("{}", value);
                    }
                    return Ok(());
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
