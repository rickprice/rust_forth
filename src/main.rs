use exit::Exit;
use rust_forth::ForthError;
use rust_forth::ForthInterpreter;
use rust_forth::HandleToken;
use rust_forth::Handled;
use rust_forth::State;
use rust_forth::Token;
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

    rf.token_handlers
        .push(Box::new(ExternalCommandHandler::new()));

    rf.execute_string("1111 123456 OUT 123456 IN")?;

    assert_eq!(
        rf.access_stack(),
        &vec![123_i64, 1, 2, 3, 34, 34, 246912, 777, 246912, 777, 777]
    );

    Ok(())
}

pub struct ExternalCommandHandler {}

impl HandleToken for ExternalCommandHandler {
    fn handle_token(&mut self, t: &Token, st: &mut State) -> Result<Handled, ForthError> {
        if let Token::Command(s) = t {
            println!("ExternalCommandHandler: Interpreting token {}", s);
            match s.as_ref() {
                "OUT" => self.out_port(st).map(|_| Ok(Handled::Handled))?,
                "IN" => self.in_port(st).map(|_| Ok(Handled::Handled))?,
                _ => Ok(Handled::NotHandled),
            }
        } else {
            Ok(Handled::NotHandled)
        }
    }
}

impl ExternalCommandHandler {
    fn out_port(&self, st: &mut State) -> Result<(), ForthError> {
        let port = st.number_stack.pop_stack()?;
        let value = st.number_stack.pop_stack()?;

        println!("Sending {} to port {}", value, port);

        Ok(())
    }

    fn in_port(&self, st: &mut State) -> Result<(), ForthError> {
        let port = st.number_stack.pop_stack()?;
        let value = 777;

        st.number_stack.push_stack(value);

        println!("Receiving {} from port {}", value, port);

        Ok(())
    }

    pub fn new() -> ExternalCommandHandler {
        ExternalCommandHandler {}
    }
}
