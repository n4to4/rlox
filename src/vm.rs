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
        dbg!(&self.chunk);

        loop {
            if DEBUG_TRACE_EXECUTION {
                disassemble_instruction(&self.chunk, self.ip);
            }

            let op = self.chunk.code[self.ip];
            match op {
                OpCode::Constant(idx) => {
                    let constant = self.read_const(idx as usize);
                    self.push(constant);
                }
                OpCode::Nil => self.push(Value::Nil),
                OpCode::True => self.push(Value::Boolean(true)),
                OpCode::False => self.push(Value::Boolean(false)),
                OpCode::Equal => {
                    let a = self.pop().expect("empty stack");
                    let b = self.pop().expect("empty stack");
                    self.push(Value::Boolean(values_equal(a, b)));
                }
                OpCode::Add
                | OpCode::Subtract
                | OpCode::Multiply
                | OpCode::Divide
                | OpCode::Greater
                | OpCode::Less => {
                    let b = match self.pop().expect("empty stack") {
                        Value::Number(number) => number,
                        _ => {
                            self.runtime_error("Operands must be numbers.");
                            return Err(InterpretError::RuntimeError);
                        }
                    };
                    let a = match self.pop().expect("empty stack") {
                        Value::Number(number) => number,
                        _ => {
                            self.runtime_error("Operands must be numbers.");
                            return Err(InterpretError::RuntimeError);
                        }
                    };
                    let val = match op {
                        OpCode::Add => Value::Number(a + b),
                        OpCode::Subtract => Value::Number(a - b),
                        OpCode::Multiply => Value::Number(a * b),
                        OpCode::Divide => Value::Number(a / b),
                        OpCode::Greater => Value::Boolean(a > b),
                        OpCode::Less => Value::Boolean(a < b),
                        _ => unreachable!(),
                    };
                    self.push(val);
                }
                OpCode::Not => {
                    let val = self.pop().expect("empty stack");
                    self.push(Value::Boolean(is_falsey(val)));
                }
                OpCode::Negate => {
                    let v = match self.pop().expect("empty stack") {
                        Value::Number(number) => number,
                        _ => {
                            self.runtime_error("Operand must be a number.");
                            return Err(InterpretError::RuntimeError);
                        }
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

    fn reset_stack(&mut self) {
        self.stack.clear();
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Option<Value> {
        self.stack.pop()
    }

    #[allow(dead_code)]
    fn peek(&self, distance: usize) -> Option<Value> {
        let offset = 1 + distance;
        if offset <= self.stack.len() {
            Some(self.stack[self.stack.len() - offset])
        } else {
            None
        }
    }

    // Error

    fn runtime_error(&mut self, message: &str) {
        eprintln!("{}", message);
        let line = self.chunk.lines[self.ip];
        eprintln!("[line {}] in script", line);
        self.reset_stack();
    }
}

fn is_falsey(value: Value) -> bool {
    matches!(value, Value::Nil | Value::Boolean(false))
}

fn values_equal(a: Value, b: Value) -> bool {
    match a {
        Value::Boolean(val_a) => matches!(b, Value::Boolean(val_b) if val_a == val_b),
        Value::Number(val_a) => matches!(b, Value::Number(val_b) if val_a == val_b),
        Value::Nil => matches!(b, Value::Nil),
    }
}
