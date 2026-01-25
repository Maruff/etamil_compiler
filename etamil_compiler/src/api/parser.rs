// API Parser Module
// Handles parsing of REST API statements from the AST

use crate::parser::Expr;

/// API Parser for route and server definitions
pub struct APIParser;

impl APIParser {
    /// Parse route definition
    pub fn parse_route(method: &str, path: &Expr) -> Result<(String, String), String> {
        let path_str = match path {
            Expr::Variable(s) if s.starts_with('"') && s.ends_with('"') => {
                s.trim_matches('"').to_string()
            }
            _ => return Err("Invalid path expression".to_string()),
        };
        
        if !super::utils::is_valid_path(&path_str) {
            return Err("Path must start with /".to_string());
        }
        
        Ok((method.to_string(), path_str))
    }
    
    /// Parse server configuration
    pub fn parse_server(host: &Expr, port: &Expr) -> Result<(String, u16), String> {
        let host_str = match host {
            Expr::Variable(s) if s.starts_with('"') && s.ends_with('"') => {
                s.trim_matches('"').to_string()
            }
            _ => "localhost".to_string(),
        };
        
        let port_num = match port {
            Expr::Number(n) => *n as u16,
            _ => 8080,
        };
        
        if !super::utils::is_valid_port(port_num) {
            return Err(format!("Invalid port: {}", port_num));
        }
        
        Ok((host_str, port_num))
    }
    
    /// Parse HTTP method
    pub fn parse_method(method: &str) -> String {
        super::utils::normalize_http_method(method)
    }
}
