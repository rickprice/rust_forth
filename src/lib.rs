#![feature(try_trait)]
use exit::Exit;

use std::option;
use std::collections::HashMap;

#[derive(Debug)]
pub enum ForthErr {
    UnknownError,
    UnknownToken,
    XParseErrorUserNum,
    XParseErrorGroupNum,
}



impl From<ForthErr> for i32 {
    fn from(err: ForthErr) -> Self {
        match err {
            ForthErr::UnknownError => 2,
            ForthErr::UnknownToken => 3,
            ForthErr::XParseErrorUserNum => 4,
            ForthErr::XParseErrorGroupNum => 5,
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

static mut command_map: HashMap<&String, Vec<Token>> = HashMap::new();

pub fn tokenize_string(s: &str) -> Vec<Token> {
    s.split_whitespace()
        .map(|x| match x.parse::<u64>() {
            Ok(n) => Token::Number(n),
            Err(_) => Token::Command(x.to_owned()),
        })
        .collect()
}

pub fn execute_token(t:Token) ->Result<(),ForthErr> {
match t {
    Token::Number(n)=>push_stack(n),
    Token::Command(s)=>{
    println!("Execute token {}", s);
    match s.as_ref() {
        "predefined1"=>println!("found predefined1"),
        "predefined2"=>println!("found predefined2"),
        s =>{
            match command_map.get(s.as_ref()) {
                Some(tl)=>execute_token_list(tl),
                None=>return Err(ForthErr::UnknownToken),
            }
            },
    }
    },
}

Ok(())
}

pub fn execute_token_list(tl:&Vec<Token>) {
        println!("Executing token list {:?}", tl);
}

fn push_stack(n:u64) {
    println!("Pushed {} on stack", n);
}

fn pop_stack() {
    println!("Popped stack");
}

pub fn run() ->Result<(),ForthErr> {
    let x = tokenize_string("abc 123 def 456 ghi");

    println!("tokenized string: {:?}", x);

    Ok(())
}
