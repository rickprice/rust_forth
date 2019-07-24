pub enum TrapHandled {
    Handled,
    NotHandled,
}

pub enum StackMachineError {
    UnkownError,
    NumberStackUnderflow,
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
    TRAP,
}

pub struct StackMachineState {
    pub number_stack: Vec<i64>,
    return_stack: Vec<usize>,
    pub opcodes: Vec<Opcode>,
    pc: usize,
}

impl StackMachineState {
    pub fn new() -> StackMachineState {
        StackMachineState {
            number_stack: Vec::new(),
            return_stack: Vec::new(),
            opcodes: Vec::new(),
            pc: 0,
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

    pub fn execute(&mut self, starting_point: usize) -> Option<StackMachineError> {
        self.st.pc = starting_point;

        loop {
            match self.st.opcodes[self.st.pc] {
                Opcode::JMP => self.st.pc = self.st.number_stack.pop().map(|x| x as usize)? - 1,
                Opcode::JR => self.st.pc += self.st.number_stack.pop().map(|x| x as usize)?,
                Opcode::CALL => {
                    self.st.return_stack.push(self.st.pc + 1);
                    self.st.pc = self.st.number_stack.pop().map(|x| x as usize)?;
                }
                Opcode::JRZ => {
                    let x = self.st.number_stack.pop()?;
                    let offset = self.st.number_stack.pop().map(|x| x as usize)?;
                    if x == 0 {
                        self.st.pc += offset;
                    }
                }
                Opcode::LDI(x) => self.st.number_stack.push(x),
                Opcode::POP => {
                    let _ = self.st.number_stack.pop()?;
                }
                Opcode::RET => match self.st.return_stack.pop() {
                    None => return None,
                    Some(oldpc) => self.st.pc = oldpc,
                },
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
            };
            // +++ FIX THIS +++ This needs to be modified for jumps and calls, or at least they need to be modified
            self.st.pc += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_jr() {
        let mut sm = StackMachine::new();

        // Populate the number stack
        sm.st.number_stack.extend_from_slice(&[321, 39483]);
        // Put the opcodes into the *memory*
        sm.st.opcodes.extend_from_slice(&[
            Opcode::LDI(0),
            Opcode::LDI(1),
            Opcode::LDI(2),
            Opcode::LDI(1), // Jump to location 6 with the JR statement, relative jump of 1
            Opcode::JR,
            Opcode::LDI(3),
            Opcode::LDI(4),
            Opcode::LDI(5),
            Opcode::RET,
        ]);

        // Execute the instructions
        sm.execute(0);

        assert_eq!(sm.st.number_stack, vec![321, 39483, 0, 1, 2, 4, 5]);
    }

    #[test]
    fn test_execute_jrz() {
        let mut sm = StackMachine::new();

        // Populate the number stack
        sm.st.number_stack.extend_from_slice(&[321, 39483]);
        // Put the opcodes into the *memory*
        sm.st.opcodes.extend_from_slice(&[
            Opcode::LDI(0),
            Opcode::LDI(1),
            Opcode::LDI(2),
            Opcode::LDI(1), // TOS for JRZ
            Opcode::LDI(1), // This won't happen because TOS won't be zero...
            Opcode::JRZ,
            Opcode::LDI(3),
            Opcode::LDI(4),
            Opcode::LDI(5),
            Opcode::LDI(1), // Relative Jump of 1
            Opcode::LDI(0),
            Opcode::JRZ, // Jump over the LDI(6)
            Opcode::LDI(6),
            Opcode::LDI(7),
            Opcode::LDI(8),
            Opcode::RET,
        ]);

        // Execute the instructions
        sm.execute(0);

        assert_eq!(sm.st.number_stack, vec![321, 39483, 0, 1, 2, 3, 4, 5, 7, 8]);
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
        sm.execute(0);

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
        sm.execute(0);

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
        sm.execute(0);

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
        sm.execute(0);

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
        sm.execute(0);

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
        sm.execute(0);

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
        sm.execute(0);

        assert_eq!(sm.st.number_stack, vec![123]);
    }
}
