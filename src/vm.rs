use crate::chunk::{disassemble_instruction, Chunk, OpCode};
use crate::common::DEBUG_TRACE_EXECUTION;
use crate::compiler::Compiler;
use crate::value::Value;

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
        let mut chunk = Chunk::new();
        let mut compiler = Compiler::new(source, &mut chunk);
        compiler
            .compile()
            .map_err(|_err| InterpretError::CompileError)?;

        let chunk = chunk;
        self.chunk = chunk;
        self.ip = 0;

        self.run()
    }

    pub fn run(&mut self) -> Result<(), InterpretError> {
        loop {
            if DEBUG_TRACE_EXECUTION {
                disassemble_instruction(&self.chunk, self.ip);
            }

            match self.chunk.code[self.ip] {
                OpCode::Constant(idx) => {
                    let constant = self.read_const(idx as usize);
                    self.push(constant);
                }
                op @ (OpCode::Add | OpCode::Subtract | OpCode::Multiply | OpCode::Divide) => {
                    let b = match self.pop().expect("empty stack") {
                        Value::Number(number) => number,
                        _ => todo!("not implemented yet"),
                    };
                    let a = match self.pop().expect("empty stack") {
                        Value::Number(number) => number,
                        _ => todo!("not implemented yet"),
                    };
                    let ret = match op {
                        OpCode::Add => a + b,
                        OpCode::Subtract => a - b,
                        OpCode::Multiply => a * b,
                        OpCode::Divide => a / b,
                        _ => unreachable!(),
                    };
                    self.push(Value::Number(ret));
                }
                OpCode::Negate => {
                    let v = match self.pop().expect("empty stack") {
                        Value::Number(number) => number,
                        _ => todo!("not implemented yet"),
                    };
                    self.push(Value::Number(-v));
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

    #[allow(dead_code)]
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
