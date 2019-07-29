use super::error::ForthError;
use super::stack_machine::GasLimit;
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
    fn tokenize_string(&self, s: &str) -> Result<Vec<Token>, ForthError> {
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
        let mut tvi: Vec<Opcode> = Vec::new();
        let mut tvc: Vec<Opcode> = Vec::new();
        let mut mode = Mode::Interpreting;

        for t in token_vector.iter() {
            let mut tv: Vec<Opcode> = Vec::new();

            match t {
                Token::Number(n) => tv.push(Opcode::LDI(*n)),
                Token::Command(s) => {
                    println!("CompiledCommands: Compiling token {}", s);

                    if let Some(offset) = self.word_addresses.get(s) {
                        tv.push(Opcode::LDI(*offset as i64));
                        tv.push(Opcode::CALL);
                    } else {
                        if let Some(ol) = self.intrinsic_words.get::<str>(s) {
                            tv.append(&mut ol.clone());
                        } else {
                            return Err(ForthError::UnknownToken(s.to_string()));
                        }
                    }
                }
                Token::Colon(s) => {
                    println!("Colon, starting compiling");
                    match mode {
                        Mode::Interpreting => {
                            mode = Mode::Compiling(String::from(s));
                        }
                        Mode::Compiling(_) => {
                            return Err(ForthError::InvalidSyntax(
                                "Second colon before semicolon".to_string(),
                            ));
                        }
                    }
                }
                Token::SemiColon => {
                    println!("Semicolon, finishing compiling");
                    match mode {
                        Mode::Interpreting => {
                            return Err(ForthError::InvalidSyntax(
                                "Semicolon before colon".to_string(),
                            ));
                        }
                        Mode::Compiling(s) => {
                            // Remove anything extraneous from the end of the opcode array,
                            // typically previous immediate mode tokens
                            self.sm.st.opcodes.resize(self.last_function, Opcode::NOP);
                            // Put a return on the end of function definition
                            tvc.push(Opcode::RET);
                            // The current function start is the end of the last function
                            let function_start = self.last_function;
                            // Move last function pointer
                            self.last_function += tvc.len();
                            // Add the function to the opcode memory
                            self.sm.st.opcodes.append(&mut tvc);
                            // Remember where to find it...
                            self.word_addresses.insert(s, function_start);
                            // Switch back to interpreting mode
                            mode = Mode::Interpreting;
                            //                            println!("Token Memory {:?}", self.sm.st.opcodes);
                            //                            println!("Word Addresses {:?}", self.word_addresses);
                            //                            println!("Last function {}", self.last_function);
                        }
                    }
                }
                Token::End => {
                    panic!("Token::End not coded yet");
                }
                Token::Error(_) => {
                    panic!("Token::Error not coded yet");
                }
            }

            match mode {
                Mode::Interpreting => {
                    tvi.append(&mut tv);
                }
                Mode::Compiling(_) => {
                    tvc.append(&mut tv);
                }
            }
        }
        return Ok(tvi);
    }

    fn execute_token_vector(
        &mut self,
        token_vector: &Vec<Token>,
        gas_limit: GasLimit,
    ) -> Result<(), ForthError> {
        let mut ol = self.compile_token_vector(token_vector)?;
        self.sm.st.opcodes.resize(self.last_function, Opcode::NOP);
        self.sm.st.opcodes.append(&mut ol);
        self.sm.st.opcodes.push(Opcode::RET);
        self.sm.execute(self.last_function, gas_limit)?;
        Ok(())
    }

    pub fn execute_string(&mut self, s: &str, gas_limit: GasLimit) -> Result<(), ForthError> {
        let tv = self.tokenize_string(s)?;
        self.execute_token_vector(&tv, gas_limit)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_intrinsics_1() {
        let mut fc = ForthCompiler::new();

        fc.execute_string("123 321 ADD 2 MUL", GasLimit::Limited(100))
            .unwrap();

        assert_eq!(&fc.sm.st.number_stack, &vec![888_i64]);

        fc.execute_string("123 321 ADD 2 MUL", GasLimit::Limited(100))
            .unwrap();

        assert_eq!(&fc.sm.st.number_stack, &vec![888_i64, 888]);
    }

    #[test]
    fn test_compile_1() {
        let mut fc = ForthCompiler::new();

        fc.execute_string(
            ": RickTest 123 321 ADD 2 MUL ; RickTest",
            GasLimit::Limited(100),
        )
        .unwrap();

        assert_eq!(&fc.sm.st.number_stack, &vec![888_i64]);

        fc.execute_string("123 321 ADD 2 MUL RickTest", GasLimit::Limited(100))
            .unwrap();

        assert_eq!(&fc.sm.st.number_stack, &vec![888_i64, 888, 888]);
    }
}
