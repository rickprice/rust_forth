#![feature(try_trait)]

use std::collections::HashMap;
use std::fs::File;
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

#[derive(Debug, Clone)]
pub enum Token {
    Number(i64),
    Command(String),
}

pub struct RustForth {
    command_map: HashMap<String, Vec<Token>>,
    number_stack: Vec<i64>,
}

impl RustForth {
    pub fn new() -> RustForth {
        RustForth {
            command_map: HashMap::new(),
            number_stack: Vec::new(),
        }
    }

    pub fn execute_token(&mut self, t: &Token) -> Result<(), ForthErr> {
        match t {
            Token::Number(n) => self.push_stack(*n),
            Token::Command(s) => {
                println!("Execute token {}", s);
                match s.as_ref() {
                    "predefined1" => println!("found predefined1"),
                    "predefined2" => println!("found predefined2"),
                    "pop" => match self.pop_stack() {
                        Ok(_) => (),
                        Err(e) => return Err(e),
                    },
                    "add" => self.internal_add()?,
                    "sub" => self.internal_sub()?,
                    "mul" => self.internal_mul()?,
                    "div" => self.internal_div()?,
                    "dup" => self.internal_dup()?,
                    s => self.execute_token_by_name(s)?,
                }
            }
        }

        println!("State of number stack {:?}", self.number_stack);

        Ok(())
    }

    pub fn tokenize_string(s: &str) -> Result<Vec<Token>, ForthErr> {
        Ok(s.split_whitespace()
            .map(|x| match x.parse::<i64>() {
                Ok(n) => Token::Number(n),
                Err(_) => Token::Command(x.to_owned()),
            })
            .collect())
    }

    fn get_token_list_for_command(&self, s: &str) -> Result<Vec<Token>, ForthErr> {
        let tl = self.command_map.get(s);
        match tl {
            Some(tl) => Ok(tl.to_vec()),
            None => return Err(ForthErr::UnknownToken),
        }
    }

    pub fn execute_token_by_name(&mut self, s: &str) -> Result<(), ForthErr> {
        let tl = self.get_token_list_for_command(s)?;

        println!("Executing token list {:?} for {}", tl, s);
        self.execute_token_vector(tl)?;
        Ok(())
    }

    pub fn execute_token_vector(&mut self, tl: Vec<Token>) -> Result<(), ForthErr> {
        println!("Executing token list {:?}", tl);
        for t in tl.iter() {
            println!("> {:?}", t);
            self.execute_token(t)?;
        }
        Ok(())
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

            let (command, command_list_string) = RustForth::split_command_initializer_line(&line)?;
            let token_list = RustForth::tokenize_string(command_list_string)?;

            self.command_map.insert(command.to_string(), token_list);
        }

        Ok(())
    }
}

impl RustForth {
    fn push_stack(&mut self, n: i64) {
        println!("Pushed {} on stack", n);
        self.number_stack.push(n);
    }

    fn pop_stack(&mut self) -> Result<i64, ForthErr> {
        println!("Popped stack");
        match self.number_stack.pop() {
            Some(x) => Ok(x),
            None => Err(ForthErr::PopOfEmptyStack),
        }
    }
}

impl RustForth {
    fn internal_mul(&mut self) -> Result<(), ForthErr> {
        let x = self.pop_stack()?;
        let y = self.pop_stack()?;
        let result = x * y;

        self.push_stack(result);

        println!("Multiplied {} by {} resulting in {}", x, y, result);

        Ok(())
    }

    fn internal_div(&mut self) -> Result<(), ForthErr> {
        let x = self.pop_stack()?;
        let y = self.pop_stack()?;
        let result = x / y;

        self.push_stack(result);

        println!("Divided {} by {} resulting in {}", x, y, result);

        Ok(())
    }
    fn internal_add(&mut self) -> Result<(), ForthErr> {
        let x = self.pop_stack()?;
        let y = self.pop_stack()?;
        let result = x + y;

        self.push_stack(result);

        println!("Added {} to {} resulting in {}", x, y, result);

        Ok(())
    }
    fn internal_sub(&mut self) -> Result<(), ForthErr> {
        let x = self.pop_stack()?;
        let y = self.pop_stack()?;
        let result = x - y;

        self.push_stack(result);

        println!("Subtracted {} by {} resulting in {}", x, y, result);

        Ok(())
    }
    fn internal_dup(&mut self) -> Result<(), ForthErr> {
        let x = self.pop_stack()?;

        self.push_stack(x);
        self.push_stack(x);

        println!("Duplicated {} ", x);

        Ok(())
    }
}

pub fn run() -> Result<(), ForthErr> {
    let mut rf = RustForth::new();

    let tl = RustForth::tokenize_string("predefined1 123 predefined2 456 pop Numbers mul add dup")?;

    let f = File::open("C:\\Users\\rprice\\Documents\\RustProjects\\rust_forth\\init.forth")?;
    rf.initialize_commands_from_file(f)?;

    println!("tokenized string: {:?}", tl);

    rf.execute_token_vector(tl)?;

    Ok(())
}
