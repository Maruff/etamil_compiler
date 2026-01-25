/// LLVM Code Generator for eTamil using llvm-sys (LLVM 18 compatible)
use llvm_sys::prelude::*;
use llvm_sys::core::*;
use llvm_sys::LLVMRealPredicate;
use std::ffi::CString;
use std::collections::HashMap;
use std::ptr;
use crate::parser::{Expr, Stmt};
use crate::fileio::csv_handler::FileIOHandler;



pub struct Compiler {
    context: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    function: LLVMValueRef,
    variables: HashMap<String, LLVMValueRef>, // Variable name -> alloca pointer
}


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
            }
        }
    }

    /// Compile the entire AST
    pub fn compile(&mut self, statements: Vec<Stmt>) {
        unsafe {
            // Compile each statement
            for stmt in statements {
                self.compile_stmt(stmt);
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
                Stmt::Assign { name, value } => {
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
                Stmt::DBQuery { query, result_var } => {
                    // தரவுசேமி_கேள்வி "SELECT * FROM table", result;
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
                    
                    // If result_var is specified, create a variable to store result count
                    if let Some(var_name) = result_var {
                        let f64_type = LLVMDoubleTypeInContext(self.context);
                        let alloca = LLVMBuildAlloca(
                            self.builder,
                            f64_type,
                            CString::new(var_name.as_str()).unwrap().as_ptr(),
                        );
                        // Store a placeholder value (e.g., number of rows)
                        let val = LLVMConstReal(f64_type, 0.0);
                        LLVMBuildStore(self.builder, val, alloca);
                        self.variables.insert(var_name, alloca);
                    }
                }
                Stmt::DBExecute { command } => {
                    // தரவுசேமி_செய் "CREATE TABLE ...";
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
                        Expr::Number(n) => *n as u32,
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
                        Expr::Number(n) => *n as u32,
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
                            Expr::Number(n) => n as u32,
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
            }
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
    fn compile_expr(&self, expr: &Expr) -> LLVMValueRef {
        unsafe {
            match expr {
                Expr::Number(n) => {
                    let f64_type = LLVMDoubleTypeInContext(self.context);
                    LLVMConstReal(f64_type, *n)
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
                Expr::Concat { left, right: _ } => {
                    // For concat in expression context, just evaluate left side
                    // (concat is mainly for print statements)
                    self.compile_expr(left)
                }
            }
        }
    }

    /// Helper to print concatenated expressions
    fn print_concat(&self, expr: &Expr) {
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

    fn print_concat_part(&self, expr: &Expr) {
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
    fn compile_comparison(&self, expr: &Expr) -> LLVMValueRef {
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

impl Drop for Compiler {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeBuilder(self.builder);
            LLVMDisposeModule(self.module);
            LLVMContextDispose(self.context);
        }
    }
}
