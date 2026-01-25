// HTTP Request Parsing Module

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
    pub body: String,
}

impl HttpRequest {
    /// Parse HTTP request from raw string
    pub fn parse(raw: &str) -> Result<Self, String> {
        let mut lines = raw.lines();
        
        // Parse request line
        let request_line = lines.next()
            .ok_or("Missing request line".to_string())?;
        
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 3 {
            return Err("Invalid request line".to_string());
        }

        let method = parts[0].to_string();
        let path_and_query = parts[1];
        let version = parts[2].to_string();

        // Parse path and query string
        let (path, query_params) = Self::parse_path_and_query(path_and_query);

        // Parse headers
        let mut headers = HashMap::new();
        let mut body = String::new();
        let mut header_section = true;

        for line in lines {
            if line.is_empty() {
                header_section = false;
                continue;
            }

            if header_section {
                if let Some(colon_idx) = line.find(':') {
                    let header_name = line[..colon_idx].trim().to_lowercase();
                    let header_value = line[colon_idx + 1..].trim().to_string();
                    headers.insert(header_name, header_value);
                }
            } else {
                body.push_str(line);
                body.push('\n');
            }
        }

        body = body.trim_end().to_string();

        Ok(HttpRequest {
            method,
            path,
            version,
            headers,
            query_params,
            body,
        })
    }

    /// Parse path and query string
    fn parse_path_and_query(path_and_query: &str) -> (String, HashMap<String, String>) {
        if let Some(question_idx) = path_and_query.find('?') {
            let path = path_and_query[..question_idx].to_string();
            let query_str = &path_and_query[question_idx + 1..];
            
            let mut params = HashMap::new();
            for param in query_str.split('&') {
                if let Some(eq_idx) = param.find('=') {
                    let key = param[..eq_idx].to_string();
                    let value = param[eq_idx + 1..].to_string();
                    params.insert(key, value);
                }
            }
            
            (path, params)
        } else {
            (path_and_query.to_string(), HashMap::new())
        }
    }

    /// Get a header value
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(&name.to_lowercase()).map(|s| s.as_str())
    }

    /// Get a query parameter
    pub fn query_param(&self, name: &str) -> Option<&str> {
        self.query_params.get(name).map(|s| s.as_str())
    }

    /// Get Content-Type
    pub fn content_type(&self) -> Option<&str> {
        self.header("content-type")
    }

    /// Get Content-Length
    pub fn content_length(&self) -> Option<usize> {
        self.header("content-length")
            .and_then(|s| s.parse::<usize>().ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_get_request() {
        let raw = "GET /api/users HTTP/1.1\r\nHost: localhost\r\nUser-Agent: test\r\n\r\n";
        let req = HttpRequest::parse(raw).unwrap();
        
        assert_eq!(req.method, "GET");
        assert_eq!(req.path, "/api/users");
        assert_eq!(req.version, "HTTP/1.1");
        assert_eq!(req.header("host"), Some("localhost"));
    }

    #[test]
    fn test_parse_query_params() {
        let raw = "GET /search?q=rust&page=1 HTTP/1.1\r\n\r\n";
        let req = HttpRequest::parse(raw).unwrap();
        
        assert_eq!(req.path, "/search");
        assert_eq!(req.query_param("q"), Some("rust"));
        assert_eq!(req.query_param("page"), Some("1"));
    }

    #[test]
    fn test_parse_post_request() {
        let raw = "POST /api/users HTTP/1.1\r\nContent-Type: application/json\r\nContent-Length: 13\r\n\r\n{\"name\":\"test\"}";
        let req = HttpRequest::parse(raw).unwrap();
        
        assert_eq!(req.method, "POST");
        assert_eq!(req.content_type(), Some("application/json"));
        assert_eq!(req.content_length(), Some(13));
    }
}
