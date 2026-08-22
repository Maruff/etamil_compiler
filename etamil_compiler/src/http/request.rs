// HTTP Request Parsing Module

use std::collections::HashMap;

/// Where does `needle` first appear in `haystack`?
fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return None;
    }
    haystack.windows(needle.len()).position(|w| w == needle)
}

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
    pub body: String,
    /// The body as it arrived. `body` is this decoded leniently, which is
    /// right for JSON and wrong for an upload: a PDF is not UTF-8, and the
    /// lenient decode replaces every byte it dislikes. Anything that might
    /// not be text reads this instead.
    pub body_bytes: Vec<u8>,
}

impl HttpRequest {
    /// Parse an HTTP request from text. Convenience over `parse_bytes` for
    /// callers that already hold a String — a test fixture, mostly.
    pub fn parse(raw: &str) -> Result<Self, String> {
        Self::parse_bytes(raw.as_bytes())
    }

    /// Parse an HTTP request from the bytes that arrived.
    ///
    /// The head is text by definition and is decoded leniently. The body is
    /// kept as bytes and only *also* offered as text, because a multipart
    /// upload stops being the file it was the moment it becomes a String.
    pub fn parse_bytes(raw: &[u8]) -> Result<Self, String> {
        let (head_end, body_at) = match find_bytes(raw, b"\r\n\r\n") {
            Some(at) => (at, at + 4),
            // Some clients (and most hand-written test fixtures) use bare LF.
            None => match find_bytes(raw, b"\n\n") {
                Some(at) => (at, at + 2),
                None => (raw.len(), raw.len()),
            },
        };
        let head = String::from_utf8_lossy(&raw[..head_end]).into_owned();
        let body_bytes = raw[body_at..].to_vec();
        let mut request = Self::parse_head(&head)?;
        request.body = String::from_utf8_lossy(&body_bytes).into_owned();
        request.body_bytes = body_bytes;
        Ok(request)
    }

    /// The request line and headers.
    fn parse_head(head: &str) -> Result<Self, String> {
        let mut lines = head.lines();

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

        // Parse headers. Names are lowercased so lookup is case-insensitive,
        // as HTTP requires.
        let mut headers = HashMap::new();
        for line in lines {
            if let Some(colon_idx) = line.find(':') {
                let header_name = line[..colon_idx].trim().to_lowercase();
                let header_value = line[colon_idx + 1..].trim().to_string();
                headers.insert(header_name, header_value);
            }
        }

        Ok(HttpRequest {
            method,
            path,
            version,
            headers,
            query_params,
            // Both filled in by parse_bytes, which is the only caller.
            body: String::new(),
            body_bytes: Vec::new(),
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
                    let key = percent_decode(&param[..eq_idx]);
                    let value = percent_decode(&param[eq_idx + 1..]);
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

/// Percent-decode one query-string component, treating '+' as a space.
///
/// Query values are the only way a GET route receives an argument, and this
/// language's arguments are frequently Tamil: without decoding, `?peyar=வரவு`
/// arrives as the literal text "%E0%AE%B5..." and every comparison against it
/// fails while looking perfectly reasonable in a log.
///
/// Decoding works on bytes rather than the &str, because a '%' can be
/// followed by bytes that are not a character boundary; slicing the str there
/// would panic on malformed input arriving from the network.
fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                let decoded = std::str::from_utf8(&bytes[i + 1..i + 3])
                    .ok()
                    .and_then(|hex| u8::from_str_radix(hex, 16).ok());
                match decoded {
                    Some(byte) => {
                        out.push(byte);
                        i += 3;
                    }
                    // Not a valid escape; keep the '%' as it was written.
                    None => {
                        out.push(b'%');
                        i += 1;
                    }
                }
            }
            byte => {
                out.push(byte);
                i += 1;
            }
        }
    }

    String::from_utf8_lossy(&out).into_owned()
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

    // The body is what a POST route actually works on, so it has to survive
    // parsing byte for byte.
    #[test]
    fn body_is_preserved_exactly() {
        let raw = "POST /api/pativu HTTP/1.1\r\nContent-Type: application/json\r\n\r\n{\"peyar\":\"வரவு\"}";
        let req = HttpRequest::parse(raw).unwrap();

        assert_eq!(req.body, "{\"peyar\":\"வரவு\"}");
    }

    // Regression: the old line-by-line parser dropped blank lines inside the
    // body and re-joined the rest with '\n', so a pretty-printed payload came
    // back altered.
    #[test]
    fn multiline_body_keeps_its_blank_lines() {
        let body = "{\n  \"a\": 1,\n\n  \"b\": 2\n}";
        let raw = format!("POST /x HTTP/1.1\r\nHost: localhost\r\n\r\n{}", body);
        let req = HttpRequest::parse(&raw).unwrap();

        assert_eq!(req.body, body);
    }

    #[test]
    fn request_without_a_body_has_an_empty_one() {
        let raw = "GET /api/kaNakku HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let req = HttpRequest::parse(raw).unwrap();

        assert_eq!(req.body, "");
    }

    // Tamil in a query value is the normal case here, not an edge case.
    #[test]
    fn query_values_are_percent_decoded() {
        let raw = "GET /q?vakY=%E0%AE%B5%E0%AE%B0%E0%AE%B5%E0%AF%81&note=two+words HTTP/1.1\r\n\r\n";
        let req = HttpRequest::parse(raw).unwrap();

        assert_eq!(req.query_param("vakY"), Some("வரவு"));
        assert_eq!(req.query_param("note"), Some("two words"));
    }

    // Malformed input arrives from the network; it must not panic.
    #[test]
    fn a_stray_percent_is_left_alone() {
        assert_eq!(percent_decode("100%"), "100%");
        assert_eq!(percent_decode("50%zz"), "50%zz");
        assert_eq!(percent_decode("%E0%AE%B5"), "வ");
    }
}
