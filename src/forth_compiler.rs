use super::error::ForthError;
use super::stack_machine::GasLimit;
use super::stack_machine::Opcode;
use super::stack_machine::StackMachine;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::convert::TryInto;

/// This Enum lists the token types that are used by the Forth interpreter
#[derive(Debug)]
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
#[derive(Debug)]
enum Mode {
    Interpreting,
    Compiling(String),
}

#[derive(Debug)]
struct DeferredIfStatement {
    if_location: usize,
    else_location: Option<usize>,
}

impl DeferredIfStatement {
    pub fn new(if_location: usize) -> DeferredIfStatement {
        DeferredIfStatement {
            if_location: if_location,
            else_location: None,
        }
    }
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
        let mut deferred_if_statements = Vec::new();
        let mut tvi: Vec<Opcode> = Vec::new();
        let mut tvc: Vec<Opcode> = Vec::new();
        let mut mode = Mode::Interpreting;

        for t in token_vector.iter() {
            let mut tv: Vec<Opcode> = Vec::new();

            match t {
                Token::Number(n) => {
                    //println!("CompiledCommands: Compiling number {}", n);
                    tv.push(Opcode::LDI(*n));
                }
                Token::Command(s) => {
                    //println!("CompiledCommands: Compiling token {}", s);
                    let current_instruction = match mode {
                        Mode::Interpreting => tvi.len(),
                        Mode::Compiling(_) => tvc.len(),
                    };

                    match s.as_ref() {
                        "IF" => {
                            deferred_if_statements
                                .push(DeferredIfStatement::new(current_instruction));
                            //println!("(IF)Deferred If Stack {:?}", deferred_if_statements);
                            tv.push(Opcode::LDI(0));
                            tv.push(Opcode::JRNZ);
                        }
                        "ELSE" => {
                            if let Some(x) = deferred_if_statements.last_mut() {
                                x.else_location = Some(current_instruction);
                                //println!("(ELSE) Deferred If Stack {:?}", deferred_if_statements);
                                tv.push(Opcode::LDI(0));
                                tv.push(Opcode::JR);
                            } else {
                                return Err(ForthError::InvalidSyntax(
                                    "ELSE without IF".to_owned(),
                                ));
                            }
                        }
                        "THEN" => {
                            // This only works if there isn't an ELSE statement, it needs to jump differently if there is an ELSE statement
                            //println!("(THEN) Deferred If Stack {:?}", deferred_if_statements);
                            if let Some(x) = deferred_if_statements.pop() {
                                //println!("(if let Some(x)) Deferred If Stack {:?}", x);
                                let if_jump_location = x.if_location;
                                let if_jump_offset = match x.else_location {
                                    None => (current_instruction as u64
                                        - (x.if_location + 1) as u64)
                                        .try_into()
                                        .unwrap(),
                                    Some(el) => (current_instruction as u64 - el as u64 + 1)
                                        .try_into()
                                        .unwrap(),
                                };
                                let (else_jump_location, else_jump_offset): (
                                    Option<usize>,
                                    Option<i64>,
                                ) = match x.else_location {
                                    Some(x) => (
                                        Some(x),
                                        Some(
                                            i64::try_from(
                                                current_instruction as u64 - (x + 1) as u64,
                                            )
                                            .unwrap(),
                                        ),
                                    ),
                                    None => (None, None),
                                };
                                match mode {
                                    Mode::Compiling(_) => {
                                        tvc[if_jump_location] = Opcode::LDI(if_jump_offset);
                                        if let (Some(location), Some(offset)) =
                                            (else_jump_location, else_jump_offset)
                                        {
                                            tvc[location] = Opcode::LDI(offset);
                                        }
                                    }
                                    Mode::Interpreting => {
                                        //println!("if structure: {:?}", x);
                                        tvi[if_jump_location] = Opcode::LDI(if_jump_offset);
                                        if let (Some(location), Some(offset)) =
                                            (else_jump_location, else_jump_offset)
                                        {
                                            tvi[location] = Opcode::LDI(offset);
                                        }
                                    }
                                }
                            } else {
                                return Err(ForthError::InvalidSyntax(
                                    "THEN without IF".to_owned(),
                                ));
                            }
                        }
                        _ => {
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
                    }
                }
                Token::Colon(s) => {
                    //println!("Colon, starting compiling");
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
                    //println!("Semicolon, finishing compiling");
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
                            //println!("Token Memory {:?}", self.sm.st.opcodes);
                            //println!("Word Addresses {:?}", self.word_addresses);
                            //println!("Last function {}", self.last_function);
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
        //println!("Compiled Codes {:?}", tvi);
        //println!("Total size of Codes {:?}", tvi.len());
        return Ok(tvi);
    }

    fn execute_token_vector(
        &mut self,
        token_vector: &Vec<Token>,
        gas_limit: GasLimit,
    ) -> Result<(), ForthError> {
        let mut ol = self.compile_token_vector(token_vector)?;
        println!("Compiled Opcodes: {:?}", ol);
        self.sm.st.opcodes.resize(self.last_function, Opcode::NOP);
        self.sm.st.opcodes.append(&mut ol);
        self.sm.st.opcodes.push(Opcode::RET);
        self.sm.execute(self.last_function, gas_limit)?;
        println!("Total opcodes defined: {}", self.sm.st.opcodes.len());
        println!("Total opcodes executed: {}", self.sm.st.gas_used());

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

    #[test]
    fn test_if_else_1() {
        let mut fc = ForthCompiler::new();

        fc.execute_string(
            "1 2 3 POP POP POP 0 IF 1 2 ADD ELSE 3 4 ADD THEN",
            GasLimit::Limited(100),
        )
        .unwrap();

        assert_eq!(&fc.sm.st.number_stack, &vec![3_i64]);
    }

    #[test]
    fn test_if_else_2() {
        let mut fc = ForthCompiler::new();

        fc.execute_string(
            "1 2 3 POP POP POP 1 IF 1 2 ADD ELSE 3 4 ADD THEN",
            GasLimit::Limited(100),
        )
        .unwrap();

        assert_eq!(&fc.sm.st.number_stack, &vec![7_i64]);
    }

    #[test]
    fn test_if_else_3() {
        let mut fc = ForthCompiler::new();

        fc.execute_string("0 IF 1 2 ADD ELSE 3 4 ADD THEN", GasLimit::Limited(100))
            .unwrap();

        assert_eq!(&fc.sm.st.number_stack, &vec![3_i64]);
    }

    #[test]
    fn test_if_else_4() {
        let mut fc = ForthCompiler::new();

        fc.execute_string("1 IF 1 2 ADD ELSE 3 4 ADD THEN", GasLimit::Limited(100))
            .unwrap();

        assert_eq!(&fc.sm.st.number_stack, &vec![7_i64]);
    }
}
