// Phase 4: Authentication & Authorization Module
// JWT-based auth with role-based access control (RBAC)

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use bcrypt::{hash, verify};

const JWT_SECRET: &[u8] = b"etamil-phase4-jwt-secret-change-in-production";
const BCRYPT_COST: u32 = 12;

/// JWT Claims structure for access tokens
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenClaims {
    pub sub: String,           // Subject (user ID)
    pub email: String,
    pub roles: Vec<String>,    // RBAC roles
    pub iat: i64,              // Issued at
    pub exp: i64,              // Expiration
}

/// User credentials for login
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Login response with tokens
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

/// User record in system
#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub roles: Vec<String>,
}

/// Authentication manager
pub struct AuthManager {
    users: HashMap<String, User>,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl AuthManager {
    /// Create a new auth manager
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            encoding_key: EncodingKey::from_secret(JWT_SECRET),
            decoding_key: DecodingKey::from_secret(JWT_SECRET),
        }
    }

    /// Register a new user
    pub fn register_user(&mut self, email: &str, password: &str, roles: Vec<String>) -> Result<User, String> {
        if self.users.values().any(|u| u.email == email) {
            return Err("User already exists".to_string());
        }

        let password_hash = hash(password, BCRYPT_COST)
            .map_err(|_| "Failed to hash password".to_string())?;

        let user = User {
            id: format!("user_{}", uuid::Uuid::new_v4()),
            email: email.to_string(),
            password_hash,
            roles,
        };

        self.users.insert(user.id.clone(), user.clone());
        Ok(user)
    }

    /// Login user and generate JWT tokens
    pub fn login(&self, email: &str, password: &str) -> Result<AuthResponse, String> {
        let user = self.users
            .values()
            .find(|u| u.email == email)
            .ok_or("User not found".to_string())?;

        verify(password, &user.password_hash)
            .map_err(|_| "Invalid credentials".to_string())?;

        let now = Utc::now();
        let access_exp = now + Duration::hours(1);
        let refresh_exp = now + Duration::days(7);

        let access_claims = TokenClaims {
            sub: user.id.clone(),
            email: user.email.clone(),
            roles: user.roles.clone(),
            iat: now.timestamp(),
            exp: access_exp.timestamp(),
        };

        let refresh_claims = TokenClaims {
            sub: user.id.clone(),
            email: user.email.clone(),
            roles: vec!["refresh".to_string()],
            iat: now.timestamp(),
            exp: refresh_exp.timestamp(),
        };

        let access_token = encode(&Header::default(), &access_claims, &self.encoding_key)
            .map_err(|_| "Failed to generate access token".to_string())?;

        let refresh_token = encode(&Header::default(), &refresh_claims, &self.encoding_key)
            .map_err(|_| "Failed to generate refresh token".to_string())?;

        Ok(AuthResponse {
            access_token,
            refresh_token,
            expires_in: 3600,
        })
    }

    /// Verify and decode JWT token
    pub fn verify_token(&self, token: &str) -> Result<TokenClaims, String> {
        decode::<TokenClaims>(token, &self.decoding_key, &Validation::default())
            .map(|data| data.claims)
            .map_err(|_| "Invalid token".to_string())
    }

    /// Check if user has required role
    pub fn has_role(&self, claims: &TokenClaims, required_role: &str) -> bool {
        claims.roles.contains(&required_role.to_string())
    }

    /// Get user by ID
    pub fn get_user(&self, user_id: &str) -> Option<User> {
        self.users.get(user_id).cloned()
    }
}

/// RBAC middleware guard
pub struct RoleGuard {
    required_roles: Vec<String>,
}

impl RoleGuard {
    pub fn new(roles: Vec<&str>) -> Self {
        Self {
            required_roles: roles.iter().map(|r| r.to_string()).collect(),
        }
    }

    pub fn check(&self, claims: &TokenClaims) -> bool {
        self.required_roles.iter().any(|role| claims.roles.contains(role))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_user() {
        let mut auth = AuthManager::new();
        let user = auth.register_user("user@example.com", "password123", vec!["user".to_string()]);
        assert!(user.is_ok());
        let user = user.unwrap();
        assert_eq!(user.email, "user@example.com");
        assert!(user.roles.contains(&"user".to_string()));
    }

    #[test]
    fn test_login_success() {
        let mut auth = AuthManager::new();
        auth.register_user("user@example.com", "password123", vec!["admin".to_string()]).unwrap();
        let result = auth.login("user@example.com", "password123");
        assert!(result.is_ok());
        let auth_resp = result.unwrap();
        assert!(!auth_resp.access_token.is_empty());
        assert!(!auth_resp.refresh_token.is_empty());
    }

    #[test]
    fn test_login_failure() {
        let auth = AuthManager::new();
        let result = auth.login("unknown@example.com", "password");
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_token() {
        let mut auth = AuthManager::new();
        auth.register_user("user@example.com", "password123", vec!["user".to_string()]).unwrap();
        let auth_resp = auth.login("user@example.com", "password123").unwrap();
        let claims = auth.verify_token(&auth_resp.access_token);
        assert!(claims.is_ok());
        let claims = claims.unwrap();
        assert_eq!(claims.email, "user@example.com");
    }

    #[test]
    fn test_role_guard() {
        let guard = RoleGuard::new(vec!["admin"]);
        let claims = TokenClaims {
            sub: "user1".to_string(),
            email: "test@example.com".to_string(),
            roles: vec!["admin".to_string()],
            iat: 1000,
            exp: 2000,
        };
        assert!(guard.check(&claims));

        let claims_no_role = TokenClaims {
            sub: "user2".to_string(),
            email: "test2@example.com".to_string(),
            roles: vec!["user".to_string()],
            iat: 1000,
            exp: 2000,
        };
        assert!(!guard.check(&claims_no_role));
    }
}
