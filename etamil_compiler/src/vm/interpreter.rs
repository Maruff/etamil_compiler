// eTamil Virtual Machine Interpreter
// Executes bytecode independently without compilation

use std::collections::HashMap;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write as IoWrite;
use rust_decimal::Decimal;
use std::str::FromStr;
use unicode_segmentation::UnicodeSegmentation;
use crate::vm::{Value, Instruction, Bytecode};

/// Split text the way a reader would: by written letter. A Tamil letter is
/// frequently several code points (consonant + vowel sign, or + pulli), so
/// counting chars would make நீளம்("வணக்கம்") 7 instead of 5.
fn letters(text: &str) -> Vec<&str> {
    text.graphemes(true).collect()
}

/// One active function call: where to resume, and that call's local names.
#[derive(Debug)]
pub struct Frame {
    pub return_ip: usize,
    pub locals: HashMap<String, Value>,
    /// Stack depth when the call began. Returning truncates back to this, so
    /// a half-evaluated expression abandoned by `?` cannot leave residue.
    pub base_len: usize,
}

/// Guards against runaway recursion before the host stack is exhausted.
const MAX_CALL_DEPTH: usize = 256;

/// Open database connections, keyed by the type name written in source.
/// Wrapped so the VM can still derive Debug — a driver handle cannot.
#[derive(Default)]
pub struct Connections(HashMap<String, Box<dyn crate::db::Database>>);

impl std::fmt::Debug for Connections {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Connections({} open)", self.0.len())
    }
}

impl Connections {
    pub fn insert(&mut self, name: String, handle: Box<dyn crate::db::Database>) {
        self.0.insert(name, handle);
    }

    pub fn remove(&mut self, name: &str) -> Option<Box<dyn crate::db::Database>> {
        self.0.remove(name)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Debug)]
pub struct VM {
    pub stack: Vec<Value>,
    pub variables: HashMap<String, Value>,
    pub instruction_pointer: usize,
    /// Mode ("read" / "write" / "append") recorded by கோப்பு_திற per file.
    pub file_modes: HashMap<String, String>,
    /// Active call frames; empty means we are at global scope.
    pub frames: Vec<Frame>,
    /// Open database connections.
    pub connections: Connections,
}

impl VM {
    pub fn new() -> Self {
        VM {
            stack: Vec::new(),
            variables: HashMap::new(),
            instruction_pointer: 0,
            file_modes: HashMap::new(),
            frames: Vec::new(),
            connections: Connections::default(),
        }
    }

    /// The connection to use for a query. There is one per database type, and
    /// with a single type open the choice is unambiguous.
    fn connection_mut(&mut self) -> Result<&mut Box<dyn crate::db::Database>, String> {
        if self.connections.0.len() == 1 {
            return Ok(self.connections.0.values_mut().next().expect("checked"));
        }
        if self.connections.is_empty() {
            return Err(
                "தரவுத்தளம் இணைக்கப்படவில்லை  (not connected to a database): \
                 use தளம்_இணை first"
                    .to_string(),
            );
        }
        Err(
            "பல தரவுத்தளங்கள் திறந்துள்ளன  (several databases are open); \
             close all but one for now"
                .to_string(),
        )
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

    /// Human-readable type name, for error messages.
    fn type_name(value: &Value) -> &'static str {
        match value {
            Value::Number(_) => "a number",
            Value::String(_) => "a string",
            Value::Boolean(_) => "a boolean",
            Value::Array(_) => "an array",
            Value::Map(_) => "a record",
            Value::Ok(_) => "a result",
            Value::Err(_) => "a result",
            Value::Null => "nil",
        }
    }

    /// Turn an index value into a valid array position, or explain why not.
    fn array_index(len: usize, index: &Value) -> Result<usize, String> {
        let raw = index.to_number();
        if raw.fract() != Decimal::ZERO {
            return Err(format!(
                "அட்டவணை முழு எண்ணாக இருக்க வேண்டும்  (array index must be a whole number, got {})",
                raw
            ));
        }
        let i = rust_decimal::prelude::ToPrimitive::to_i64(&raw).unwrap_or(-1);
        if i < 0 || i as usize >= len {
            return Err(format!(
                "அட்டவணை {} வரம்பிற்கு வெளியே (நீளம் {})  (index {} out of bounds, length {})",
                raw, len, raw, len
            ));
        }
        Ok(i as usize)
    }

    /// `base[index]` for arrays (by position) and records (by key).
    fn index_of(base: &Value, index: &Value) -> Result<Value, String> {
        match base {
            Value::Array(items) => {
                let i = Self::array_index(items.len(), index)?;
                Ok(items[i].clone())
            }
            Value::Map(fields) => {
                let key = index.to_string();
                fields.get(&key).cloned().ok_or_else(|| {
                    format!("புலம் '{}' இல்லை  (no field '{}' on this record)", key, key)
                })
            }
            Value::String(s) => {
                let parts = letters(s);
                let i = Self::array_index(parts.len(), index)?;
                Ok(Value::String(parts[i].to_string()))
            }
            other => Err(format!(
                "அட்டவணைப்படுத்த முடியாது  (cannot index into {})",
                Self::type_name(other)
            )),
        }
    }

    /// Builtins, callable under Tamil, romanized or English names. This is
    /// the extension point the tax and accounting builtins will plug into.
    fn call_builtin(&mut self, name: &str, argc: usize) -> Result<Value, String> {
        let mut args = Vec::with_capacity(argc);
        for _ in 0..argc {
            args.push(self.pop()?);
        }
        args.reverse();

        match name {
            // நீளம் — length of an array, record or string
            "நீளம்" | "nILam" | "_length" => {
                Self::expect_args(name, &args, 1)?;
                let n = match &args[0] {
                    Value::Array(items) => items.len(),
                    Value::Map(fields) => fields.len(),
                    Value::String(s) => letters(s).len(),
                    other => {
                        return Err(format!(
                            "நீளம் ஒரு அணி/பொருள்/சொல் தேவை  (length needs an array, record or string, got {})",
                            Self::type_name(other)
                        ));
                    }
                };
                Ok(Value::Number(Decimal::from(n)))
            }
            // இணை — append to an array, returning the extended array.
            // (சேர் / cEr is already the SQL JOIN keyword.)
            "இணை" | "iNY" | "_append" => {
                Self::expect_args(name, &args, 2)?;
                match &args[0] {
                    Value::Array(items) => {
                        let mut items = items.clone();
                        items.push(args[1].clone());
                        Ok(Value::Array(items))
                    }
                    other => Err(format!(
                        "சேர் ஒரு அணி தேவை  (append needs an array, got {})",
                        Self::type_name(other)
                    )),
                }
            }
            // --- Dates ---
            // ISO-8601 text throughout, which sorts chronologically, so
            // comparison needs no primitive. Arithmetic does: the calendar
            // cannot be derived from string operations.
            // இன்று() — today, in UTC
            "இன்று" | "iZRu" | "_today" => {
                Self::expect_args(name, &args, 0)?;
                let seconds = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|_| "கடிகாரம் படிக்க முடியவில்லை  (cannot read the clock)")?
                    .as_secs() as i64;
                Ok(Value::String(Self::format_date(seconds / 86_400)))
            }
            // நாள்_வேறுபாடு(a, b) — whole days from a to b, negative if b is earlier
            "நாள்_வேறுபாடு" | "nAL_vERupAtu" | "_daysBetween" => {
                Self::expect_args(name, &args, 2)?;
                let from = Self::parse_date(&args[0])?;
                let to = Self::parse_date(&args[1])?;
                Ok(Value::Number(Decimal::from(to - from)))
            }
            // நாள்_கூட்டு(நாள், நாட்கள்)
            "நாள்_கூட்டு" | "nAL_kUttu" | "_addDays" => {
                Self::expect_args(name, &args, 2)?;
                let date = Self::parse_date(&args[0])?;
                let days = rust_decimal::prelude::ToPrimitive::to_i64(&args[1].to_number())
                    .ok_or("நாட்கள் ஒரு முழு எண்ணாக இருக்க வேண்டும்  (days must be a whole number)")?;
                Ok(Value::String(Self::format_date(date + days)))
            }
            // வகை — the type of a value, as a string
            "வகை" | "vakY" | "_typeof" => {
                Self::expect_args(name, &args, 1)?;
                Ok(Value::String(Self::type_name(&args[0]).to_string()))
            }
            // --- Results, following Rust ---
            // சரி(v) — Ok
            "சரி" | "cari" | "_ok" => {
                Self::expect_args(name, &args, 1)?;
                Ok(Value::Ok(Box::new(args[0].clone())))
            }
            // தவறு(e) — Err
            "தவறு" | "qavaRu" | "_err" => {
                Self::expect_args(name, &args, 1)?;
                Ok(Value::Err(Box::new(args[0].clone())))
            }
            // சரியா(r) — is_ok
            "சரியா" | "cariyA" | "_isOk" => {
                Self::expect_args(name, &args, 1)?;
                Ok(Value::Boolean(matches!(args[0], Value::Ok(_))))
            }
            // தவறா(r) — is_err
            "தவறா" | "qavaRA" | "_isErr" => {
                Self::expect_args(name, &args, 1)?;
                Ok(Value::Boolean(matches!(args[0], Value::Err(_))))
            }
            // மதிப்பு(r) — unwrap; a தவறு here is a runtime error, as in Rust
            "மதிப்பு" | "maqippu" | "_unwrap" => {
                Self::expect_args(name, &args, 1)?;
                match &args[0] {
                    Value::Ok(inner) => Ok((**inner).clone()),
                    Value::Err(error) => Err(format!(
                        "தவறான முடிவை விரித்தது: {}  (unwrap on an error: {})",
                        error.to_string(),
                        error.to_string()
                    )),
                    other => Err(format!(
                        "மதிப்பு க்கு ஒரு முடிவு தேவை  (unwrap needs a result, got {})",
                        Self::type_name(other)
                    )),
                }
            }
            // இயல்பு(r, d) — unwrap_or
            "இயல்பு" | "iyalpu" | "_unwrapOr" => {
                Self::expect_args(name, &args, 2)?;
                match &args[0] {
                    Value::Ok(inner) => Ok((**inner).clone()),
                    Value::Err(_) => Ok(args[1].clone()),
                    other => Err(format!(
                        "இயல்பு க்கு ஒரு முடிவு தேவை  (unwrapOr needs a result, got {})",
                        Self::type_name(other)
                    )),
                }
            }
            // --- Numeric primitives ---
            // These need the decimal type itself, so they cannot be written
            // in eTamil. Everything derivable from them lives in nUlakam/.
            // வட்டமிடு(n, இடங்கள்) — round to n decimal places, half away
            // from zero, which is what Indian tax rules expect.
            "வட்டமிடு" | "vattamitu" | "_round" => {
                Self::expect_args(name, &args, 2)?;
                let places = rust_decimal::prelude::ToPrimitive::to_u32(&args[1].to_number())
                    .ok_or("வட்டமிடு: இடங்கள் ஒரு முழு எண்  (round: places must be a whole number)")?;
                Ok(Value::Number(args[0].to_number().round_dp_with_strategy(
                    places,
                    rust_decimal::RoundingStrategy::MidpointAwayFromZero,
                )))
            }
            // தரை(n) — floor
            "தரை" | "qarY" | "_floor" => {
                Self::expect_args(name, &args, 1)?;
                Ok(Value::Number(args[0].to_number().floor()))
            }
            // மேல்(n) — ceiling
            "மேல்" | "mEl" | "_ceil" => {
                Self::expect_args(name, &args, 1)?;
                Ok(Value::Number(args[0].to_number().ceil()))
            }
            // சொல்லாக்கு(v) — render any value as text
            "சொல்லாக்கு" | "collAkku" | "_toString" => {
                Self::expect_args(name, &args, 1)?;
                Ok(Value::String(args[0].to_string()))
            }
            // எண்ணாக்கு(s) — parse text as a number, as a result
            "எண்ணாக்கு" | "eNNAkku" | "_toNumber" => {
                Self::expect_args(name, &args, 1)?;
                let text = args[0].to_string();
                match Decimal::from_str(text.trim()) {
                    Ok(number) => Ok(Value::Ok(Box::new(Value::Number(number)))),
                    Err(_) => Ok(Value::Err(Box::new(Value::String(format!(
                        "'{}' ஒரு எண் அல்ல  ('{}' is not a number)",
                        text, text
                    ))))),
                }
            }
            // Case folding matters for PAN/GSTIN style codes; Tamil has no
            // case, so these only affect the Latin characters.
            "மேல்_எழுத்து" | "mEl_ezuqqu" | "_upper" => {
                Self::expect_args(name, &args, 1)?;
                Ok(Value::String(args[0].to_string().to_uppercase()))
            }
            "கீழ்_எழுத்து" | "kIz_ezuqqu" | "_lower" => {
                Self::expect_args(name, &args, 1)?;
                Ok(Value::String(args[0].to_string().to_lowercase()))
            }
            // --- Authentication ---
            // bcrypt, HMAC-SHA256, base64 and randomness are not expressible
            // in eTamil, so they live in the host. Everything above them —
            // who a user is, which route needs which role — stays in the
            // language. A token's payload crosses as JSON text, which
            // nUlakam/jEcAZ.qmz reads and writes.
            // கடவுச்சொல்_மறை(கடவுச்சொல்) — hash a password for storage
            "கடவுச்சொல்_மறை" | "kataveuccol_maRY" | "_hashPassword" => {
                Self::expect_args(name, &args, 1)?;
                Ok(Value::String(crate::http::auth::hash_password(
                    &args[0].to_string(),
                )?))
            }
            // கடவுச்சொல்_சரியா(கடவுச்சொல், மறையீடு) — does it match?
            "கடவுச்சொல்_சரியா" | "kataveuccol_cariyA" | "_verifyPassword" => {
                Self::expect_args(name, &args, 2)?;
                Ok(Value::Boolean(crate::http::auth::verify_password(
                    &args[0].to_string(),
                    &args[1].to_string(),
                )?))
            }
            // சீட்டு_ஆக்கு(சுமை_ஜேசான், நொடிகள்) — sign a token.
            //
            // Named சீட்டு rather than the more literal குறியீடு because that
            // word is already the SQL INDEX keyword and is hard reserved: a
            // caller writing the obvious `குறியீடு = குறியீடு_ஆக்கு(...)`
            // would get a parse error on their own variable.
            "சீட்டு_ஆக்கு" | "cIttu_Akku" | "_issueToken" => {
                Self::expect_args(name, &args, 2)?;
                let seconds = rust_decimal::prelude::ToPrimitive::to_i64(&args[1].to_number())
                    .ok_or("நொடிகள் ஒரு முழு எண்  (the lifetime must be a whole number of seconds)")?;
                Ok(Value::String(crate::http::auth::issue_token(
                    &args[0].to_string(),
                    seconds,
                )?))
            }
            // சீட்டு_சரிபார்(சீட்டு) — verify, yielding the claims as JSON
            // text. A bad or expired token is a தவறு, not an error, so
            // rejecting a request is ordinary control flow.
            "சீட்டு_சரிபார்" | "cIttu_caripAr" | "_readToken" => {
                Self::expect_args(name, &args, 1)?;
                match crate::http::auth::read_token(&args[0].to_string()) {
                    Ok(claims) => Ok(Value::Ok(Box::new(Value::String(claims)))),
                    Err(message) => Ok(Value::Err(Box::new(Value::String(message)))),
                }
            }
            unknown => Err(format!(
                "அறியப்படாத செயல் '{}'  (unknown function '{}')",
                unknown, unknown
            )),
        }
    }

    /// Read an ISO-8601 date as a day number, counting from 1970-01-01.
    ///
    /// Done here rather than with a date crate: the conversion is a closed
    /// form (Howard Hinnant's civil-calendar algorithm), it is exact for any
    /// year, and it keeps a calendar library out of the interpreter.
    fn parse_date(value: &Value) -> Result<i64, String> {
        let text = value.to_string();
        let bad = || {
            format!(
                "'{}' ஒரு நாள் அல்ல  ('{}' is not a date; expected YYYY-MM-DD)",
                text, text
            )
        };

        let parts: Vec<&str> = text.trim().split('-').collect();
        if parts.len() != 3 {
            return Err(bad());
        }
        let year: i64 = parts[0].parse().map_err(|_| bad())?;
        let month: i64 = parts[1].parse().map_err(|_| bad())?;
        let day: i64 = parts[2].parse().map_err(|_| bad())?;

        if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
            return Err(bad());
        }
        Ok(Self::days_from_civil(year, month, day))
    }

    /// Days from 1970-01-01 to a civil date.
    fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
        let y = if month <= 2 { year - 1 } else { year };
        let era = if y >= 0 { y } else { y - 399 } / 400;
        let yoe = y - era * 400;
        let mp = if month > 2 { month - 3 } else { month + 9 };
        let doy = (153 * mp + 2) / 5 + day - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146097 + doe - 719468
    }

    /// The inverse: a civil date from a day number.
    fn civil_from_days(days: i64) -> (i64, i64, i64) {
        let z = days + 719468;
        let era = if z >= 0 { z } else { z - 146096 } / 146097;
        let doe = z - era * 146097;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let y = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        (if m <= 2 { y + 1 } else { y }, m, d)
    }

    /// Format a day number back as ISO-8601.
    fn format_date(days: i64) -> String {
        let (year, month, day) = Self::civil_from_days(days);
        format!("{:04}-{:02}-{:02}", year, month, day)
    }

    fn expect_args(name: &str, args: &[Value], want: usize) -> Result<(), String> {
        if args.len() != want {
            return Err(format!(
                "செயல் '{}' {} அளவுருக்களை எதிர்பார்க்கிறது, {} வழங்கப்பட்டது  \
                 (function '{}' expects {} argument(s), got {})",
                name,
                want,
                args.len(),
                name,
                want,
                args.len()
            ));
        }
        Ok(())
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
                Instruction::MakeArray(count) => {
                    let mut items = Vec::with_capacity(count);
                    for _ in 0..count {
                        items.push(self.pop()?);
                    }
                    items.reverse(); // pushed left to right
                    self.stack.push(Value::Array(items));
                }
                Instruction::MakeRecord(keys) => {
                    let mut fields = HashMap::with_capacity(keys.len());
                    for key in keys.into_iter().rev() {
                        let value = self.pop()?;
                        fields.insert(key, value);
                    }
                    self.stack.push(Value::Map(fields));
                }
                Instruction::Index => {
                    let index = self.pop()?;
                    let base = self.pop()?;
                    self.stack.push(Self::index_of(&base, &index)?);
                }
                Instruction::Field(name) => {
                    let base = self.pop()?;
                    match base {
                        Value::Map(fields) => {
                            let value = fields.get(&name).cloned().ok_or_else(|| {
                                format!("புலம் '{}' இல்லை  (no field '{}' on this record)", name, name)
                            })?;
                            self.stack.push(value);
                        }
                        other => {
                            return Err(format!(
                                "'{}' ஒரு பொருள் அல்ல  ('.{}' needs a record, got {})",
                                name,
                                name,
                                Self::type_name(&other)
                            ));
                        }
                    }
                }
                Instruction::SetIndex(name) => {
                    let value = self.pop()?;
                    let index = self.pop()?;
                    let mut base = self.get_var(&name).ok_or_else(|| {
                        format!("அறிவிக்கப்படாத மாறி '{}'  (undefined variable '{}')", name, name)
                    })?;
                    match &mut base {
                        Value::Array(items) => {
                            let i = Self::array_index(items.len(), &index)?;
                            items[i] = value;
                        }
                        Value::Map(fields) => {
                            fields.insert(index.to_string(), value);
                        }
                        other => {
                            return Err(format!(
                                "'{}' ஐ அட்டவணைப்படுத்த முடியாது  (cannot index into {})",
                                name,
                                Self::type_name(other)
                            ));
                        }
                    }
                    self.set_var(name, base);
                }
                Instruction::SetField(name, field) => {
                    let value = self.pop()?;
                    let mut base = self.get_var(&name).ok_or_else(|| {
                        format!("அறிவிக்கப்படாத மாறி '{}'  (undefined variable '{}')", name, name)
                    })?;
                    match &mut base {
                        Value::Map(fields) => {
                            fields.insert(field, value);
                        }
                        other => {
                            return Err(format!(
                                "'{}' ஒரு பொருள் அல்ல  ('{}.{}' needs a record, got {})",
                                name,
                                name,
                                field,
                                Self::type_name(other)
                            ));
                        }
                    }
                    self.set_var(name, base);
                }
                Instruction::Length => {
                    let value = self.pop()?;
                    let n = match &value {
                        Value::Array(items) => items.len(),
                        Value::Map(fields) => fields.len(),
                        Value::String(s) => letters(s).len(),
                        other => {
                            return Err(format!(
                                "இதை சுற்ற முடியாது  (cannot iterate over {})",
                                Self::type_name(other)
                            ));
                        }
                    };
                    self.stack.push(Value::Number(Decimal::from(n)));
                }
                Instruction::NthOrKey => {
                    let index = self.pop()?;
                    let base = self.pop()?;
                    let value = match &base {
                        Value::Array(items) => {
                            let i = Self::array_index(items.len(), &index)?;
                            items[i].clone()
                        }
                        Value::Map(fields) => {
                            // Sorted so iteration order is stable run to run.
                            let mut keys: Vec<&String> = fields.keys().collect();
                            keys.sort();
                            let i = Self::array_index(keys.len(), &index)?;
                            Value::String(keys[i].clone())
                        }
                        Value::String(s) => {
                            let parts = letters(s);
                            let i = Self::array_index(parts.len(), &index)?;
                            Value::String(parts[i].to_string())
                        }
                        other => {
                            return Err(format!(
                                "இதை சுற்ற முடியாது  (cannot iterate over {})",
                                Self::type_name(other)
                            ));
                        }
                    };
                    self.stack.push(value);
                }
                Instruction::Call(name, argc) => {
                    // User-defined functions shadow builtins.
                    if !bytecode.functions.contains_key(&name) {
                        let result = self.call_builtin(&name, argc)?;
                        self.stack.push(result);
                        self.instruction_pointer += 1;
                        continue;
                    }
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
                        base_len: self.stack.len(),
                    });
                    self.instruction_pointer = info.start;
                    continue;
                }
                Instruction::Return => {
                    let value = self.pop()?;
                    let frame = self.frames.pop().ok_or(
                        "செயலுக்கு வெளியே திரும்பு  (return outside of a function)",
                    )?;
                    self.stack.truncate(frame.base_len);
                    self.instruction_pointer = frame.return_ip;
                    self.stack.push(value);
                    continue;
                }
                Instruction::TryUnwrap => {
                    let value = self.pop()?;
                    match value {
                        Value::Ok(inner) => self.stack.push(*inner),
                        Value::Err(error) => {
                            // Rust's `?`: hand the failure to the caller.
                            match self.frames.pop() {
                                Some(frame) => {
                                    self.stack.truncate(frame.base_len);
                                    self.instruction_pointer = frame.return_ip;
                                    self.stack.push(Value::Err(error));
                                    continue;
                                }
                                None => {
                                    return Err(format!(
                                        "கையாளப்படாத தவறு: {}  (unhandled error at top level: {})",
                                        error.to_string(),
                                        error.to_string()
                                    ));
                                }
                            }
                        }
                        other => {
                            return Err(format!(
                                "'?' க்கு ஒரு முடிவு தேவை  ('?' needs a result, got {})",
                                Self::type_name(&other)
                            ));
                        }
                    }
                }
                Instruction::DBConnect(db_type) => {
                    let connection = self.pop()?.to_string();
                    let handle = crate::db::open(&db_type, &connection)?;
                    self.connections.insert(db_type, handle);
                }
                Instruction::DBDisconnect(db_type) => {
                    match self.connections.remove(&db_type) {
                        Some(mut handle) => handle.close()?,
                        None => {
                            return Err(format!(
                                "'{}' இணைக்கப்படவில்லை  (not connected to {})",
                                db_type, db_type
                            ));
                        }
                    }
                }
                Instruction::DBExecute => {
                    let params = crate::db::params_from(&self.pop()?)?;
                    let sql = self.pop()?.to_string();
                    let handle = self.connection_mut()?;
                    handle.execute(&sql, &params)?;
                }
                Instruction::DBQuery => {
                    let params = crate::db::params_from(&self.pop()?)?;
                    let sql = self.pop()?.to_string();
                    let handle = self.connection_mut()?;
                    // One record per row, so a result set is an array of
                    // records — a table in the language's own terms.
                    let rows = handle.query(&sql, &params)?;
                    self.stack.push(Value::Array(rows));
                }
                Instruction::SendResponse => {
                    let headers = self.pop()?;
                    let body = self.pop()?;
                    let status = self.pop()?;
                    // Written to globals, not the current frame: the server
                    // reads them from the VM once the handler has returned,
                    // and பதில் is often called from inside a function.
                    self.variables
                        .insert("response_status".to_string(), status);
                    self.variables
                        .insert("response_body".to_string(), Value::String(body.to_string()));
                    self.variables
                        .insert("response_headers".to_string(), headers);
                }
                Instruction::DefineRoute(_, _) | Instruction::StartServer(_, _) => {
                    return Err(
                        "வழங்கி செயல்பாடுகள் VM இல் இன்னும் இல்லை  (server operations are not implemented in the VM yet)"
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
