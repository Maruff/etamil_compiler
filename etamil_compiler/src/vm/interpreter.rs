// eTamil Virtual Machine Interpreter
// Executes bytecode independently without compilation

use std::collections::HashMap;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write as IoWrite;
use crate::vm::{Value, Instruction, Bytecode};

#[derive(Debug)]
pub struct VM {
    pub stack: Vec<Value>,
    pub variables: HashMap<String, Value>,
    pub instruction_pointer: usize,
    /// Mode ("read" / "write" / "append") recorded by கோப்பு_திற per file.
    pub file_modes: HashMap<String, String>,
}

impl VM {
    pub fn new() -> Self {
        VM {
            stack: Vec::new(),
            variables: HashMap::new(),
            instruction_pointer: 0,
            file_modes: HashMap::new(),
        }
    }

    /// Pop a value, or report a stack underflow.
    fn pop(&mut self) -> Result<Value, String> {
        self.stack.pop().ok_or_else(|| "Stack underflow".to_string())
    }

    /// Append one line to a file, creating it if needed.
    fn append_line(filename: &str, data: &str) -> Result<(), String> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(filename)
            .map_err(|e| format!("கோப்பு '{}' எழுத முடியவில்லை  (cannot write '{}'): {}", filename, filename, e))?;
        writeln!(file, "{}", data)
            .map_err(|e| format!("கோப்பு '{}' எழுத முடியவில்லை  (cannot write '{}'): {}", filename, filename, e))
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
                    // An unknown name used to silently load Null, which
                    // to_number() then turned into 0.0 — a typo became a
                    // wrong answer with no diagnostic.
                    let value = self.variables.get(&name).cloned().ok_or_else(|| {
                        format!(
                            "அறிவிக்கப்படாத மாறி '{}'  (undefined variable '{}')",
                            name, name
                        )
                    })?;
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
                Instruction::And => {
                    let right = self.pop()?;
                    let left = self.pop()?;
                    self.stack.push(Value::Boolean(left.is_truthy() && right.is_truthy()));
                }
                Instruction::Or => {
                    let right = self.pop()?;
                    let left = self.pop()?;
                    self.stack.push(Value::Boolean(left.is_truthy() || right.is_truthy()));
                }
                Instruction::Not => {
                    let value = self.pop()?;
                    self.stack.push(Value::Boolean(!value.is_truthy()));
                }
                Instruction::FileOpen(mode) => {
                    let filename = self.pop()?.to_string();
                    // Opening for writing truncates once; later writes append,
                    // so a sequence of writes reads back in order.
                    if mode == "write" {
                        fs::write(&filename, "")
                            .map_err(|e| format!("கோப்பு '{}' திறக்க முடியவில்லை  (cannot open '{}' for writing): {}", filename, filename, e))?;
                    }
                    self.file_modes.insert(filename, mode);
                }
                Instruction::FileClose => {
                    let filename = self.pop()?.to_string();
                    self.file_modes.remove(&filename);
                }
                Instruction::FileWrite => {
                    let data = self.pop()?.to_string();
                    let filename = self.pop()?.to_string();
                    Self::append_line(&filename, &data)?;
                }
                Instruction::FileRead => {
                    let filename = self.pop()?.to_string();
                    let contents = fs::read_to_string(&filename)
                        .map_err(|e| format!("கோப்பு '{}' படிக்க முடியவில்லை  (cannot read '{}'): {}", filename, filename, e))?;
                    self.stack.push(Value::String(contents.trim_end_matches('\n').to_string()));
                }
                Instruction::ReadCSV => {
                    let filename = self.pop()?.to_string();
                    let contents = fs::read_to_string(&filename)
                        .map_err(|e| format!("கோப்பு '{}' படிக்க முடியவில்லை  (cannot read '{}'): {}", filename, filename, e))?;
                    // Count data rows, excluding the header line.
                    let rows = contents.lines().filter(|l| !l.trim().is_empty()).count();
                    let data_rows = if rows > 0 { rows - 1 } else { 0 };
                    self.stack.push(Value::Number(data_rows as f64));
                }
                Instruction::WriteCSV => {
                    let row = self.pop()?.to_string();
                    let filename = self.pop()?.to_string();
                    Self::append_line(&filename, &row)?;
                }
                Instruction::Nop => {}
                Instruction::Unsupported(what) => {
                    return Err(format!(
                        "{} — இந்த VM இல் இன்னும் செயல்படுத்தப்படவில்லை  (not implemented in the VM yet)",
                        what
                    ));
                }
                Instruction::DBConnect(_) | Instruction::DBQuery | Instruction::DBExecute
                | Instruction::DefineRoute(_, _) | Instruction::StartServer(_, _)
                | Instruction::Call(_) | Instruction::Return => {
                    return Err(
                        "தரவுத்தளம்/வழங்கி செயல்பாடுகள் VM இல் இன்னும் இல்லை  (database and server operations are not implemented in the VM yet)"
                            .to_string(),
                    );
                }
                Instruction::Halt => {
                    break;
                }
            }
            
            self.instruction_pointer += 1;
        }
        
        Ok(())
    }
}
