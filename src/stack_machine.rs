use std::convert::TryFrom;

pub enum GasLimit {
    Unlimited,
    Limited(u64),
}

#[derive(Debug)]
pub enum StackMachineError {
    UnkownError,
    NumberStackUnderflow,
    UnhandledTrap,
    RanOutOfGas,
}

pub enum TrapHandled {
    Handled,
    NotHandled,
}

// Chain of Command Pattern
pub trait HandleTrap {
    fn handle_trap(
        &mut self,
        trap_id: i64,
        st: &mut StackMachineState,
    ) -> Result<TrapHandled, StackMachineError>;
}

pub struct TrapHandler<'a> {
    handled_trap: i64,
    to_run: Box<dyn Fn(i64, &mut StackMachineState) -> Result<TrapHandled, StackMachineError> + 'a>,
}

impl<'a> TrapHandler<'a> {
    pub fn new<C>(handled_trap: i64, f: C) -> TrapHandler<'a>
    where
        C: Fn(i64, &mut StackMachineState) -> Result<TrapHandled, StackMachineError> + 'a,
    {
        TrapHandler {
            handled_trap: handled_trap,
            to_run: Box::new(f),
        }
    }
}

impl<'a> HandleTrap for TrapHandler<'a> {
    fn handle_trap(
        &mut self,
        trap_number: i64,
        st: &mut StackMachineState,
    ) -> Result<TrapHandled, StackMachineError> {
        if trap_number == self.handled_trap {
            return (self.to_run)(self.handled_trap, st);
        }
        Ok(TrapHandled::NotHandled)
    }
}

#[derive(Debug, Clone)]
pub enum Opcode {
    JMP,
    JR,
    JRZ,
    JRNZ,
    CALL,
    CMPZ,
    CMPNZ,
    LDI(i64),
    POP,
    SWAP,
    RET,
    ADD,
    SUB,
    MUL,
    DIV,
    NOT,
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

impl StackMachineState {
    pub fn gas_used(&self) -> u64 {
        self.gas_used
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
        self.st.gas_used = 0;
        self.st.pc = starting_point;
        loop {
            let mut pc_reset = false;
            match self.st.opcodes[self.st.pc] {
                Opcode::JMP => {
                    self.st.pc = self
                        .st
                        .number_stack
                        .pop()
                        .map(|x| x as usize)
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    pc_reset = true;
                }
                Opcode::JR => {
                    let new_offset = self.st.pc as i128
                        + self
                            .st
                            .number_stack
                            .pop()
                            .ok_or(StackMachineError::NumberStackUnderflow)?
                            as i128;
                    self.st.pc = usize::try_from(new_offset).unwrap();
                    pc_reset = true;
                }
                Opcode::CALL => {
                    self.st.return_stack.push(self.st.pc + 1);
                    self.st.pc = self
                        .st
                        .number_stack
                        .pop()
                        .map(|x| x as usize)
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    pc_reset = true;
                }
                Opcode::CMPZ => {
                    let x = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    if x == 0 {
                        self.st.number_stack.push(0);
                    } else {
                        self.st.number_stack.push(-1);
                    }
                }
                Opcode::CMPNZ => {
                    let x = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    if x == 0 {
                        self.st.number_stack.push(-1);
                    } else {
                        self.st.number_stack.push(0);
                    }
                }
                Opcode::JRZ => {
                    let new_offset = self.st.pc as i128
                        + self
                            .st
                            .number_stack
                            .pop()
                            .ok_or(StackMachineError::NumberStackUnderflow)?
                            as i128;
                    let x = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    if x == 0 {
                        self.st.pc = usize::try_from(new_offset).unwrap();
                        pc_reset = true;
                    }
                }
                Opcode::JRNZ => {
                    let new_offset = self.st.pc as i128
                        + self
                            .st
                            .number_stack
                            .pop()
                            .ok_or(StackMachineError::NumberStackUnderflow)?
                            as i128;
                    let x = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    if x != 0 {
                        self.st.pc = usize::try_from(new_offset).unwrap();
                        pc_reset = true;
                    }
                }
                Opcode::LDI(x) => self.st.number_stack.push(x),
                Opcode::POP => {
                    let _ = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                }
                Opcode::RET => {
                    match self.st.return_stack.pop() {
                        None => return Ok(()),
                        Some(oldpc) => self.st.pc = oldpc,
                    };
                    pc_reset = true;
                }
                Opcode::ADD => {
                    let x = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    let y = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    self.st.number_stack.push(x + y);
                }
                Opcode::SUB => {
                    let x = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    let y = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    self.st.number_stack.push(x - y);
                }
                Opcode::MUL => {
                    let x = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    let y = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    self.st.number_stack.push(x * y);
                }
                Opcode::DIV => {
                    let x = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    let y = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    self.st.number_stack.push(x / y);
                }
                Opcode::NOT => {
                    let x = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    self.st.number_stack.push(!x);
                }
                Opcode::DUP => {
                    let x = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    self.st.number_stack.push(x);
                    self.st.number_stack.push(x);
                }
                Opcode::SWAP => {
                    let x = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    let y = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    self.st.number_stack.push(x);
                    self.st.number_stack.push(y);
                }
                Opcode::TRAP => {
                    // We are going to say that TRAPs always have a numeric code on the number stack to define which TRAP is being called
                    let trap_id = self
                        .st
                        .number_stack
                        .pop()
                        .ok_or(StackMachineError::NumberStackUnderflow)?;
                    for h in self.trap_handlers.iter_mut() {
                        if let TrapHandled::Handled = h.handle_trap(trap_id, &mut self.st)? {
                            return Ok(());
                        }
                    }
                    return Err(StackMachineError::UnhandledTrap);
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
            Opcode::LDI(1), // This won't happen because TOS won't be zero...
            Opcode::LDI(2), // TOS for JRZ
            Opcode::JRZ,
            Opcode::LDI(3),
            Opcode::LDI(4),
            Opcode::LDI(5),
            Opcode::LDI(0),
            Opcode::LDI(2), // Relative Jump of 1
            Opcode::JRZ,    // Jump over the LDI(6)
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
            Opcode::LDI(1),  // This won't happen because TOS won't be zero...
            Opcode::LDI(-2), // TOS for JRZ
            Opcode::JRZ,
            Opcode::LDI(3),
            Opcode::LDI(4),
            Opcode::LDI(5),
            Opcode::LDI(0),
            Opcode::LDI(-12), // Relative Jump to start of code
            Opcode::JRZ,      // Jump over the LDI(6)
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
    fn test_execute_jrnz_forward() {
        let mut sm = StackMachine::new();

        // Populate the number stack
        sm.st.number_stack.extend_from_slice(&[321, 39483]);
        // Put the opcodes into the *memory*
        sm.st.opcodes.extend_from_slice(&[
            Opcode::LDI(0),
            Opcode::LDI(1),
            Opcode::LDI(2),
            Opcode::LDI(0), // This won't happen because TOS is zero...
            Opcode::LDI(2), // TOS for JRZ
            Opcode::JRNZ,
            Opcode::LDI(3),
            Opcode::LDI(4),
            Opcode::LDI(5),
            Opcode::LDI(1),
            Opcode::LDI(2), // Relative Jump of 1
            Opcode::JRNZ,   // Jump over the LDI(6)
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
    fn test_execute_jrnz_backward() {
        let mut sm = StackMachine::new();

        // Populate the number stack
        sm.st.number_stack.extend_from_slice(&[321, 39483]);
        // Put the opcodes into the *memory*
        sm.st.opcodes.extend_from_slice(&[
            Opcode::LDI(0),
            Opcode::RET,
            Opcode::LDI(1),
            Opcode::LDI(2),
            Opcode::LDI(0),  // This won't happen because TOS is zero...
            Opcode::LDI(-2), // TOS for JRZ
            Opcode::JRNZ,
            Opcode::LDI(3),
            Opcode::LDI(4),
            Opcode::LDI(5),
            Opcode::LDI(1),
            Opcode::LDI(-12), // Relative Jump to start of code
            Opcode::JRNZ,     // Jump over the LDI(6)
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
    fn test_execute_cmpz_1() {
        let mut sm = StackMachine::new();

        // Populate the number stack
        sm.st.number_stack.extend_from_slice(&[123, 321, 0]);
        // Put the opcodes into the *memory*
        sm.st
            .opcodes
            .extend_from_slice(&[Opcode::CMPZ, Opcode::RET]);

        // Execute the instructions
        sm.execute(0, GasLimit::Limited(100)).unwrap();

        assert_eq!(sm.st.number_stack, vec![123_i64, 321, 0]);
    }

    #[test]
    fn test_execute_cmpz_2() {
        let mut sm = StackMachine::new();

        // Populate the number stack
        sm.st.number_stack.extend_from_slice(&[123, 321, 1]);
        // Put the opcodes into the *memory*
        sm.st
            .opcodes
            .extend_from_slice(&[Opcode::CMPZ, Opcode::RET]);

        // Execute the instructions
        sm.execute(0, GasLimit::Limited(100)).unwrap();

        assert_eq!(sm.st.number_stack, vec![123_i64, 321, -1]);
    }

    #[test]
    fn test_execute_cmpnz_1() {
        let mut sm = StackMachine::new();

        // Populate the number stack
        sm.st.number_stack.extend_from_slice(&[123, 321, 0]);
        // Put the opcodes into the *memory*
        sm.st
            .opcodes
            .extend_from_slice(&[Opcode::CMPNZ, Opcode::RET]);

        // Execute the instructions
        sm.execute(0, GasLimit::Limited(100)).unwrap();

        assert_eq!(sm.st.number_stack, vec![123_i64, 321, -1]);
    }

    #[test]
    fn test_execute_cmpnz_2() {
        let mut sm = StackMachine::new();

        // Populate the number stack
        sm.st.number_stack.extend_from_slice(&[123, 321, 1]);
        // Put the opcodes into the *memory*
        sm.st
            .opcodes
            .extend_from_slice(&[Opcode::CMPNZ, Opcode::RET]);

        // Execute the instructions
        sm.execute(0, GasLimit::Limited(100)).unwrap();

        assert_eq!(sm.st.number_stack, vec![123_i64, 321, 0]);
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
    #[should_panic]
    fn test_execute_pop_error() {
        let mut sm = StackMachine::new();

        // Populate the number stack
        sm.st.number_stack.extend_from_slice(&[321, 39483]);
        // Put the opcodes into the *memory*
        sm.st.opcodes.extend_from_slice(&[
            Opcode::LDI(0),
            Opcode::LDI(1),
            Opcode::POP,
            Opcode::POP,
            Opcode::POP,
            Opcode::POP,
            Opcode::POP,
            Opcode::LDI(2),
            Opcode::RET,
        ]);

        // Execute the instructions
        sm.execute(0, GasLimit::Limited(100)).unwrap();
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
    fn test_execute_not() {
        let mut sm = StackMachine::new();

        // Populate the number stack
        sm.st.number_stack.extend_from_slice(&[321, 0]);
        // Put the opcodes into the *memory*
        sm.st.opcodes.extend_from_slice(&[Opcode::NOT, Opcode::RET]);

        // Execute the instructions
        sm.execute(0, GasLimit::Limited(100)).unwrap();

        assert_eq!(sm.st.number_stack, vec![321_i64, -1]);
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
    }

    #[test]
    fn test_handle_trap_1() {
        let mut sm = StackMachine::new();

        sm.trap_handlers
            .push(Box::from(TrapHandler::new(100, |_trap_id, st| {
                st.number_stack
                    .pop()
                    .ok_or(StackMachineError::NumberStackUnderflow)?;
                st.number_stack.push(200);
                Ok(TrapHandled::Handled)
            })));

        // Populate the number stack
        sm.st.number_stack.extend_from_slice(&[50_i64, 100]);
        // Put the opcodes into the *memory*
        sm.st
            .opcodes
            .extend_from_slice(&[Opcode::TRAP, Opcode::RET]);

        // Execute the instructions
        sm.execute(0, GasLimit::Limited(100)).unwrap();

        assert_eq!(sm.st.number_stack, vec![200]);
    }

    #[test]
    fn test_handle_trap_2() {
        let mut sm = StackMachine::new();

        sm.trap_handlers
            .push(Box::from(TrapHandler::new(-100, |_trap_id, st| {
                st.number_stack
                    .pop()
                    .ok_or(StackMachineError::NumberStackUnderflow)?;
                st.number_stack.push(-100);
                Ok(TrapHandled::Handled)
            })));
        sm.trap_handlers
            .push(Box::from(TrapHandler::new(100, |_trap_id, st| {
                st.number_stack
                    .pop()
                    .ok_or(StackMachineError::NumberStackUnderflow)?;
                st.number_stack.push(200);
                Ok(TrapHandled::Handled)
            })));
        sm.trap_handlers
            .push(Box::from(TrapHandler::new(-200, |_trap_id, st| {
                st.number_stack
                    .pop()
                    .ok_or(StackMachineError::NumberStackUnderflow)?;
                st.number_stack.push(-200);
                Ok(TrapHandled::Handled)
            })));

        // Populate the number stack, with a value (50), and the trap number (100)
        sm.st.number_stack.extend_from_slice(&[50_i64, 100]);
        // Put the opcodes into the *memory*
        sm.st
            .opcodes
            .extend_from_slice(&[Opcode::TRAP, Opcode::RET]);

        // Execute the instructions
        sm.execute(0, GasLimit::Limited(100)).unwrap();

        assert_eq!(sm.st.number_stack, vec![200]);
    }

    #[test]
    fn test_unhandled_trap_1() {
        let mut sm = StackMachine::new();

        // Populate the number stack, with a value (50), and the trap number (100)
        sm.st.number_stack.extend_from_slice(&[50_i64, 100]);

        // Put the opcodes into the *memory*
        sm.st
            .opcodes
            .extend_from_slice(&[Opcode::TRAP, Opcode::RET]);

        // Execute the instructions
        match sm.execute(0, GasLimit::Limited(100)) {
            Err(StackMachineError::UnhandledTrap) => (),
            r => panic!("Incorrect error type returned {:?}", r),
        }
    }
}
