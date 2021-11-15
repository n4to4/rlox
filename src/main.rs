use rlox::vm::VM;
use std::io::BufRead;

fn main() -> anyhow::Result<()> {
    let mut it = std::env::args();
    if it.len() == 1 {
        repl()?;
    } else if it.len() == 2 {
        run_file(&it.next().unwrap())?;
    } else {
        eprintln!("Usage: rlox [path]");
    }
    Ok(())
}

fn repl() -> std::io::Result<()> {
    let mut vm = VM::new();
    let stdin = std::io::stdin();
    let mut line = String::new();
    let mut handle = stdin.lock();

    loop {
        print!("> ");
        let n = handle.read_line(&mut line)?;
        if n == 0 || line.trim().is_empty() {
            return Ok(());
        }

        let _ = vm.interpret(&line);
    }
}

fn run_file(path: &str) -> anyhow::Result<()> {
    let mut vm = VM::new();
    let source = std::fs::read_to_string(path)?;
    vm.interpret(&source)?;
    Ok(())
}
