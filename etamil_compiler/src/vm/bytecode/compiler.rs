// Bytecode compiler: Converts AST to bytecode instructions
use crate::parser::{Expr, Stmt};
use crate::vm::bytecode::{Bytecode, FunctionInfo, Instruction};
use crate::vm::Value;

pub struct BytecodeCompiler {
    bytecode: Bytecode,
}

impl BytecodeCompiler {
    pub fn new() -> Self {
        BytecodeCompiler {
            bytecode: Bytecode::new(),
        }
    }

    pub fn compile_statements(statements: Vec<Stmt>) -> Bytecode {
        let mut compiler = BytecodeCompiler::new();
        for stmt in statements {
            compiler.compile_stmt(stmt);
        }
        compiler.bytecode.push(Instruction::Halt);
        compiler.bytecode
    }

    fn compile_stmt(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::Assign { name, value } => {
                self.compile_expr(value);
                self.bytecode.push(Instruction::StoreVar(name));
            }
            Stmt::FunctionDef { name, params, body } => {
                // The body is emitted inline, so execution has to jump over it.
                let jump_idx = self.bytecode.len();
                self.bytecode.push(Instruction::Jump(0)); // patched below

                let start = self.bytecode.len();
                for stmt in body {
                    self.compile_stmt(stmt);
                }
                // Falling off the end returns nil.
                self.bytecode.push(Instruction::Push(Value::Null));
                self.bytecode.push(Instruction::Return);

                let end = self.bytecode.len();
                self.bytecode.instructions[jump_idx] = Instruction::Jump(end);
                self.bytecode
                    .functions
                    .insert(name, FunctionInfo { start, params });
            }
            Stmt::Return(value) => {
                match value {
                    Some(expr) => self.compile_expr(expr),
                    None => self.bytecode.push(Instruction::Push(Value::Null)),
                }
                self.bytecode.push(Instruction::Return);
            }
            Stmt::Expression(expr) => {
                // Evaluated for its effect; discard whatever it left behind.
                self.compile_expr(expr);
                self.bytecode.push(Instruction::Pop);
            }
            Stmt::Print(expr) => {
                self.compile_expr(expr);
                self.bytecode.push(Instruction::Print);
            }
            Stmt::Input(expr) => {
                self.bytecode.push(Instruction::Input);
                if let Expr::Variable(name) = expr {
                    self.bytecode.push(Instruction::StoreVar(name));
                }
            }
            Stmt::If { condition, then_branch, else_branch } => {
                self.compile_expr(condition);
                
                let jump_false_idx = self.bytecode.len();
                self.bytecode.push(Instruction::JumpIfFalse(0)); // Placeholder
                
                for stmt in then_branch {
                    self.compile_stmt(stmt);
                }
                
                match else_branch {
                    Some(else_stmts) => {
                        let jump_idx = self.bytecode.len();
                        self.bytecode.push(Instruction::Jump(0)); // Placeholder
                        
                        // Patch jump_if_false to skip to else
                        let else_start = self.bytecode.len();
                        if let Instruction::JumpIfFalse(_) = &mut self.bytecode.instructions[jump_false_idx] {
                            self.bytecode.instructions[jump_false_idx] = Instruction::JumpIfFalse(else_start);
                        }
                        
                        for stmt in else_stmts {
                            self.compile_stmt(stmt);
                        }
                        
                        // Patch final jump
                        let end = self.bytecode.len();
                        if let Instruction::Jump(_) = &mut self.bytecode.instructions[jump_idx] {
                            self.bytecode.instructions[jump_idx] = Instruction::Jump(end);
                        }
                    }
                    None => {
                        let end = self.bytecode.len();
                        if let Instruction::JumpIfFalse(_) = &mut self.bytecode.instructions[jump_false_idx] {
                            self.bytecode.instructions[jump_false_idx] = Instruction::JumpIfFalse(end);
                        }
                    }
                }
            }
            Stmt::Loop { condition, body } => {
                let loop_start = self.bytecode.len();
                
                self.compile_expr(condition);
                let jump_false_idx = self.bytecode.len();
                self.bytecode.push(Instruction::JumpIfFalse(0)); // Placeholder
                
                for stmt in body {
                    self.compile_stmt(stmt);
                }
                
                self.bytecode.push(Instruction::Jump(loop_start));
                
                let end = self.bytecode.len();
                if let Instruction::JumpIfFalse(_) = &mut self.bytecode.instructions[jump_false_idx] {
                    self.bytecode.instructions[jump_false_idx] = Instruction::JumpIfFalse(end);
                }
            }
            Stmt::FileOpen { filename, mode } => {
                self.compile_expr(filename);
                self.bytecode.push(Instruction::FileOpen(mode));
            }
            Stmt::FileClose { filename } => {
                self.compile_expr(filename);
                self.bytecode.push(Instruction::FileClose);
            }
            Stmt::FileWrite { filename, data } => {
                self.compile_expr(filename);
                self.compile_expr(data);
                self.bytecode.push(Instruction::FileWrite);
            }
            Stmt::FileRead { filename, variable } => {
                self.compile_expr(filename);
                self.bytecode.push(Instruction::FileRead);
                self.bytecode.push(Instruction::StoreVar(variable));
            }
            Stmt::ReadCSV { filename, variable } => {
                self.compile_expr(filename);
                self.bytecode.push(Instruction::ReadCSV);
                self.bytecode.push(Instruction::StoreVar(variable));
            }
            Stmt::WriteCSV { filename, data } => {
                self.compile_expr(filename);
                self.compile_expr(data);
                self.bytecode.push(Instruction::WriteCSV);
            }
            // Database and server statements parse, but the VM has no runtime
            // for them yet. Emit an instruction that fails loudly rather than
            // silently doing nothing.
            other => {
                let label = Self::stmt_label(&other);
                self.bytecode.push(Instruction::Unsupported(label));
            }
        }
    }

    /// Human-readable name for a statement the VM cannot execute.
    fn stmt_label(stmt: &Stmt) -> String {
        match stmt {
            Stmt::DBConnect { .. } => "தளம்_இணை (database connect)",
            Stmt::DBDisconnect { .. } => "தளம்_பிரி (database disconnect)",
            Stmt::DBQuery { .. } => "தளம்_வினா (database query)",
            Stmt::DBExecute { .. } => "தளம்_செய் (database execute)",
            Stmt::DBInsert { .. } => "தளம்_செருக (database insert)",
            Stmt::DBUpdate { .. } => "தளம்_புதுப்பி (database update)",
            Stmt::DBDelete { .. } => "தளம்_நீக்கு (database delete)",
            Stmt::CreateTable { .. } => "அட்டை_ஆக்கு (create table)",
            Stmt::Select { .. } => "தேர்வெடு (select)",
            Stmt::DefineRoute { .. } => "வழி (route)",
            Stmt::StartServer { .. } => "வழங்கி_தொடங்கு (start server)",
            Stmt::StopServer => "வழங்கி_நிறுத்து (stop server)",
            Stmt::SendResponse { .. } => "பதில் (response)",
            Stmt::SendJSON { .. } => "ஜேசான்_உரை (json response)",
            Stmt::GetRequestBody { .. } => "உடல் (request body)",
            Stmt::GetRequestParam { .. } => "அளவுரு (request param)",
            Stmt::GetHeader { .. } | Stmt::SetHeader { .. } => "தலைப்பு (header)",
            _ => "this statement",
        }
        .to_string()
    }

    fn compile_expr(&mut self, expr: Expr) {
        match expr {
            Expr::Number(n) => {
                self.bytecode.push(Instruction::Push(Value::Number(n)));
            }
            Expr::String(s) => {
                self.bytecode.push(Instruction::Push(Value::String(s)));
            }
            Expr::Boolean(b) => {
                self.bytecode.push(Instruction::Push(Value::Boolean(b)));
            }
            Expr::Null => {
                self.bytecode.push(Instruction::Push(Value::Null));
            }
            Expr::Variable(name) => {
                self.bytecode.push(Instruction::LoadVar(name));
            }
            Expr::BinaryOp { op, left, right } => {
                self.compile_expr(*left);
                self.compile_expr(*right);
                
                match op.as_str() {
                    "+" => self.bytecode.push(Instruction::Add),
                    "-" => self.bytecode.push(Instruction::Subtract),
                    "*" => self.bytecode.push(Instruction::Multiply),
                    "/" => self.bytecode.push(Instruction::Divide),
                    "%" => self.bytecode.push(Instruction::Modulo),
                    _ => {}
                }
            }
            Expr::Comparison { left, op, right } => {
                self.compile_expr(*left);
                self.compile_expr(*right);
                
                match op.as_str() {
                    // The parser emits "==" for equality; matching "=" here
                    // meant no instruction was emitted and both operands were
                    // left on the stack, so every == comparison silently
                    // tested the truthiness of its right-hand side instead.
                    "==" => self.bytecode.push(Instruction::Equal),
                    "!=" => self.bytecode.push(Instruction::NotEqual),
                    "<" => self.bytecode.push(Instruction::LessThan),
                    "<=" => self.bytecode.push(Instruction::LessOrEqual),
                    ">" => self.bytecode.push(Instruction::GreaterThan),
                    ">=" => self.bytecode.push(Instruction::GreaterOrEqual),
                    other => panic!("Unknown comparison operator: {}", other),
                }
            }
            Expr::Logical { op, left, right } => {
                self.compile_expr(*left);
                self.compile_expr(*right);

                match op.as_str() {
                    "&&" => self.bytecode.push(Instruction::And),
                    "||" => self.bytecode.push(Instruction::Or),
                    other => panic!("Unknown logical operator: {}", other),
                }
            }
            Expr::Not(inner) => {
                self.compile_expr(*inner);
                self.bytecode.push(Instruction::Not);
            }
            Expr::Call { name, args } => {
                let argc = args.len();
                for arg in args {
                    self.compile_expr(arg);
                }
                self.bytecode.push(Instruction::Call(name, argc));
            }
            Expr::Concat { left, right } => {
                self.compile_expr(*left);
                self.compile_expr(*right);
                self.bytecode.push(Instruction::Concat);
            }
        }
    }
}
