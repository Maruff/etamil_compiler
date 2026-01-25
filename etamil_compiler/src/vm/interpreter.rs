// eTamil Virtual Machine Interpreter
// Executes bytecode independently without compilation

use std::collections::HashMap;
use crate::vm::{Value, Instruction, Bytecode};

pub struct VM {
    pub stack: Vec<Value>,
    pub variables: HashMap<String, Value>,
    pub instruction_pointer: usize,
}

impl VM {
    pub fn new() -> Self {
        VM {
            stack: Vec::new(),
            variables: HashMap::new(),
            instruction_pointer: 0,
        }
    }

    pub fn execute(&mut self, bytecode: Bytecode) -> Result<(), String> {
        while self.instruction_pointer < bytecode.instructions.len() {
            let instruction = bytecode.instructions[self.instruction_pointer].clone();
            
            match instruction {
                Instruction::Push(value) => {
                    self.stack.push(value);
                }
                Instruction::Pop => {
                    self.stack.pop();
                }
                Instruction::StoreVar(name) => {
                    if let Some(value) = self.stack.pop() {
                        self.variables.insert(name, value);
                    }
                }
                Instruction::LoadVar(name) => {
                    let value = self.variables.get(&name)
                        .cloned()
                        .unwrap_or(Value::Null);
                    self.stack.push(value);
                }
                Instruction::Add => {
                    let right = self.stack.pop().ok_or("Stack underflow")?;
                    let left = self.stack.pop().ok_or("Stack underflow")?;
                    self.stack.push(Value::Number(left.to_number() + right.to_number()));
                }
                Instruction::Subtract => {
                    let right = self.stack.pop().ok_or("Stack underflow")?;
                    let left = self.stack.pop().ok_or("Stack underflow")?;
                    self.stack.push(Value::Number(left.to_number() - right.to_number()));
                }
                Instruction::Multiply => {
                    let right = self.stack.pop().ok_or("Stack underflow")?;
                    let left = self.stack.pop().ok_or("Stack underflow")?;
                    self.stack.push(Value::Number(left.to_number() * right.to_number()));
                }
                Instruction::Divide => {
                    let right = self.stack.pop().ok_or("Stack underflow")?;
                    let left = self.stack.pop().ok_or("Stack underflow")?;
                    let divisor = right.to_number();
                    if divisor == 0.0 {
                        return Err("Division by zero".to_string());
                    }
                    self.stack.push(Value::Number(left.to_number() / divisor));
                }
                Instruction::Modulo => {
                    let right = self.stack.pop().ok_or("Stack underflow")?;
                    let left = self.stack.pop().ok_or("Stack underflow")?;
                    self.stack.push(Value::Number(left.to_number() % right.to_number()));
                }
                Instruction::Equal => {
                    let right = self.stack.pop().ok_or("Stack underflow")?;
                    let left = self.stack.pop().ok_or("Stack underflow")?;
                    self.stack.push(Value::Boolean(left == right));
                }
                Instruction::NotEqual => {
                    let right = self.stack.pop().ok_or("Stack underflow")?;
                    let left = self.stack.pop().ok_or("Stack underflow")?;
                    self.stack.push(Value::Boolean(left != right));
                }
                Instruction::LessThan => {
                    let right = self.stack.pop().ok_or("Stack underflow")?;
                    let left = self.stack.pop().ok_or("Stack underflow")?;
                    let result = left.partial_cmp(&right)
                        .map(|ord| ord == std::cmp::Ordering::Less)
                        .unwrap_or(false);
                    self.stack.push(Value::Boolean(result));
                }
                Instruction::LessOrEqual => {
                    let right = self.stack.pop().ok_or("Stack underflow")?;
                    let left = self.stack.pop().ok_or("Stack underflow")?;
                    let result = left.partial_cmp(&right)
                        .map(|ord| ord != std::cmp::Ordering::Greater)
                        .unwrap_or(false);
                    self.stack.push(Value::Boolean(result));
                }
                Instruction::GreaterThan => {
                    let right = self.stack.pop().ok_or("Stack underflow")?;
                    let left = self.stack.pop().ok_or("Stack underflow")?;
                    let result = left.partial_cmp(&right)
                        .map(|ord| ord == std::cmp::Ordering::Greater)
                        .unwrap_or(false);
                    self.stack.push(Value::Boolean(result));
                }
                Instruction::GreaterOrEqual => {
                    let right = self.stack.pop().ok_or("Stack underflow")?;
                    let left = self.stack.pop().ok_or("Stack underflow")?;
                    let result = left.partial_cmp(&right)
                        .map(|ord| ord != std::cmp::Ordering::Less)
                        .unwrap_or(false);
                    self.stack.push(Value::Boolean(result));
                }
                Instruction::Concat => {
                    let right = self.stack.pop().ok_or("Stack underflow")?;
                    let left = self.stack.pop().ok_or("Stack underflow")?;
                    let result = format!("{}{}", left.to_string(), right.to_string());
                    self.stack.push(Value::String(result));
                }
                Instruction::Print => {
                    if let Some(value) = self.stack.pop() {
                        println!("{}", value.to_string());
                    }
                }
                Instruction::Input => {
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)
                        .map_err(|e| e.to_string())?;
                    self.stack.push(Value::String(input.trim().to_string()));
                }
                Instruction::JumpIfFalse(target) => {
                    if let Some(value) = self.stack.pop() {
                        if !value.is_truthy() {
                            self.instruction_pointer = target;
                            continue;
                        }
                    }
                }
                Instruction::Jump(target) => {
                    self.instruction_pointer = target;
                    continue;
                }
                Instruction::Halt => {
                    break;
                }
                _ => {
                    // Other instructions handled as needed
                }
            }
            
            self.instruction_pointer += 1;
        }
        
        Ok(())
    }
}
