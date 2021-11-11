use rlox::chunk::{Chunk, OpCode};
use rlox::value::Value;
use rlox::vm::VM;

fn main() {
    let mut chunk = Chunk::new();
    let constant = chunk.add_constant(Value(1.2));
    chunk.write_chunk(OpCode::Constant(constant as u8), 123);
    chunk.write_chunk(OpCode::Return, 123);

    dbg!(&chunk);
    chunk.disassemble("test");

    println!("{}", "=".repeat(80));

    let mut vm = VM::new(chunk);
    vm.run();
}
