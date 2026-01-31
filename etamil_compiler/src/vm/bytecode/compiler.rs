// Bytecode compiler: Converts AST to bytecode instructions
use crate::parser::{Expr, Stmt};
use crate::vm::bytecode::{Bytecode, Instruction};
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
            Stmt::FileWrite { filename, data } => {
                self.compile_expr(filename);
                self.bytecode.push(Instruction::FileWrite);
                self.compile_expr(data);
                self.bytecode.push(Instruction::FileWrite);
            }
            Stmt::FileRead { filename, variable } => {
                self.compile_expr(filename);
                self.bytecode.push(Instruction::FileRead);
                self.bytecode.push(Instruction::StoreVar(variable));
            }
            Stmt::DBQuery { query, result_var } => {
                self.compile_expr(query);
                self.bytecode.push(Instruction::DBQuery);
                if let Some(var) = result_var {
                    self.bytecode.push(Instruction::StoreVar(var));
                }
            }
            _ => {
                // Other statements handled as needed
            }
        }
    }

    fn compile_expr(&mut self, expr: Expr) {
        match expr {
            Expr::Number(n) => {
                self.bytecode.push(Instruction::Push(Value::Number(n)));
            }
            Expr::String(s) => {
                self.bytecode.push(Instruction::Push(Value::String(s)));
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
                    "=" => self.bytecode.push(Instruction::Equal),
                    "!=" => self.bytecode.push(Instruction::NotEqual),
                    "<" => self.bytecode.push(Instruction::LessThan),
                    "<=" => self.bytecode.push(Instruction::LessOrEqual),
                    ">" => self.bytecode.push(Instruction::GreaterThan),
                    ">=" => self.bytecode.push(Instruction::GreaterOrEqual),
                    _ => {}
                }
            }
            Expr::Concat { left, right } => {
                self.compile_expr(*left);
                self.compile_expr(*right);
                self.bytecode.push(Instruction::Concat);
            }
        }
    }
}
