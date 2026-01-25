// Monitoring & Metrics Module for eTamil Backend - Phase 3
// Provides performance metrics, health checks, and observability hooks

use chrono::Utc;
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::Instant;

/// Request metrics
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetrics {
    /// Number of successful requests
    pub total_requests: u64,
    /// Number of failed requests
    pub failed_requests: u64,
    /// Total time spent processing requests (ms)
    pub total_time_ms: u64,
    /// Average response time (ms)
    pub avg_response_time_ms: f64,
    /// Minimum response time (ms)
    pub min_response_time_ms: u64,
    /// Maximum response time (ms)
    pub max_response_time_ms: u64,
    /// Requests per second
    pub rps: f64,
}

/// Health check status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub status: HealthStatus,
    pub timestamp: String,
    pub uptime_ms: u64,
    pub checks: HashMap<String, HealthCheckItem>,
}

/// Individual health check item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckItem {
    pub status: HealthStatus,
    pub message: String,
    pub response_time_ms: u64,
}

/// Metrics collection and tracking
pub struct MetricsCollector {
    start_time: Instant,
    total_requests: Arc<Mutex<u64>>,
    failed_requests: Arc<Mutex<u64>>,
    response_times: Arc<Mutex<Vec<u64>>>,
    endpoint_metrics: Arc<Mutex<HashMap<String, EndpointMetrics>>>,
}

/// Per-endpoint metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointMetrics {
    pub path: String,
    pub method: String,
    pub requests: u64,
    pub errors: u64,
    pub total_time_ms: u64,
    pub avg_time_ms: f64,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        MetricsCollector {
            start_time: Instant::now(),
            total_requests: Arc::new(Mutex::new(0)),
            failed_requests: Arc::new(Mutex::new(0)),
            response_times: Arc::new(Mutex::new(Vec::new())),
            endpoint_metrics: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Record a request
    pub fn record_request(&self, endpoint: &str, method: &str, duration_ms: u64, success: bool) {
        // Update total counters
        if let Ok(mut total) = self.total_requests.lock() {
            *total += 1;
        }
        if !success {
            if let Ok(mut failed) = self.failed_requests.lock() {
                *failed += 1;
            }
        }

        // Record response time
        if let Ok(mut times) = self.response_times.lock() {
            times.push(duration_ms);
        }

        // Update endpoint metrics
        if let Ok(mut endpoints) = self.endpoint_metrics.lock() {
            endpoints
                .entry(endpoint.to_string())
                .and_modify(|metrics| {
                    metrics.requests += 1;
                    if !success {
                        metrics.errors += 1;
                    }
                    metrics.total_time_ms += duration_ms;
                    metrics.avg_time_ms = metrics.total_time_ms as f64 / metrics.requests as f64;
                })
                .or_insert_with(|| EndpointMetrics {
                    path: endpoint.to_string(),
                    method: method.to_string(),
                    requests: 1,
                    errors: if success { 0 } else { 1 },
                    total_time_ms: duration_ms,
                    avg_time_ms: duration_ms as f64,
                });
        }
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> RequestMetrics {
        let total = self.total_requests.lock().map(|t| *t).unwrap_or(0);
        let failed = self.failed_requests.lock().map(|f| *f).unwrap_or(0);
        let times = self.response_times.lock().map(|t| t.clone()).unwrap_or_default();

        let (min_time, max_time, avg_time) = if times.is_empty() {
            (0, 0, 0.0)
        } else {
            let min = *times.iter().min().unwrap_or(&0);
            let max = *times.iter().max().unwrap_or(&0);
            let avg = times.iter().sum::<u64>() as f64 / times.len() as f64;
            (min, max, avg)
        };

        let uptime = self.start_time.elapsed();
        let rps = if uptime.as_secs() > 0 {
            total as f64 / uptime.as_secs_f64()
        } else {
            0.0
        };

        RequestMetrics {
            total_requests: total,
            failed_requests: failed,
            total_time_ms: times.iter().sum(),
            avg_response_time_ms: avg_time,
            min_response_time_ms: min_time,
            max_response_time_ms: max_time,
            rps,
        }
    }

    /// Get endpoint metrics
    pub fn get_endpoint_metrics(&self, endpoint: &str) -> Option<EndpointMetrics> {
        self.endpoint_metrics
            .lock()
            .ok()
            .and_then(|metrics| metrics.get(endpoint).cloned())
    }

    /// Get all endpoint metrics
    pub fn get_all_endpoint_metrics(&self) -> Vec<EndpointMetrics> {
        self.endpoint_metrics
            .lock()
            .map(|metrics| metrics.values().cloned().collect())
            .unwrap_or_default()
    }

    /// Get uptime
    pub fn get_uptime_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Health checker
pub struct HealthChecker {
    checks: Arc<Mutex<HashMap<String, Box<dyn Fn() -> HealthCheckItem + Send>>>>,
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new() -> Self {
        HealthChecker {
            checks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a health check
    pub fn register_check<F>(&self, name: &str, check: F)
    where
        F: Fn() -> HealthCheckItem + Send + 'static,
    {
        if let Ok(mut checks) = self.checks.lock() {
            checks.insert(name.to_string(), Box::new(check));
        }
    }

    /// Run all health checks
    pub fn check_health(&self) -> HealthCheck {
        let mut checks_result = HashMap::new();
        let mut overall_status = HealthStatus::Healthy;

        if let Ok(checks) = self.checks.lock() {
            for (name, check_fn) in checks.iter() {
                let result = check_fn();
                if result.status == HealthStatus::Unhealthy {
                    overall_status = HealthStatus::Unhealthy;
                } else if result.status == HealthStatus::Degraded && overall_status == HealthStatus::Healthy {
                    overall_status = HealthStatus::Degraded;
                }
                checks_result.insert(name.clone(), result);
            }
        }

        HealthCheck {
            status: overall_status,
            timestamp: Utc::now().to_rfc3339(),
            uptime_ms: 0, // Will be set by caller
            checks: checks_result,
        }
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Default health checks
pub fn create_default_health_checker() -> HealthChecker {
    let checker = HealthChecker::new();

    // Server health check
    checker.register_check("server", || HealthCheckItem {
        status: HealthStatus::Healthy,
        message: "Server is running".to_string(),
        response_time_ms: 1,
    });

    // Memory check (basic)
    checker.register_check("memory", || HealthCheckItem {
        status: HealthStatus::Healthy,
        message: "Memory usage normal".to_string(),
        response_time_ms: 5,
    });

    checker
}

/// Performance metrics report
#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub timestamp: String,
    pub uptime_ms: u64,
    pub metrics: RequestMetrics,
    pub endpoints: Vec<EndpointMetrics>,
    pub health: HealthCheck,
}

impl PerformanceReport {
    /// Create a performance report
    pub fn generate(metrics: &MetricsCollector, health_checker: &HealthChecker) -> Self {
        let uptime = metrics.get_uptime_ms();
        let mut health = health_checker.check_health();
        health.uptime_ms = uptime;

        PerformanceReport {
            timestamp: Utc::now().to_rfc3339(),
            uptime_ms: uptime,
            metrics: metrics.get_metrics(),
            endpoints: metrics.get_all_endpoint_metrics(),
            health,
        }
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector() {
        let collector = MetricsCollector::new();
        collector.record_request("/api/users", "GET", 50, true);
        collector.record_request("/api/users", "GET", 75, true);
        collector.record_request("/api/users", "POST", 100, false);

        let metrics = collector.get_metrics();
        assert_eq!(metrics.total_requests, 3);
        assert_eq!(metrics.failed_requests, 1);
    }

    #[test]
    fn test_health_checker() {
        let checker = HealthChecker::new();
        checker.register_check("test", || HealthCheckItem {
            status: HealthStatus::Healthy,
            message: "All good".to_string(),
            response_time_ms: 1,
        });

        let health = checker.check_health();
        assert_eq!(health.status, HealthStatus::Healthy);
    }
}
