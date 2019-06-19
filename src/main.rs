use exit::Exit;
use rust_forth::ForthErr;
use rust_forth::RustForth;
use std::fs::File;

fn main() -> Exit<ForthErr> {
    println!("Hello, world!");

    run()?;

    Exit::Ok
}

fn run() -> Result<(), ForthErr> {
    let mut rf = RustForth::new();

    let f = File::open("C:\\Users\\rprice\\Documents\\RustProjects\\rust_forth\\init.forth")?;
    rf.initialize_commands_from_file(f)?;

    rf.execute_string("predefined1 123 predefined2 456 pop Numbers mul add dup")?;

    rf.execute_string(": RickCommand 123456 dup add ; RickCommand")?;

/*
    let tl = RustForth::tokenize_string("predefined1 123 predefined2 456 pop Numbers mul add dup")?;

    println!("tokenized string: {:?}", tl);

    rf.execute_token_vector(tl)?;
*/
    Ok(())
}
