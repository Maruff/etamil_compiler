// Phase 2: Async HTTP Handler for eTamil
// Integrates with existing Tokio runtime and connection pooling

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AsyncRequestContext {
    pub method: String,
    pub path: String,
    pub query_params: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub body: String,
}

pub struct AsyncHandlerResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

/// Async request handler that executes eTamil code
pub async fn handle_request_async(
    context: AsyncRequestContext,
    etamil_code: String,
) -> AsyncHandlerResponse {
    // Use tokio::task::spawn_blocking to run eTamil (synchronous code)
    // in a worker thread without blocking the async runtime
    let result = tokio::task::spawn_blocking(move || {
        execute_etamil_code(&etamil_code, &context)
    })
    .await;

    match result {
        Ok(response) => response,
        Err(_) => AsyncHandlerResponse {
            status: 500,
            headers: HashMap::from([
                ("Content-Type".to_string(), "application/json".to_string()),
            ]),
            body: r#"{"error": "Task execution failed"}"#.to_string(),
        },
    }
}

/// Execute eTamil code synchronously (in blocking thread)
fn execute_etamil_code(
    code: &str,
    context: &AsyncRequestContext,
) -> AsyncHandlerResponse {
    // This is synchronous but runs in a thread pool
    // so it doesn't block the async runtime

    // Inject request context as eTamil variables
    let mut etamil_with_context = format!(
        r#"
request_method = "{}"
request_path = "{}"
response_status = 200
response_body = "OK"
{}
"#,
        context.method, context.path, code
    );

    // Add query parameters as variables
    for (key, value) in &context.query_params {
        etamil_with_context.push_str(&format!(
            r#"{} = "{}"
"#,
            key, value
        ));
    }

    // Try to compile and execute
    match crate::vm::bytecode::compiler::Compiler::new().compile(&etamil_with_context) {
        Ok(bytecode) => {
            let mut vm = crate::vm::VM::new();

            // Set up variable access
            vm.variables.insert("request_method".to_string(), crate::vm::Value::Number(0.0));
            vm.variables.insert("request_path".to_string(), crate::vm::Value::Number(0.0));
            vm.variables.insert("response_status".to_string(), crate::vm::Value::Number(200.0));
            vm.variables.insert("response_body".to_string(), crate::vm::Value::Number(0.0));

            match vm.execute(&bytecode) {
                Ok(_) => {
                    // Extract response from VM
                    let status = vm.variables
                        .get("response_status")
                        .and_then(|v| match v {
                            crate::vm::Value::Number(n) => Some(*n as u16),
                            _ => None,
                        })
                        .unwrap_or(200);

                    let body = vm.variables
                        .get("response_body")
                        .map(|v| format!("{:?}", v))
                        .unwrap_or_else(|| "Handler executed successfully".to_string());

                    AsyncHandlerResponse {
                        status,
                        headers: HashMap::from([
                            ("Content-Type".to_string(), "application/json".to_string()),
                            ("Access-Control-Allow-Origin".to_string(), "*".to_string()),
                        ]),
                        body: format!(r#"{{"message": "{}"}}"#, body),
                    }
                }
                Err(_) => AsyncHandlerResponse {
                    status: 500,
                    headers: HashMap::from([
                        ("Content-Type".to_string(), "application/json".to_string()),
                    ]),
                    body: r#"{"error": "Execution failed"}"#.to_string(),
                },
            }
        }
        Err(_) => AsyncHandlerResponse {
            status: 400,
            headers: HashMap::from([
                ("Content-Type".to_string(), "application/json".to_string()),
            ]),
            body: r#"{"error": "Compilation failed"}"#.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_request_handling() {
        let context = AsyncRequestContext {
            method: "GET".to_string(),
            path: "/test".to_string(),
            query_params: HashMap::new(),
            headers: HashMap::new(),
            body: String::new(),
        };

        let code = r#"
response_status = 200
response_body = "Test OK"
"#
        .to_string();

        let response = handle_request_async(context, code).await;
        assert_eq!(response.status, 200);
    }

    #[tokio::test]
    async fn test_concurrent_requests() {
        let mut handles = vec![];

        // Spawn 10 concurrent requests
        for i in 0..10 {
            let handle = tokio::spawn(async move {
                let context = AsyncRequestContext {
                    method: "GET".to_string(),
                    path: format!("/test/{}", i),
                    query_params: HashMap::new(),
                    headers: HashMap::new(),
                    body: String::new(),
                };

                let code = format!(
                    r#"
response_status = 200
response_body = "Request {}"
"#,
                    i
                );

                handle_request_async(context, code).await
            });

            handles.push(handle);
        }

        // Wait for all to complete
        for handle in handles {
            let response = handle.await.unwrap();
            assert_eq!(response.status, 200);
        }
    }
}
