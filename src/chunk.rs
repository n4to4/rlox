use crate::value::{Value, ValueArray};

#[derive(Debug, Clone, Copy)]
pub(crate) enum OpCode {
    Constant(u8),
    Return,
}

#[derive(Debug)]
pub(crate) struct Chunk {
    code: Vec<OpCode>,
    lines: Vec<i32>,
    constants: ValueArray,
}

impl Chunk {
    pub(crate) fn new() -> Self {
        Chunk {
            code: Vec::new(),
            lines: Vec::new(),
            constants: ValueArray::new(),
        }
    }

    pub(crate) fn write_chunk(&mut self, byte: OpCode, line: i32) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub(crate) fn add_constant(&mut self, value: Value) -> usize {
        self.constants.write_value_array(value);
        self.constants.len() - 1
    }

    pub(crate) fn disassemble(&self, name: &str) {
        println!("=== {} ===", name);

        for (offset, c) in self.code.iter().enumerate() {
            print!("{:04} ", offset);
            match *c {
                OpCode::Return => println!("{:?}", OpCode::Return),
                OpCode::Constant(off) => {
                    println!("Constant {:}", self.constants[off].0);
                }
            }
        }
    }
}
