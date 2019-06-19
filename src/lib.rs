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
    InvalidSyntax(String),
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
            ForthErr::InvalidSyntax(_) => 8,
            ForthErr::Io(_) => 9,
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
    Colon(String),
    SemiColon,
}

enum Mode {
    Interpreting,
    Compiling(String),
}

pub struct RustForth {
    command_map: HashMap<String, Vec<Token>>,
    number_stack: Vec<i64>,
    mode: Mode,
}

impl RustForth {
    pub fn new() -> RustForth {
        RustForth {
            command_map: HashMap::new(),
            number_stack: Vec::new(),
            mode: Mode::Interpreting,
        }
    }

    pub fn execute_string(&mut self, s: &str) -> Result<(), ForthErr> {
        let tl = RustForth::tokenize_string(s)?;

        println!("tokenized string: {:?}", tl);

        self.execute_token_vector(tl)?;

        Ok(())
    }

    fn execute_token(&mut self, t: &Token) -> Result<(), ForthErr> {
        match &self.mode {
            Mode::Interpreting => {
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
                    Token::Colon(s) => {
                        self.mode = Mode::Compiling(String::from(s));
                    }
                    Token::SemiColon => {
                        panic!("Token::SemiColon case should not happen here");
                    }
                }

                println!("State of number stack {:?}", self.number_stack);
            }
            Mode::Compiling(c) => match t {
                Token::Number(n) => {
                    self.command_map
                        .entry(c.to_string())
                        .or_insert(Vec::new())
                        .push(Token::Number(*n));
                }
                Token::Command(s) => {
                    self.command_map
                        .entry(c.to_string())
                        .or_insert(Vec::new())
                        .push(Token::Command(s.to_string()));
                }
                Token::Colon(_) => {
                    panic!("Token::Colon case should not happen here");
                }
                Token::SemiColon => {
                    self.mode = Mode::Interpreting;
                }
            },
        }

        Ok(())
    }

    fn tokenize_string(s: &str) -> Result<Vec<Token>, ForthErr> {
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
                                    return Err(ForthErr::InvalidSyntax(String::from(
                                        "No token after :",
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

    fn get_token_list_for_command(&self, s: &str) -> Result<Vec<Token>, ForthErr> {
        let tl = self.command_map.get(s);
        match tl {
            Some(tl) => Ok(tl.to_vec()),
            None => return Err(ForthErr::UnknownToken),
        }
    }

    fn execute_token_by_name(&mut self, s: &str) -> Result<(), ForthErr> {
        let tl = self.get_token_list_for_command(s)?;

        println!("Executing token list {:?} for {}", tl, s);
        self.execute_token_vector(tl)?;
        Ok(())
    }

    fn execute_token_vector(&mut self, tl: Vec<Token>) -> Result<(), ForthErr> {
        println!("Interpreting token list {:?}", tl);
        for t in tl.iter() {
            println!("Executing token vector {:?}", t);
            self.execute_token(t)?;
        }
        Ok(())
    }

    pub fn execute_commands_from_file(&mut self, f: File) -> Result<(), ForthErr> {
        let reader = BufReader::new(f);

        for line in reader.lines() {
            let line = line?;

            self.execute_string(&line)?;
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
