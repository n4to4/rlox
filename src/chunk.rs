use crate::value::{Value, ValueArray};

#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    Constant(u8),
    Nil,
    True,
    False,
    Pop,
    GetLocal(u8),
    GetGlobal(u8),
    DefineGlobal(u8),
    SetLocal(u8),
    SetGlobal(u8),
    Equal,
    Greater,
    Less,
    Add,
    Subtract,
    Multiply,
    Divide,
    Not,
    Negate,
    Print,
    Return,
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub code: Vec<OpCode>,
    pub lines: Vec<i32>,
    pub constants: ValueArray,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: Vec::new(),
            lines: Vec::new(),
            constants: ValueArray::new(),
        }
    }

    pub fn write_chunk(&mut self, byte: OpCode, line: i32) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.write_value_array(value);
        self.constants.len() - 1
    }

    pub fn disassemble(&self, name: &str) {
        println!("=== {} ===", name);

        for offset in 0..self.code.len() {
            disassemble_instruction(self, offset);
        }
    }
}

pub fn disassemble_instruction(chunk: &Chunk, offset: usize) {
    print!("{:04} ", offset);

    if offset > 0 && chunk.lines[offset] == chunk.lines[offset - 1] {
        print!("   | ");
    } else {
        print!("{:4} ", chunk.lines[offset]);
    }
    match chunk.code[offset] {
        OpCode::Constant(off) => {
            println!("Constant {}", chunk.constants[off]);
        }
        _ => println!("{:?}", &chunk.code[offset]),
    }
}
