use super::error::ForthError;
use super::state::State;

pub enum Handled {
    Handled,
    NotHandled,
}

/// This Enum lists the token types that are used by the Forth interpreter
#[derive(Debug, Clone)]
pub enum Token {
    Number(i64),
    Command(String),
    Colon(String),
    SemiColon,
}

// Chain of Command Pattern
pub trait HandleToken {
    fn handle_token(&mut self, t: &Token, st: &mut State) -> Result<Handled, ForthError>;
}

pub mod internals {
    use super::super::error::ForthError;
    use super::HandleToken;
    use super::Handled;
    use super::State;
    use super::Token;
    use std::collections::HashMap;

    pub struct ForthInternalCommandHandler {}

    impl HandleToken for ForthInternalCommandHandler {
        fn handle_token(&mut self, t: &Token, st: &mut State) -> Result<Handled, ForthError> {
            if let Token::Command(s) = t {
                println!("ForthInternalCommandHandler: Interpreting token {}", s);
                match s.as_ref() {
                    "POP" => st.number_stack.pop_stack().map(|_| Ok(Handled::Handled))?,
                    "ADD" => self.add(st).map(|_| Ok(Handled::Handled))?,
                    "SUB" => self.sub(st).map(|_| Ok(Handled::Handled))?,
                    "MUL" => self.mul(st).map(|_| Ok(Handled::Handled))?,
                    "DIV" => self.div(st).map(|_| Ok(Handled::Handled))?,
                    "DUP" => self.dup(st).map(|_| Ok(Handled::Handled))?,
                    "SWAP" => self.swap(st).map(|_| Ok(Handled::Handled))?,
                    _ => Ok(Handled::NotHandled),
                }
            } else {
                Ok(Handled::NotHandled)
            }
        }
    }

    impl ForthInternalCommandHandler {
        fn mul(&self, st: &mut State) -> Result<(), ForthError> {
            let x = st.number_stack.pop_stack()?;
            let y = st.number_stack.pop_stack()?;
            let result = x * y;

            st.number_stack.push_stack(result);

            Ok(())
        }

        fn div(&self, st: &mut State) -> Result<(), ForthError> {
            let x = st.number_stack.pop_stack()?;
            let y = st.number_stack.pop_stack()?;
            let result = x / y;

            st.number_stack.push_stack(result);

            Ok(())
        }

        fn add(&self, st: &mut State) -> Result<(), ForthError> {
            let x = st.number_stack.pop_stack()?;
            let y = st.number_stack.pop_stack()?;
            let result = x + y;

            st.number_stack.push_stack(result);

            Ok(())
        }

        fn sub(&self, st: &mut State) -> Result<(), ForthError> {
            let x = st.number_stack.pop_stack()?;
            let y = st.number_stack.pop_stack()?;
            let result = x - y;

            st.number_stack.push_stack(result);

            Ok(())
        }

        fn dup(&self, st: &mut State) -> Result<(), ForthError> {
            let x = st.number_stack.pop_stack()?;

            st.number_stack.push_stack(x);
            st.number_stack.push_stack(x);

            Ok(())
        }

        fn swap(&self, st: &mut State) -> Result<(), ForthError> {
            let x = st.number_stack.pop_stack()?;
            let y = st.number_stack.pop_stack()?;

            st.number_stack.push_stack(x);
            st.number_stack.push_stack(y);

            Ok(())
        }

        pub fn new() -> ForthInternalCommandHandler {
            ForthInternalCommandHandler {}
        }
    }

    /// This Enum determines whether the Forth interpreter is in Interpreting mode or Compiling mode
    enum Mode {
        Interpreting,
        Compiling(String),
    }
    pub struct CompiledCommands {
        command_map: HashMap<String, Vec<Token>>,
        mode: Mode,
    }

    impl HandleToken for CompiledCommands {
        fn handle_token(&mut self, t: &Token, st: &mut State) -> Result<Handled, ForthError> {
            match &self.mode {
                Mode::Interpreting => {
                    match t {
                        Token::Number(n) => st.number_stack.push_stack(*n),
                        Token::Command(s) => {
                            println!("CompiledCommands: Interpreting token {}", s);

                            match self.get_token_list_for_command(s) {
                                Result::Ok(mut tl) => {
                                    // Because we append, we need the tokens in reverse order so they can be popped in the correct order
                                    tl.reverse();

                                    st.token_stack.append(&mut tl);

                                    return Ok(Handled::Handled);
                                }
                                Result::Err(ForthError::UnknownToken(_)) => {
                                    return Ok(Handled::NotHandled)
                                }
                                Result::Err(e) => return Err(e),
                            }
                        }
                        Token::Colon(s) => {
                            println!("Colon, starting compiling");
                            self.mode = Mode::Compiling(String::from(s));
                        }
                        Token::SemiColon => {
                            panic!("Token::SemiColon case should not happen here; are you missing a prior semicolon?");
                        }
                    }

                    println!("State of number stack {:?}", st.number_stack);
                }

                Mode::Compiling(c) => match t {
                    Token::Number(n) => {
                        println!("Compiling number {}", n);
                        self.command_map
                            .entry(c.to_string())
                            .or_insert(Vec::new())
                            .push(Token::Number(*n));
                    }
                    Token::Command(s) => {
                        println!("Compiling token {}", s);
                        self.command_map
                            .entry(c.to_string())
                            .or_insert(Vec::new())
                            .push(Token::Command(s.to_string()));
                    }
                    Token::Colon(_) => {
                        panic!("Token::Colon case should not happen here");
                    }
                    Token::SemiColon => {
                        println!("SemiColon, finished compiling");
                        self.mode = Mode::Interpreting;
                    }
                },
            }

            Ok(Handled::Handled)
        }
    }

    impl CompiledCommands {
        fn get_token_list_for_command(&self, s: &str) -> Result<Vec<Token>, ForthError> {
            let tl = self.command_map.get(s);
            match tl {
                Some(tl) => Ok(tl.to_vec()),
                None => return Err(ForthError::UnknownToken(s.to_owned())),
            }
        }

        pub fn new() -> CompiledCommands {
            CompiledCommands {
                command_map: HashMap::new(),
                mode: Mode::Interpreting,
            }
        }
    }

    /// This Enum determines whether the Forth interpreter is in Interpreting mode or Compiling mode
    enum IfThenMode {
        Interpreting,
        Skipping,
    }
    pub struct IfThenCommands {
        mode: IfThenMode,
        deferral: u16,
    }

    impl HandleToken for IfThenCommands {
        fn handle_token(&mut self, t: &Token, st: &mut State) -> Result<Handled, ForthError> {
            match &self.mode {
                // IF ELSE THEN
                IfThenMode::Interpreting => match t {
                    Token::Command(s) => match s.as_ref() {
                        "IF" => {
                            if let 0 = st.number_stack.pop_stack()? {
                                self.mode = IfThenMode::Skipping
                            }
                        }
                        "ELSE" => self.mode = IfThenMode::Skipping,
                        "THEN" => return Ok(Handled::Handled),
                        _ => return Ok(Handled::NotHandled),
                    },
                    _ => return Ok(Handled::NotHandled),
                },
                IfThenMode::Skipping => match t {
                    Token::Command(s) => match s.as_ref() {
                        "IF" => self.deferral += 1,
                        "ELSE" if self.deferral == 0 => self.mode = IfThenMode::Interpreting,
                        "THEN" if self.deferral != 0 => self.deferral -= 1,
                        "THEN" if self.deferral == 0 => self.mode = IfThenMode::Interpreting,
                        _ => return Ok(Handled::Handled), // We eat commands until we are out of skipping mode
                    },
                    _ => return Ok(Handled::Handled), // We eat *whatever* until we are out of skipping mode
                },
            }

            Ok(Handled::Handled)
        }
    }

    impl IfThenCommands {
        pub fn new() -> IfThenCommands {
            IfThenCommands {
                mode: IfThenMode::Interpreting,
                deferral: 0,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_if_statement_if_part() {
        let mut rf = ForthInterpreter::new();

        rf.push_stack(123);
        rf.push_stack(321);
        rf.push_stack(1);
        rf.execute_string("IF ADD 2 MUL ELSE ADD 3 MUL THEN")
            .unwrap();
        let n = rf.pop_stack().unwrap();

        assert_eq!(n, 888);
    }

    #[test]
    fn test_if_statement_else_part() {
        let mut rf = ForthInterpreter::new();

        rf.push_stack(123);
        rf.push_stack(321);
        rf.push_stack(0);
        rf.execute_string("IF ADD 2 MUL ELSE ADD 3 MUL THEN")
            .unwrap();
        let n = rf.pop_stack().unwrap();

        assert_eq!(n, 1332);
    }

    #[test]
    fn test_compound_if_statement_if_if_part() {
        let mut rf = ForthInterpreter::new();

        rf.push_stack(123);
        rf.push_stack(321);
        rf.execute_string("1 IF 2 IF ADD 3 MUL THEN ELSE ADD 4 MUL THEN")
            .unwrap();
        let n = rf.pop_stack().unwrap();

        assert_eq!(n, 1332);
    }

    #[test]
    fn test_compound_if_statement_then_part() {
        let mut rf = ForthInterpreter::new();

        rf.push_stack(123);
        rf.push_stack(321);
        rf.execute_string("0 IF 2 IF ADD 3 MUL THEN ELSE ADD 4 MUL THEN")
            .unwrap();
        let n = rf.pop_stack().unwrap();

        assert_eq!(n, 1776);
    }
    #[test]
    fn test_compound_if_statement_no_match() {
        let mut rf = ForthInterpreter::new();

        rf.push_stack(123);
        rf.push_stack(321);
        rf.execute_string("1 IF 0 IF ADD 3 MUL THEN ELSE ADD 4 MUL THEN")
            .unwrap();
        let n = rf.pop_stack().unwrap();

        assert_eq!(n, 321);
    }

    #[test]
    fn test_compound_if_statement_then_if_part() {
        let mut rf = ForthInterpreter::new();

        rf.push_stack(123);
        rf.push_stack(321);
        rf.execute_string("0 IF 2 IF ADD 3 MUL THEN ELSE 1 IF ADD 4 MUL ELSE ADD 5 MUL THEN THEN")
            .unwrap();
        let n = rf.pop_stack().unwrap();

        assert_eq!(n, 1776);
    }

    #[test]
    fn test_compound_if_statement_then_then_part() {
        let mut rf = ForthInterpreter::new();

        rf.push_stack(123);
        rf.push_stack(321);
        rf.execute_string("0 IF 2 IF ADD 3 MUL THEN ELSE 0 IF ADD 4 MUL ELSE ADD 5 MUL THEN THEN")
            .unwrap();
        let n = rf.pop_stack().unwrap();

        assert_eq!(n, 2220);
    }
}
