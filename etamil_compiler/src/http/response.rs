// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Mohammed Maruff (Esan Maruff) <esan@etamil.in>
// HTTP Response Module

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status_code: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    /// A body that is not text — a PDF, an .odt, a picture. When this is set
    /// it is what gets sent and `body` is ignored. It is `Vec<u8>` and not a
    /// `String` for the obvious reason: a PDF is not valid UTF-8, and going
    /// through a String would replace every byte it did not like.
    pub bytes: Option<Vec<u8>>,
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
            bytes: None,
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

    /// The whole response as it goes onto the socket.
    ///
    /// Bytes, not a String: the head is text but the body need not be, and a
    /// response carrying a PDF has to arrive as the bytes that were read.
    pub fn to_http_bytes(&self) -> Vec<u8> {
        let mut head = format!("HTTP/1.1 {} {}\r\n", self.status_code, self.status_text);

        for (name, value) in &self.headers {
            head.push_str(&format!("{}: {}\r\n", name, value));
        }

        // Add CORS headers for MVP
        head.push_str("Access-Control-Allow-Origin: *\r\n");
        head.push_str("Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS\r\n");
        head.push_str("Access-Control-Allow-Headers: Content-Type\r\n");

        head.push_str("\r\n");

        let mut out = head.into_bytes();
        match &self.bytes {
            Some(raw) => out.extend_from_slice(raw),
            None => out.extend_from_slice(self.body.as_bytes()),
        }
        out
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
        let http_str = String::from_utf8(resp.to_http_bytes()).unwrap();
        
        assert!(http_str.contains("HTTP/1.1 200 OK"));
        assert!(http_str.contains("Content-Type: application/json"));
        assert!(http_str.contains("Test"));
    }

    // A PDF is not valid UTF-8. Before this, the body went out through a
    // String and every byte the decoder disliked became U+FFFD, so the file
    // that arrived was not the file that was read.
    #[test]
    fn a_byte_body_is_sent_unchanged() {
        let mut resp = HttpResponse::success(200, String::new());
        resp.bytes = Some(vec![0x25, 0x50, 0x44, 0x46, 0xFF, 0xFE, 0x00]);

        let raw = resp.to_http_bytes();
        let split = raw
            .windows(4)
            .position(|w| w == b"\r\n\r\n")
            .expect("head and body are separated");

        assert_eq!(&raw[split + 4..], &[0x25, 0x50, 0x44, 0x46, 0xFF, 0xFE, 0x00]);
    }

    #[test]
    fn test_set_header() {
        let mut resp = HttpResponse::success(200, "OK".to_string());
        resp.set_header("X-Custom", "value");
        
        assert_eq!(resp.headers.get("X-Custom"), Some(&"value".to_string()));
    }
}
