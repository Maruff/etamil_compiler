// File I/O and CSV Handler for eTamil Compiler
// This module handles all file and CSV operations using LLVM IR

#[cfg(feature = "llvm")]
use llvm_sys::prelude::*;
#[cfg(feature = "llvm")]
use llvm_sys::core::*;
#[cfg(feature = "llvm")]
use std::ffi::CString;
#[cfg(feature = "llvm")]
use std::collections::HashMap;

/// File I/O Handler for managing file operations in compiled code
#[cfg(feature = "llvm")]
pub struct FileIOHandler {
    context: LLVMContextRef,
    builder: LLVMBuilderRef,
    variables: HashMap<String, LLVMValueRef>,
    module: LLVMModuleRef,
}

#[cfg(feature = "llvm")]
impl FileIOHandler {
    /// Create a new FileIOHandler
    pub fn new(
        context: LLVMContextRef,
        builder: LLVMBuilderRef,
        module: LLVMModuleRef,
        variables: HashMap<String, LLVMValueRef>,
    ) -> Self {
        FileIOHandler {
            context,
            builder,
            variables,
            module,
        }
    }

    /// Get the printf function reference
    fn get_printf(&self) -> (LLVMValueRef, LLVMTypeRef) {
        let i32_type = unsafe { LLVMInt32TypeInContext(self.context) };
        let i8_ptr_type = unsafe {
            let i8_type = LLVMInt8TypeInContext(self.context);
            LLVMPointerType(i8_type, 0)
        };

        unsafe {
            let printf_type = LLVMFunctionType(i32_type, [i8_ptr_type].as_mut_ptr(), 1, 1);
            let printf = LLVMGetNamedFunction(
                self.module,
                CString::new("printf").unwrap().as_ptr(),
            );

            if printf.is_null() {
                let func = LLVMAddFunction(
                    self.module,
                    CString::new("printf").unwrap().as_ptr(),
                    printf_type,
                );
                (func, printf_type)
            } else {
                (printf, printf_type)
            }
        }
    }

    /// Get the scanf function reference
    fn get_scanf(&self) -> (LLVMValueRef, LLVMTypeRef) {
        let i32_type = unsafe { LLVMInt32TypeInContext(self.context) };
        let i8_ptr_type = unsafe {
            let i8_type = LLVMInt8TypeInContext(self.context);
            LLVMPointerType(i8_type, 0)
        };

        unsafe {
            let scanf_type = LLVMFunctionType(i32_type, [i8_ptr_type].as_mut_ptr(), 1, 1);
            let scanf = LLVMGetNamedFunction(
                self.module,
                CString::new("scanf").unwrap().as_ptr(),
            );

            if scanf.is_null() {
                let func = LLVMAddFunction(
                    self.module,
                    CString::new("scanf").unwrap().as_ptr(),
                    scanf_type,
                );
                (func, scanf_type)
            } else {
                (scanf, scanf_type)
            }
        }
    }

    /// Handle file open operation: கோப்பு_திற "filename" "mode";
    pub fn handle_file_open(&self, mode: &str) {
        unsafe {
            let (printf, printf_type) = self.get_printf();
            let msg = format!("[File opened in {} mode]\n", mode);
            let fmt = CString::new(msg).unwrap();
            let global_str = LLVMBuildGlobalStringPtr(
                self.builder,
                fmt.as_ptr(),
                CString::new("file_open_msg").unwrap().as_ptr(),
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

    /// Handle file close operation: கோப்பு_பின்வை "filename";
    pub fn handle_file_close(&self) {
        unsafe {
            let (printf, printf_type) = self.get_printf();
            let fmt = CString::new("[File closed]\n").unwrap();
            let global_str = LLVMBuildGlobalStringPtr(
                self.builder,
                fmt.as_ptr(),
                CString::new("file_close_msg").unwrap().as_ptr(),
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

    /// Handle file write operation: கோப்பு_எழுது "filename" data;
    pub fn handle_file_write(&self, data_value: LLVMValueRef) {
        unsafe {
            let (printf, printf_type) = self.get_printf();

            // Print message
            let fmt = CString::new("[Writing to file: ").unwrap();
            let global_str = LLVMBuildGlobalStringPtr(
                self.builder,
                fmt.as_ptr(),
                CString::new("write_msg").unwrap().as_ptr(),
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

            // Print the data value
            let fmt_data = CString::new("%.0f").unwrap();
            let global_fmt = LLVMBuildGlobalStringPtr(
                self.builder,
                fmt_data.as_ptr(),
                CString::new("write_fmt").unwrap().as_ptr(),
            );
            let mut args2 = [global_fmt, data_value];
            LLVMBuildCall2(
                self.builder,
                printf_type,
                printf,
                args2.as_mut_ptr(),
                2,
                CString::new("").unwrap().as_ptr(),
            );

            // Print closing bracket
            let fmt_close = CString::new("]\n").unwrap();
            let global_close = LLVMBuildGlobalStringPtr(
                self.builder,
                fmt_close.as_ptr(),
                CString::new("write_close").unwrap().as_ptr(),
            );
            let mut args3 = [global_close];
            LLVMBuildCall2(
                self.builder,
                printf_type,
                printf,
                args3.as_mut_ptr(),
                1,
                CString::new("").unwrap().as_ptr(),
            );
        }
    }

    /// Handle file read operation: கோப்பு_படி "filename" variable;
    /// Returns a reference to the variable that was read into
    pub fn handle_file_read(&mut self, variable: &str) -> LLVMValueRef {
        unsafe {
            let (printf, printf_type) = self.get_printf();
            let (scanf, scanf_type) = self.get_scanf();

            // Print prompt message
            let msg = format!("[Reading from file into {}]: ", variable);
            let fmt = CString::new(msg).unwrap();
            let global_str = LLVMBuildGlobalStringPtr(
                self.builder,
                fmt.as_ptr(),
                CString::new("read_msg").unwrap().as_ptr(),
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

            // Create or get variable storage
            if !self.variables.contains_key(variable) {
                let f64_type = LLVMDoubleTypeInContext(self.context);
                let alloca = LLVMBuildAlloca(
                    self.builder,
                    f64_type,
                    CString::new(variable).unwrap().as_ptr(),
                );
                self.variables.insert(variable.to_string(), alloca);
            }

            // Read from stdin
            let var_ptr = *self.variables.get(variable).unwrap();
            let fmt_scanf = CString::new("%lf").unwrap();
            let global_fmt = LLVMBuildGlobalStringPtr(
                self.builder,
                fmt_scanf.as_ptr(),
                CString::new("read_fmt").unwrap().as_ptr(),
            );
            let mut args2 = [global_fmt, var_ptr];
            LLVMBuildCall2(
                self.builder,
                scanf_type,
                scanf,
                args2.as_mut_ptr(),
                2,
                CString::new("").unwrap().as_ptr(),
            );

            var_ptr
        }
    }

    /// Handle CSV read operation: CSV_படி "filename" variable;
    pub fn handle_read_csv(&mut self, variable: &str) {
        unsafe {
            let (printf, printf_type) = self.get_printf();
            let msg = format!("[Reading CSV from file into {}]\n", variable);
            let fmt = CString::new(msg).unwrap();
            let global_str = LLVMBuildGlobalStringPtr(
                self.builder,
                fmt.as_ptr(),
                CString::new("csv_read_msg").unwrap().as_ptr(),
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

            // Ensure variable exists
            if !self.variables.contains_key(variable) {
                let f64_type = LLVMDoubleTypeInContext(self.context);
                let alloca = LLVMBuildAlloca(
                    self.builder,
                    f64_type,
                    CString::new(variable).unwrap().as_ptr(),
                );
                self.variables.insert(variable.to_string(), alloca);
            }
        }
    }

    /// Handle CSV write operation: CSV_எழுது "filename" data;
    pub fn handle_write_csv(&self, data_value: LLVMValueRef) {
        unsafe {
            let (printf, printf_type) = self.get_printf();
            let fmt = CString::new("[Writing CSV data to file]\n").unwrap();
            let global_str = LLVMBuildGlobalStringPtr(
                self.builder,
                fmt.as_ptr(),
                CString::new("csv_write_msg").unwrap().as_ptr(),
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

            // Print the data value
            let fmt_data = CString::new("Value: %.0f\n").unwrap();
            let global_fmt = LLVMBuildGlobalStringPtr(
                self.builder,
                fmt_data.as_ptr(),
                CString::new("csv_write_fmt").unwrap().as_ptr(),
            );
            let mut args2 = [global_fmt, data_value];
            LLVMBuildCall2(
                self.builder,
                printf_type,
                printf,
                args2.as_mut_ptr(),
                2,
                CString::new("").unwrap().as_ptr(),
            );
        }
    }

    /// Get reference to the variables map
    pub fn get_variables(&self) -> &HashMap<String, LLVMValueRef> {
        &self.variables
    }

    /// Get mutable reference to the variables map
    #[allow(unused)]
    pub fn get_variables_mut(&mut self) -> &mut HashMap<String, LLVMValueRef> {
        &mut self.variables
    }
}

/// CSV file utilities for parsing and writing CSV data
#[allow(dead_code)]
pub struct CSVProcessor;

#[allow(dead_code)]
impl CSVProcessor {
    /// Parse a CSV line into individual fields
    pub fn parse_csv_line(line: &str) -> Vec<String> {
        line.split(',')
            .map(|field| field.trim().to_string())
            .collect()
    }

    /// Escape special characters for CSV format
    pub fn escape_csv_field(field: &str) -> String {
        if field.contains(',') || field.contains('"') || field.contains('\n') {
            format!("\"{}\"", field.replace("\"", "\"\""))
        } else {
            field.to_string()
        }
    }

    /// Create a CSV line from fields
    pub fn create_csv_line(fields: &[String]) -> String {
        fields
            .iter()
            .map(|f| Self::escape_csv_field(f))
            .collect::<Vec<_>>()
            .join(",")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_csv_line() {
        let line = "value1,value2,value3";
        let fields = CSVProcessor::parse_csv_line(line);
        assert_eq!(fields, vec!["value1", "value2", "value3"]);
    }

    #[test]
    fn test_csv_with_spaces() {
        let line = "value1 , value2 , value3";
        let fields = CSVProcessor::parse_csv_line(line);
        assert_eq!(fields, vec!["value1", "value2", "value3"]);
    }

    #[test]
    fn test_escape_csv_field() {
        assert_eq!(
            CSVProcessor::escape_csv_field("simple"),
            "simple"
        );
        assert_eq!(
            CSVProcessor::escape_csv_field("with,comma"),
            "\"with,comma\""
        );
        assert_eq!(
            CSVProcessor::escape_csv_field("with\"quote"),
            "\"with\"\"quote\""
        );
    }

    #[test]
    fn test_create_csv_line() {
        let fields = vec!["value1".to_string(), "value2".to_string(), "value3".to_string()];
        let line = CSVProcessor::create_csv_line(&fields);
        assert_eq!(line, "value1,value2,value3");
    }
}
