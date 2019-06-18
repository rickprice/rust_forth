#![feature(try_trait)]
//use exit::Exit;

use std::collections::HashMap;
use std::fs::File;
use std::io::Error;
use std::io::{BufRead, BufReader};
use std::option;

#[derive(Debug)]
pub enum ForthErr {
    UnknownError,
    UnknownToken,
    PopOfEmptyStack,
    XParseErrorUserNum,
    XParseErrorGroupNum,
    InvalidInitializationLine,
    Io(std::io::Error),
}

impl From<std::io::Error> for ForthErr {
    fn from(err: std::io::Error) -> ForthErr {
        ForthErr::Io(err)
    }
}

impl From<ForthErr> for i32 {
    fn from(err: ForthErr) -> Self {
        match err {
            ForthErr::UnknownError => 2,
            ForthErr::UnknownToken => 3,
            ForthErr::PopOfEmptyStack => 4,
            ForthErr::XParseErrorUserNum => 5,
            ForthErr::XParseErrorGroupNum => 6,
            ForthErr::InvalidInitializationLine => 7,
            ForthErr::Io(_) => 8,
        }
    }
}

impl From<option::NoneError> for ForthErr {
    fn from(_: option::NoneError) -> Self {
        ForthErr::UnknownError
    }
}

#[derive(Debug)]
pub enum Token {
    Number(u64),
    Command(String),
}

pub struct RustForth {
    command_map: HashMap<String, Vec<Token>>,
    number_stack: Vec<u64>,
}

impl RustForth {
    pub fn new() -> RustForth {
        RustForth {
            command_map: HashMap::new(),
            number_stack: Vec::new(),
        }
    }

    pub fn execute_token(&mut self, t: Token) -> Result<(), ForthErr> {
        match t {
            Token::Number(n) => self.push_stack(n),
            Token::Command(s) => {
                println!("Execute token {}", s);
                match s.as_ref() {
                    "predefined1" => println!("found predefined1"),
                    "predefined2" => println!("found predefined2"),
                    "pop" => match self.pop_stack() {
                        Some(_) => (),
                        None => return Err(ForthErr::PopOfEmptyStack),
                    },
                    s => match self.command_map.get(s) {
                        Some(tl) => self.execute_token_list(tl),
                        None => return Err(ForthErr::UnknownToken),
                    },
                }
            }
        }

        Ok(())
    }

    pub fn tokenize_string(s: &str) -> Vec<Token> {
        s.split_whitespace()
            .map(|x| match x.parse::<u64>() {
                Ok(n) => Token::Number(n),
                Err(_) => Token::Command(x.to_owned()),
            })
            .collect()
    }

    pub fn execute_token_list(&self, tl: &Vec<Token>) {
        println!("Executing token list {:?}", tl);

        for x in tl.iter() {
            println!("> {:?}", x);
        }
    }

    fn push_stack(&mut self, n: u64) {
        println!("Pushed {} on stack", n);
        self.number_stack.push(n);
    }

    fn pop_stack(&mut self) -> Option<u64> {
        println!("Popped stack");
        self.number_stack.pop()
    }

    fn split_command_initializer_line(in_string: &str) -> Result<(&str, &str), ForthErr> {
        let mut splitter = in_string.splitn(2, "=>");
        let first = splitter
            .next()
            .ok_or(ForthErr::InvalidInitializationLine)?
            .trim();
        let second = splitter
            .next()
            .ok_or(ForthErr::InvalidInitializationLine)?
            .trim();
        Ok((first, second))
    }

    fn initialize_commands_from_file(&mut self, f: File) -> Result<(), ForthErr> {
        let reader = BufReader::new(f);

        // Read the file line by line using the lines() iterator from std::io::BufRead.
        for line in reader.lines() {
            let line = line?;

            let (command, command_list) = RustForth::split_command_initializer_line(&line)?;

            //            let line = line?;         // Show the line and its number.
            //            t,tl =
            //            println!("{}. {}", index + 1, line);
        }

        Ok(())
    }
}

pub fn run() -> Result<(), ForthErr> {
    let x = RustForth::tokenize_string("abc 123 def 456 ghi");

    println!("tokenized string: {:?}", x);

    Ok(())
}
