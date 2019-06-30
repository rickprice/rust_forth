use super::error::ForthError;
use super::tokenHandler::Token;

pub struct State {
    pub number_stack: NumberStack,
    pub token_stack: Vec<Token>,
}

impl State {
    pub fn new() -> State {
        State {
            number_stack: NumberStack::new(),
            token_stack: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct NumberStack {
    pub number_stack: Vec<i64>,
}

impl NumberStack {
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

    pub fn new() -> NumberStack {
        NumberStack {
            number_stack: Vec::new(),
        }
    }
}
