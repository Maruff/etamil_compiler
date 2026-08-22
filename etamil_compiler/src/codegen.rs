/// LLVM Code Generator for eTamil using llvm-sys (LLVM 18 compatible)
#[cfg(feature = "llvm")]
use llvm_sys::prelude::*;
#[cfg(feature = "llvm")]
use llvm_sys::core::*;
#[cfg(feature = "llvm")]
use llvm_sys::{LLVMIntPredicate, LLVMLinkage};
#[cfg(feature = "llvm")]
use std::ffi::CString;
#[cfg(feature = "llvm")]
use std::collections::HashMap;
#[cfg(feature = "llvm")]
use std::ptr;
use crate::parser::Stmt;
#[cfg(feature = "llvm")]
use crate::parser::Expr;
#[cfg(feature = "llvm")]
use crate::codegen_limits::WholeNumberBuiltin;

#[cfg(feature = "llvm")]
#[derive(Clone, Copy)]
struct ArrayInfo {
    pointer: LLVMValueRef,
    element_count: usize,
    array_type: LLVMTypeRef,
}

#[cfg(feature = "llvm")]
type RecordInfo = HashMap<String, LLVMValueRef>;


#[cfg(feature = "llvm")]
pub struct Compiler {
    context: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    function: LLVMValueRef,
    variables: HashMap<String, LLVMValueRef>, // Variable name -> alloca pointer
    /// Top-level names, as module globals rather than stack slots.
    ///
    /// A `செயல்` can *read* a global — nUlakam is full of functions that do,
    /// starting with `kAcu.qmz` reading `பைசா_ஒரு_ரூபாய்` — but a function's
    /// stack frame cannot reach `main`'s. Before this, a function body looked
    /// the name up in an empty map and compiled a `0.0` in its place, so
    /// `ரூபாயாக(2)` answered 0 instead of 200 with nothing said.
    globals: HashMap<String, LLVMValueRef>,
    arrays: HashMap<String, ArrayInfo>,
    records: HashMap<String, RecordInfo>,
    functions: HashMap<String, LLVMValueRef>,
    in_function: bool,
    terminated: bool,
    /// Constructs this backend cannot build. The VM supports considerably
    /// more of the language than the LLVM path does, and emitting IR that
    /// drops a statement or evaluates an expression as 0.0 would make the
    /// compiled program quietly disagree with the same source run on the VM.
    /// The caller must refuse to emit when this is non-empty.
    unsupported: Vec<String>,
}

#[cfg(not(feature = "llvm"))]
pub struct Compiler {
    // Placeholder struct for non-LLVM builds
}


#[cfg(feature = "llvm")]
impl Compiler {
    /// Create a new LLVM compiler instance
    pub fn new() -> Self {
        unsafe {
            let context = LLVMContextCreate();
            let module = LLVMModuleCreateWithNameInContext(
                CString::new("etamil_module").unwrap().as_ptr(),
                context,
            );
            let builder = LLVMCreateBuilderInContext(context);

            // Create main function
            let i32_type = LLVMInt32TypeInContext(context);
            let fn_type = LLVMFunctionType(i32_type, ptr::null_mut(), 0, 0);
            let function = LLVMAddFunction(
                module,
                CString::new("main").unwrap().as_ptr(),
                fn_type,
            );
            
            let entry_block = LLVMAppendBasicBlockInContext(
                context,
                function,
                CString::new("entry").unwrap().as_ptr(),
            );
            
            LLVMPositionBuilderAtEnd(builder, entry_block);

            Compiler {
                context,
                module,
                builder,
                function,
                variables: HashMap::new(),
                globals: HashMap::new(),
                arrays: HashMap::new(),
                records: HashMap::new(),
                functions: HashMap::new(),
                in_function: false,
                terminated: false,
                unsupported: Vec::new(),
            }
        }
    }

    /// Constructs encountered that this backend cannot compile.
    pub fn unsupported(&self) -> &[String] {
        &self.unsupported
    }

    // --- Numbers -----------------------------------------------------------
    //
    // Every number that reaches this backend is a whole number, because
    // `codegen_limits` refuses a program with a fractional literal in it
    // before any IR exists. So the representation is `i64`, not `double`.
    //
    // That is the whole of the change, and it is why money works here now:
    // an i64 is exact to 2^63 where a double is exact to 2^53, and `sdiv`
    // divides exactly where `fdiv` was out by one in the last place — which
    // for a figure held in paise is a paisa. The alternative was decimal
    // arithmetic over a runtime library, twenty-three instruction sites and a
    // second artefact to link. See docs/llvm-backend-gaps.md.

    fn number_type(&self) -> LLVMTypeRef {
        unsafe { LLVMInt64TypeInContext(self.context) }
    }

    fn number_const(&self, value: i64) -> LLVMValueRef {
        unsafe { LLVMConstInt(self.number_type(), value as u64, 1) }
    }

    /// An `alloca` at the top of the current function's entry block, wherever
    /// the builder happens to be standing.
    ///
    /// A variable first assigned inside a branch used to allocate its storage
    /// inside that branch, and a load after the branch then referred to
    /// storage the load did not dominate. That is invalid IR, and LLVM only
    /// says so later — at verification, or as a miscompile. Entry-block
    /// allocas are also the shape every optimiser expects.
    fn entry_alloca(&mut self, kind: LLVMTypeRef, name: &str) -> LLVMValueRef {
        unsafe {
            let here = LLVMGetInsertBlock(self.builder);
            let entry = LLVMGetEntryBasicBlock(self.function);
            let first = LLVMGetFirstInstruction(entry);
            if first.is_null() {
                LLVMPositionBuilderAtEnd(self.builder, entry);
            } else {
                LLVMPositionBuilderBefore(self.builder, first);
            }
            let pointer = LLVMBuildAlloca(
                self.builder,
                kind,
                CString::new(name).unwrap().as_ptr(),
            );
            LLVMPositionBuilderAtEnd(self.builder, here);
            pointer
        }
    }

    /// Storage for a top-level name, as a module global so that a function can
    /// read it. Declared before any function body is compiled; the value is
    /// stored when the assignment itself runs, which is always before a call
    /// that reads it, because every call is reached from `main`.
    fn declare_global(&mut self, name: &str) -> LLVMValueRef {
        if let Some(pointer) = self.globals.get(name).copied() {
            return pointer;
        }
        unsafe {
            let global = LLVMAddGlobal(
                self.module,
                self.number_type(),
                CString::new(name).unwrap().as_ptr(),
            );
            LLVMSetInitializer(global, self.number_const(0));
            LLVMSetLinkage(global, LLVMLinkage::LLVMInternalLinkage);
            self.globals.insert(name.to_string(), global);
            global
        }
    }

    /// Where an assignment to this name should write.
    ///
    /// Inside a `செயல்` a name is always local, even when a global of the same
    /// name exists — assigning to one in a function makes a local, which is
    /// what the VM does and what nUlakam is written around. At top level the
    /// storage is the global declared for it.
    fn storage_for(&mut self, name: &str) -> LLVMValueRef {
        if let Some(pointer) = self.variables.get(name).copied() {
            return pointer;
        }
        if !self.in_function {
            if let Some(pointer) = self.globals.get(name).copied() {
                return pointer;
            }
        }
        let kind = self.number_type();
        let pointer = self.entry_alloca(kind, name);
        self.variables.insert(name.to_string(), pointer);
        pointer
    }

    /// Where a name's value is read from: a local first, then a global.
    fn lookup(&self, name: &str) -> Option<LLVMValueRef> {
        self.variables
            .get(name)
            .copied()
            .or_else(|| self.globals.get(name).copied())
    }

    /// What to call a statement this backend will not build.
    ///
    /// This is the roadmap `scripts/run_parity.sh` ranks, so a name that says
    /// which statement it was is worth more than a category. Everything below
    /// the first group used to be "handled": each printed a log line the VM
    /// never prints, or stored a placeholder zero, and `கோப்பு_படி` read stdin
    /// instead of the file it named. None of that failed — it produced
    /// different output from the same source on the VM, which is worse. They
    /// are refusals now.
    fn stmt_label(statement: &Stmt) -> &'static str {
        match statement {
            Stmt::FunctionDef { .. } => "செயல் (function definition)",
            Stmt::Return(_) => "திரும்பு (return)",
            Stmt::ForEach { .. } => "ஒவ்வொரு (for-each)",
            Stmt::SetIndex { .. } => "a[i] = v (index assignment)",
            Stmt::SetField { .. } => "r.f = v (field assignment)",
            Stmt::Import(_) => "இறக்கு (import)",
            Stmt::Expression(_) => "an expression statement",
            // Files
            Stmt::FileOpen { .. } => "கோப்பு_திற (open a file)",
            Stmt::FileClose { .. } => "கோப்பு_மூடு (close a file)",
            Stmt::FileRead { .. } => "கோப்பு_படி (read a file)",
            Stmt::FileWrite { .. } => "கோப்பு_எழுது (write a file)",
            Stmt::ReadCSV { .. } => "CSV_படி (read a CSV)",
            Stmt::WriteCSV { .. } => "CSV_எழுது (write a CSV)",
            // Database
            Stmt::DBConnect { .. } => "தரவுசேமி_இணை (connect to a database)",
            Stmt::DBDisconnect { .. } => "தரவுசேமி_பிரி (disconnect)",
            Stmt::DBQuery { .. } => "தளம்_வினா (query)",
            Stmt::DBExecute { .. } => "தளம்_செய் (execute)",
            Stmt::DBInsert { .. } => "தளம்_நுழை (insert)",
            Stmt::DBUpdate { .. } => "தளம்_புதுப்பி (update)",
            Stmt::DBDelete { .. } => "தளம்_நீக்கு (delete)",
            Stmt::CreateTable { .. } => "அட்டவணை_உருவாக்கு (create a table)",
            Stmt::Select { .. } => "தேர்ந்தெடு (select)",
            // Server
            Stmt::DefineRoute { .. } => "வழி (a route)",
            Stmt::StartServer { .. } => "சேவையகம்_தொடங்கு (start a server)",
            Stmt::StopServer => "சேவையகம்_நிறுத்து (stop a server)",
            Stmt::SendResponse { .. } => "பதில்_அனுப்பு (send a response)",
            Stmt::SendJSON { .. } => "ஜேசான்_உரை (send JSON)",
            Stmt::GetRequestBody { .. } => "வேண்டுகோள்_உடல் (the request body)",
            Stmt::GetRequestParam { .. } => "வேண்டுகோள்_அளபுரு (a request parameter)",
            Stmt::GetHeader { .. } => "தலைப்பு_பெறு (read a header)",
            Stmt::SetHeader { .. } => "தலைப்பு_அமை (set a header)",
            _ => "a statement this backend does not build",
        }
    }

    fn expr_label(expression: &Expr) -> &'static str {
        match expression {
            Expr::Call { .. } => "a function call",
            Expr::ArrayLiteral(_) => "an array literal",
            Expr::RecordLiteral(_) => "a record literal",
            Expr::Index { .. } => "an index",
            Expr::Field { .. } => "a field access",
            Expr::Try(_) => "the ? operator",
            Expr::String(_) => "உரை (a text value)",
            Expr::Logical { .. } => "a logical operator",
            Expr::Not(_) => "இல்லை (not)",
            Expr::Boolean(_) => "a boolean literal",
            Expr::Null => "இன்மை (nil)",
            _ => "this expression",
        }
    }

    /// Compile the entire AST
    pub fn compile(&mut self, statements: Vec<Stmt>) {
        // Decided by reading the program, before any IR exists. The largest
        // thing this backend gets wrong is not a missing construct but a
        // present one: it computes in f64, so decimal arithmetic compiles to
        // IR that runs and answers something slightly other than the VM does.
        //
        // That is the one failure this project refuses everywhere else — a
        // wrong answer with no warning — and the field below already says the
        // caller must refuse to emit when it is non-empty. See
        // src/codegen_limits.rs, which is where the reasoning and its tests
        // live; it walks the AST and needs no LLVM, so it is checkable on
        // machines that cannot build this file.
        self.unsupported
            .extend(crate::codegen_limits::refusals(&statements));

        // Top-level names get their storage before any function body is
        // compiled, because a function body may read one and function bodies
        // are compiled first — so a name assigned further down the file would
        // otherwise be invisible from inside a `செயல்` written above it.
        // Arrays and records are not included: they live in their own maps as
        // stack slots and a function cannot reach them either way, which is
        // reported rather than guessed at.
        for statement in &statements {
            if let Stmt::Assign { name, value, .. } = statement {
                if !matches!(value, Expr::ArrayLiteral(_) | Expr::RecordLiteral(_)) {
                    self.declare_global(name);
                }
            }
        }

        unsafe {
            for statement in &statements {
                if let Stmt::FunctionDef { name, params, .. } = statement {
                    self.declare_function(name, params.len());
                }
            }

            for statement in &statements {
                if let Stmt::FunctionDef { name, params, body } = statement {
                    self.compile_function(name, params, body);
                }
            }

            for stmt in statements {
                if !matches!(stmt, Stmt::FunctionDef { .. }) {
                    self.compile_stmt(stmt);
                }
            }
            
            // Return 0 from main
            let i32_type = LLVMInt32TypeInContext(self.context);
            let zero = LLVMConstInt(i32_type, 0, 0);
            LLVMBuildRet(self.builder, zero);
        }
    }

    /// Compile a statement
    fn compile_stmt(&mut self, stmt: Stmt) {
        unsafe {
            match stmt {
                Stmt::Assign { name, value, .. } => {
                    // A name that becomes an array or a record stops being a
                    // number, and the global declared for it holds whatever it
                    // last held. Dropping it makes a later read of the name as
                    // a number a refusal rather than a stale figure.
                    if let Expr::ArrayLiteral(items) = &value {
                        let array = self.compile_array_literal(items);
                        self.variables.remove(&name);
                        self.globals.remove(&name);
                        self.records.remove(&name);
                        self.arrays.insert(name, array);
                        return;
                    }
                    if let Expr::RecordLiteral(fields) = &value {
                        let record = self.compile_record_literal(fields);
                        self.variables.remove(&name);
                        self.globals.remove(&name);
                        self.arrays.remove(&name);
                        self.records.insert(name, record);
                        return;
                    }

                    let val = self.compile_expr(&value);
                    let var_ptr = self.storage_for(&name);
                    LLVMBuildStore(self.builder, val, var_ptr);
                }
                Stmt::FunctionDef { .. } => {}
                Stmt::Return(value) => {
                    if self.in_function {
                        let val = match value.as_ref() {
                            Some(expr) => self.compile_expr(expr),
                            None => self.number_const(0),
                        };
                        LLVMBuildRet(self.builder, val);
                        self.terminated = true;
                    } else {
                        self.unsupported.push("திரும்பு (return)".to_string());
                    }
                }
                Stmt::SetIndex { name, index, value } => {
                    if let Some(array) = self.arrays.get(&name).copied() {
                        let index = self.compile_array_index(&index);
                        let value = self.compile_expr(&value);
                        let mut indices = [LLVMConstInt(LLVMInt32TypeInContext(self.context), 0, 0), index];
                        let element = LLVMBuildGEP2(
                            self.builder,
                            array.array_type,
                            array.pointer,
                            indices.as_mut_ptr(),
                            2,
                            CString::new("array_element").unwrap().as_ptr(),
                        );
                        LLVMBuildStore(self.builder, value, element);
                    } else {
                        self.unsupported.push("array assignment".to_string());
                    }
                }
                Stmt::SetField { name, field, value } => {
                    let pointer = self.records.get(&name).and_then(|record| record.get(&field).copied());
                    if let Some(pointer) = pointer {
                        let value = self.compile_expr(&value);
                        LLVMBuildStore(self.builder, value, pointer);
                    } else {
                        self.unsupported.push(format!("record field {}", field));
                    }
                }
                Stmt::Print(expr) => {
                    // Emit a printf call; strings and numbers are handled differently
                    let (printf, printf_type) = self.get_printf();
                    assert!(!printf.is_null(), "printf declaration missing");
                    assert!(!printf_type.is_null(), "printf type missing");

                    match expr {
                        // A literal needs no value representation — it is
                        // already bytes, and printing bytes is the one thing
                        // this backend can do with text. Which matters more
                        // than it sounds: nearly every example opens with a
                        // banner line, and the arm this replaces looked for
                        // `Variable("\"text\"")`, a shape the parser stopped
                        // producing, so every one of them was refused.
                        Expr::String(text) => {
                            self.print_text(&format!("{}\n", text));
                        }
                        Expr::Concat { .. } => {
                            // Handle concatenated expressions by evaluating and printing
                            self.print_concat(&expr);
                        }
                        _ => {
                            // A whole number, so %lld rather than %.0f: the
                            // VM prints 6, not 6.0, and printf given a double
                            // where the format says integer prints garbage.
                            let val = self.compile_expr(&expr);
                            let fmt = CString::new("%lld\n").unwrap();
                            let global_fmt = LLVMBuildGlobalStringPtr(
                                self.builder,
                                fmt.as_ptr(),
                                CString::new("fmt").unwrap().as_ptr(),
                            );

                            let mut args = [global_fmt, val];

                            LLVMBuildCall2(
                                self.builder,
                                printf_type,
                                printf,
                                args.as_mut_ptr(),
                                args.len() as u32,
                                CString::new("").unwrap().as_ptr(),
                            );
                        }
                    }
                }
                Stmt::Input(expr) => {
                    // Input statement: print prompt and read value from stdin
                    let (scanf, scanf_type) = self.get_scanf();

                    match expr {
                        Expr::Concat { left, right } => {
                            // Print the prompt (left side)
                            if let Expr::String(text) = left.as_ref() {
                                let text = text.clone();
                                self.print_text(&text);
                            }

                            // Read into the variable (right side). Whether the
                            // name already has storage is `storage_for`'s
                            // business, and it is also what decides between a
                            // local and a global — which the two hand-written
                            // branches this replaces both got wrong, always
                            // allocating a local and so shadowing the global a
                            // function would go on to read.
                            if let Expr::Variable(var_name) = right.as_ref() {
                                let var_ptr = self.storage_for(var_name);
                                let fmt = CString::new("%lld").unwrap();
                                let global_fmt = LLVMBuildGlobalStringPtr(
                                    self.builder,
                                    fmt.as_ptr(),
                                    CString::new("scanf_fmt").unwrap().as_ptr(),
                                );
                                let mut args = [global_fmt, var_ptr];
                                LLVMBuildCall2(
                                    self.builder,
                                    scanf_type,
                                    scanf,
                                    args.as_mut_ptr(),
                                    args.len() as u32,
                                    CString::new("").unwrap().as_ptr(),
                                );
                            } else {
                                // Printing the prompt and then reading nothing
                                // would leave the program waiting for input it
                                // never consumes.
                                self.unsupported.push(format!(
                                    "உள்ளிடு reading into {}",
                                    Self::expr_label(right.as_ref())
                                ));
                            }
                        }
                        Expr::String(text) => {
                            // A prompt with nothing to read into.
                            self.print_text(&text);
                        }
                        other => {
                            self.unsupported.push(format!(
                                "உள்ளிடு with {}",
                                Self::expr_label(&other)
                            ));
                        }
                    }
                }
                Stmt::If { condition, then_branch, else_branch } => {
                    let cond_val = self.compile_comparison(&condition);
                    
                    let then_bb = LLVMAppendBasicBlockInContext(
                        self.context,
                        self.function,
                        CString::new("then").unwrap().as_ptr(),
                    );
                    let else_bb = LLVMAppendBasicBlockInContext(
                        self.context,
                        self.function,
                        CString::new("else").unwrap().as_ptr(),
                    );
                    let merge_bb = LLVMAppendBasicBlockInContext(
                        self.context,
                        self.function,
                        CString::new("merge").unwrap().as_ptr(),
                    );
                    
                    LLVMBuildCondBr(self.builder, cond_val, then_bb, else_bb);

                    // A branch that returns has already terminated its block,
                    // and a second terminator after `ret` is invalid IR. This
                    // used to be unreachable in practice because no function
                    // with a guard clause could get this far; a guard clause is
                    // the first thing a money function has —
                    // `(கீழ் == 0) எனில் { திரும்பு 0; }` — so it is reachable
                    // now.
                    LLVMPositionBuilderAtEnd(self.builder, then_bb);
                    self.terminated = false;
                    for stmt in then_branch {
                        if self.terminated {
                            break;
                        }
                        self.compile_stmt(stmt);
                    }
                    let then_returned = self.terminated;
                    if !then_returned {
                        LLVMBuildBr(self.builder, merge_bb);
                    }

                    LLVMPositionBuilderAtEnd(self.builder, else_bb);
                    self.terminated = false;
                    if let Some(stmts) = else_branch {
                        for stmt in stmts {
                            if self.terminated {
                                break;
                            }
                            self.compile_stmt(stmt);
                        }
                    }
                    let else_returned = self.terminated;
                    if !else_returned {
                        LLVMBuildBr(self.builder, merge_bb);
                    }

                    // Continue after if. When both arms returned, nothing
                    // branches here — but a block still needs a terminator, and
                    // `unreachable` is the honest one.
                    LLVMPositionBuilderAtEnd(self.builder, merge_bb);
                    self.terminated = then_returned && else_returned;
                    if self.terminated {
                        LLVMBuildUnreachable(self.builder);
                    }
                }
                Stmt::Loop { condition, body } => {
                    let loop_cond_bb = LLVMAppendBasicBlockInContext(
                        self.context,
                        self.function,
                        CString::new("loop_cond").unwrap().as_ptr(),
                    );
                    let loop_body_bb = LLVMAppendBasicBlockInContext(
                        self.context,
                        self.function,
                        CString::new("loop_body").unwrap().as_ptr(),
                    );
                    let after_loop_bb = LLVMAppendBasicBlockInContext(
                        self.context,
                        self.function,
                        CString::new("after_loop").unwrap().as_ptr(),
                    );
                    
                    LLVMBuildBr(self.builder, loop_cond_bb);
                    
                    // Loop condition
                    LLVMPositionBuilderAtEnd(self.builder, loop_cond_bb);
                    let cond_val = self.compile_comparison(&condition);
                    LLVMBuildCondBr(self.builder, cond_val, loop_body_bb, after_loop_bb);
                    
                    // Loop body. A `திரும்பு` inside the loop terminates the
                    // block it is in, so there is no back edge to build.
                    LLVMPositionBuilderAtEnd(self.builder, loop_body_bb);
                    self.terminated = false;
                    for stmt in body {
                        if self.terminated {
                            break;
                        }
                        self.compile_stmt(stmt);
                    }
                    if !self.terminated {
                        LLVMBuildBr(self.builder, loop_cond_bb);
                    }

                    // After loop. Reached from the condition however the body
                    // ended, so compilation carries on from here either way.
                    LLVMPositionBuilderAtEnd(self.builder, after_loop_bb);
                    self.terminated = false;
                }
                Stmt::ForEach { var, collection, body } => {
                    let array = match &collection {
                        Expr::Variable(name) => self.arrays.get(name).copied(),
                        Expr::ArrayLiteral(items) => Some(self.compile_array_literal(items)),
                        _ => None,
                    };

                    if let Some(array) = array {
                        self.compile_array_foreach(&var, array, &body);
                    } else {
                        self.unsupported.push("ஒவ்வொரு (for-each) over a numeric array".to_string());
                    }
                }
                other => {
                    // Recording rather than ignoring: a statement the LLVM
                    // backend drops would make the compiled program quietly
                    // disagree with the same source run on the VM.
                    self.unsupported
                        .push(format!("statement {}", Self::stmt_label(&other)));
                }
            }
        }
    }

    fn llvm_function_type(&self, parameter_count: usize) -> LLVMTypeRef {
        unsafe {
            let number = self.number_type();
            let mut parameters = vec![number; parameter_count];
            LLVMFunctionType(
                number,
                parameters.as_mut_ptr(),
                parameters.len() as u32,
                0,
            )
        }
    }

    fn declare_function(&mut self, name: &str, parameter_count: usize) -> LLVMValueRef {
        if let Some(function) = self.functions.get(name).copied() {
            return function;
        }

        unsafe {
            let function = LLVMAddFunction(
                self.module,
                CString::new(name).unwrap().as_ptr(),
                self.llvm_function_type(parameter_count),
            );
            self.functions.insert(name.to_string(), function);
            function
        }
    }

    fn compile_function(&mut self, name: &str, params: &[String], body: &[Stmt]) {
        unsafe {
            let function = self.declare_function(name, params.len());
            let saved_function = self.function;
            let saved_block = LLVMGetInsertBlock(self.builder);
            let saved_variables = std::mem::take(&mut self.variables);
            let saved_arrays = std::mem::take(&mut self.arrays);
            let saved_records = std::mem::take(&mut self.records);
            let saved_in_function = self.in_function;
            let saved_terminated = self.terminated;

            self.function = function;
            self.in_function = true;
            self.terminated = false;
            let entry = LLVMAppendBasicBlockInContext(
                self.context,
                function,
                CString::new("entry").unwrap().as_ptr(),
            );
            LLVMPositionBuilderAtEnd(self.builder, entry);

            let number = self.number_type();
            for (index, parameter) in params.iter().enumerate() {
                let pointer = LLVMBuildAlloca(
                    self.builder,
                    number,
                    CString::new(parameter.as_str()).unwrap().as_ptr(),
                );
                LLVMBuildStore(self.builder, LLVMGetParam(function, index as u32), pointer);
                self.variables.insert(parameter.clone(), pointer);
            }

            for statement in body {
                if self.terminated {
                    break;
                }
                self.compile_stmt(statement.clone());
            }

            if !self.terminated {
                let zero = self.number_const(0);
                LLVMBuildRet(self.builder, zero);
            }

            self.function = saved_function;
            self.variables = saved_variables;
            self.arrays = saved_arrays;
            self.records = saved_records;
            self.in_function = saved_in_function;
            self.terminated = saved_terminated;
            LLVMPositionBuilderAtEnd(self.builder, saved_block);
        }
    }

    fn compile_array_literal(&mut self, items: &[Expr]) -> ArrayInfo {
        unsafe {
            let number = self.number_type();
            let element_count = items.len();
            let array_type = LLVMArrayType2(number, element_count.max(1) as u64);
            let pointer = self.entry_alloca(array_type, "array");

            for (index, item) in items.iter().enumerate() {
                let mut indices = [
                    LLVMConstInt(LLVMInt32TypeInContext(self.context), 0, 0),
                    LLVMConstInt(LLVMInt32TypeInContext(self.context), index as u64, 0),
                ];
                let element = LLVMBuildGEP2(
                    self.builder,
                    array_type,
                    pointer,
                    indices.as_mut_ptr(),
                    2,
                    CString::new("array_element").unwrap().as_ptr(),
                );
                LLVMBuildStore(self.builder, self.compile_expr(item), element);
            }

            ArrayInfo {
                pointer,
                element_count,
                array_type,
            }
        }
    }

    fn compile_record_literal(&mut self, fields: &[(String, Expr)]) -> RecordInfo {
        let mut record = HashMap::new();
        for (field, value) in fields {
            let number = self.number_type();
            let pointer = self.entry_alloca(number, &format!("record_{}", field));
            let stored = self.compile_expr(value);
            unsafe {
                LLVMBuildStore(self.builder, stored, pointer);
            }
            record.insert(field.clone(), pointer);
        }
        record
    }

    fn compile_array_index(&mut self, index: &Expr) -> LLVMValueRef {
        unsafe {
            // A number is an i64 now, so this narrows rather than converting
            // from a double. Array lengths here are what fits in a literal.
            let value = self.compile_expr(index);
            LLVMBuildTrunc(
                self.builder,
                value,
                LLVMInt32TypeInContext(self.context),
                CString::new("array_index").unwrap().as_ptr(),
            )
        }
    }

    fn compile_array_foreach(&mut self, variable: &str, array: ArrayInfo, body: &[Stmt]) {
        unsafe {
            let i32_type = LLVMInt32TypeInContext(self.context);
            let index_pointer = LLVMBuildAlloca(
                self.builder,
                i32_type,
                CString::new("each_index").unwrap().as_ptr(),
            );
            LLVMBuildStore(self.builder, LLVMConstInt(i32_type, 0, 0), index_pointer);

            let condition_block = LLVMAppendBasicBlockInContext(
                self.context,
                self.function,
                CString::new("each_condition").unwrap().as_ptr(),
            );
            let body_block = LLVMAppendBasicBlockInContext(
                self.context,
                self.function,
                CString::new("each_body").unwrap().as_ptr(),
            );
            let after_block = LLVMAppendBasicBlockInContext(
                self.context,
                self.function,
                CString::new("each_after").unwrap().as_ptr(),
            );
            LLVMBuildBr(self.builder, condition_block);

            LLVMPositionBuilderAtEnd(self.builder, condition_block);
            let index = LLVMBuildLoad2(
                self.builder,
                i32_type,
                index_pointer,
                CString::new("each_index_value").unwrap().as_ptr(),
            );
            let limit = LLVMConstInt(i32_type, array.element_count as u64, 0);
            let condition = LLVMBuildICmp(
                self.builder,
                LLVMIntPredicate::LLVMIntULT,
                index,
                limit,
                CString::new("each_has_value").unwrap().as_ptr(),
            );
            LLVMBuildCondBr(self.builder, condition, body_block, after_block);

            LLVMPositionBuilderAtEnd(self.builder, body_block);
            let mut indices = [LLVMConstInt(i32_type, 0, 0), index];
            let element = LLVMBuildGEP2(
                self.builder,
                array.array_type,
                array.pointer,
                indices.as_mut_ptr(),
                2,
                CString::new("each_element").unwrap().as_ptr(),
            );
            let value = LLVMBuildLoad2(
                self.builder,
                self.number_type(),
                element,
                CString::new("each_value").unwrap().as_ptr(),
            );
            let variable_pointer = self.storage_for(variable);
            LLVMBuildStore(self.builder, value, variable_pointer);

            self.terminated = false;
            for statement in body {
                if self.terminated {
                    break;
                }
                self.compile_stmt(statement.clone());
            }
            if !self.terminated {
                let next = LLVMBuildAdd(
                    self.builder,
                    index,
                    LLVMConstInt(i32_type, 1, 0),
                    CString::new("each_next").unwrap().as_ptr(),
                );
                LLVMBuildStore(self.builder, next, index_pointer);
                LLVMBuildBr(self.builder, condition_block);
            }

            // Reached from the condition whatever the body did, so compilation
            // carries on here even if the body returned.
            LLVMPositionBuilderAtEnd(self.builder, after_block);
            self.terminated = false;
        }
    }

    /// Get or declare printf
    fn get_printf(&self) -> (LLVMValueRef, LLVMTypeRef) {
        unsafe {
            let name = CString::new("printf").unwrap();
            let i8_ptr = LLVMPointerType(LLVMInt8TypeInContext(self.context), 0);
            let mut param_types = [i8_ptr];
            let fn_type = LLVMFunctionType(
                LLVMInt32TypeInContext(self.context),
                param_types.as_mut_ptr(),
                param_types.len() as u32,
                1, // varargs
            );

            let existing = LLVMGetNamedFunction(self.module, name.as_ptr());
            if !existing.is_null() {
                return (existing, fn_type);
            }

            let func = LLVMAddFunction(self.module, name.as_ptr(), fn_type);
            (func, fn_type)
        }
    }

    /// Get or declare scanf
    fn get_scanf(&self) -> (LLVMValueRef, LLVMTypeRef) {
        unsafe {
            let name = CString::new("scanf").unwrap();
            let i8_ptr = LLVMPointerType(LLVMInt8TypeInContext(self.context), 0);
            let mut param_types = [i8_ptr];
            let fn_type = LLVMFunctionType(
                LLVMInt32TypeInContext(self.context),
                param_types.as_mut_ptr(),
                param_types.len() as u32,
                1, // varargs
            );

            let existing = LLVMGetNamedFunction(self.module, name.as_ptr());
            if !existing.is_null() {
                return (existing, fn_type);
            }

            let func = LLVMAddFunction(self.module, name.as_ptr(), fn_type);
            (func, fn_type)
        }
    }

    /// Compile an expression to an LLVM value
    fn compile_expr(&mut self, expr: &Expr) -> LLVMValueRef {
        unsafe {
            match expr {
                Expr::Number(n) => {
                    // codegen_limits refuses both of the failures below before
                    // any IR exists. Checking again here is what makes that a
                    // guarantee rather than a convention held in another file.
                    match rust_decimal::prelude::ToPrimitive::to_i64(n) {
                        Some(whole) if n.fract().is_zero() => self.number_const(whole),
                        _ => {
                            self.unsupported
                                .push(format!("the number {} (no exact i64 holds it)", n));
                            self.number_const(0)
                        }
                    }
                }
                Expr::Variable(name) => {
                    if let Some(var_ptr) = self.lookup(name) {
                        LLVMBuildLoad2(
                            self.builder,
                            self.number_type(),
                            var_ptr,
                            CString::new("load").unwrap().as_ptr(),
                        )
                    } else if self.arrays.contains_key(name) {
                        // The name is defined — it holds an array, and an array
                        // is not a value here, so it cannot be returned or
                        // passed or added to anything.
                        //
                        // Saying "nothing defines it" instead sent 33 refusals
                        // in the parity run to a cause that did not exist:
                        // nUlakam/aNi.qmz builds up `விடை = []` and returns it,
                        // and the honest reason is this one, which belongs to
                        // the boxed-value gap rather than to name resolution.
                        self.unsupported.push(format!(
                            "the name {} holds an அணி (an array is not a value here)",
                            name
                        ));
                        self.number_const(0)
                    } else if self.records.contains_key(name) {
                        self.unsupported.push(format!(
                            "the name {} holds a பொருள் (a record is not a value here)",
                            name
                        ));
                        self.number_const(0)
                    } else if name.starts_with('"') && name.ends_with('"') {
                        // The parser hands a string literal through as a
                        // Variable whose name is the quoted text, and print
                        // handles that shape before it reaches here. Anywhere
                        // else it is a text value, which has no form in this IR.
                        self.unsupported
                            .push("a text value (strings have no representation here)".to_string());
                        self.number_const(0)
                    } else {
                        // This used to answer 0.0 in silence, and that was the
                        // worst line in the file: a name a function could not
                        // see — every global, before the map above existed —
                        // compiled to zero and the program ran on.
                        self.unsupported
                            .push(format!("the name {} (nothing here defines it)", name));
                        self.number_const(0)
                    }
                }
                Expr::BinaryOp { op, left, right } => {
                    let lhs = self.compile_expr(left);
                    let rhs = self.compile_expr(right);

                    match op.as_str() {
                        "+" => LLVMBuildAdd(self.builder, lhs, rhs, CString::new("add").unwrap().as_ptr()),
                        "-" => LLVMBuildSub(self.builder, lhs, rhs, CString::new("sub").unwrap().as_ptr()),
                        "*" => LLVMBuildMul(self.builder, lhs, rhs, CString::new("mul").unwrap().as_ptr()),
                        // A bare division is refused by codegen_limits, because
                        // the quotient of two whole numbers usually is not one
                        // and an i64 has nowhere to keep the rest. It is exact
                        // under தரை or மேல், and that is handled at the call.
                        //
                        // Its own words, verbatim: a second phrasing of the
                        // same cause listed twice in the parity summary and
                        // halved the rank of what is actually the commonest
                        // arithmetic gap in the corpus.
                        "/" => {
                            self.unsupported
                                .push(crate::codegen_limits::DIVISION.to_string());
                            self.number_const(0)
                        }
                        other => {
                            // The parser builds only + - * / today. Recording
                            // rather than answering zero is what keeps that
                            // true of tomorrow's parser too.
                            self.unsupported.push(format!("the operator {}", other));
                            self.number_const(0)
                        }
                    }
                }
                Expr::Comparison { .. } => {
                    // As a branch condition a comparison is fine, and
                    // `compile_comparison` builds that. As a *value* it is a
                    // மெய் or a பொய், and the VM prints those as words — so an
                    // i64 1 here would not be a narrower answer, it would be a
                    // different one.
                    self.unsupported.push(
                        "a comparison used as a value (மெய்/பொய் has no representation here)"
                            .to_string(),
                    );
                    self.number_const(0)
                }
                Expr::Call { name, args } => {
                    // A function the author wrote wins over a builtin of the
                    // same name, which is the order the VM resolves in too.
                    let function = self.functions.get(name).copied();
                    if let Some(function) = function {
                        let mut values: Vec<LLVMValueRef> =
                            args.iter().map(|arg| self.compile_expr(arg)).collect();
                        LLVMBuildCall2(
                            self.builder,
                            self.llvm_function_type(values.len()),
                            function,
                            values.as_mut_ptr(),
                            values.len() as u32,
                            CString::new("call").unwrap().as_ptr(),
                        )
                    } else if let Some(builtin) = crate::codegen_limits::whole_number_builtin(name) {
                        self.compile_whole_number_builtin(name, builtin, args)
                    } else {
                        self.unsupported.push(format!("function call {}", name));
                        self.number_const(0)
                    }
                }
                Expr::Index { base, index } => {
                    let array = match base.as_ref() {
                        Expr::Variable(name) => self.arrays.get(name).copied(),
                        _ => None,
                    };
                    if let Some(array) = array {
                        let index = self.compile_array_index(index);
                        let mut indices = [LLVMConstInt(LLVMInt32TypeInContext(self.context), 0, 0), index];
                        let element = LLVMBuildGEP2(
                            self.builder,
                            array.array_type,
                            array.pointer,
                            indices.as_mut_ptr(),
                            2,
                            CString::new("array_element").unwrap().as_ptr(),
                        );
                        LLVMBuildLoad2(
                            self.builder,
                            self.number_type(),
                            element,
                            CString::new("array_value").unwrap().as_ptr(),
                        )
                    } else {
                        self.unsupported.push("array index".to_string());
                        self.number_const(0)
                    }
                }
                Expr::Field { base, name } => {
                    let pointer = match base.as_ref() {
                        Expr::Variable(variable) => self
                            .records
                            .get(variable)
                            .and_then(|record| record.get(name).copied()),
                        _ => None,
                    };
                    if let Some(pointer) = pointer {
                        LLVMBuildLoad2(
                            self.builder,
                            self.number_type(),
                            pointer,
                            CString::new("record_value").unwrap().as_ptr(),
                        )
                    } else {
                        self.unsupported.push(format!("record field {}", name));
                        self.number_const(0)
                    }
                }
                Expr::Concat { left, right: _ } => {
                    // For concat in expression context, just evaluate left side
                    // (concat is mainly for print statements)
                    self.compile_expr(left)
                }
                other => {
                    // Same reasoning: yielding 0 for an expression this
                    // backend cannot build would silently change the answer.
                    self.unsupported
                        .push(format!("expression {}", Self::expr_label(other)));
                    self.number_const(0)
                }
            }
        }
    }

    /// `printf("%s", text)` — through `%s` rather than as the format itself,
    /// so that a `%` in the text stays a `%` instead of being read as a
    /// conversion and printing whatever happens to be in a register.
    fn print_text(&mut self, text: &str) {
        let literal = match CString::new(text) {
            Ok(literal) => literal,
            Err(_) => {
                // A NUL inside the text would end the C string early, so the
                // program would print less than it asked to.
                self.unsupported
                    .push("text with a NUL byte in it".to_string());
                return;
            }
        };
        unsafe {
            let (printf, printf_type) = self.get_printf();
            let buffer = LLVMBuildGlobalStringPtr(
                self.builder,
                literal.as_ptr(),
                CString::new("text").unwrap().as_ptr(),
            );
            let format = CString::new("%s").unwrap();
            let global_format = LLVMBuildGlobalStringPtr(
                self.builder,
                format.as_ptr(),
                CString::new("text_fmt").unwrap().as_ptr(),
            );
            let mut args = [global_format, buffer];
            LLVMBuildCall2(
                self.builder,
                printf_type,
                printf,
                args.as_mut_ptr(),
                args.len() as u32,
                CString::new("").unwrap().as_ptr(),
            );
        }
    }

    /// Helper to print concatenated expressions
    fn print_concat(&mut self, expr: &Expr) {
        unsafe {
            let (printf, printf_type) = self.get_printf();
            
            match expr {
                Expr::Concat { left, right } => {
                    // Recursively print left side
                    self.print_concat_part(left);
                    // Then print right side
                    self.print_concat_part(right);
                }
                _ => self.print_concat_part(expr),
            }
            
            // Print final newline
            let newline = CString::new("\n").unwrap();
            let global_nl = LLVMBuildGlobalStringPtr(
                self.builder,
                newline.as_ptr(),
                CString::new("nl").unwrap().as_ptr(),
            );
            let mut args = [global_nl];
            LLVMBuildCall2(
                self.builder,
                printf_type,
                printf,
                args.as_mut_ptr(),
                1,
                CString::new("").unwrap().as_ptr(),
            );
        }
    }

    fn print_concat_part(&mut self, expr: &Expr) {
        // `&` between a label and a number is how nearly every example says
        // anything, so both halves have to work for either to be worth having.
        match expr {
            Expr::String(text) => {
                self.print_text(text);
                return;
            }
            Expr::Concat { left, right } => {
                self.print_concat_part(left);
                self.print_concat_part(right);
                return;
            }
            _ => {}
        }

        let val = self.compile_expr(expr);
        unsafe {
            let (printf, printf_type) = self.get_printf();
            let fmt = CString::new("%lld").unwrap();
            let global_fmt = LLVMBuildGlobalStringPtr(
                self.builder,
                fmt.as_ptr(),
                CString::new("fmt").unwrap().as_ptr(),
            );
            let mut args = [global_fmt, val];
            LLVMBuildCall2(
                self.builder,
                printf_type,
                printf,
                args.as_mut_ptr(),
                args.len() as u32,
                CString::new("").unwrap().as_ptr(),
            );
        }
    }

    /// A branch condition: the `i1` that `எனில்` and `சுற்று` need.
    ///
    /// A comparison compiles straight to the integer compare. Anything else is
    /// "not zero", which is what the VM does with a number in a condition —
    /// checked against it rather than assumed: 5 takes the branch and 0 does
    /// not.
    ///
    /// This used to be whatever `compile_expr` happened to return, an `i1` for
    /// a comparison and a `double` for everything else, and a `double` handed
    /// to a conditional branch is not IR that builds at all.
    fn compile_comparison(&mut self, expr: &Expr) -> LLVMValueRef {
        if let Expr::Comparison { left, op, right } = expr {
            let lhs = self.compile_expr(left);
            let rhs = self.compile_expr(right);
            let pred = match op.as_str() {
                ">" => LLVMIntPredicate::LLVMIntSGT,
                "<" => LLVMIntPredicate::LLVMIntSLT,
                ">=" => LLVMIntPredicate::LLVMIntSGE,
                "<=" => LLVMIntPredicate::LLVMIntSLE,
                "==" => LLVMIntPredicate::LLVMIntEQ,
                "!=" => LLVMIntPredicate::LLVMIntNE,
                _ => LLVMIntPredicate::LLVMIntEQ,
            };
            return unsafe {
                LLVMBuildICmp(
                    self.builder,
                    pred,
                    lhs,
                    rhs,
                    CString::new("cmp").unwrap().as_ptr(),
                )
            };
        }

        let value = self.compile_expr(expr);
        unsafe {
            let zero = self.number_const(0);
            LLVMBuildICmp(
                self.builder,
                LLVMIntPredicate::LLVMIntNE,
                value,
                zero,
                CString::new("condition").unwrap().as_ptr(),
            )
        }
    }

    /// The three numeric builtins that are exact on whole numbers, and so the
    /// only ones this backend can reach at all: every other builtin takes or
    /// returns a string, an array, a record or a result, and none of those has
    /// a representation in the emitted IR.
    ///
    /// `தரை(அ / ஆ)` is the one that earns its place. Over anything that is not
    /// a division both `தரை` and `மேல்` are the identity, because a whole
    /// number is already floored and already ceilinged — and so is `வட்டமிடு`,
    /// at any number of places.
    fn compile_whole_number_builtin(
        &mut self,
        name: &str,
        builtin: WholeNumberBuiltin,
        args: &[Expr],
    ) -> LLVMValueRef {
        let wanted = if matches!(builtin, WholeNumberBuiltin::Round) {
            2
        } else {
            1
        };
        if args.len() != wanted {
            self.unsupported.push(format!(
                "{} given {} arguments, and it takes {}",
                name,
                args.len(),
                wanted
            ));
            return self.number_const(0);
        }

        if matches!(builtin, WholeNumberBuiltin::Round) {
            // Rounding a whole number to any number of decimal places is the
            // number itself. A negative count is a runtime error on the VM and
            // a computed one cannot be read from here, so only a literal count
            // that the VM would also accept is compiled.
            let places_are_readable = match &args[1] {
                Expr::Number(places) => {
                    places.fract().is_zero()
                        && rust_decimal::prelude::ToPrimitive::to_u32(places).is_some()
                }
                _ => false,
            };
            if !places_are_readable {
                self.unsupported
                    .push(format!("{} with a place count this backend cannot read", name));
                return self.number_const(0);
            }
            return self.compile_expr(&args[0]);
        }

        if let Expr::BinaryOp { op, left, right } = &args[0] {
            if op == "/" {
                let lhs = self.compile_expr(left);
                let rhs = self.compile_expr(right);
                return self.build_integer_division(lhs, rhs, builtin);
            }
        }
        self.compile_expr(&args[0])
    }

    /// `தரை(அ / ஆ)` or `மேல்(அ / ஆ)` over whole numbers, exactly.
    ///
    /// This is the operation the whole change is for. `sdiv` truncates towards
    /// zero, which is the floor when the quotient is positive and the ceiling
    /// when it is negative — so exactly one of the two directions needs a
    /// correction of one, and which one depends on the sign of the answer. The
    /// sign of the answer is the two operands' signs xor'd together.
    ///
    /// Getting that correction wrong is one paisa, in the direction that makes
    /// a split fail to add up to the amount that was split.
    fn build_integer_division(
        &mut self,
        lhs: LLVMValueRef,
        rhs: LLVMValueRef,
        rounding: WholeNumberBuiltin,
    ) -> LLVMValueRef {
        unsafe {
            let zero = self.number_const(0);

            // The VM stops with "பூஜ்ஜியத்தால் வகுத்தல்" rather than dividing
            // by zero. An `sdiv` by zero is undefined behaviour — on x86 it
            // faults, which is a crash with no message and a different answer
            // from the VM's — so the check is emitted rather than left to the
            // hardware.
            let by_zero = LLVMBuildICmp(
                self.builder,
                LLVMIntPredicate::LLVMIntEQ,
                rhs,
                zero,
                CString::new("divisor_is_zero").unwrap().as_ptr(),
            );
            let die_block = LLVMAppendBasicBlockInContext(
                self.context,
                self.function,
                CString::new("divide_by_zero").unwrap().as_ptr(),
            );
            let ok_block = LLVMAppendBasicBlockInContext(
                self.context,
                self.function,
                CString::new("divide").unwrap().as_ptr(),
            );
            LLVMBuildCondBr(self.builder, by_zero, die_block, ok_block);

            LLVMPositionBuilderAtEnd(self.builder, die_block);
            self.build_die("பூஜ்ஜியத்தால் வகுத்தல்  (division by zero)\n");

            LLVMPositionBuilderAtEnd(self.builder, ok_block);
            let quotient = LLVMBuildSDiv(
                self.builder,
                lhs,
                rhs,
                CString::new("quotient").unwrap().as_ptr(),
            );
            let remainder = LLVMBuildSRem(
                self.builder,
                lhs,
                rhs,
                CString::new("remainder").unwrap().as_ptr(),
            );
            let inexact = LLVMBuildICmp(
                self.builder,
                LLVMIntPredicate::LLVMIntNE,
                remainder,
                zero,
                CString::new("inexact").unwrap().as_ptr(),
            );
            let signs = LLVMBuildXor(
                self.builder,
                lhs,
                rhs,
                CString::new("signs").unwrap().as_ptr(),
            );

            // Truncation went the wrong way when the exact quotient lies on the
            // side being rounded towards: negative for a floor, positive for a
            // ceiling. And only when the division left a remainder — an exact
            // division is already the answer.
            let (towards, step) = match rounding {
                WholeNumberBuiltin::Ceil => (LLVMIntPredicate::LLVMIntSGE, 1),
                _ => (LLVMIntPredicate::LLVMIntSLT, -1),
            };
            let rounds_away = LLVMBuildICmp(
                self.builder,
                towards,
                signs,
                zero,
                CString::new("rounds_away").unwrap().as_ptr(),
            );
            let correcting = LLVMBuildAnd(
                self.builder,
                inexact,
                rounds_away,
                CString::new("needs_correction").unwrap().as_ptr(),
            );
            let one_further = self.number_const(step);
            let stepped = LLVMBuildAdd(
                self.builder,
                quotient,
                one_further,
                CString::new("stepped").unwrap().as_ptr(),
            );
            LLVMBuildSelect(
                self.builder,
                correcting,
                stepped,
                quotient,
                CString::new("rounded").unwrap().as_ptr(),
            )
        }
    }

    /// Write a message to standard error and stop, the way the VM does when a
    /// program asks arithmetic for something it cannot give.
    ///
    /// `write` and `exit`, not `fprintf(stderr, ...)`: the `stderr` FILE* is a
    /// different symbol on glibc than on macOS, and this backend builds on
    /// both.
    fn build_die(&mut self, message: &str) {
        unsafe {
            let (write, write_type) = self.get_write();
            let (exit, exit_type) = self.get_exit();
            let i32_type = LLVMInt32TypeInContext(self.context);

            let text = CString::new(message).unwrap();
            let buffer = LLVMBuildGlobalStringPtr(
                self.builder,
                text.as_ptr(),
                CString::new("die_message").unwrap().as_ptr(),
            );
            let mut told = [
                LLVMConstInt(i32_type, 2, 0),
                buffer,
                LLVMConstInt(self.number_type(), message.len() as u64, 0),
            ];
            LLVMBuildCall2(
                self.builder,
                write_type,
                write,
                told.as_mut_ptr(),
                told.len() as u32,
                CString::new("").unwrap().as_ptr(),
            );

            let mut status = [LLVMConstInt(i32_type, 1, 0)];
            LLVMBuildCall2(
                self.builder,
                exit_type,
                exit,
                status.as_mut_ptr(),
                status.len() as u32,
                CString::new("").unwrap().as_ptr(),
            );
            LLVMBuildUnreachable(self.builder);
        }
    }

    /// `ssize_t write(int, const void *, size_t)`
    fn get_write(&self) -> (LLVMValueRef, LLVMTypeRef) {
        unsafe {
            let name = CString::new("write").unwrap();
            let mut parameters = [
                LLVMInt32TypeInContext(self.context),
                LLVMPointerType(LLVMInt8TypeInContext(self.context), 0),
                self.number_type(),
            ];
            let kind = LLVMFunctionType(
                self.number_type(),
                parameters.as_mut_ptr(),
                parameters.len() as u32,
                0,
            );

            let existing = LLVMGetNamedFunction(self.module, name.as_ptr());
            if !existing.is_null() {
                return (existing, kind);
            }
            (LLVMAddFunction(self.module, name.as_ptr(), kind), kind)
        }
    }

    /// `void exit(int)`
    fn get_exit(&self) -> (LLVMValueRef, LLVMTypeRef) {
        unsafe {
            let name = CString::new("exit").unwrap();
            let mut parameters = [LLVMInt32TypeInContext(self.context)];
            let kind = LLVMFunctionType(
                LLVMVoidTypeInContext(self.context),
                parameters.as_mut_ptr(),
                parameters.len() as u32,
                0,
            );

            let existing = LLVMGetNamedFunction(self.module, name.as_ptr());
            if !existing.is_null() {
                return (existing, kind);
            }
            (LLVMAddFunction(self.module, name.as_ptr(), kind), kind)
        }
    }

    /// Emit LLVM IR to a file
    pub fn emit_ir(&self, filename: &str) -> Result<(), String> {
        let c_filename = CString::new(filename).map_err(|e| e.to_string())?;
        unsafe {
            let mut error: *mut i8 = std::ptr::null_mut();
            let success = LLVMPrintModuleToFile(self.module, c_filename.as_ptr(), &mut error);
            
            if success != 0 {
                let error_msg = if !error.is_null() {
                    std::ffi::CStr::from_ptr(error)
                        .to_string_lossy()
                        .to_string()
                } else {
                    "Unknown LLVM error".to_string()
                };
                LLVMDisposeMessage(error);
                return Err(error_msg);
            }
        }
        Ok(())
    }

    /// Print the module to stderr for debugging
    pub fn dump_module(&self) {
        unsafe {
            LLVMDumpModule(self.module);
        }
    }
}

// Placeholder implementations for non-LLVM builds (e.g., Windows without LLVM)
#[cfg(not(feature = "llvm"))]
impl Compiler {
    pub fn new() -> Self {
        Compiler {}
    }

    pub fn compile(&mut self, _statements: Vec<Stmt>) {
        eprintln!("WARNING: LLVM code generation is not available on this platform.");
        eprintln!("Please use --vm flag or install LLVM to use --llvm flag.");
    }

    pub fn emit_ir(&self, _filename: &str) -> Result<(), String> {
        Err("LLVM code generation is not available on this platform. Use --vm flag instead.".to_string())
    }

    pub fn dump_module(&self) {
        eprintln!("LLVM module dumping is not available on this platform.");
    }
}

#[cfg(feature = "llvm")]
impl Drop for Compiler {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeBuilder(self.builder);
            LLVMDisposeModule(self.module);
            LLVMContextDispose(self.context);
        }
    }
}
