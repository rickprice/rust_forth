use exit::Exit;
use rust_forth::error::ForthError;
use rust_forth::RustForth;
use std::fs;
fn main() -> Exit<ForthError> {
    println!("Hello, world!");

    run()?;

    Exit::Ok
}

fn run() -> Result<(), ForthError> {
    let mut rf = RustForth::new();

    let startup =
        fs::read_to_string("C:\\Users\\rprice\\Documents\\RustProjects\\rust_forth\\init.forth")?;
    rf.execute_string(&startup)?;

    rf.execute_string("predefined1 123 predefined2 456 pop Numbers mul add dup")?;

    rf.execute_string(": RickCommand 123456 dup add 777 ; RickCommand RickCommand")?;

    Ok(())
}
