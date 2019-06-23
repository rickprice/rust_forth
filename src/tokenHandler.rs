use super::state::State;
use super::Token;

enum Handled {
    Handled,
    NotHandled,
}

// Chain of Command Pattern
trait HandleToken {
    fn handle_token(&mut self, t: &Token, st: &mut State) -> Result<Handled,ForthError>;
}

mod internals {
    pub struct ForthInternalCommandHandler {

    }

    impl ForthInternalCommandHandler {
        fn handle_token(&mut self, t: &Token, st: &mut State) -> Result<Handled,ForthError> {
                match t {
                    Token::Command(s) => {
                        println!("Interpreting token {}", s);
                        match s.as_ref() {
                            "POP" => match self.pop_stack(st) {
                                Ok(_) => (),
                                Err(e) => return Err(e),
                            },
                            "ADD" => self.add(st)?,
                            "SUB" => self.sub(st)?,
                            "MUL" => self.mul(st)?,
                            "DIV" => self.div(st)?,
                            "DUP" => self.dup(st)?,
                            "SWAP" => self.swap(st)?,
                            _=>Ok(NotHandledToken),
                        },
                    }
                }
                Ok(Handled)
            }
        }

   fn mul(&self,st: &mut State) -> Result<(), ForthError> {
        let x = st.number_stack.pop_stack()?;
        let y = st.number_stack.pop_stack()?;
        let result = x * y;

        st.number_stack.push_stack(result);

        println!("Multiplied {} by {} resulting in {}", x, y, result);

        Ok(())
    }

    fn div(&self,st: &mut State) -> Result<(), ForthError> {
        let x = st.number_stack.pop_stack()?;
        let y = st.number_stack.pop_stack()?;
        let result = x / y;

        self.st.number_stack.push_stack(result);

        println!("Divided {} by {} resulting in {}", x, y, result);

        Ok(())
    }

    fn add(&self,st: &mut State) -> Result<(), ForthError> {
        let x = st.number_stack.pop_stack()?;
        let y = st.number_stack.pop_stack()?;
        let result = x + y;

        self.st.number_stack.push_stack(result);

        println!("Added {} to {} resulting in {}", x, y, result);

        Ok(())
    }

    fn sub(&self,st: &mut State) -> Result<(), ForthError> {
        let x = st.number_stack.pop_stack()?;
        let y = st.number_stack.pop_stack()?;
        let result = x - y;

        st.number_stack.push_stack(result);

        println!("Subtracted {} by {} resulting in {}", x, y, result);

        Ok(())
    }

    fn dup(&self,st: &mut State) -> Result<(), ForthError> {
        let x = st.number_stack.pop_stack()?;

        st.number_stack.push_stack(x);
        st.number_stack.push_stack(x);

        println!("Duplicated {} ", x);

        Ok(())
    }

    fn swap(&self,st: &mut State) -> Result<(), ForthError> {
        let x = st.number_stack.pop_stack()?;
        let y = st.number_stack.pop_stack()?;

        st.number_stack.push_stack(x);
        st.number_stack.push_stack(y);

        println!("Swapped top items on stack {} {}", x, y);

        Ok(())
    }        

    pub struct CompiledCommands {
   command_map: HashMap<String, Vec<Token>>,
    mode: Mode,
    }

    impl CompiledCommands {
        fn handle_token(&mut self, t: &Token, st: &mut State) -> Result<Handled,ForthError> {
        match &self.mode {
            Mode::Interpreting => {
                match t {
                Token::Number(n) => {
                    println!("Compiling number {}", n);
                    self.command_map
                        .entry(c.to_string())
                        .or_insert(Vec::new())
                        .push(Token::Number(*n));
                },
                    Token::Command(s) => {
                        println!("Interpreting token {}", s);
                            self.execute_token_by_name(s)?,                        
                    },
                    Token::Colon(s) => {
                        println!("Colon, starting compiling");
                        self.mode = Mode::Compiling(String::from(s));
                    },
                    Token::SemiColon => {
                        panic!("Token::SemiColon case should not happen here; are you missing a prior semicolon?");
                    },
                }

                println!("State of number stack {:?}", self.number_stack);
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

        Ok(())
    }

    }
}