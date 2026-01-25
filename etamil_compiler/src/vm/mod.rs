// eTamil Virtual Machine (VM)
// Executes bytecode independently without LLVM or Rust compilation

pub mod bytecode;
pub mod interpreter;
pub mod value;

pub use bytecode::{Bytecode, Instruction};
pub use interpreter::VM;
pub use value::Value;

/// Initialize the eTamil VM with bytecode
pub fn create_vm() -> VM {
    VM::new()
}

/// Compile AST to bytecode (from parser module)
pub fn compile_to_bytecode(statements: Vec<crate::parser::Stmt>) -> Bytecode {
    bytecode::compiler::BytecodeCompiler::compile_statements(statements)
}

/// Execute bytecode directly
pub fn execute(bytecode: Bytecode) -> Result<(), String> {
    let mut vm = VM::new();
    vm.execute(bytecode)
}

// Re-export for convenience
pub use bytecode::compiler::BytecodeCompiler;
