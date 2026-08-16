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

    // Collections
    /// Pop n values into an array (அணி — a column).
    MakeArray(usize),
    /// Pop one value per key into a record (பொருள் — a row).
    MakeRecord(Vec<String>),
    /// Pop index then base; push the element.
    Index,
    /// Pop base; push the named field.
    Field(String),
    /// Pop index and value; store into the named variable's collection.
    SetIndex(String),
    /// Pop value; store into the named variable's field.
    SetField(String, String),
    /// Pop a collection; push how many elements it has.
    Length,
    /// Pop index then collection; push the nth element of an array, the nth
    /// key of a record (keys sorted so iteration order is stable), or the
    /// nth character of a string. Used to desugar ஒவ்வொரு.
    NthOrKey,
    
    // File I/O — each pops its operands off the stack
    FileOpen(String), // mode; pops filename
    FileClose,        // pops filename
    FileWrite,        // pops data, then filename
    FileRead,         // pops filename, pushes contents
    ReadCSV,          // pops filename, pushes the number of data rows
    WriteCSV,         // pops row, then filename

    // Database
    DBConnect(String), // db_type
    DBQuery,
    DBExecute,

    // API
    DefineRoute(String, String), // method, path
    StartServer(String, u16),    // host, port

    // Functions
    /// Call a named function with this many arguments already on the stack.
    Call(String, usize),
    /// Pop the return value, restore the caller's frame, push the value back.
    Return,

    // Misc
    Nop,
    /// A statement the VM cannot execute. Carries the message shown to the
    /// user; executing it is a runtime error rather than a silent no-op.
    Unsupported(String),
    Halt,
}

/// Where a function's body starts, and the names its arguments bind to.
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub start: usize,
    pub params: Vec<String>,
}

/// Complete bytecode program
#[derive(Debug, Clone)]
pub struct Bytecode {
    pub instructions: Vec<Instruction>,
    /// Function bodies are emitted inline and jumped over; this maps a name
    /// to its entry point. Resolution happens at call time, so functions may
    /// be defined in any order and may recurse.
    pub functions: std::collections::HashMap<String, FunctionInfo>,
}

impl Bytecode {
    pub fn new() -> Self {
        Bytecode {
            instructions: Vec::new(),
            functions: std::collections::HashMap::new(),
        }
    }

    pub fn push(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    pub fn len(&self) -> usize {
        self.instructions.len()
    }
}
