/// LLVM Code Generator for eTamil using llvm-sys (LLVM 18 compatible)
#[cfg(feature = "llvm")]
use llvm_sys::prelude::*;
#[cfg(feature = "llvm")]
use llvm_sys::core::*;
#[cfg(feature = "llvm")]
use llvm_sys::{LLVMIntPredicate, LLVMRealPredicate};
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
use crate::fileio::csv_handler::FileIOHandler;

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

    fn stmt_label(statement: &Stmt) -> &'static str {
        match statement {
            Stmt::FunctionDef { .. } => "செயல் (function definition)",
            Stmt::Return(_) => "திரும்பு (return)",
            Stmt::ForEach { .. } => "ஒவ்வொரு (for-each)",
            Stmt::SetIndex { .. } => "a[i] = v (index assignment)",
            Stmt::SetField { .. } => "r.f = v (field assignment)",
            Stmt::Import(_) => "இறக்கு (import)",
            Stmt::Expression(_) => "an expression statement",
            _ => "a database or server statement",
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
            Expr::Logical { .. } => "a logical operator",
            Expr::Not(_) => "இல்லை (not)",
            Expr::Boolean(_) => "a boolean literal",
            Expr::Null => "இன்மை (nil)",
            _ => "this expression",
        }
    }

    /// Compile the entire AST
    pub fn compile(&mut self, statements: Vec<Stmt>) {
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
                    if let Expr::ArrayLiteral(items) = &value {
                        let array = self.compile_array_literal(items);
                        self.variables.remove(&name);
                        self.records.remove(&name);
                        self.arrays.insert(name, array);
                        return;
                    }
                    if let Expr::RecordLiteral(fields) = &value {
                        let record = self.compile_record_literal(fields);
                        self.variables.remove(&name);
                        self.arrays.remove(&name);
                        self.records.insert(name, record);
                        return;
                    }

                    let val = self.compile_expr(&value);
                    
                    // Create or get variable allocation
                    if !self.variables.contains_key(&name) {
                        let f64_type = LLVMDoubleTypeInContext(self.context);
                        let alloca = LLVMBuildAlloca(
                            self.builder,
                            f64_type,
                            CString::new(name.as_str()).unwrap().as_ptr(),
                        );
                        self.variables.insert(name.clone(), alloca);
                    }
                    
                    let var_ptr = self.variables.get(&name).unwrap();
                    LLVMBuildStore(self.builder, val, *var_ptr);
                }
                Stmt::FunctionDef { .. } => {}
                Stmt::Return(value) => {
                    if self.in_function {
                        let val = value
                            .as_ref()
                            .map(|expr| self.compile_expr(expr))
                            .unwrap_or_else(|| {
                                LLVMConstReal(
                                    LLVMDoubleTypeInContext(self.context),
                                    0.0,
                                )
                            });
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
                        // String literal is stored as Variable("\"text\"") in the current parser
                        Expr::Variable(s) if s.starts_with('"') && s.ends_with('"') => {
                            let clean = s.trim_matches('"');
                            // Append newline and NUL terminator
                            let fmt = CString::new(format!("{}\n", clean)).unwrap();
                            let global_str = LLVMBuildGlobalStringPtr(
                                self.builder,
                                fmt.as_ptr(),
                                CString::new("str").unwrap().as_ptr(),
                            );

                            let mut args = [global_str];

                            LLVMBuildCall2(
                                self.builder,
                                printf_type,
                                printf,
                                args.as_mut_ptr(),
                                args.len() as u32,
                                CString::new("").unwrap().as_ptr(),
                            );
                        }
                        Expr::Concat { .. } => {
                            // Handle concatenated expressions by evaluating and printing
                            self.print_concat(&expr);
                        }
                        _ => {
                            // Treat as numeric expression and print with %.0f\n
                            let val = self.compile_expr(&expr);
                            let fmt = CString::new("%.0f\n").unwrap();
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
                    let (printf, printf_type) = self.get_printf();
                    let (scanf, scanf_type) = self.get_scanf();

                    match expr {
                        Expr::Concat { left, right } => {
                            // Print the prompt (left side)
                            if let Expr::Variable(s) = left.as_ref() {
                                if s.starts_with('"') && s.ends_with('"') {
                                    let clean = s.trim_matches('"');
                                    let fmt = CString::new(clean).unwrap();
                                    let global_str = LLVMBuildGlobalStringPtr(
                                        self.builder,
                                        fmt.as_ptr(),
                                        CString::new("prompt").unwrap().as_ptr(),
                                    );
                                    let mut args = [global_str];
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
                            
                            // Read into the variable (right side)
                            if let Expr::Variable(var_name) = right.as_ref() {
                                if let Some(var_ptr) = self.variables.get(var_name) {
                                    // Variable exists, scanf into it
                                    let fmt = CString::new("%lf").unwrap();
                                    let global_fmt = LLVMBuildGlobalStringPtr(
                                        self.builder,
                                        fmt.as_ptr(),
                                        CString::new("scanf_fmt").unwrap().as_ptr(),
                                    );
                                    let mut args = [global_fmt, *var_ptr];
                                    LLVMBuildCall2(
                                        self.builder,
                                        scanf_type,
                                        scanf,
                                        args.as_mut_ptr(),
                                        2,
                                        CString::new("").unwrap().as_ptr(),
                                    );
                                } else {
                                    // Variable doesn't exist yet, create it first
                                    let f64_type = LLVMDoubleTypeInContext(self.context);
                                    let alloca = LLVMBuildAlloca(
                                        self.builder,
                                        f64_type,
                                        CString::new(var_name.as_str()).unwrap().as_ptr(),
                                    );
                                    self.variables.insert(var_name.clone(), alloca);
                                    
                                    let fmt = CString::new("%lf").unwrap();
                                    let global_fmt = LLVMBuildGlobalStringPtr(
                                        self.builder,
                                        fmt.as_ptr(),
                                        CString::new("scanf_fmt").unwrap().as_ptr(),
                                    );
                                    let mut args = [global_fmt, alloca];
                                    LLVMBuildCall2(
                                        self.builder,
                                        scanf_type,
                                        scanf,
                                        args.as_mut_ptr(),
                                        2,
                                        CString::new("").unwrap().as_ptr(),
                                    );
                                }
                            }
                        }
                        _ => {
                            // Simple prompt, just print
                            if let Expr::Variable(s) = expr {
                                if s.starts_with('"') && s.ends_with('"') {
                                    let clean = s.trim_matches('"');
                                    let fmt = CString::new(clean).unwrap();
                                    let global_str = LLVMBuildGlobalStringPtr(
                                        self.builder,
                                        fmt.as_ptr(),
                                        CString::new("prompt").unwrap().as_ptr(),
                                    );
                                    let mut args = [global_str];
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
                    
                    // Then branch
                    LLVMPositionBuilderAtEnd(self.builder, then_bb);
                    for stmt in then_branch {
                        self.compile_stmt(stmt);
                    }
                    LLVMBuildBr(self.builder, merge_bb);
                    
                    // Else branch
                    LLVMPositionBuilderAtEnd(self.builder, else_bb);
                    if let Some(stmts) = else_branch {
                        for stmt in stmts {
                            self.compile_stmt(stmt);
                        }
                    }
                    LLVMBuildBr(self.builder, merge_bb);
                    
                    // Continue after if
                    LLVMPositionBuilderAtEnd(self.builder, merge_bb);
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
                    
                    // Loop body
                    LLVMPositionBuilderAtEnd(self.builder, loop_body_bb);
                    for stmt in body {
                        self.compile_stmt(stmt);
                    }
                    LLVMBuildBr(self.builder, loop_cond_bb);
                    
                    // After loop
                    LLVMPositionBuilderAtEnd(self.builder, after_loop_bb);
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
                // File I/O operations
                Stmt::FileOpen { filename: _, mode } => {
                    // Create a file handler and delegate to it
                    let file_handler = FileIOHandler::new(
                        self.context,
                        self.builder,
                        self.module,
                        self.variables.clone(),
                    );
                    file_handler.handle_file_open(&mode);
                    // Update variables from handler
                    let handler_vars = file_handler.get_variables().clone();
                    self.variables = handler_vars;
                }

                Stmt::FileClose { filename: _ } => {
                    // Create a file handler and delegate to it
                    let file_handler = FileIOHandler::new(
                        self.context,
                        self.builder,
                        self.module,
                        self.variables.clone(),
                    );
                    file_handler.handle_file_close();
                }

                Stmt::FileWrite { filename: _, data } => {
                    // Create a file handler and delegate to it
                    let file_handler = FileIOHandler::new(
                        self.context,
                        self.builder,
                        self.module,
                        self.variables.clone(),
                    );
                    let val = self.compile_expr(&data);
                    file_handler.handle_file_write(val);
                }

                Stmt::FileRead { filename: _, variable } => {
                    // Create a file handler and delegate to it
                    let mut file_handler = FileIOHandler::new(
                        self.context,
                        self.builder,
                        self.module,
                        self.variables.clone(),
                    );
                    file_handler.handle_file_read(&variable);
                    // Update variables from handler
                    let handler_vars = file_handler.get_variables().clone();
                    self.variables = handler_vars;
                }

                Stmt::ReadCSV { filename: _, variable } => {
                    // CSV_படி "data.csv", varName;
                    let mut file_handler = FileIOHandler::new(
                        self.context,
                        self.builder,
                        self.module,
                        self.variables.clone(),
                    );
                    file_handler.handle_read_csv(&variable);
                    // Update variables from handler
                    let handler_vars = file_handler.get_variables().clone();
                    self.variables = handler_vars;
                }
                Stmt::WriteCSV { filename: _, data } => {
                    // CSV_எழுது "data.csv", data;
                    let file_handler = FileIOHandler::new(
                        self.context,
                        self.builder,
                        self.module,
                        self.variables.clone(),
                    );
                    let val = self.compile_expr(&data);
                    file_handler.handle_write_csv(val);
                }
                // Database operations
                Stmt::DBConnect { db_type, connection_string } => {
                    // தரவுசேமி_இணை SQL, "connection_string";
                    let (printf, printf_type) = self.get_printf();
                    
                    // Extract connection string if it's a string literal
                    let conn_str = match &connection_string {
                        Expr::Variable(s) if s.starts_with('"') && s.ends_with('"') => {
                            s.trim_matches('"').to_string()
                        }
                        _ => "database_connection".to_string(),
                    };
                    
                    let msg = format!("Connecting to {} database: {}\\n", db_type, conn_str);
                    let fmt = CString::new(msg).unwrap();
                    let global_str = LLVMBuildGlobalStringPtr(
                        self.builder,
                        fmt.as_ptr(),
                        CString::new("db_connect_msg").unwrap().as_ptr(),
                    );
                    let mut args = [global_str];
                    LLVMBuildCall2(
                        self.builder,
                        printf_type,
                        printf,
                        args.as_mut_ptr(),
                        1,
                        CString::new("").unwrap().as_ptr(),
                    );
                }
                Stmt::DBDisconnect { db_type } => {
                    // தரவுசேமி_பிரிந்து SQL;
                    let (printf, printf_type) = self.get_printf();
                    let msg = format!("Disconnecting from {} database\\n", db_type);
                    let fmt = CString::new(msg).unwrap();
                    let global_str = LLVMBuildGlobalStringPtr(
                        self.builder,
                        fmt.as_ptr(),
                        CString::new("db_disconnect_msg").unwrap().as_ptr(),
                    );
                    let mut args = [global_str];
                    LLVMBuildCall2(
                        self.builder,
                        printf_type,
                        printf,
                        args.as_mut_ptr(),
                        1,
                        CString::new("").unwrap().as_ptr(),
                    );
                }
                // The bound parameter array is matched but unused: this
                // backend does not run the query, it only reports it, so
                // there is nothing to bind the values to. Adding `params` to
                // the AST for the SQLite work left this arm — and the one
                // below — naming too few fields, which made `--features llvm`
                // stop compiling altogether.
                Stmt::DBQuery { query, params: _, result_var } => {
                    // தளம்_வினா "SELECT * FROM table", [params], result;
                    let (printf, printf_type) = self.get_printf();
                    
                    // Extract query string
                    let query_str = match &query {
                        Expr::Variable(s) if s.starts_with('"') && s.ends_with('"') => {
                            s.trim_matches('"').to_string()
                        }
                        _ => "SELECT query".to_string(),
                    };
                    
                    let msg = format!("Executing query: {}\\n", query_str);
                    let fmt = CString::new(msg).unwrap();
                    let global_str = LLVMBuildGlobalStringPtr(
                        self.builder,
                        fmt.as_ptr(),
                        CString::new("db_query_msg").unwrap().as_ptr(),
                    );
                    let mut args = [global_str];
                    LLVMBuildCall2(
                        self.builder,
                        printf_type,
                        printf,
                        args.as_mut_ptr(),
                        1,
                        CString::new("").unwrap().as_ptr(),
                    );
                    
                    // Create a variable to store the placeholder result count.
                    let f64_type = LLVMDoubleTypeInContext(self.context);
                    let alloca = LLVMBuildAlloca(
                        self.builder,
                        f64_type,
                        CString::new(result_var.as_str()).unwrap().as_ptr(),
                    );
                    let val = LLVMConstReal(f64_type, 0.0);
                    LLVMBuildStore(self.builder, val, alloca);
                    self.variables.insert(result_var, alloca);
                }
                Stmt::DBExecute { command, params: _ } => {
                    // தளம்_செய் "CREATE TABLE ...", [params];
                    let (printf, printf_type) = self.get_printf();
                    
                    let cmd_str = match &command {
                        Expr::Variable(s) if s.starts_with('"') && s.ends_with('"') => {
                            s.trim_matches('"').to_string()
                        }
                        _ => "SQL command".to_string(),
                    };
                    
                    let msg = format!("Executing command: {}\\n", cmd_str);
                    let fmt = CString::new(msg).unwrap();
                    let global_str = LLVMBuildGlobalStringPtr(
                        self.builder,
                        fmt.as_ptr(),
                        CString::new("db_exec_msg").unwrap().as_ptr(),
                    );
                    let mut args = [global_str];
                    LLVMBuildCall2(
                        self.builder,
                        printf_type,
                        printf,
                        args.as_mut_ptr(),
                        1,
                        CString::new("").unwrap().as_ptr(),
                    );
                }
                Stmt::DBInsert { table, data } => {
                    // தரவுசேமி_செருக students, "John, 20, A";
                    let (printf, printf_type) = self.get_printf();
                    
                    let data_str = match &data {
                        Expr::Variable(s) if s.starts_with('"') && s.ends_with('"') => {
                            s.trim_matches('"').to_string()
                        }
                        _ => "data".to_string(),
                    };
                    
                    let msg = format!("Inserting into {}: {}\\n", table, data_str);
                    let fmt = CString::new(msg).unwrap();
                    let global_str = LLVMBuildGlobalStringPtr(
                        self.builder,
                        fmt.as_ptr(),
                        CString::new("db_insert_msg").unwrap().as_ptr(),
                    );
                    let mut args = [global_str];
                    LLVMBuildCall2(
                        self.builder,
                        printf_type,
                        printf,
                        args.as_mut_ptr(),
                        1,
                        CString::new("").unwrap().as_ptr(),
                    );
                }
                Stmt::DBUpdate { table, data, condition } => {
                    // தரவுசேமி_புதுப்பி students, "age=21", "name='John'";
                    let (printf, printf_type) = self.get_printf();
                    
                    let data_str = match &data {
                        Expr::Variable(s) if s.starts_with('"') && s.ends_with('"') => {
                            s.trim_matches('"').to_string()
                        }
                        _ => "data".to_string(),
                    };
                    
                    let cond_str = if let Some(cond) = condition {
                        match cond {
                            Expr::Variable(s) if s.starts_with('"') && s.ends_with('"') => {
                                format!(" WHERE {}", s.trim_matches('"'))
                            }
                            _ => String::new(),
                        }
                    } else {
                        String::new()
                    };
                    
                    let msg = format!("Updating {}: SET {}{}\\n", table, data_str, cond_str);
                    let fmt = CString::new(msg).unwrap();
                    let global_str = LLVMBuildGlobalStringPtr(
                        self.builder,
                        fmt.as_ptr(),
                        CString::new("db_update_msg").unwrap().as_ptr(),
                    );
                    let mut args = [global_str];
                    LLVMBuildCall2(
                        self.builder,
                        printf_type,
                        printf,
                        args.as_mut_ptr(),
                        1,
                        CString::new("").unwrap().as_ptr(),
                    );
                }
                Stmt::DBDelete { table, condition } => {
                    // தரவுசேமி_நீக்கு students, "age>25";
                    let (printf, printf_type) = self.get_printf();
                    
                    let cond_str = match &condition {
                        Expr::Variable(s) if s.starts_with('"') && s.ends_with('"') => {
                            s.trim_matches('"').to_string()
                        }
                        _ => "condition".to_string(),
                    };
                    
                    let msg = format!("Deleting from {} WHERE {}\\n", table, cond_str);
                    let fmt = CString::new(msg).unwrap();
                    let global_str = LLVMBuildGlobalStringPtr(
                        self.builder,
                        fmt.as_ptr(),
                        CString::new("db_delete_msg").unwrap().as_ptr(),
                    );
                    let mut args = [global_str];
                    LLVMBuildCall2(
                        self.builder,
                        printf_type,
                        printf,
                        args.as_mut_ptr(),
                        1,
                        CString::new("").unwrap().as_ptr(),
                    );
                }
                Stmt::CreateTable { table, schema } => {
                    // அட்டை_ஆக்கு students, "id INT, name TEXT, age INT";
                    let (printf, printf_type) = self.get_printf();
                    
                    let schema_str = match &schema {
                        Expr::Variable(s) if s.starts_with('"') && s.ends_with('"') => {
                            s.trim_matches('"').to_string()
                        }
                        _ => "schema".to_string(),
                    };
                    
                    let msg = format!("Creating table {} with schema: {}\\n", table, schema_str);
                    let fmt = CString::new(msg).unwrap();
                    let global_str = LLVMBuildGlobalStringPtr(
                        self.builder,
                        fmt.as_ptr(),
                        CString::new("create_table_msg").unwrap().as_ptr(),
                    );
                    let mut args = [global_str];
                    LLVMBuildCall2(
                        self.builder,
                        printf_type,
                        printf,
                        args.as_mut_ptr(),
                        1,
                        CString::new("").unwrap().as_ptr(),
                    );
                }
                Stmt::Select { columns, from_table, where_clause } => {
                    // தேர்வெடு name, age இதனில் students விதி age > 18;
                    let (printf, printf_type) = self.get_printf();
                    
                    let cols = columns.join(", ");
                    let where_str = if let Some(cond) = where_clause {
                        match cond {
                            Expr::Variable(s) if s.starts_with('"') && s.ends_with('"') => {
                                format!(" WHERE {}", s.trim_matches('"'))
                            }
                            _ => String::new(),
                        }
                    } else {
                        String::new()
                    };
                    
                    let msg = format!("SELECT {} FROM {}{}\\n", cols, from_table, where_str);
                    let fmt = CString::new(msg).unwrap();
                    let global_str = LLVMBuildGlobalStringPtr(
                        self.builder,
                        fmt.as_ptr(),
                        CString::new("select_msg").unwrap().as_ptr(),
                    );
                    let mut args = [global_str];
                    LLVMBuildCall2(
                        self.builder,
                        printf_type,
                        printf,
                        args.as_mut_ptr(),
                        1,
                        CString::new("").unwrap().as_ptr(),
                    );
                }
                // REST API Operations
                Stmt::DefineRoute { method, path, handler } => {
                    // வழி GET "/api/users" { handler };
                    let (printf, printf_type) = self.get_printf();
                    
                    let path_str = match &path {
                        Expr::Variable(s) if s.starts_with('"') && s.ends_with('"') => {
                            s.trim_matches('"').to_string()
                        }
                        _ => "/api/route".to_string(),
                    };
                    
                    let msg = format!("Defining route: {} {}\\n", method, path_str);
                    let fmt = CString::new(msg).unwrap();
                    let global_str = LLVMBuildGlobalStringPtr(
                        self.builder,
                        fmt.as_ptr(),
                        CString::new("route_msg").unwrap().as_ptr(),
                    );
                    let mut args = [global_str];
                    LLVMBuildCall2(
                        self.builder,
                        printf_type,
                        printf,
                        args.as_mut_ptr(),
                        1,
                        CString::new("").unwrap().as_ptr(),
                    );
                    
                    // Compile handler statements
                    for stmt in handler {
                        self.compile_stmt(stmt);
                    }
                }
                Stmt::StartServer { host, port } => {
                    // வழங்கி_தொடங்கு "localhost", 8080;
                    let (printf, printf_type) = self.get_printf();
                    
                    let host_str = match &host {
                        Expr::Variable(s) if s.starts_with('"') && s.ends_with('"') => {
                            s.trim_matches('"').to_string()
                        }
                        _ => "localhost".to_string(),
                    };
                    
                    let port_num = match &port {
                        Expr::Number(n) => rust_decimal::prelude::ToPrimitive::to_u32(n).unwrap_or(0),
                        _ => 8080,
                    };
                    
                    let msg = format!("Starting server on {}:{}\\n", host_str, port_num);
                    let fmt = CString::new(msg).unwrap();
                    let global_str = LLVMBuildGlobalStringPtr(
                        self.builder,
                        fmt.as_ptr(),
                        CString::new("server_start_msg").unwrap().as_ptr(),
                    );
                    let mut args = [global_str];
                    LLVMBuildCall2(
                        self.builder,
                        printf_type,
                        printf,
                        args.as_mut_ptr(),
                        1,
                        CString::new("").unwrap().as_ptr(),
                    );
                }
                Stmt::StopServer => {
                    // வழங்கி_நிறுத்து;
                    let (printf, printf_type) = self.get_printf();
                    let msg = "Stopping server\\n";
                    let fmt = CString::new(msg).unwrap();
                    let global_str = LLVMBuildGlobalStringPtr(
                        self.builder,
                        fmt.as_ptr(),
                        CString::new("server_stop_msg").unwrap().as_ptr(),
                    );
                    let mut args = [global_str];
                    LLVMBuildCall2(
                        self.builder,
                        printf_type,
                        printf,
                        args.as_mut_ptr(),
                        1,
                        CString::new("").unwrap().as_ptr(),
                    );
                }
                Stmt::SendResponse { status_code, body, headers: _ } => {
                    // பதில் 200, "Success message";
                    let (printf, printf_type) = self.get_printf();
                    
                    let status = match &status_code {
                        Expr::Number(n) => rust_decimal::prelude::ToPrimitive::to_u32(n).unwrap_or(0),
                        _ => 200,
                    };
                    
                    let body_str = match &body {
                        Expr::Variable(s) if s.starts_with('"') && s.ends_with('"') => {
                            s.trim_matches('"').to_string()
                        }
                        _ => "OK".to_string(),
                    };
                    
                    let msg = format!("Sending response: {} - {}\\n", status, body_str);
                    let fmt = CString::new(msg).unwrap();
                    let global_str = LLVMBuildGlobalStringPtr(
                        self.builder,
                        fmt.as_ptr(),
                        CString::new("response_msg").unwrap().as_ptr(),
                    );
                    let mut args = [global_str];
                    LLVMBuildCall2(
                        self.builder,
                        printf_type,
                        printf,
                        args.as_mut_ptr(),
                        1,
                        CString::new("").unwrap().as_ptr(),
                    );
                }
                Stmt::SendJSON { data, status_code } => {
                    // ஜேசான்_உரை data, 200;
                    let (printf, printf_type) = self.get_printf();
                    
                    let status = if let Some(code) = status_code {
                        match code {
                            Expr::Number(n) => rust_decimal::prelude::ToPrimitive::to_u32(&n).unwrap_or(0),
                            _ => 200,
                        }
                    } else {
                        200
                    };
                    
                    let data_str = match &data {
                        Expr::Variable(s) if s.starts_with('"') && s.ends_with('"') => {
                            s.trim_matches('"').to_string()
                        }
                        _ => "{}".to_string(),
                    };
                    
                    let msg = format!("Sending JSON ({} status): {}\\n", status, data_str);
                    let fmt = CString::new(msg).unwrap();
                    let global_str = LLVMBuildGlobalStringPtr(
                        self.builder,
                        fmt.as_ptr(),
                        CString::new("json_response_msg").unwrap().as_ptr(),
                    );
                    let mut args = [global_str];
                    LLVMBuildCall2(
                        self.builder,
                        printf_type,
                        printf,
                        args.as_mut_ptr(),
                        1,
                        CString::new("").unwrap().as_ptr(),
                    );
                }
                Stmt::GetRequestBody { variable } => {
                    // Store a placeholder value for request body
                    let f64_type = LLVMDoubleTypeInContext(self.context);
                    let alloca = LLVMBuildAlloca(
                        self.builder,
                        f64_type,
                        CString::new(variable.as_str()).unwrap().as_ptr(),
                    );
                    let val = LLVMConstReal(f64_type, 0.0);
                    LLVMBuildStore(self.builder, val, alloca);
                    self.variables.insert(variable, alloca);
                }
                Stmt::GetRequestParam { param_name: _, variable } => {
                    // Store a placeholder value for request param
                    let f64_type = LLVMDoubleTypeInContext(self.context);
                    let alloca = LLVMBuildAlloca(
                        self.builder,
                        f64_type,
                        CString::new(variable.as_str()).unwrap().as_ptr(),
                    );
                    let val = LLVMConstReal(f64_type, 0.0);
                    LLVMBuildStore(self.builder, val, alloca);
                    self.variables.insert(variable, alloca);
                }
                Stmt::GetHeader { header_name: _, variable } => {
                    // Store a placeholder value for header
                    let f64_type = LLVMDoubleTypeInContext(self.context);
                    let alloca = LLVMBuildAlloca(
                        self.builder,
                        f64_type,
                        CString::new(variable.as_str()).unwrap().as_ptr(),
                    );
                    let val = LLVMConstReal(f64_type, 0.0);
                    LLVMBuildStore(self.builder, val, alloca);
                    self.variables.insert(variable, alloca);
                }
                Stmt::SetHeader { header_name, value: _ } => {
                    // Log header setting
                    let (printf, printf_type) = self.get_printf();
                    
                    let header_str = match &header_name {
                        Expr::Variable(s) if s.starts_with('"') && s.ends_with('"') => {
                            s.trim_matches('"').to_string()
                        }
                        _ => "Header".to_string(),
                    };
                    
                    let msg = format!("Setting header: {}\\n", header_str);
                    let fmt = CString::new(msg).unwrap();
                    let global_str = LLVMBuildGlobalStringPtr(
                        self.builder,
                        fmt.as_ptr(),
                        CString::new("header_msg").unwrap().as_ptr(),
                    );
                    let mut args = [global_str];
                    LLVMBuildCall2(
                        self.builder,
                        printf_type,
                        printf,
                        args.as_mut_ptr(),
                        1,
                        CString::new("").unwrap().as_ptr(),
                    );
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
            let f64_type = LLVMDoubleTypeInContext(self.context);
            let mut parameters = vec![f64_type; parameter_count];
            LLVMFunctionType(
                f64_type,
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

            let f64_type = LLVMDoubleTypeInContext(self.context);
            for (index, parameter) in params.iter().enumerate() {
                let pointer = LLVMBuildAlloca(
                    self.builder,
                    f64_type,
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
                LLVMBuildRet(
                    self.builder,
                    LLVMConstReal(f64_type, 0.0),
                );
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
            let f64_type = LLVMDoubleTypeInContext(self.context);
            let element_count = items.len();
            let array_type = LLVMArrayType2(f64_type, element_count.max(1) as u64);
            let pointer = LLVMBuildAlloca(
                self.builder,
                array_type,
                CString::new("array").unwrap().as_ptr(),
            );

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
            unsafe {
                let pointer = LLVMBuildAlloca(
                    self.builder,
                    LLVMDoubleTypeInContext(self.context),
                    CString::new(format!("record_{}", field)).unwrap().as_ptr(),
                );
                LLVMBuildStore(self.builder, self.compile_expr(value), pointer);
                record.insert(field.clone(), pointer);
            }
        }
        record
    }

    fn compile_array_index(&mut self, index: &Expr) -> LLVMValueRef {
        unsafe {
            LLVMBuildFPToSI(
                self.builder,
                self.compile_expr(index),
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
                LLVMDoubleTypeInContext(self.context),
                element,
                CString::new("each_value").unwrap().as_ptr(),
            );
            let variable_pointer = self.variables.entry(variable.to_string()).or_insert_with(|| {
                LLVMBuildAlloca(
                    self.builder,
                    LLVMDoubleTypeInContext(self.context),
                    CString::new(variable).unwrap().as_ptr(),
                )
            });
            LLVMBuildStore(self.builder, value, *variable_pointer);

            for statement in body {
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

            LLVMPositionBuilderAtEnd(self.builder, after_block);
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
                    let f64_type = LLVMDoubleTypeInContext(self.context);
                    LLVMConstReal(f64_type, rust_decimal::prelude::ToPrimitive::to_f64(n).unwrap_or(0.0))
                }
                Expr::Variable(name) => {
                    if let Some(var_ptr) = self.variables.get(name) {
                        let f64_type = LLVMDoubleTypeInContext(self.context);
                        LLVMBuildLoad2(
                            self.builder,
                            f64_type,
                            *var_ptr,
                            CString::new("load").unwrap().as_ptr(),
                        )
                    } else {
                        // Variable not found, return 0.0
                        let f64_type = LLVMDoubleTypeInContext(self.context);
                        LLVMConstReal(f64_type, 0.0)
                    }
                }
                Expr::BinaryOp { op, left, right } => {
                    let lhs = self.compile_expr(left);
                    let rhs = self.compile_expr(right);
                    
                    match op.as_str() {
                        "+" => LLVMBuildFAdd(self.builder, lhs, rhs, CString::new("add").unwrap().as_ptr()),
                        "-" => LLVMBuildFSub(self.builder, lhs, rhs, CString::new("sub").unwrap().as_ptr()),
                        "*" => LLVMBuildFMul(self.builder, lhs, rhs, CString::new("mul").unwrap().as_ptr()),
                        "/" => LLVMBuildFDiv(self.builder, lhs, rhs, CString::new("div").unwrap().as_ptr()),
                        _ => {
                            let f64_type = LLVMDoubleTypeInContext(self.context);
                            LLVMConstReal(f64_type, 0.0)
                        }
                    }
                }
                Expr::Comparison { left, op, right } => {
                    let lhs = self.compile_expr(left);
                    let rhs = self.compile_expr(right);
                    
                    let pred = match op.as_str() {
                        ">" => LLVMRealPredicate::LLVMRealOGT,
                        "<" => LLVMRealPredicate::LLVMRealOLT,
                        ">=" => LLVMRealPredicate::LLVMRealOGE,
                        "<=" => LLVMRealPredicate::LLVMRealOLE,
                        "==" => LLVMRealPredicate::LLVMRealOEQ,
                        "!=" => LLVMRealPredicate::LLVMRealONE,
                        _ => LLVMRealPredicate::LLVMRealOEQ,
                    };
                    
                    LLVMBuildFCmp(self.builder, pred, lhs, rhs, CString::new("cmp").unwrap().as_ptr())
                }
                Expr::Call { name, args } => {
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
                    } else {
                        self.unsupported.push(format!("function call {}", name));
                        LLVMConstReal(LLVMDoubleTypeInContext(self.context), 0.0)
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
                            LLVMDoubleTypeInContext(self.context),
                            element,
                            CString::new("array_value").unwrap().as_ptr(),
                        )
                    } else {
                        self.unsupported.push("array index".to_string());
                        LLVMConstReal(LLVMDoubleTypeInContext(self.context), 0.0)
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
                            LLVMDoubleTypeInContext(self.context),
                            pointer,
                            CString::new("record_value").unwrap().as_ptr(),
                        )
                    } else {
                        self.unsupported.push(format!("record field {}", name));
                        LLVMConstReal(LLVMDoubleTypeInContext(self.context), 0.0)
                    }
                }
                Expr::Concat { left, right: _ } => {
                    // For concat in expression context, just evaluate left side
                    // (concat is mainly for print statements)
                    self.compile_expr(left)
                }
                other => {
                    // Same reasoning: yielding 0.0 for an expression this
                    // backend cannot build would silently change the answer.
                    self.unsupported
                        .push(format!("expression {}", Self::expr_label(other)));
                    let f64_type = LLVMDoubleTypeInContext(self.context);
                    LLVMConstReal(f64_type, 0.0)
                }
            }
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
        unsafe {
            let (printf, printf_type) = self.get_printf();
            
            match expr {
                Expr::Variable(s) if s.starts_with('"') && s.ends_with('"') => {
                    let clean = s.trim_matches('"');
                    let fmt = CString::new(clean).unwrap();
                    let global_str = LLVMBuildGlobalStringPtr(
                        self.builder,
                        fmt.as_ptr(),
                        CString::new("str").unwrap().as_ptr(),
                    );
                    let mut args = [global_str];
                    LLVMBuildCall2(
                        self.builder,
                        printf_type,
                        printf,
                        args.as_mut_ptr(),
                        1,
                        CString::new("").unwrap().as_ptr(),
                    );
                }
                Expr::Concat { left, right } => {
                    self.print_concat_part(left);
                    self.print_concat_part(right);
                }
                _ => {
                    let val = self.compile_expr(expr);
                    let fmt = CString::new("%.0f").unwrap();
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
                        2,
                        CString::new("").unwrap().as_ptr(),
                    );
                }
            }
        }
    }

    /// Compile a comparison expression (helper for If/Loop)
    fn compile_comparison(&mut self, expr: &Expr) -> LLVMValueRef {
        self.compile_expr(expr)
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
