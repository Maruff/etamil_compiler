// Bytecode compiler: Converts AST to bytecode instructions
use crate::parser::{Expr, Stmt};
use crate::vm::bytecode::{Bytecode, FunctionInfo, Instruction};
use crate::vm::Value;
use rust_decimal::Decimal;

pub struct BytecodeCompiler {
    bytecode: Bytecode,
    /// Makes each ஒவ்வொரு loop's hidden variables unique.
    loop_id: usize,
}

impl BytecodeCompiler {
    pub fn new() -> Self {
        BytecodeCompiler {
            bytecode: Bytecode::new(),
            loop_id: 0,
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
            // The declared type is the checker's business, not the VM's: by
            // the time bytecode is emitted the program has already been
            // accepted, so there is nothing left to enforce here.
            Stmt::Assign { name, value, declared: _, at: _ } => {
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
            Stmt::SetIndex { name, index, value } => {
                self.compile_expr(index);
                self.compile_expr(value);
                self.bytecode.push(Instruction::SetIndex(name));
            }
            Stmt::SetField { name, field, value } => {
                self.compile_expr(value);
                self.bytecode.push(Instruction::SetField(name, field));
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
            // ovvoru item il collection { ... } is desugared into an index
            // loop over hidden variables. The '#' prefix cannot appear in a
            // user identifier, so these can never collide with a real name.
            Stmt::ForEach { var, collection, body } => {
                let id = self.loop_id;
                self.loop_id += 1;
                let items = format!("#each_items_{}", id);
                let index = format!("#each_i_{}", id);

                self.compile_expr(collection);
                self.bytecode.push(Instruction::StoreVar(items.clone()));
                self.bytecode
                    .push(Instruction::Push(Value::Number(Decimal::ZERO)));
                self.bytecode.push(Instruction::StoreVar(index.clone()));

                let start = self.bytecode.len();
                self.bytecode.push(Instruction::LoadVar(index.clone()));
                self.bytecode.push(Instruction::LoadVar(items.clone()));
                self.bytecode.push(Instruction::Length);
                self.bytecode.push(Instruction::LessThan);
                let jump_false_idx = self.bytecode.len();
                self.bytecode.push(Instruction::JumpIfFalse(0)); // patched below

                self.bytecode.push(Instruction::LoadVar(items.clone()));
                self.bytecode.push(Instruction::LoadVar(index.clone()));
                self.bytecode.push(Instruction::NthOrKey);
                self.bytecode.push(Instruction::StoreVar(var));

                for stmt in body {
                    self.compile_stmt(stmt);
                }

                self.bytecode.push(Instruction::LoadVar(index.clone()));
                self.bytecode
                    .push(Instruction::Push(Value::Number(Decimal::ONE)));
                self.bytecode.push(Instruction::Add);
                self.bytecode.push(Instruction::StoreVar(index));
                self.bytecode.push(Instruction::Jump(start));

                let end = self.bytecode.len();
                self.bytecode.instructions[jump_false_idx] = Instruction::JumpIfFalse(end);
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
            Stmt::SendResponse { status_code, body, headers } => {
                self.compile_expr(status_code);
                self.compile_expr(body);
                // A response always leaves three values for the instruction,
                // so the stack shape does not depend on whether the author
                // supplied headers.
                match headers {
                    Some(expr) => self.compile_expr(expr),
                    None => self.bytecode.push(Instruction::Push(Value::Null)),
                }
                self.bytecode.push(Instruction::SendResponse);
            }
            Stmt::DBConnect { db_type, connection_string } => {
                self.compile_expr(connection_string);
                self.bytecode.push(Instruction::DBConnect(db_type));
            }
            Stmt::DBDisconnect { db_type } => {
                self.bytecode.push(Instruction::DBDisconnect(db_type));
            }
            Stmt::DBExecute { command, params } => {
                self.compile_expr(command);
                self.compile_expr(params);
                self.bytecode.push(Instruction::DBExecute);
            }
            Stmt::DBQuery { query, params, result_var } => {
                self.compile_expr(query);
                self.compile_expr(params);
                self.bytecode.push(Instruction::DBQuery);
                self.bytecode.push(Instruction::StoreVar(result_var));
            }
            // The remaining database and server statements parse, but the VM
            // has no runtime for them yet. Emit an instruction that fails
            // loudly rather than silently doing nothing.
            other => {
                let label = Self::stmt_label(&other);
                self.bytecode.push(Instruction::Unsupported(label));
            }
        }
    }

    /// Human-readable name for a statement the VM cannot execute.
    fn stmt_label(stmt: &Stmt) -> String {
        match stmt {
            // Imports are resolved by module.rs before compilation; reaching
            // here means the program was compiled without resolving them.
            Stmt::Import(_) => "இறக்கு (import)",
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
            Stmt::Schedule { .. } => "இடைவெளி (schedule)",
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
            Expr::ArrayLiteral(items) => {
                let count = items.len();
                for item in items {
                    self.compile_expr(item);
                }
                self.bytecode.push(Instruction::MakeArray(count));
            }
            Expr::RecordLiteral(fields) => {
                let mut keys = Vec::with_capacity(fields.len());
                for (key, value) in fields {
                    keys.push(key);
                    self.compile_expr(value);
                }
                self.bytecode.push(Instruction::MakeRecord(keys));
            }
            Expr::Index { base, index } => {
                self.compile_expr(*base);
                self.compile_expr(*index);
                self.bytecode.push(Instruction::Index);
            }
            Expr::Field { base, name } => {
                self.compile_expr(*base);
                self.bytecode.push(Instruction::Field(name));
            }
            Expr::Try(inner) => {
                self.compile_expr(*inner);
                self.bytecode.push(Instruction::TryUnwrap);
            }
            Expr::Concat { left, right } => {
                self.compile_expr(*left);
                self.compile_expr(*right);
                self.bytecode.push(Instruction::Concat);
            }
        }
    }
}
