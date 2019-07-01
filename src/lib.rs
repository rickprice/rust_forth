#![feature(try_trait)]

//! This is documentation for the rust_forth module
//!
//!

pub use error::ForthError;

pub mod error;
mod state;
mod token_handler;

use state::State;
use token_handler::internals::CompiledCommands;
use token_handler::internals::ForthInternalCommandHandler;
use token_handler::HandleToken;
use token_handler::Handled;
use token_handler::Token;

/// This Struct holds all the information the Forth Interpreter needs to run.
/// If you want to run more than one Forth interpreter, then create another copy
/// of this struct.
///
///
/// ```
/// # use std::error::Error;
/// use rust_forth::ForthInterpreter;
/// use rust_forth::ForthError;
/// use std::fs;
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
    state: State,
    token_handlers: Vec<Box<HandleToken>>,
}

/// This method executes Forth commands contained inside the string, these can be commands to be compiled, or interpreted commands
///}

impl ForthInterpreter {
    /// This creates a new instance of a Forth Interpreter, it only understands the built in commands.
    /// If you want something more than the bare bones words, you will need to load a source file.
    /// In the source code directory there is a file called 'init.forth' that has basic words
    pub fn new() -> ForthInterpreter {
        ForthInterpreter {
            state: State::new(),
            token_handlers: vec![
                Box::new(CompiledCommands::new()),
                Box::new(ForthInternalCommandHandler::new()),
            ],
        }
    }

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
        let mut tl = ForthInterpreter::tokenize_string(s)?;

        println!("tokenized string: {:?}", tl);

        // Because we append, we need the tokens in reverse order so they can be popped in the correct order
        tl.reverse();

        self.state.token_stack.append(&mut tl);

        self.execute_token_stack()?;

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
        self.state.number_stack.push_stack(n);
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
        self.state.number_stack.pop_stack()
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
        &mut self.state.number_stack.number_stack
    }
}

// This function has two modes, Interpreting, and Compiling.
// In the Interpreting mode, each token is executed as a command
// and if the colon is encountered, Compiling mode is entered.
// In Compiling mode, the tokens are added to the map of commands
// until a semicolon is encountered, at which point things switch
// back to Interpreting mode.

impl ForthInterpreter {
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

    fn execute_token_stack(&mut self) -> Result<(), ForthError> {
        loop {
            match self.state.token_stack.pop() {
                Some(t) => self.execute_token(&t)?,
                None => break,
            }
        }
        Ok(())
    }

    fn execute_token(&mut self, t: &Token) -> Result<(), ForthError> {
        for th in self.token_handlers.iter_mut() {
            if let Handled::Handled = th.handle_token(t, &mut self.state)? {
                break;
            }
        }

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
