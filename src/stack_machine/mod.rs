use std::convert::TryFrom;
use std::option;

pub enum GasLimit {
    Unlimited,
    Limited(u64),
}

pub enum TrapHandled {
    Handled,
    NotHandled,
}

#[derive(Debug)]
pub enum StackMachineError {
    UnkownError,
    NoneError,
    NumberStackUnderflow,
    RanOutOfGas,
}

/// Helper to convert a Some/None return to a ForthError error code.
impl From<option::NoneError> for StackMachineError {
    fn from(_: option::NoneError) -> Self {
        StackMachineError::NoneError
    }
}

// Chain of Command Pattern
pub trait HandleTrap {
    fn handle_trap(&mut self, st: &mut StackMachineState)
        -> Result<TrapHandled, StackMachineError>;
}

#[derive(Debug, Clone)]
pub enum Opcode {
    JMP,
    JR,
    JRZ,
    CALL,
    LDI(i64),
    POP,
    SWAP,
    RET,
    ADD,
    SUB,
    MUL,
    DIV,
    DUP,
    TRAP,
    NOP,
}

pub struct StackMachineState {
    pub number_stack: Vec<i64>,
    return_stack: Vec<usize>,
    pub opcodes: Vec<Opcode>,
    pc: usize,
    gas_used: u64,
}

impl StackMachineState {
    pub fn new() -> StackMachineState {
        StackMachineState {
            number_stack: Vec::new(),
            return_stack: Vec::new(),
            opcodes: Vec::new(),
            pc: 0,
            gas_used: 0,
        }
    }
}

pub struct StackMachine {
    pub st: StackMachineState,
    pub trap_handlers: Vec<Box<dyn HandleTrap>>,
}

impl StackMachine {
    pub fn new() -> StackMachine {
        StackMachine {
            st: StackMachineState::new(),
            trap_handlers: Vec::new(),
        }
    }

    pub fn execute(
        &mut self,
        starting_point: usize,
        gas_limit: GasLimit,
    ) -> Result<(), StackMachineError> {
        self.st.pc = starting_point;
        loop {
            let mut pc_reset = false;
            match self.st.opcodes[self.st.pc] {
                Opcode::JMP => {
                    self.st.pc = self.st.number_stack.pop().map(|x| x as usize)?;
                    pc_reset = true;
                }
                Opcode::JR => {
                    let new_offset = self.st.pc as i128 + self.st.number_stack.pop()? as i128;
                    self.st.pc = usize::try_from(new_offset).unwrap();
                    pc_reset = true;
                }
                Opcode::CALL => {
                    self.st.return_stack.push(self.st.pc + 1);
                    self.st.pc = self.st.number_stack.pop().map(|x| x as usize)?;
                    pc_reset = true;
                }
                Opcode::JRZ => {
                    let x = self.st.number_stack.pop()?;
                    let new_offset = self.st.pc as i128 + self.st.number_stack.pop()? as i128;
                    if x == 0 {
                        self.st.pc = usize::try_from(new_offset).unwrap();
                        pc_reset = true;
                    }
                }
                Opcode::LDI(x) => self.st.number_stack.push(x),
                Opcode::POP => {
                    let _ = self.st.number_stack.pop()?;
                }
                Opcode::RET => {
                    match self.st.return_stack.pop() {
                        None => return Ok(()),
                        Some(oldpc) => self.st.pc = oldpc,
                    };
                    pc_reset = true;
                }
                Opcode::ADD => {
                    let x = self.st.number_stack.pop()?;
                    let y = self.st.number_stack.pop()?;
                    self.st.number_stack.push(x + y);
                }
                Opcode::SUB => {
                    let x = self.st.number_stack.pop()?;
                    let y = self.st.number_stack.pop()?;
                    self.st.number_stack.push(x - y);
                }
                Opcode::MUL => {
                    let x = self.st.number_stack.pop()?;
                    let y = self.st.number_stack.pop()?;
                    self.st.number_stack.push(x * y);
                }
                Opcode::DIV => {
                    let x = self.st.number_stack.pop()?;
                    let y = self.st.number_stack.pop()?;
                    self.st.number_stack.push(x / y);
                }
                Opcode::DUP => {
                    let x = self.st.number_stack.pop()?;
                    self.st.number_stack.push(x);
                    self.st.number_stack.push(x);
                }
                Opcode::SWAP => {
                    let x = self.st.number_stack.pop()?;
                    let y = self.st.number_stack.pop()?;
                    self.st.number_stack.push(x);
                    self.st.number_stack.push(y);
                }
                Opcode::TRAP => {
                    for h in self.trap_handlers.iter_mut() {
                        if let TrapHandled::Handled = h.handle_trap(&mut self.st).ok()? {
                            break;
                        }
                    }
                }
                Opcode::NOP => {}
            };
            if pc_reset == false {
                self.st.pc += 1;
            }

            self.st.gas_used += 1;

            if let GasLimit::Limited(x) = gas_limit {
                if self.st.gas_used > x {
                    return Err(StackMachineError::RanOutOfGas);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_jr_forward() {
        let mut sm = StackMachine::new();

        // Populate the number stack
        sm.st.number_stack.extend_from_slice(&[321, 39483]);
        // Put the opcodes into the *memory*
        sm.st.opcodes.extend_from_slice(&[
            Opcode::LDI(0),
            Opcode::LDI(1),
            Opcode::LDI(2),
            Opcode::LDI(2), // Jump to location 6 with the JR statement, relative jump of 1
            Opcode::JR,
            Opcode::LDI(3),
            Opcode::LDI(4),
            Opcode::LDI(5),
            Opcode::RET,
        ]);

        // Execute the instructions
        sm.execute(0, GasLimit::Limited(100)).unwrap();

        assert_eq!(sm.st.number_stack, vec![321, 39483, 0, 1, 2, 4, 5]);
    }

    #[test]
    fn test_execute_jr_backward() {
        let mut sm = StackMachine::new();

        // Populate the number stack
        sm.st.number_stack.extend_from_slice(&[321, 39483]);
        // Put the opcodes into the *memory*
        sm.st.opcodes.extend_from_slice(&[
            Opcode::LDI(0),
            Opcode::LDI(1),
            Opcode::RET,
            Opcode::LDI(2),
            Opcode::LDI(-5), // Jump to the LDI(0)
            Opcode::JR,
        ]);

        // Execute the instructions
        sm.execute(3, GasLimit::Limited(100)).unwrap();

        assert_eq!(sm.st.number_stack, vec![321, 39483, 2, 0, 1]);
    }

    #[test]
    fn test_execute_jrz_forward() {
        let mut sm = StackMachine::new();

        // Populate the number stack
        sm.st.number_stack.extend_from_slice(&[321, 39483]);
        // Put the opcodes into the *memory*
        sm.st.opcodes.extend_from_slice(&[
            Opcode::LDI(0),
            Opcode::LDI(1),
            Opcode::LDI(2),
            Opcode::LDI(2), // TOS for JRZ
            Opcode::LDI(1), // This won't happen because TOS won't be zero...
            Opcode::JRZ,
            Opcode::LDI(3),
            Opcode::LDI(4),
            Opcode::LDI(5),
            Opcode::LDI(2), // Relative Jump of 1
            Opcode::LDI(0),
            Opcode::JRZ, // Jump over the LDI(6)
            Opcode::LDI(6),
            Opcode::LDI(7),
            Opcode::LDI(8),
            Opcode::RET,
        ]);

        // Execute the instructions
        sm.execute(0, GasLimit::Limited(100)).unwrap();

        assert_eq!(sm.st.number_stack, vec![321, 39483, 0, 1, 2, 3, 4, 5, 7, 8]);
    }

    #[test]
    fn test_execute_jrz_backward() {
        let mut sm = StackMachine::new();

        // Populate the number stack
        sm.st.number_stack.extend_from_slice(&[321, 39483]);
        // Put the opcodes into the *memory*
        sm.st.opcodes.extend_from_slice(&[
            Opcode::LDI(0),
            Opcode::RET,
            Opcode::LDI(1),
            Opcode::LDI(2),
            Opcode::LDI(-2), // TOS for JRZ
            Opcode::LDI(1),  // This won't happen because TOS won't be zero...
            Opcode::JRZ,
            Opcode::LDI(3),
            Opcode::LDI(4),
            Opcode::LDI(5),
            Opcode::LDI(-12), // Relative Jump to start of code
            Opcode::LDI(0),
            Opcode::JRZ, // Jump over the LDI(6)
            Opcode::LDI(6),
            Opcode::LDI(7),
            Opcode::LDI(8),
            Opcode::RET,
        ]);

        // Execute the instructions
        sm.execute(2, GasLimit::Limited(100)).unwrap();

        assert_eq!(sm.st.number_stack, vec![321, 39483, 1, 2, 3, 4, 5, 0]);
    }

    #[test]
    fn test_execute_call() {
        let mut sm = StackMachine::new();

        // Populate the number stack
        sm.st.number_stack.extend_from_slice(&[321, 39483]);
        // Put the opcodes into the *memory*
        sm.st.opcodes.extend_from_slice(&[
            Opcode::LDI(0),
            Opcode::LDI(5),
            Opcode::CALL,
            Opcode::LDI(1),
            Opcode::RET,
            Opcode::LDI(2),
            Opcode::LDI(10),
            Opcode::CALL,
            Opcode::LDI(3),
            Opcode::RET,
            Opcode::LDI(4),
            Opcode::LDI(15),
            Opcode::CALL,
            Opcode::LDI(5),
            Opcode::RET,
            Opcode::LDI(6),
            Opcode::LDI(20),
            Opcode::CALL,
            Opcode::LDI(7),
            Opcode::RET,
            Opcode::LDI(8),
            Opcode::LDI(25),
            Opcode::CALL,
            Opcode::LDI(9),
            Opcode::RET,
            Opcode::RET,
        ]);

        // Execute the instructions
        sm.execute(0, GasLimit::Limited(100)).unwrap();

        assert_eq!(
            sm.st.number_stack,
            vec![321, 39483, 0, 2, 4, 6, 8, 9, 7, 5, 3, 1]
        );
    }

    #[test]
    fn test_execute_ldi() {
        let mut sm = StackMachine::new();

        // Populate the number stack
        sm.st.number_stack.extend_from_slice(&[321, 39483]);
        // Put the opcodes into the *memory*
        sm.st.opcodes.extend_from_slice(&[
            Opcode::LDI(0),
            Opcode::LDI(1),
            Opcode::LDI(2),
            Opcode::RET,
        ]);

        // Execute the instructions
        sm.execute(0, GasLimit::Limited(100)).unwrap();

        assert_eq!(sm.st.number_stack, vec![321, 39483, 0, 1, 2]);
    }

    #[test]
    fn test_execute_pop() {
        let mut sm = StackMachine::new();

        // Populate the number stack
        sm.st.number_stack.extend_from_slice(&[321, 39483]);
        // Put the opcodes into the *memory*
        sm.st.opcodes.extend_from_slice(&[
            Opcode::LDI(0),
            Opcode::LDI(1),
            Opcode::POP,
            Opcode::LDI(2),
            Opcode::RET,
        ]);

        // Execute the instructions
        sm.execute(0, GasLimit::Limited(100)).unwrap();

        assert_eq!(sm.st.number_stack, vec![321, 39483, 0, 2]);
    }

    #[test]
    fn test_execute_swap() {
        let mut sm = StackMachine::new();

        // Populate the number stack
        sm.st.number_stack.extend_from_slice(&[321, 39483]);
        // Put the opcodes into the *memory*
        sm.st.opcodes.extend_from_slice(&[
            Opcode::LDI(0),
            Opcode::LDI(1),
            Opcode::SWAP,
            Opcode::LDI(2),
            Opcode::RET,
        ]);

        // Execute the instructions
        sm.execute(0, GasLimit::Limited(100)).unwrap();

        assert_eq!(sm.st.number_stack, vec![321, 39483, 1, 0, 2]);
    }

    #[test]
    fn test_execute_add() {
        let mut sm = StackMachine::new();

        // Populate the number stack
        sm.st.number_stack.extend_from_slice(&[123, 321]);
        // Put the opcodes into the *memory*
        sm.st.opcodes.extend_from_slice(&[Opcode::ADD, Opcode::RET]);

        // Execute the instructions
        sm.execute(0, GasLimit::Limited(100)).unwrap();

        assert_eq!(sm.st.number_stack, vec![444]);
    }

    #[test]
    fn test_execute_sub() {
        let mut sm = StackMachine::new();

        // Populate the number stack
        sm.st.number_stack.extend_from_slice(&[321, 444]);
        // Put the opcodes into the *memory*
        sm.st.opcodes.extend_from_slice(&[Opcode::SUB, Opcode::RET]);

        // Execute the instructions
        sm.execute(0, GasLimit::Limited(100)).unwrap();

        assert_eq!(sm.st.number_stack, vec![123]);
    }

    #[test]
    fn test_execute_mul() {
        let mut sm = StackMachine::new();

        // Populate the number stack
        sm.st.number_stack.extend_from_slice(&[321, 123]);
        // Put the opcodes into the *memory*
        sm.st.opcodes.extend_from_slice(&[Opcode::MUL, Opcode::RET]);

        // Execute the instructions
        sm.execute(0, GasLimit::Limited(100)).unwrap();

        assert_eq!(sm.st.number_stack, vec![39483]);
    }

    #[test]
    fn test_execute_div() {
        let mut sm = StackMachine::new();

        // Populate the number stack
        sm.st.number_stack.extend_from_slice(&[321, 39483]);
        // Put the opcodes into the *memory*
        sm.st.opcodes.extend_from_slice(&[Opcode::DIV, Opcode::RET]);

        // Execute the instructions
        sm.execute(0, GasLimit::Limited(100)).unwrap();

        assert_eq!(sm.st.number_stack, vec![123]);
    }

    #[test]
    fn test_execute_dup() {
        let mut sm = StackMachine::new();

        // Populate the number stack
        sm.st.number_stack.extend_from_slice(&[123, 39483]);
        // Put the opcodes into the *memory*
        sm.st.opcodes.extend_from_slice(&[Opcode::DUP, Opcode::RET]);

        // Execute the instructions
        sm.execute(0, GasLimit::Limited(100)).unwrap();

        assert_eq!(sm.st.number_stack, vec![123, 39483, 39483]);
    }

    #[test]
    #[should_panic]
    fn test_execute_run_out_of_gas() {
        let mut sm = StackMachine::new();

        // Populate the number stack
        sm.st.number_stack.extend_from_slice(&[321, 39483]);
        // Put the opcodes into the *memory*
        sm.st.opcodes.extend_from_slice(&[
            Opcode::LDI(0),
            Opcode::LDI(5),
            Opcode::CALL,
            Opcode::LDI(1),
            Opcode::RET,
            Opcode::LDI(2),
            Opcode::LDI(10),
            Opcode::CALL,
            Opcode::LDI(3),
            Opcode::RET,
            Opcode::LDI(4),
            Opcode::LDI(15),
            Opcode::CALL,
            Opcode::LDI(5),
            Opcode::RET,
            Opcode::LDI(6),
            Opcode::LDI(20),
            Opcode::CALL,
            Opcode::LDI(7),
            Opcode::RET,
            Opcode::LDI(8),
            Opcode::LDI(25),
            Opcode::CALL,
            Opcode::LDI(9),
            Opcode::RET,
            Opcode::RET,
        ]);

        // Execute the instructions
        sm.execute(0, GasLimit::Limited(10)).unwrap();

        assert_eq!(
            sm.st.number_stack,
            vec![321, 39483, 0, 2, 4, 6, 8, 9, 7, 5, 3, 1]
        );
    }
}
