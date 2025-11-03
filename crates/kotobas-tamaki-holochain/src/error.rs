//! エラーハンドリングとリトライロジック
//!
//! Holochain Kotobasos用のエラーハンドリングとリトライ機能を提供します。

use crate::HolochainKotobasosError;
use std::time::Duration;
use tracing::{warn, error};

/// リトライ設定
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// 最大リトライ回数
    pub max_retries: u32,
    /// 初期リトライ間隔（秒）
    pub initial_delay: Duration,
    /// 最大リトライ間隔（秒）
    pub max_delay: Duration,
    /// 指数バックオフの倍率
    pub backoff_multiplier: f64,
    /// リトライ可能なエラーかどうかを判定する関数
    pub retryable_error: fn(&HolochainKotobasosError) -> bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            retryable_error: |e| {
                matches!(
                    e,
                    HolochainKotobasosError::Dht(_) | 
                    HolochainKotobasosError::Hdk(_)
                )
            },
        }
    }
}

/// リトライ実行器
pub struct RetryExecutor {
    config: RetryConfig,
}

impl RetryExecutor {
    /// 新しいリトライ実行器を作成
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// デフォルト設定でリトライ実行器を作成
    pub fn default() -> Self {
        Self::new(RetryConfig::default())
    }

    /// リトライ可能な操作を実行
    pub async fn execute<F, Fut, T>(&self, operation: F) -> Result<T, HolochainKotobasosError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, HolochainKotobasosError>> + Send,
    {
        let mut delay = self.config.initial_delay;
        let mut last_error: Option<String> = None;

        for attempt in 0..=self.config.max_retries {
            match operation().await {
                Ok(result) => {
                    if attempt > 0 {
                        warn!("Operation succeeded after {} retries", attempt);
                    }
                    return Ok(result);
                }
                Err(e) => {
                    // リトライ可能かどうかを確認
                    if !(self.config.retryable_error)(&e) {
                        return Err(e);
                    }

                    last_error = Some(format!("{:?}", e));

                    if attempt < self.config.max_retries {
                        warn!(
                            "Operation failed (attempt {}/{}), retrying in {:?}",
                            attempt + 1,
                            self.config.max_retries,
                            delay
                        );
                        tokio::time::sleep(delay).await;
                        delay = std::cmp::min(
                            Duration::from_secs_f64(
                                delay.as_secs_f64() * self.config.backoff_multiplier
                            ),
                            self.config.max_delay,
                        );
                    }
                }
            }
        }

        error!("Operation failed after {} retries: {:?}", self.config.max_retries, last_error);
        Err(crate::HolochainKotobasosError::Other(anyhow::anyhow!(
            "Operation failed after {} retries: {:?}",
            self.config.max_retries,
            last_error
        )))
    }
}

/// エラーエスカレーション設定
#[derive(Debug, Clone)]
pub struct EscalationConfig {
    /// エスカレーションレベル
    pub levels: Vec<EscalationLevel>,
}

/// エスカレーションレベル
#[derive(Debug, Clone)]
pub struct EscalationLevel {
    /// エラーの種類
    pub error_type: String,
    /// エスカレーション先（ログ、メトリクス、通知など）
    pub targets: Vec<EscalationTarget>,
}

/// エスカレーション先
#[derive(Debug, Clone)]
pub enum EscalationTarget {
    /// ログに記録
    Log,
    /// メトリクスに記録
    Metrics,
    /// 通知を送信
    Notification(String),
}

/// エラーエスカレーター
pub struct ErrorEscalator {
    config: EscalationConfig,
}

impl ErrorEscalator {
    /// 新しいエラーエスカレーターを作成
    pub fn new(config: EscalationConfig) -> Self {
        Self { config }
    }

    /// デフォルト設定でエラーエスカレーターを作成
    pub fn default() -> Self {
        Self::new(EscalationConfig {
            levels: vec![
                EscalationLevel {
                    error_type: "Dht".to_string(),
                    targets: vec![EscalationTarget::Log, EscalationTarget::Metrics],
                },
                EscalationLevel {
                    error_type: "Hdk".to_string(),
                    targets: vec![EscalationTarget::Log, EscalationTarget::Metrics],
                },
                EscalationLevel {
                    error_type: "Kernel".to_string(),
                    targets: vec![
                        EscalationTarget::Log,
                        EscalationTarget::Metrics,
                        EscalationTarget::Notification("kernel_errors".to_string()),
                    ],
                },
            ],
        })
    }

    /// エラーをエスカレート
    pub fn escalate(&self, error: &HolochainKotobasosError) {
        let error_type = match error {
            HolochainKotobasosError::Hdk(_) => "Hdk",
            HolochainKotobasosError::Dht(_) => "Dht",
            HolochainKotobasosError::Kernel(_) => "Kernel",
            HolochainKotobasosError::Actor(_) => "Actor",
            HolochainKotobasosError::Provenance(_) => "Provenance",
            HolochainKotobasosError::Evolution(_) => "Evolution",
            HolochainKotobasosError::MerkleDag(_) => "MerkleDag",
            HolochainKotobasosError::Serialization(_) => "Serialization",
            HolochainKotobasosError::Other(_) => "Other",
        };

        for level in &self.config.levels {
            if level.error_type == error_type {
                for target in &level.targets {
                    match target {
                        EscalationTarget::Log => {
                            error!("Escalated error: {:?}", error);
                        }
                        EscalationTarget::Metrics => {
                            // TODO: メトリクスに記録
                            warn!("Metrics recording not implemented yet");
                        }
                        EscalationTarget::Notification(channel) => {
                            // TODO: 通知を送信
                            warn!("Notification to {} not implemented yet", channel);
                        }
                    }
                }
            }
        }
    }
}

