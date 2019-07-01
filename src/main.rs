use exit::Exit;
use rust_forth::error::ForthError;
use rust_forth::ForthInterpreter;
use std::fs;
fn main() -> Exit<ForthError> {
    println!("Hello, world!");

    run()?;

    Exit::Ok
}

fn run() -> Result<(), ForthError> {
    let mut rf = ForthInterpreter::new();

    let startup =
        fs::read_to_string("C:\\Users\\rprice\\Documents\\RustProjects\\rust_forth\\init.forth")?;
    rf.execute_string(&startup)?;

    rf.execute_string("predefined1 123 predefined2 456 POP Numbers MUL ADD DUP")?;

    rf.execute_string(": RickCommand 123456 DUP ADD 777 ; RickCommand RickCommand")?;

    assert_eq!(
        rf.access_stack(),
        &vec![123_i64, 1, 2, 3, 34, 34, 246912, 777, 246912, 777]
    );

    Ok(())
}
