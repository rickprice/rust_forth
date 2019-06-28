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

fn execute_token(th: Vec<Box<HandleToken>>,t: &Token, st: &mut State) -> Result<(), ForthError> {
    for th in th.iter_mut() {
        if let Handled::Handled = th.handle_token(t, st)? {
            break;
        }
    }

    Ok(())
}

mod internals {
    use super::super::error::ForthError;
    use super::HandleToken;
    use super::Handled;
    use super::State;
    use super::Token;
    use super::execute_token;
    use std::collections::HashMap;

    pub struct ForthInternalCommandHandler {}

    impl HandleToken for ForthInternalCommandHandler {
        fn handle_token(&mut self, t: &Token, st: &mut State) -> Result<Handled, ForthError> {
            if let Token::Command(s) = t {
                println!("Interpreting token {}", s);
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

            println!("Multiplied {} by {} resulting in {}", x, y, result);

            Ok(())
        }

        fn div(&self, st: &mut State) -> Result<(), ForthError> {
            let x = st.number_stack.pop_stack()?;
            let y = st.number_stack.pop_stack()?;
            let result = x / y;

            st.number_stack.push_stack(result);

            println!("Divided {} by {} resulting in {}", x, y, result);

            Ok(())
        }

        fn add(&self, st: &mut State) -> Result<(), ForthError> {
            let x = st.number_stack.pop_stack()?;
            let y = st.number_stack.pop_stack()?;
            let result = x + y;

            st.number_stack.push_stack(result);

            println!("Added {} to {} resulting in {}", x, y, result);

            Ok(())
        }

        fn sub(&self, st: &mut State) -> Result<(), ForthError> {
            let x = st.number_stack.pop_stack()?;
            let y = st.number_stack.pop_stack()?;
            let result = x - y;

            st.number_stack.push_stack(result);

            println!("Subtracted {} by {} resulting in {}", x, y, result);

            Ok(())
        }

        fn dup(&self, st: &mut State) -> Result<(), ForthError> {
            let x = st.number_stack.pop_stack()?;

            st.number_stack.push_stack(x);
            st.number_stack.push_stack(x);

            println!("Duplicated {} ", x);

            Ok(())
        }

        fn swap(&self, st: &mut State) -> Result<(), ForthError> {
            let x = st.number_stack.pop_stack()?;
            let y = st.number_stack.pop_stack()?;

            st.number_stack.push_stack(x);
            st.number_stack.push_stack(y);

            println!("Swapped top items on stack {} {}", x, y);

            Ok(())
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
                            println!("Interpreting token {}", s);
                            self.execute_token_by_name(s, st)?
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

            Ok(Handled::NotHandled)
        }
    }
    impl CompiledCommands {
        fn execute_token_by_name(&mut self, s: &str, st: &mut State) -> Result<(), ForthError> {
            let tl = self.get_token_list_for_command(s)?;

            println!("Executing token list {:?} for {}", tl, s);
            self.execute_token_vector(tl, st)?;
            Ok(())
        }

        fn execute_token_vector(
            &mut self,
            tl: Vec<Token>,
            st: &mut State,
        ) -> Result<(), ForthError> {
            println!("Interpreting token list {:?}", tl);
            for t in tl.iter() {
                println!("Executing token vector {:?}", t);
                execute_token(t, st)?;
            }
            Ok(())
        }

        fn get_token_list_for_command(&self, s: &str) -> Result<Vec<Token>, ForthError> {
            let tl = self.command_map.get(s);
            match tl {
                Some(tl) => Ok(tl.to_vec()),
                None => return Err(ForthError::UnknownToken(s.to_owned())),
            }
        }
    }
}
