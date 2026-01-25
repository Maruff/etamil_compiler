// Request Handler Module

use crate::http::{HttpRequest, HttpResponse};
use crate::parser::Stmt;
use crate::vm::VM;

pub struct RequestHandler;

impl RequestHandler {
    /// Execute a handler for a request
    pub fn execute(
        request: &HttpRequest,
        handler_stmts: &[Stmt],
    ) -> Result<HttpResponse, String> {
        let mut vm = VM::new();

        // Store request data in VM variables
        vm.variables.insert(
            "request_method".to_string(),
            crate::vm::Value::String(request.method.clone()),
        );
        vm.variables.insert(
            "request_path".to_string(),
            crate::vm::Value::String(request.path.clone()),
        );

        // Store query parameters
        let mut params = std::collections::HashMap::new();
        for (key, value) in &request.query_params {
            params.insert(
                key.clone(),
                crate::vm::Value::String(value.clone()),
            );
        }
        vm.variables.insert(
            "query_params".to_string(),
            crate::vm::Value::Map(params),
        );

        // Store headers
        let mut headers = std::collections::HashMap::new();
        for (key, value) in &request.headers {
            headers.insert(
                key.clone(),
                crate::vm::Value::String(value.clone()),
            );
        }
        vm.variables.insert(
            "headers".to_string(),
            crate::vm::Value::Map(headers),
        );

        // Compile and execute handler
        let bytecode =
            crate::vm::bytecode::compiler::BytecodeCompiler::compile_statements(handler_stmts.to_vec());

        vm.execute(bytecode)
            .map_err(|e| format!("Handler execution error: {}", e))?;

        // Get response from VM
        let status = vm.variables
            .get("response_status")
            .and_then(|v| match v {
                crate::vm::Value::Number(n) => Some(*n as u16),
                _ => None,
            })
            .unwrap_or(200);

        let body = vm.variables
            .get("response_body")
            .map(|v| v.to_string())
            .unwrap_or_else(|| "Handler executed successfully".to_string());

        Ok(HttpResponse::success(status, body))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler_execution() {
        let request = HttpRequest {
            method: "GET".to_string(),
            path: "/api/test".to_string(),
            version: "HTTP/1.1".to_string(),
            headers: std::collections::HashMap::new(),
            query_params: std::collections::HashMap::new(),
            body: String::new(),
        };

        let handler_stmts = vec![];

        let result = RequestHandler::execute(&request, &handler_stmts);
        assert!(result.is_ok());
    }
}
