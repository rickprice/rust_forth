use exit::Exit;
use rust_forth::ForthErr;

fn main() -> Exit<ForthErr> {   
     println!("Hello, world!");

    rust_forth::run()?;

    Exit::Ok
}
