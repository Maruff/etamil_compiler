// Phase 4: Resilience Patterns (Circuit Breaker, Retries, Timeouts)

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::time::sleep;

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitBreakerState {
    Closed,      // Normal operation
    Open,        // Failing, rejecting requests
    HalfOpen,    // Testing if service is back
}

/// Circuit breaker configuration
#[derive(Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,  // Failures before opening
    pub success_threshold: u32,  // Successes in HalfOpen to close
    pub timeout: Duration,        // How long to stay open
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            timeout: Duration::from_secs(60),
        }
    }
}

/// Circuit breaker implementation
pub struct CircuitBreaker {
    state: Arc<tokio::sync::RwLock<CircuitBreakerState>>,
    failure_count: Arc<AtomicU32>,
    success_count: Arc<AtomicU32>,
    last_failure_time: Arc<tokio::sync::RwLock<Option<SystemTime>>>,
    config: CircuitBreakerConfig,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: Arc::new(tokio::sync::RwLock::new(CircuitBreakerState::Closed)),
            failure_count: Arc::new(AtomicU32::new(0)),
            success_count: Arc::new(AtomicU32::new(0)),
            last_failure_time: Arc::new(tokio::sync::RwLock::new(None)),
            config,
        }
    }

    pub async fn get_state(&self) -> CircuitBreakerState {
        *self.state.read().await
    }

    pub async fn record_success(&self) {
        let state = *self.state.read().await;
        match state {
            CircuitBreakerState::Closed => {
                self.failure_count.store(0, Ordering::SeqCst);
            }
            CircuitBreakerState::HalfOpen => {
                let succ = self.success_count.fetch_add(1, Ordering::SeqCst);
                if succ + 1 >= self.config.success_threshold {
                    let mut s = self.state.write().await;
                    *s = CircuitBreakerState::Closed;
                    self.failure_count.store(0, Ordering::SeqCst);
                    self.success_count.store(0, Ordering::SeqCst);
                }
            }
            CircuitBreakerState::Open => {}
        }
    }

    pub async fn record_failure(&self) {
        let mut state = self.state.write().await;
        let fail_count = self.failure_count.fetch_add(1, Ordering::SeqCst);

        *self.last_failure_time.write().await = Some(SystemTime::now());

        if fail_count + 1 >= self.config.failure_threshold {
            *state = CircuitBreakerState::Open;
        }
    }

    pub async fn try_request<F, T>(&self, f: F) -> Result<T, String>
    where
        F: std::future::Future<Output = Result<T, String>>,
    {
        let state = *self.state.read().await;

        match state {
            CircuitBreakerState::Closed => {
                match f.await {
                    Ok(result) => {
                        self.record_success().await;
                        Ok(result)
                    }
                    Err(e) => {
                        self.record_failure().await;
                        Err(e)
                    }
                }
            }
            CircuitBreakerState::Open => {
                let last_failure = *self.last_failure_time.read().await;
                if let Some(last_time) = last_failure {
                    let elapsed = last_time.elapsed().unwrap_or(Duration::from_secs(0));
                    if elapsed > self.config.timeout {
                        let mut s = self.state.write().await;
                        *s = CircuitBreakerState::HalfOpen;
                        self.success_count.store(0, Ordering::SeqCst);
                        // Proceed with request in HalfOpen state
                        return match f.await {
                            Ok(result) => {
                                self.record_success().await;
                                Ok(result)
                            }
                            Err(e) => {
                                self.record_failure().await;
                                Err(e)
                            }
                        };
                    }
                }
                Err("Circuit breaker is open".to_string())
            }
            CircuitBreakerState::HalfOpen => {
                match f.await {
                    Ok(result) => {
                        self.record_success().await;
                        Ok(result)
                    }
                    Err(e) => {
                        self.record_failure().await;
                        Err(e)
                    }
                }
            }
        }
    }
}

/// Retry configuration
#[derive(Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_factor: 2.0,
        }
    }
}

/// Retry helper
pub struct Retry {
    config: RetryConfig,
}

impl Retry {
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    pub async fn execute<F, Fut, T>(&self, mut f: F) -> Result<T, String>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, String>>,
    {
        let mut delay = self.config.initial_delay;
        let mut attempt = 0;

        loop {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempt += 1;
                    if attempt > self.config.max_retries {
                        return Err(format!("Max retries exceeded: {}", e));
                    }

                    sleep(delay).await;

                    // Exponential backoff
                    let next_delay_secs =
                        delay.as_secs_f64() * self.config.backoff_factor;
                    delay = Duration::from_secs_f64(next_delay_secs.min(
                        self.config.max_delay.as_secs_f64(),
                    ));
                }
            }
        }
    }
}

/// Request timeout helper
pub async fn with_timeout<F, T>(
    duration: Duration,
    future: F,
) -> Result<T, String>
where
    F: std::future::Future<Output = T>,
{
    match tokio::time::timeout(duration, future).await {
        Ok(result) => Ok(result),
        Err(_) => Err("Request timeout".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_closed() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig::default());
        assert_eq!(cb.get_state().await, CircuitBreakerState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            ..Default::default()
        };
        let cb = CircuitBreaker::new(config);

        // Simulate failures
        let result = cb.try_request(async {
            Err::<(), String>("error".to_string())
        }).await;
        assert!(result.is_err());
        
        let result = cb.try_request(async {
            Err::<(), String>("error".to_string())
        }).await;
        assert!(result.is_err());

        // Should be open now
        assert_eq!(cb.get_state().await, CircuitBreakerState::Open);
    }

    #[tokio::test]
    async fn test_retry_success() {
        let retry = Retry::new(RetryConfig::default());
        let attempts = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let attempts_clone = attempts.clone();

        let result = retry.execute(|| {
            let attempts = attempts_clone.clone();
            async move {
                let count = attempts.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if count < 1 {
                    Err::<(), String>("retry".to_string())
                } else {
                    Ok(())
                }
            }
        }).await;

        assert!(result.is_ok());
        assert_eq!(attempts.load(std::sync::atomic::Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_timeout() {
        let result = with_timeout(
            Duration::from_millis(100),
            async {
                sleep(Duration::from_secs(1)).await;
                "done"
            },
        ).await;

        assert!(result.is_err());
    }
}
