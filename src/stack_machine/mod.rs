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
                Opcode::JMP => self.st.pc = self.st.number_stack.pop().map(|x| x as usize)?,
                Opcode::JR => self.st.pc += self.st.number_stack.pop().map(|x| x as usize)?,
                Opcode::CALL => {
                    self.st.return_stack.push(self.st.pc + 1);
                    self.st.pc = self.st.number_stack.pop().map(|x| x as usize)?;
                }
                Opcode::JRZ => {
                    let x = self.st.number_stack.pop()?;
                    if x == 0 {
                        self.st.pc += self.st.number_stack.pop().map(|x| x as usize)?;
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
