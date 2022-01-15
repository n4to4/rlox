use std::collections::HashSet;

use crate::chunk::{disassemble_instruction, Chunk, OpCode};
use crate::common::DEBUG_TRACE_EXECUTION;
use crate::compiler::Compiler;
use crate::object::*;
use crate::value::Value;

pub struct VM {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
    strings: HashSet<String>, // intern
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
            strings: HashSet::new(),
        }
    }

    pub fn new_string(&mut self, s: impl Into<String>) -> Value {
        let s = s.into();
        self.strings.insert(s.clone());
        Value::new_string(s)
    }

    pub fn interpret(&mut self, source: &str) -> anyhow::Result<(), InterpretError> {
        let mut chunk = Chunk::new();
        let mut compiler = Compiler::new(self, source, &mut chunk);
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
                OpCode::Add => {
                    let b = self.peek(1).expect("empty stack");
                    let a = self.peek(1).expect("empty stack");

                    match (a, b) {
                        (Value::Number(_), Value::Number(_)) => {
                            self.number_binop(|a, b| Value::Number(a + b))?
                        }
                        (Value::Obj(_), Value::Obj(_)) => {
                            let b = self.pop().expect("empty stack");
                            let a = self.pop().expect("empty stack");
                            match (a.as_obj(), b.as_obj()) {
                                (Some(Object::String(str_a)), Some(Object::String(str_b))) => {
                                    let new = str_a.to_owned() + str_b;
                                    let string = self.new_string(new);
                                    self.push(string);
                                }
                                _ => {
                                    self.runtime_error("Operands must be strings.");
                                    return Err(InterpretError::RuntimeError);
                                }
                            }
                        }
                        _ => {
                            self.runtime_error("Operands must be numbers.");
                            return Err(InterpretError::RuntimeError);
                        }
                    }
                }
                OpCode::Subtract => self.number_binop(|a, b| Value::Number(a - b))?,
                OpCode::Multiply => self.number_binop(|a, b| Value::Number(a * b))?,
                OpCode::Divide => self.number_binop(|a, b| Value::Number(a / b))?,
                OpCode::Greater => self.number_binop(|a, b| Value::Boolean(a > b))?,
                OpCode::Less => self.number_binop(|a, b| Value::Boolean(a < b))?,

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

    fn number_binop<F>(&mut self, f: F) -> Result<(), InterpretError>
    where
        F: Fn(f64, f64) -> Value,
    {
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
        self.push(f(a, b));
        Ok(())
    }

    fn read_const(&self, idx: usize) -> Value {
        self.chunk.constants.values[idx].clone()
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
            Some(self.stack[self.stack.len() - offset].clone())
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
        Value::Obj(obj_a) => {
            matches!(b, Value::Obj(obj_b) if Object::values_equal(obj_a.as_ref(), obj_b.as_ref()))
        }
    }
}
