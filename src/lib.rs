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

pub fn run() {
    let x = tokenize_string("abc 123 def 456 ghi");

    println!("tokenized string: {:?}", x)
}
