#![feature(try_trait)]
use exit::Exit;

use std::option;

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
    Token(String),
}

pub fn tokenize_string(s: &str) -> Vec<Token> {
    s.split_whitespace()
        .map(|x| match x.parse::<u64>() {
            Ok(n) => Token::Number(n),
            Err(_) => Token::Token(x.to_owned()),
        })
        .collect()
}

pub fn execute_token(t:Token) ->Result<(),ForthErr> {

Ok(())
}

pub fn run() ->Result<(),ForthErr> {
    let x = tokenize_string("abc 123 def 456 ghi");

    println!("tokenized string: {:?}", x);

    Ok(())
}
