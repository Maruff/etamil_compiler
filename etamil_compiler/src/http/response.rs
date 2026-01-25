// HTTP Response Module

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status_code: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl HttpResponse {
    /// Create a new HTTP response
    pub fn new(status_code: u16, status_text: &str, body: String) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("Content-Length".to_string(), body.len().to_string());
        
        HttpResponse {
            status_code,
            status_text: status_text.to_string(),
            headers,
            body,
        }
    }

    /// Create a 200 OK response
    pub fn success(status_code: u16, body: String) -> Self {
        let status_text = match status_code {
            200 => "OK",
            201 => "Created",
            202 => "Accepted",
            204 => "No Content",
            _ => "OK",
        };
        
        HttpResponse::new(status_code, status_text, body)
    }

    /// Create a 400 Bad Request response
    pub fn bad_request(message: &str) -> Self {
        let body = format!(r#"{{"error": "{}"}}"#, message);
        HttpResponse::new(400, "Bad Request", body)
    }

    /// Create a 404 Not Found response
    pub fn not_found() -> Self {
        let body = r#"{"error": "Not Found"}"#.to_string();
        HttpResponse::new(404, "Not Found", body)
    }

    /// Create a 500 Internal Server Error response
    pub fn internal_error(message: &str) -> Self {
        let body = format!(r#"{{"error": "{}"}}"#, message);
        HttpResponse::new(500, "Internal Server Error", body)
    }

    /// Create a response with custom status
    pub fn custom(status_code: u16, body: String) -> Self {
        let status_text = Self::status_text_for_code(status_code);
        HttpResponse::new(status_code, status_text, body)
    }

    /// Set a response header
    pub fn set_header(&mut self, name: &str, value: &str) {
        self.headers.insert(name.to_string(), value.to_string());
    }

    /// Convert response to HTTP string format
    pub fn to_http_string(&self) -> String {
        let mut response = format!(
            "HTTP/1.1 {} {}\r\n",
            self.status_code,
            self.status_text
        );

        // Add headers
        for (name, value) in &self.headers {
            response.push_str(&format!("{}: {}\r\n", name, value));
        }

        // Add CORS headers for MVP
        response.push_str("Access-Control-Allow-Origin: *\r\n");
        response.push_str("Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS\r\n");
        response.push_str("Access-Control-Allow-Headers: Content-Type\r\n");

        response.push_str("\r\n");
        response.push_str(&self.body);

        response
    }

    /// Get status text for a status code
    fn status_text_for_code(code: u16) -> &'static str {
        match code {
            100 => "Continue",
            101 => "Switching Protocols",
            200 => "OK",
            201 => "Created",
            202 => "Accepted",
            204 => "No Content",
            300 => "Multiple Choices",
            301 => "Moved Permanently",
            302 => "Found",
            304 => "Not Modified",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            409 => "Conflict",
            500 => "Internal Server Error",
            501 => "Not Implemented",
            502 => "Bad Gateway",
            503 => "Service Unavailable",
            _ => "Unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_success_response() {
        let resp = HttpResponse::success(200, "Hello, World!".to_string());
        assert_eq!(resp.status_code, 200);
        assert_eq!(resp.body, "Hello, World!");
    }

    #[test]
    fn test_create_error_response() {
        let resp = HttpResponse::not_found();
        assert_eq!(resp.status_code, 404);
        assert!(resp.body.contains("Not Found"));
    }

    #[test]
    fn test_http_string_format() {
        let resp = HttpResponse::success(200, "Test".to_string());
        let http_str = resp.to_http_string();
        
        assert!(http_str.contains("HTTP/1.1 200 OK"));
        assert!(http_str.contains("Content-Type: application/json"));
        assert!(http_str.contains("Test"));
    }

    #[test]
    fn test_set_header() {
        let mut resp = HttpResponse::success(200, "OK".to_string());
        resp.set_header("X-Custom", "value");
        
        assert_eq!(resp.headers.get("X-Custom"), Some(&"value".to_string()));
    }
}
