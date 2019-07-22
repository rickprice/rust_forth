
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
    fn handle_trap(&mut self, sm: &mut StackMachine) -> Result<TrapHandled, StackMachineError>;
}


#[derive(Debug, Clone)]
pub enum Opcode {
    JMP,
    JR,
    JRZ,
    CALL,
    LDI(i64),
    POP,
    RET,
    ADD,
    SUB,
    MUL,
    DIV,
    TRAP,
}

pub struct StackMachine {
    pub number_stack: Vec<i64>,
    return_stack: Vec<usize>,
    pub opcodes: Vec<Opcode>,
    pub trap_handlers: Vec<Box<dyn HandleTrap>>,
    pc: usize,
}

impl StackMachine {
    pub fn new() -> StackMachine {
        StackMachine {
            number_stack: Vec::new(),
            return_stack: Vec::new(),
            opcodes: Vec::new(),
            trap_handlers: Vec::new(),
            pc: 0,
        }
    }

    pub fn execute(&mut self, startingPoint: usize)->Option<StackMachineError> {
        self.pc = startingPoint;

        loop {
            match self.opcodes[self.pc] {
                JMP=> self.pc = self.number_stack.pop().map(|x| x as usize)?,
                JR=> self.pc += self.number_stack.pop().map(|x| x as usize)?,
                CALL=> {
                    self.return_stack.push(self.pc+1);
                    self.pc = self.number_stack.pop().map(|x| x as usize)?;
                },
                JRZ=> {
                    let x = self.number_stack.pop()?;
                    if x == 0 {
                        self.pc += self.number_stack.pop().map(|x| x as usize)?;
                    }
                },
                LDI(x)=>self.number_stack.push(x),
                POP=> { let _ = self.number_stack.pop()?;},
                RET=> {
                    match self.return_stack.pop() {
                        None=>return None,
                        Some(oldpc)=>self.pc=oldpc,
                    },
                }
                ADD=> {
                    let x = self.number_stack.pop()?;
                    let y = self.number_stack.pop()?;
                    self.number_stack.push(x+y);
                },
                SUB=> {
                    let x = self.number_stack.pop()?;
                    let y = self.number_stack.pop()?;
                    self.number_stack.push(x-y);
                },
                MUL=> {
                    let x = self.number_stack.pop()?;
                    let y = self.number_stack.pop()?;
                    self.number_stack.push(x*y);
                },
                DIV=> {
                    let x = self.number_stack.pop()?;
                    let y = self.number_stack.pop()?;
                    self.number_stack.push(x/y);
                },
                SWAP=> {
                    let x = self.number_stack.pop()?;
                    let y = self.number_stack.pop()?;
                    self.number_stack.push(x);
                    self.number_stack.push(y);
                },
                TRAP=>{
                    for h in self.trap_handlers.iter() {
                        if let TrapHandled::Handled = h.handle_trap(&mut self)? {
                            break;
                        }
                    }
                }
            }
        }
    }
}
