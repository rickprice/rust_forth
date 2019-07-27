use super::error::ForthError;
use super::stack_machine::Opcode;
use super::stack_machine::StackMachine;
use std::collections::HashMap;

/// This Enum lists the token types that are used by the Forth interpreter
#[derive(Debug, Clone)]
pub enum Token {
    Number(i64),
    Command(String),
    Colon(String),
    SemiColon,
    End,
    Error(String),
}

macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

pub struct ForthCompiler {
    pub sm: StackMachine,
    intrinsic_words: HashMap<&'static str, Vec<Opcode>>,
    word_addresses: HashMap<String, usize>,
    last_function: usize,
}

impl ForthCompiler {
    pub fn new() -> ForthCompiler {
        ForthCompiler {
            sm: StackMachine::new(),
            intrinsic_words: hashmap![
            "POP" => vec![Opcode::POP],
            "SWAP" => vec![Opcode::SWAP],
            "ADD" => vec![Opcode::ADD],
            "SUB" => vec![Opcode::SUB],
            "MUL" => vec![Opcode::MUL],
            "DIV" => vec![Opcode::DIV],
            "DUP" => vec![Opcode::DUP],
            "TRAP" => vec![Opcode::TRAP],
            "INC" => vec![Opcode::LDI(1),Opcode::ADD],
            "DEC" => vec![Opcode::LDI(-1),Opcode::ADD]
            ],
            word_addresses: HashMap::new(),
            last_function: 0,
        }
    }
}

/// This Enum determines whether the Forth interpreter is in Interpreting mode or Compiling mode
enum Mode {
    Interpreting,
    Compiling(String),
}

impl ForthCompiler {
    fn tokenize_string(s: &str) -> Result<Vec<Token>, ForthError> {
        let mut tv = Vec::new();

        let mut string_iter = s.split_whitespace();

        loop {
            match string_iter.next() {
                None => return Ok(tv),
                Some(string_token) => {
                    tv.push(match string_token.parse::<i64>() {
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

    fn compile_token_vector(
        &mut self,
        token_vector: &Vec<Token>,
    ) -> Result<Vec<Opcode>, ForthError> {
        let mut tv: Vec<Opcode> = Vec::new();
        let mut mode = Mode::Interpreting;

        for t in token_vector.iter() {
            match t {
                Token::Number(n) => tv.push(Opcode::LDI(*n)),
                Token::Command(s) => {
                    println!("CompiledCommands: Compiling token {}", s);

                    if let Some(offset) = self.word_addresses.get(s) {
                        tv.push(Opcode::LDI(*offset as i64));
                        tv.push(Opcode::CALL);
                    } else {
                        let ol = self.intrinsic_words.get::<str>(s)?;
                        tv.append(&mut ol.clone());
                    }
                }
                Token::Colon(s) => {
                    println!("Colon, starting compiling");
                    mode = Mode::Compiling(String::from(s));
                }
                Token::SemiColon => {
                    panic!("Token::SemiColon case should not happen here; are you missing a prior semicolon?");
                }
                Token::End => {
                    panic!("Token::End not coded yet");
                }
                Token::Error(_) => {
                    panic!("Token::Error not coded yet");
                }
            }
        }
        return Ok(tv);
    }

    fn execute_token_vector(&mut self, token_vector: &Vec<Token>) -> Result<(), ForthError> {
        let ol = self.compile_token_vector(token_vector)?;
        self.sm.st.opcodes.resize(self.last_function, Opcode::NOP);
        Ok(())
    }
}
