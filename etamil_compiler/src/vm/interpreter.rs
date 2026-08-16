// eTamil Virtual Machine Interpreter
// Executes bytecode independently without compilation

use std::collections::HashMap;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write as IoWrite;
use rust_decimal::Decimal;
use crate::vm::{Value, Instruction, Bytecode};

/// One active function call: where to resume, and that call's local names.
#[derive(Debug)]
pub struct Frame {
    pub return_ip: usize,
    pub locals: HashMap<String, Value>,
}

/// Guards against runaway recursion before the host stack is exhausted.
const MAX_CALL_DEPTH: usize = 256;

#[derive(Debug)]
pub struct VM {
    pub stack: Vec<Value>,
    pub variables: HashMap<String, Value>,
    pub instruction_pointer: usize,
    /// Mode ("read" / "write" / "append") recorded by கோப்பு_திற per file.
    pub file_modes: HashMap<String, String>,
    /// Active call frames; empty means we are at global scope.
    pub frames: Vec<Frame>,
}

impl VM {
    pub fn new() -> Self {
        VM {
            stack: Vec::new(),
            variables: HashMap::new(),
            instruction_pointer: 0,
            file_modes: HashMap::new(),
            frames: Vec::new(),
        }
    }

    /// Read a name: the current call's locals shadow globals.
    fn get_var(&self, name: &str) -> Option<Value> {
        if let Some(frame) = self.frames.last() {
            if let Some(value) = frame.locals.get(name) {
                return Some(value.clone());
            }
        }
        self.variables.get(name).cloned()
    }

    /// Write a name. Inside a function this always creates or updates a
    /// local, so a function cannot silently clobber a global — assigning to
    /// an outer name shadows it for the duration of the call.
    fn set_var(&mut self, name: String, value: Value) {
        match self.frames.last_mut() {
            Some(frame) => {
                frame.locals.insert(name, value);
            }
            None => {
                self.variables.insert(name, value);
            }
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
                        self.set_var(name, value);
                    }
                }
                Instruction::LoadVar(name) => {
                    // An unknown name used to silently load Null, which
                    // to_number() then turned into 0.0 — a typo became a
                    // wrong answer with no diagnostic.
                    let value = self.get_var(&name).ok_or_else(|| {
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
                    if divisor == Decimal::ZERO {
                        return Err("பூஜ்ஜியத்தால் வகுத்தல்  (division by zero)".to_string());
                    }
                    // Division stays exact to the decimal type's full
                    // precision. Rounding is deliberately not applied here:
                    // Indian tax computation rounds once at the end, and
                    // rounding every intermediate would compound error in a
                    // chained calculation.
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
                    self.stack.push(Value::Number(Decimal::from(data_rows)));
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
                Instruction::Call(name, argc) => {
                    let info = bytecode.functions.get(&name).cloned().ok_or_else(|| {
                        format!(
                            "அறியப்படாத செயல் '{}'  (unknown function '{}')",
                            name, name
                        )
                    })?;
                    if info.params.len() != argc {
                        return Err(format!(
                            "செயல் '{}' {} அளவுருக்களை எதிர்பார்க்கிறது, {} வழங்கப்பட்டது  \
                             (function '{}' expects {} argument(s), got {})",
                            name,
                            info.params.len(),
                            argc,
                            name,
                            info.params.len(),
                            argc
                        ));
                    }
                    if self.frames.len() >= MAX_CALL_DEPTH {
                        return Err(format!(
                            "செயல் அழைப்பு ஆழம் மிகுதி ({})  (call depth exceeded — infinite recursion?)",
                            MAX_CALL_DEPTH
                        ));
                    }

                    // Arguments were pushed left to right, so bind in reverse.
                    let mut locals = HashMap::new();
                    for param in info.params.iter().rev() {
                        let value = self.pop()?;
                        locals.insert(param.clone(), value);
                    }

                    self.frames.push(Frame {
                        return_ip: self.instruction_pointer + 1,
                        locals,
                    });
                    self.instruction_pointer = info.start;
                    continue;
                }
                Instruction::Return => {
                    let value = self.pop()?;
                    let frame = self.frames.pop().ok_or(
                        "செயலுக்கு வெளியே திரும்பு  (return outside of a function)",
                    )?;
                    self.instruction_pointer = frame.return_ip;
                    self.stack.push(value);
                    continue;
                }
                Instruction::DBConnect(_) | Instruction::DBQuery | Instruction::DBExecute
                | Instruction::DefineRoute(_, _) | Instruction::StartServer(_, _) => {
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
