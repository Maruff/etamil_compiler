// API Code Generator Module
// Handles LLVM IR generation for REST API operations

use llvm_sys::prelude::*;
use llvm_sys::core::*;
use std::ffi::CString;

/// API Code Generator for LLVM IR generation
pub struct APICodegen;

impl APICodegen {
    /// Generate code for route definition
    pub fn emit_route_definition(
        builder: LLVMBuilderRef,
        _context: LLVMContextRef,
        module: LLVMModuleRef,
        method: &str,
        path: &str,
    ) {
        unsafe {
            let (printf, printf_type) = Self::get_printf(module, _context);
            
            let msg = format!("Defining route: {} {}\\n", method, path);
            let fmt = CString::new(msg).unwrap();
            let global_str = LLVMBuildGlobalStringPtr(
                builder,
                fmt.as_ptr(),
                CString::new("route_msg").unwrap().as_ptr(),
            );
            let mut args = [global_str];
            LLVMBuildCall2(
                builder,
                printf_type,
                printf,
                args.as_mut_ptr(),
                1,
                CString::new("").unwrap().as_ptr(),
            );
        }
    }
    
    /// Generate code for server startup
    pub fn emit_server_start(
        builder: LLVMBuilderRef,
        _context: LLVMContextRef,
        module: LLVMModuleRef,
        host: &str,
        port: u16,
    ) {
        unsafe {
            let (printf, printf_type) = Self::get_printf(module, _context);
            
            let msg = format!("Starting server on {}:{}\\n", host, port);
            let fmt = CString::new(msg).unwrap();
            let global_str = LLVMBuildGlobalStringPtr(
                builder,
                fmt.as_ptr(),
                CString::new("server_start_msg").unwrap().as_ptr(),
            );
            let mut args = [global_str];
            LLVMBuildCall2(
                builder,
                printf_type,
                printf,
                args.as_mut_ptr(),
                1,
                CString::new("").unwrap().as_ptr(),
            );
        }
    }
    
    /// Generate code for server shutdown
    pub fn emit_server_stop(
        builder: LLVMBuilderRef,
        _context: LLVMContextRef,
        module: LLVMModuleRef,
    ) {
        unsafe {
            let (printf, printf_type) = Self::get_printf(module, _context);
            let msg = "Stopping server\\n";
            let fmt = CString::new(msg).unwrap();
            let global_str = LLVMBuildGlobalStringPtr(
                builder,
                fmt.as_ptr(),
                CString::new("server_stop_msg").unwrap().as_ptr(),
            );
            let mut args = [global_str];
            LLVMBuildCall2(
                builder,
                printf_type,
                printf,
                args.as_mut_ptr(),
                1,
                CString::new("").unwrap().as_ptr(),
            );
        }
    }
    
    /// Generate code for JSON response
    pub fn emit_json_response(
        builder: LLVMBuilderRef,
        _context: LLVMContextRef,
        module: LLVMModuleRef,
        data: &str,
        status_code: u32,
    ) {
        unsafe {
            let (printf, printf_type) = Self::get_printf(module, _context);
            
            let msg = format!("Sending JSON ({} status): {}\\n", status_code, data);
            let fmt = CString::new(msg).unwrap();
            let global_str = LLVMBuildGlobalStringPtr(
                builder,
                fmt.as_ptr(),
                CString::new("json_response_msg").unwrap().as_ptr(),
            );
            let mut args = [global_str];
            LLVMBuildCall2(
                builder,
                printf_type,
                printf,
                args.as_mut_ptr(),
                1,
                CString::new("").unwrap().as_ptr(),
            );
        }
    }
    
    /// Generate code for text response
    pub fn emit_text_response(
        builder: LLVMBuilderRef,
        _context: LLVMContextRef,
        module: LLVMModuleRef,
        status_code: u32,
        body: &str,
    ) {
        unsafe {
            let (printf, printf_type) = Self::get_printf(module, _context);
            
            let msg = format!("Sending response: {} - {}\\n", status_code, body);
            let fmt = CString::new(msg).unwrap();
            let global_str = LLVMBuildGlobalStringPtr(
                builder,
                fmt.as_ptr(),
                CString::new("response_msg").unwrap().as_ptr(),
            );
            let mut args = [global_str];
            LLVMBuildCall2(
                builder,
                printf_type,
                printf,
                args.as_mut_ptr(),
                1,
                CString::new("").unwrap().as_ptr(),
            );
        }
    }
    
    /// Get or declare printf function
    fn get_printf(module: LLVMModuleRef, context: LLVMContextRef) -> (LLVMValueRef, LLVMTypeRef) {
        unsafe {
            let name = CString::new("printf").unwrap();
            let i8_ptr = LLVMPointerType(LLVMInt8TypeInContext(context), 0);
            let mut param_types = [i8_ptr];
            let fn_type = LLVMFunctionType(
                LLVMInt32TypeInContext(context),
                param_types.as_mut_ptr(),
                param_types.len() as u32,
                1, // varargs
            );
            
            let existing = LLVMGetNamedFunction(module, name.as_ptr());
            if !existing.is_null() {
                return (existing, fn_type);
            }
            
            let func = LLVMAddFunction(module, name.as_ptr(), fn_type);
            (func, fn_type)
        }
    }
}
