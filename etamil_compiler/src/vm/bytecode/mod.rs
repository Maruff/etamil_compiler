// Bytecode instruction set for the eTamil VM
pub mod compiler;

/// Bytecode instruction set
#[derive(Debug, Clone)]
pub enum Instruction {
    // Stack operations
    Push(crate::vm::Value),
    Pop,
    
    // Variable operations
    StoreVar(String),
    LoadVar(String),
    
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    
    // Comparison
    Equal,
    NotEqual,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,
    
    // Logical
    And,
    Or,
    Not,
    
    // Control flow
    JumpIfFalse(usize),
    Jump(usize),
    
    // I/O
    Print,
    Input,
    
    // String operations
    Concat,
    
    // File I/O
    FileOpen(String), // mode
    FileClose,
    FileWrite,
    FileRead,
    
    // Database
    DBConnect(String), // db_type
    DBQuery,
    DBExecute,
    
    // API
    DefineRoute(String, String), // method, path
    StartServer(String, u16),    // host, port
    
    // Functions
    Call(String),
    Return,
    
    // Misc
    Nop,
    Halt,
}

/// Complete bytecode program
#[derive(Debug, Clone)]
pub struct Bytecode {
    pub instructions: Vec<Instruction>,
}

impl Bytecode {
    pub fn new() -> Self {
        Bytecode {
            instructions: Vec::new(),
        }
    }

    pub fn push(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    pub fn len(&self) -> usize {
        self.instructions.len()
    }
}
