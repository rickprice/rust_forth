use super::stack_machine::Opcode;
use super::stack_machine::StackMachine;
use std::collections::HashMap;

macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = ::std::collections::HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}

static intrinsic_words: HashMap<&str, Vec<Opcode>> = hashmap![
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
 ];

pub struct ForthCompiler {
    pub sm: StackMachine,
    word_addresses: HashMap<String,usize>,
    last_function: usize,
}

impl ForthCompiler {
    pub fn new() -> ForthCompiler {
        ForthCompiler {
            sm: StackMachine::new(),
            word_addresses: HashMap::new(),
            last_function: 0,
        }
    }
} 
