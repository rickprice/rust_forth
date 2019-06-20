use super::error::ForthError;
use super::Mode;
use super::RustForth;
use super::Token;

impl RustForth {
    fn execute_token(&mut self, t: &Token) -> Result<(), ForthError> {
        match &self.mode {
            Mode::Interpreting => {
                match t {
                    Token::Number(n) => self.push_stack(*n),
                    Token::Command(s) => {
                        println!("Interpreting token {}", s);
                        match s.as_ref() {
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

    pub fn tokenize_string(s: &str) -> Result<Vec<Token>, ForthError> {
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
            None => return Err(ForthError::UnknownToken),
        }
    }

    pub fn execute_token_vector(&mut self, tl: Vec<Token>) -> Result<(), ForthError> {
        println!("Interpreting token list {:?}", tl);
        for t in tl.iter() {
            println!("Executing token vector {:?}", t);
            self.execute_token(t)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
