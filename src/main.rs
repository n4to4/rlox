mod chunk;

use chunk::{Chunk, OpCode};

fn main() {
    let mut chunk = Chunk::new();
    chunk.write_chunk(OpCode::Return);

    dbg!(&chunk);

    chunk.disassemble("test");
}
