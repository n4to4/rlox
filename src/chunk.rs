#[derive(Debug, Clone, Copy)]
enum OpCode {
    Return,
}

#[derive(Debug)]
struct Chunk {
    code: Vec<OpCode>,
}

impl Chunk {
    fn new() -> Self {
        Chunk { code: Vec::new() }
    }

    fn write_chunk(&mut self, byte: OpCode) {
        self.code.push(byte);
    }
}
