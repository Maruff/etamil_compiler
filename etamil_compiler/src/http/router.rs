// HTTP Router Module

#[derive(Debug, Clone)]
pub struct Route {
    pub method: String,
    pub path: String,
}

pub struct Router {
    pub routes: Vec<Route>,
}

impl Router {
    /// Create a new router
    pub fn new() -> Self {
        Router {
            routes: Vec::new(),
        }
    }

    /// Add a route to the router
    pub fn add_route(&mut self, method: &str, path: &str) {
        let route = Route {
            method: method.to_uppercase(),
            path: path.to_string(),
        };
        self.routes.push(route);
    }

    /// Find a matching route for a request
    pub fn find_route(&self, method: &str, path: &str) -> Option<&Route> {
        let method_upper = method.to_uppercase();
        
        for route in &self.routes {
            if route.method == method_upper && self.path_matches(&route.path, path) {
                return Some(route);
            }
        }
        
        None
    }

    /// Check if a route path matches a request path
    fn path_matches(&self, pattern: &str, path: &str) -> bool {
        let pattern_parts: Vec<&str> = pattern.split('/').filter(|p| !p.is_empty()).collect();
        let path_parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();

        if pattern_parts.len() != path_parts.len() {
            return false;
        }

        for (pattern_part, path_part) in pattern_parts.iter().zip(path_parts.iter()) {
            if !pattern_part.starts_with(':') && pattern_part != path_part {
                return false;
            }
        }

        true
    }

    /// Get route count
    pub fn len(&self) -> usize {
        self.routes.len()
    }

    /// Check if router is empty
    pub fn is_empty(&self) -> bool {
        self.routes.is_empty()
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_router() {
        let router = Router::new();
        assert!(router.is_empty());
    }

    #[test]
    fn test_add_route() {
        let mut router = Router::new();
        router.add_route("GET", "/api/users");
        
        assert_eq!(router.len(), 1);
        assert_eq!(router.routes[0].method, "GET");
        assert_eq!(router.routes[0].path, "/api/users");
    }

    #[test]
    fn test_find_route() {
        let mut router = Router::new();
        router.add_route("GET", "/api/users");
        router.add_route("POST", "/api/users");
        
        assert!(router.find_route("GET", "/api/users").is_some());
        assert!(router.find_route("POST", "/api/users").is_some());
        assert!(router.find_route("DELETE", "/api/users").is_none());
    }

    #[test]
    fn test_path_with_parameters() {
        let mut router = Router::new();
        router.add_route("GET", "/api/users/:id");
        
        assert!(router.find_route("GET", "/api/users/123").is_some());
        assert!(router.find_route("GET", "/api/users/456").is_some());
    }
}
