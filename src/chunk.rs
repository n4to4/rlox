#[derive(Debug, Clone, Copy)]
pub(crate) enum OpCode {
    Return,
}

impl OpCode {
    pub(crate) fn disassemble(&self, offset: usize) {
        println!("{:04} {:?}", offset, self);
    }
}

#[derive(Debug)]
pub(crate) struct Chunk {
    code: Vec<OpCode>,
}

impl Chunk {
    pub(crate) fn new() -> Self {
        Chunk { code: Vec::new() }
    }

    pub(crate) fn write_chunk(&mut self, byte: OpCode) {
        self.code.push(byte);
    }

    pub(crate) fn disassemble(&self, name: &str) {
        println!("=== {} ===", name);

        for (offset, c) in self.code.iter().enumerate() {
            c.disassemble(offset);
        }
    }
}
