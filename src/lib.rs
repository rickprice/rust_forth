#![feature(try_trait)]

use error::ForthError;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub mod error;

mod internal_execution;
mod internal_implementation;

pub struct RustForth {
    command_map: HashMap<String, Vec<Token>>,
    number_stack: Vec<i64>,
    mode: Mode,
}

#[derive(Debug, Clone)]
pub enum Token {
    Number(i64),
    Command(String),
    Colon(String),
    SemiColon,
}

enum Mode {
    Interpreting,
    Compiling(String),
}

impl RustForth {
    pub fn new() -> RustForth {
        RustForth {
            command_map: HashMap::new(),
            number_stack: Vec::new(),
            mode: Mode::Interpreting,
        }
    }

    pub fn execute_string(&mut self, s: &str) -> Result<(), ForthError> {
        println!("Executing string: {}", s);
        let tl = RustForth::tokenize_string(s)?;

        println!("tokenized string: {:?}", tl);

        self.execute_token_vector(tl)?;

        Ok(())
    }
}
