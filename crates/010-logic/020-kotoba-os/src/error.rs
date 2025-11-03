//! Error handling and retry mechanisms for KotobaOS
//!
//! Provides comprehensive error classification, retry logic with exponential backoff,
//! and error escalation mechanisms.

use crate::KotobaOsError;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{error, warn, info};

/// Error category for classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// Transient error (can be retried)
    Transient,
    /// Permanent error (should not be retried)
    Permanent,
    /// System error (may require escalation)
    System,
    /// Validation error (data issue)
    Validation,
    /// Network/IO error (may be transient)
    Network,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial delay between retries (seconds)
    pub initial_delay_secs: u64,
    /// Maximum delay between retries (seconds)
    pub max_delay_secs: u64,
    /// Exponential backoff multiplier
    pub backoff_multiplier: f64,
    /// Retryable error categories
    pub retryable_categories: Vec<ErrorCategory>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_secs: 1,
            max_delay_secs: 60,
            backoff_multiplier: 2.0,
            retryable_categories: vec![
                ErrorCategory::Transient,
                ErrorCategory::Network,
            ],
        }
    }
}

/// Error context for tracking and escalation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    /// Error category
    pub category: ErrorCategory,
    /// Original error message
    pub message: String,
    /// Retry attempt count
    pub retry_count: u32,
    /// Process ID (if applicable)
    pub process_id: Option<String>,
    /// Timestamp
    pub timestamp: String,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new(category: ErrorCategory, message: String) -> Self {
        Self {
            category,
            message,
            retry_count: 0,
            process_id: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Create error context from KotobaOsError
    pub fn from_error(err: &KotobaOsError, process_id: Option<String>) -> Self {
        let (category, message) = match err {
            KotobaOsError::JsonLdParse(e) => {
                (ErrorCategory::Validation, format!("JSON-LD parsing error: {}", e))
            }
            KotobaOsError::StoryValidation(msg) => {
                (ErrorCategory::Validation, format!("Story validation error: {}", msg))
            }
            KotobaOsError::ActorSelection(msg) => {
                (ErrorCategory::System, format!("Actor selection error: {}", msg))
            }
            KotobaOsError::ProcessExecution(msg) => {
                (ErrorCategory::Transient, format!("Process execution error: {}", msg))
            }
            KotobaOsError::ProvenanceError(msg) => {
                (ErrorCategory::System, format!("Provenance error: {}", msg))
            }
            KotobaOsError::Io(e) => {
                (ErrorCategory::Network, format!("IO error: {}", e))
            }
            KotobaOsError::Other(e) => {
                (ErrorCategory::System, format!("Other error: {}", e))
            }
        };

        Self {
            category,
            message,
            retry_count: 0,
            process_id,
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Check if error is retryable
    pub fn is_retryable(&self, config: &RetryConfig) -> bool {
        config.retryable_categories.contains(&self.category)
            && self.retry_count < config.max_retries
    }

    /// Increment retry count
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }
}

/// Retry executor with exponential backoff
pub struct RetryExecutor {
    config: RetryConfig,
}

impl RetryExecutor {
    /// Create a new retry executor
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(RetryConfig::default())
    }

    /// Execute a function with retry logic
    pub async fn execute<F, T, E>(
        &self,
        operation: F,
        operation_name: &str,
    ) -> Result<T, ErrorContext>
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
        E: std::fmt::Display,
    {
        let mut last_error: Option<ErrorContext> = None;

        for attempt in 0..=self.config.max_retries {
            match operation().await {
                Ok(result) => {
                    if attempt > 0 {
                        info!(
                            "[RetryExecutor] Operation '{}' succeeded after {} retries",
                            operation_name, attempt
                        );
                    }
                    return Ok(result);
                }
                Err(e) => {
                    let mut error_ctx = ErrorContext::new(
                        ErrorCategory::Transient, // Default category
                        format!("{}", e),
                    );
                    error_ctx.retry_count = attempt;

                    if !error_ctx.is_retryable(&self.config) {
                        error!(
                            "[RetryExecutor] Operation '{}' failed after {} attempts: {}",
                            operation_name, attempt, e
                        );
                        return Err(error_ctx);
                    }

                    if attempt < self.config.max_retries {
                        let delay = self.calculate_delay(attempt);
                        warn!(
                            "[RetryExecutor] Operation '{}' failed (attempt {}/{}), retrying in {:?}: {}",
                            operation_name,
                            attempt + 1,
                            self.config.max_retries,
                            delay,
                            e
                        );
                        tokio::time::sleep(delay).await;
                    }

                    last_error = Some(error_ctx);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            ErrorContext::new(
                ErrorCategory::Permanent,
                format!("Operation '{}' failed after all retries", operation_name),
            )
        }))
    }

    /// Calculate delay for exponential backoff
    fn calculate_delay(&self, attempt: u32) -> Duration {
        let delay_secs = (self.config.initial_delay_secs as f64
            * self.config.backoff_multiplier.powi(attempt as i32))
            .min(self.config.max_delay_secs as f64) as u64;
        Duration::from_secs(delay_secs)
    }
}

/// Error escalation levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EscalationLevel {
    /// No escalation needed
    None,
    /// Log warning
    Warning,
    /// Log error and notify
    Error,
    /// Critical: requires immediate attention
    Critical,
}

/// Error escalation handler
pub struct ErrorEscalator {
    /// Threshold for error escalation
    pub error_threshold: u32,
    /// Threshold for critical escalation
    pub critical_threshold: u32,
}

impl ErrorEscalator {
    /// Create a new error escalator
    pub fn new(error_threshold: u32, critical_threshold: u32) -> Self {
        Self {
            error_threshold,
            critical_threshold,
        }
    }

    /// Determine escalation level based on error context
    pub fn escalate(&self, error_ctx: &ErrorContext) -> EscalationLevel {
        match error_ctx.category {
            ErrorCategory::System => {
                if error_ctx.retry_count >= self.critical_threshold {
                    EscalationLevel::Critical
                } else if error_ctx.retry_count >= self.error_threshold {
                    EscalationLevel::Error
                } else {
                    EscalationLevel::Warning
                }
            }
            ErrorCategory::Validation => EscalationLevel::Error,
            ErrorCategory::Permanent => EscalationLevel::Error,
            _ => {
                if error_ctx.retry_count >= self.error_threshold {
                    EscalationLevel::Warning
                } else {
                    EscalationLevel::None
                }
            }
        }
    }

    /// Handle escalation
    pub fn handle_escalation(&self, error_ctx: &ErrorContext, level: EscalationLevel) {
        match level {
            EscalationLevel::None => {}
            EscalationLevel::Warning => {
                warn!(
                    "[ErrorEscalator] Warning: {} (retry: {})",
                    error_ctx.message, error_ctx.retry_count
                );
            }
            EscalationLevel::Error => {
                error!(
                    "[ErrorEscalator] Error: {} (retry: {}, process: {:?})",
                    error_ctx.message,
                    error_ctx.retry_count,
                    error_ctx.process_id
                );
            }
            EscalationLevel::Critical => {
                error!(
                    "[ErrorEscalator] CRITICAL: {} (retry: {}, process: {:?})",
                    error_ctx.message,
                    error_ctx.retry_count,
                    error_ctx.process_id
                );
                // In a production system, this would trigger alerts/notifications
            }
        }
    }
}

impl Default for ErrorEscalator {
    fn default() -> Self {
        Self::new(2, 5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_retry_executor_success() {
        let executor = RetryExecutor::default();
        let mut attempt = 0;

        let result = executor
            .execute(
                || {
                    attempt += 1;
                    Box::pin(async move {
                        if attempt < 2 {
                            Err::<(), _>("Temporary failure")
                        } else {
                            Ok(())
                        }
                    })
                },
                "test_operation",
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(attempt, 2);
    }

    #[tokio::test]
    async fn test_retry_executor_max_retries() {
        let executor = RetryExecutor::default();

        let result = executor
            .execute(
                || Box::pin(async move { Err::<(), _>("Always fails") }),
                "test_operation",
            )
            .await;

        assert!(result.is_err());
        let error_ctx = result.unwrap_err();
        assert_eq!(error_ctx.retry_count, executor.config.max_retries);
    }

    #[test]
    fn test_error_context_retryable() {
        let config = RetryConfig::default();
        let mut error_ctx = ErrorContext::new(
            ErrorCategory::Transient,
            "Test error".to_string(),
        );

        assert!(error_ctx.is_retryable(&config));

        error_ctx.retry_count = config.max_retries;
        assert!(!error_ctx.is_retryable(&config));
    }

    #[test]
    fn test_error_escalator() {
        let escalator = ErrorEscalator::default();
        let mut error_ctx = ErrorContext::new(
            ErrorCategory::System,
            "System error".to_string(),
        );

        assert_eq!(escalator.escalate(&error_ctx), EscalationLevel::Warning);

        error_ctx.retry_count = escalator.error_threshold;
        assert_eq!(escalator.escalate(&error_ctx), EscalationLevel::Error);

        error_ctx.retry_count = escalator.critical_threshold;
        assert_eq!(escalator.escalate(&error_ctx), EscalationLevel::Critical);
    }
}

