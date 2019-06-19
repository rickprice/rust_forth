use exit::Exit;
use rust_forth::ForthError;
use rust_forth::RustForth;
use std::fs::File;

fn main() -> Exit<ForthError> {
    println!("Hello, world!");

    run()?;

    Exit::Ok
}

fn run() -> Result<(), ForthError> {
    let mut rf = RustForth::new();

    let f = File::open("C:\\Users\\rprice\\Documents\\RustProjects\\rust_forth\\init.forth")?;
    rf.execute_commands_from_file(f)?;

    rf.execute_string("predefined1 123 predefined2 456 pop Numbers mul add dup")?;

    rf.execute_string(": RickCommand 123456 dup add 777 ; RickCommand RickCommand")?;

    Ok(())
}
