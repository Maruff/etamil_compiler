// Request Handler Module

use std::collections::HashMap;

use crate::http::{HttpRequest, HttpResponse};
use crate::parser::Stmt;
use crate::vm::{VM, Value};

/// Make one request readable from eTamil.
///
/// Every variable a handler can see is set here, in one place. The server and
/// this module each used to bind the request themselves, and the two copies
/// drifted: the path-parameter path bound neither the query string nor the
/// headers, and neither of them ever bound the body at all, so a POST payload
/// was unreachable from the language.
///
/// The names are deliberately plain — `query_params["id"]` reads the same in
/// Tamil or romanized source, because they are data, not keywords.
pub fn bind_request(vm: &mut VM, request: &HttpRequest, path_params: &HashMap<String, String>) {
    vm.variables.insert(
        "request_method".to_string(),
        Value::String(request.method.clone()),
    );
    vm.variables.insert(
        "request_path".to_string(),
        Value::String(request.path.clone()),
    );
    // The body arrives as text. Parsing it — as JSON or anything else — is
    // the language's own job; see nUlakam/jEcAZ.qmz.
    vm.variables.insert(
        "request_body".to_string(),
        Value::String(request.body.clone()),
    );

    vm.variables.insert(
        "query_params".to_string(),
        Value::Map(as_value_map(&request.query_params)),
    );
    vm.variables.insert(
        "headers".to_string(),
        Value::Map(as_value_map(&request.headers)),
    );
    vm.variables.insert(
        "path_params".to_string(),
        Value::Map(as_value_map(path_params)),
    );

    // Also as param_<name>, which is how path parameters have always been
    // exposed; programs written against that keep working.
    for (name, value) in path_params {
        vm.variables.insert(
            format!("param_{}", name),
            Value::String(value.clone()),
        );
    }
}

fn as_value_map(source: &HashMap<String, String>) -> HashMap<String, Value> {
    source
        .iter()
        .map(|(key, value)| (key.clone(), Value::String(value.clone())))
        .collect()
}

/// Read the response a handler left behind.
///
/// `பதில்` writes `response_status`, `response_body` and `response_headers`
/// into the VM's globals; a handler that sets none of them still answers 200,
/// so a route that only prints is not an error.
pub fn response_from(vm: &VM) -> HttpResponse {
    let status = vm
        .variables
        .get("response_status")
        .and_then(|value| match value {
            Value::Number(n) => rust_decimal::prelude::ToPrimitive::to_u16(n),
            _ => None,
        })
        .unwrap_or(200);

    let body = vm
        .variables
        .get("response_body")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "Handler executed successfully".to_string());

    let mut response = HttpResponse::success(status, body);

    // A handler's own headers override the defaults, which is what lets a
    // route serve HTML or a CSV export instead of the JSON this server
    // otherwise assumes.
    if let Some(Value::Map(fields)) = vm.variables.get("response_headers") {
        for (name, value) in fields {
            response.set_header(name, &value.to_string());
        }
    }

    // Content-Length always describes the body actually being sent, never a
    // value the handler supplied.
    let length = response.body.len().to_string();
    response.set_header("Content-Length", &length);

    response
}

pub struct RequestHandler;

impl RequestHandler {
    /// Execute a handler for a request
    pub fn execute(
        request: &HttpRequest,
        handler_stmts: &[Stmt],
    ) -> Result<HttpResponse, String> {
        let mut vm = VM::new();
        bind_request(&mut vm, request, &HashMap::new());

        // Compile and execute handler
        let bytecode =
            crate::vm::bytecode::compiler::BytecodeCompiler::compile_statements(handler_stmts.to_vec());

        vm.execute(bytecode)
            .map_err(|e| format!("Handler execution error: {}", e))?;

        Ok(response_from(&vm))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(method: &str, path: &str) -> HttpRequest {
        HttpRequest {
            method: method.to_string(),
            path: path.to_string(),
            version: "HTTP/1.1".to_string(),
            headers: HashMap::new(),
            query_params: HashMap::new(),
            body: String::new(),
        }
    }

    fn text_var(vm: &VM, name: &str) -> String {
        vm.variables
            .get(name)
            .unwrap_or_else(|| panic!("'{}' was never bound", name))
            .to_string()
    }

    #[test]
    fn test_handler_execution() {
        let result = RequestHandler::execute(&request("GET", "/api/test"), &[]);
        assert!(result.is_ok());
    }

    // The whole point of the change: a POST body has to be readable from
    // eTamil. It was parsed off the socket and then dropped.
    #[test]
    fn the_body_is_bound_for_the_handler_to_read() {
        let mut req = request("POST", "/api/pativu");
        req.body = r#"{"qokY":1500}"#.to_string();

        let mut vm = VM::new();
        bind_request(&mut vm, &req, &HashMap::new());

        assert_eq!(text_var(&vm, "request_body"), r#"{"qokY":1500}"#);
    }

    // Regression: the path-parameter branch bound neither of these.
    #[test]
    fn path_parameters_do_not_cost_the_query_string_or_headers() {
        let mut req = request("GET", "/api/kaNakku/1000");
        req.query_params.insert("from".to_string(), "2026-04-01".to_string());
        req.headers.insert("authorization".to_string(), "Bearer t".to_string());

        let mut params = HashMap::new();
        params.insert("id".to_string(), "1000".to_string());

        let mut vm = VM::new();
        bind_request(&mut vm, &req, &params);

        assert_eq!(text_var(&vm, "param_id"), "1000");
        match vm.variables.get("path_params") {
            Some(Value::Map(fields)) => {
                assert_eq!(fields.get("id").map(|v| v.to_string()), Some("1000".to_string()));
            }
            other => panic!("path_params should be a record, got {:?}", other),
        }
        match vm.variables.get("query_params") {
            Some(Value::Map(fields)) => {
                assert_eq!(
                    fields.get("from").map(|v| v.to_string()),
                    Some("2026-04-01".to_string())
                );
            }
            other => panic!("query_params should be a record, got {:?}", other),
        }
        match vm.variables.get("headers") {
            Some(Value::Map(fields)) => {
                assert_eq!(
                    fields.get("authorization").map(|v| v.to_string()),
                    Some("Bearer t".to_string())
                );
            }
            other => panic!("headers should be a record, got {:?}", other),
        }
    }

    #[test]
    fn a_handler_that_sets_nothing_still_answers_200() {
        let vm = VM::new();
        assert_eq!(response_from(&vm).status_code, 200);
    }
}
