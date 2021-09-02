mod chunk;
mod value;

use chunk::{Chunk, OpCode};
use value::Value;

fn main() {
    let mut chunk = Chunk::new();
    chunk.add_constant(Value(1.2));
    chunk.write_chunk(OpCode::Return);

    dbg!(&chunk);

    chunk.disassemble("test");
}
