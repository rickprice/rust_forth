#![feature(try_trait)]

//! This is documentation for the rust_forth module
//!
//!

pub use error::ForthError;
use std::collections::HashMap;

pub mod error;
mod state;
mod tokenHandler::TokenHandler;

/// This Struct holds all the information the Forth Interpreter needs to run.
/// If you want to run more than one Forth interpreter, then create another copy
/// of this struct.
///
///
/// ```
/// # use std::error::Error;
/// use rust_forth::ForthInterpreter;
/// use std::fs;
/// use rust_forth::ForthError;
/// # use exit::Exit;
/// #
/// #   fn main() -> Result<(), ForthError> {
/// #
///
///    let mut rf = ForthInterpreter::new();
///
///    let startup = fs::read_to_string("C:\\Users\\rprice\\Documents\\RustProjects\\rust_forth\\init.forth")?;
///    rf.execute_string(&startup)?;
///
///    rf.execute_string("123 321 ADD 2 MUL")?;
///
///    rf.execute_string(": TestCommand 123456 DUP ADD 777 ; TestCommand TestCommand")?;
///
/// #
/// #   Ok(())
/// # }
/// ```
pub struct ForthInterpreter {
    token_handlers : Vec<TokenHandler>,
    state : State,

    command_map: HashMap<String, Vec<Token>>,
    number_stack: Vec<i64>,
    mode: Mode,
}

/// This Enum lists the token types that are used by the Forth interpreter
#[derive(Debug, Clone)]
enum Token {
    Number(i64),
    Command(String),
    Colon(String),
    SemiColon,
}

/// This Enum determines whether the Forth interpreter is in Interpreting mode or Compiling mode
enum Mode {
    Interpreting,
    Compiling(String),
}

impl ForthInterpreter {
    /// This creates a new instance of a Forth Interpreter, it only understands the built in commands.
    /// If you want something more than the bare bones words, you will need to load a source file.
    /// In the source code directory there is a file called 'init.forth' that has basic words
    pub fn new() -> ForthInterpreter {
        ForthInterpreter {
            command_map: HashMap::new(),
            number_stack: Vec::new(),
            mode: Mode::Interpreting,
        }
    }

    /// This method executes Forth commands contained inside the string, these can be commands to be compiled, or interpreted commands
    ///
    /// # Arguments
    ///
    /// * 'str' - A string slice that contains forth commands to execute (or compile)
    ///
    /// # Example
    ///
    ///
    /// ```
    /// # use std::error::Error;
    /// use rust_forth::ForthInterpreter;
    /// use rust_forth::ForthError;
    /// # use exit::Exit;
    /// #
    /// #   fn main() -> Result<(), ForthError> {
    /// #
    ///
    ///    let mut rf = ForthInterpreter::new();
    ///
    ///    rf.execute_string("123 321 ADD 2 MUL")?;
    ///
    ///    rf.execute_string(": TestCommand 123456 DUP ADD 777 ; TestCommand TestCommand")?;
    ///
    /// #
    /// #   Ok(())
    /// # }
    /// ```    
    pub fn execute_string(&mut self, s: &str) -> Result<(), ForthError> {
        println!("Executing string: {}", s);
        let tl = ForthInterpreter::tokenize_string(s)?;

        println!("tokenized string: {:?}", tl);

        self.execute_token_vector(tl)?;

        Ok(())
    }
}

impl ForthInterpreter {
    /// This method pushes a number onto the Forth stack
    ///
    /// # Arguments
    ///
    /// * 'n' - A number to be pushed onto the top of the Forth stack
    ///
    /// # Example
    ///
    /// ```
    /// # use std::error::Error;
    /// use rust_forth::ForthInterpreter;
    /// use rust_forth::ForthError;
    /// # use exit::Exit;
    /// #
    /// #   fn main() -> Result<(), ForthError> {
    /// #
    ///
    ///    let mut rf = ForthInterpreter::new();
    ///
    ///     rf.push_stack(123);
    ///     rf.push_stack(321);
    ///
    ///     rf.execute_string("ADD 2 MUL")?;
    ///
    ///     let n = rf.pop_stack()?;
    ///     println!("Found {} on top of stack",n);
    /// #
    /// #   Ok(())
    /// # }
    /// ```    
    pub fn push_stack(&mut self, n: i64) {
        println!("Pushed {} on stack", n);
        self.number_stack.push(n);
    }

    /// This method pops a number off the Forth stack
    ///
    /// # Arguments
    ///
    /// # Example
    ///
    /// ```
    /// # use std::error::Error;
    /// use rust_forth::ForthInterpreter;
    /// use rust_forth::ForthError;
    /// # use exit::Exit;
    /// #
    /// #   fn main() -> Result<(), ForthError> {
    /// #
    ///
    ///    let mut rf = ForthInterpreter::new();
    ///
    ///     rf.push_stack(123);
    ///     rf.push_stack(321);
    ///
    ///     rf.execute_string("ADD 2 MUL")?;
    ///
    ///     let n = rf.pop_stack()?;
    ///     println!("Found {} on top of stack",n);
    /// #
    /// #   Ok(())
    /// # }
    /// ```    
    pub fn pop_stack(&mut self) -> Result<i64, ForthError> {
        println!("Popped stack");
        match self.number_stack.pop() {
            Some(x) => Ok(x),
            None => Err(ForthError::PopOfEmptyStack),
        }
    }

    /// This method lets you bulk modify/read the Forth stack
    ///
    /// # Arguments
    ///
    /// # Example
    ///
    /// ```
    /// # use std::error::Error;
    /// use rust_forth::ForthInterpreter;
    /// use rust_forth::ForthError;
    /// # use exit::Exit;
    /// #
    /// #   fn main() -> Result<(), ForthError> {
    /// #
    ///
    ///     let mut rf = ForthInterpreter::new();
    ///
    ///     let mut vector = vec![5_i64,4,3,2,1];
    ///     rf.access_stack().append(&mut vector);
    ///
    ///     rf.push_stack(123);
    ///     rf.push_stack(321);
    ///
    ///     rf.execute_string("ADD 2 MUL")?;
    ///
    ///     for n in rf.access_stack() {
    ///         println!("Found {} (backwards) on stack",n)
    ///     }
    ///
    ///
    /// #
    /// #   Ok(())
    /// # }
    /// ```    
    pub fn access_stack(&mut self) -> &mut Vec<i64> {
        &mut self.number_stack
    }
}

// This function has two modes, Interpreting, and Compiling.
// In the Interpreting mode, each token is executed as a command
// and if the colon is encountered, Compiling mode is entered.
// In Compiling mode, the tokens are added to the map of commands
// until a semicolon is encountered, at which point things switch
// back to Interpreting mode.

impl ForthInterpreter {
    fn execute_token(&mut self, t: &Token) -> Result<(), ForthError> {

        for th in self.token_handlers {
            match th.handle_token(t,self.state) {
                Handled=>return,
                UnHandled=>(),
            }
        }




        match &self.mode {
            Mode::Interpreting => {
                match t {
                    Token::Number(n) => self.push_stack(*n),
                    Token::Command(s) => {
                        println!("Interpreting token {}", s);
                        match s.as_ref() {
                            "POP" => match self.pop_stack() {
                                Ok(_) => (),
                                Err(e) => return Err(e),
                            },
                            "ADD" => self.internal_add()?,
                            "SUB" => self.internal_sub()?,
                            "MUL" => self.internal_mul()?,
                            "DIV" => self.internal_div()?,
                            "DUP" => self.internal_dup()?,
                            "SWAP" => self.internal_swap()?,
                            s => self.execute_token_by_name(s)?,
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

    fn tokenize_string(s: &str) -> Result<Vec<Token>, ForthError> {
        let mut tl = Vec::new();

        let mut string_iter = s.split_whitespace();

        loop {
            match string_iter.next() {
                None => return Ok(tl),
                Some(string_token) => {
                    tl.push(match string_token.parse::<i64>() {
                        Ok(n) => Token::Number(n),
                        Err(_) => match string_token {
                            ":" => match &string_iter.next() {
                                Some(next_token) => Token::Colon(next_token.to_string()),
                                None => {
                                    return Err(ForthError::InvalidSyntax(String::from(
                                        "No token after :, but one needed to compile",
                                    )))
                                }
                            },
                            ";" => Token::SemiColon,
                            _ => Token::Command(string_token.to_owned()),
                        },
                    });
                }
            }
        }
    }

    fn execute_token_by_name(&mut self, s: &str) -> Result<(), ForthError> {
        let tl = self.get_token_list_for_command(s)?;

        println!("Executing token list {:?} for {}", tl, s);
        self.execute_token_vector(tl)?;
        Ok(())
    }

    fn get_token_list_for_command(&self, s: &str) -> Result<Vec<Token>, ForthError> {
        let tl = self.command_map.get(s);
        match tl {
            Some(tl) => Ok(tl.to_vec()),
            None => return Err(ForthError::UnknownToken(s.to_owned())),
        }
    }

    fn execute_token_vector(&mut self, tl: Vec<Token>) -> Result<(), ForthError> {
        println!("Interpreting token list {:?}", tl);
        for t in tl.iter() {
            println!("Executing token vector {:?}", t);
            self.execute_token(t)?;
        }
        Ok(())
    }
}

impl ForthInterpreter {
    fn internal_mul(&mut self) -> Result<(), ForthError> {
        let x = self.pop_stack()?;
        let y = self.pop_stack()?;
        let result = x * y;

        self.push_stack(result);

        println!("Multiplied {} by {} resulting in {}", x, y, result);

        Ok(())
    }

    fn internal_div(&mut self) -> Result<(), ForthError> {
        let x = self.pop_stack()?;
        let y = self.pop_stack()?;
        let result = x / y;

        self.push_stack(result);

        println!("Divided {} by {} resulting in {}", x, y, result);

        Ok(())
    }

    fn internal_add(&mut self) -> Result<(), ForthError> {
        let x = self.pop_stack()?;
        let y = self.pop_stack()?;
        let result = x + y;

        self.push_stack(result);

        println!("Added {} to {} resulting in {}", x, y, result);

        Ok(())
    }

    fn internal_sub(&mut self) -> Result<(), ForthError> {
        let x = self.pop_stack()?;
        let y = self.pop_stack()?;
        let result = x - y;

        self.push_stack(result);

        println!("Subtracted {} by {} resulting in {}", x, y, result);

        Ok(())
    }

    fn internal_dup(&mut self) -> Result<(), ForthError> {
        let x = self.pop_stack()?;

        self.push_stack(x);
        self.push_stack(x);

        println!("Duplicated {} ", x);

        Ok(())
    }

    fn internal_swap(&mut self) -> Result<(), ForthError> {
        let x = self.pop_stack()?;
        let y = self.pop_stack()?;

        self.push_stack(x);
        self.push_stack(y);

        println!("Swapped top items on stack {} {}", x, y);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_string_1() {
        let mut rf = ForthInterpreter::new();

        rf.push_stack(123);
        rf.push_stack(321);
        rf.execute_string("ADD 2 MUL").unwrap();
        let n = rf.pop_stack().unwrap();

        assert_eq!(n, 888);
    }

    #[test]
    fn test_execute_string_2() {
        let mut rf = ForthInterpreter::new();

        rf.push_stack(123456);
        rf.push_stack(111112);
        rf.execute_string(": TEST ADD 2 SWAP DIV ; TEST").unwrap();
        let n = rf.pop_stack().unwrap();

        assert_eq!(n, 117284);
    }
}
