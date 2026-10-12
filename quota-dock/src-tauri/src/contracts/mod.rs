use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderId {
    OpenaiCodex,
    GoogleAntigravity,
    OpencodeLocal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OriginKind {
    OpencodeOpenaiUsage,
    CodexLocalUsage,
    OpenaiQuotaLogin,
    CodexAppServer,
    CodexWebManual,
    OpenaiUndocumentedOptIn,
    AntigravityPluginCache,
    AntigravityPrivateOptIn,
    OpencodeStats,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Availability {
    Ok,
    Partial,
    Stale,
    Disabled,
    NotConfigured,
    NotInstalled,
    UnauthorizedScope,
    Forbidden,
    RateLimited,
    Offline,
    NetworkError,
    SourceChanged,
    NoData,
    UnknownError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaSnapshot {
    pub schema_version: u32,
    pub provider_id: ProviderId,
    pub origin: OriginKind,
    pub account_key: String,
    pub account_label: String,
    pub status: Availability,
    pub observed_at: Option<String>,
    pub attempted_at: Option<String>,
    pub expires_at: Option<String>,
    pub source_version: Option<String>,
    pub note: Option<String>,
    pub windows: Vec<QuotaWindow>,
    pub models: Vec<ModelQuota>,
    pub warnings: Vec<String>,
    pub raw_error_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaWindow {
    pub id: String,
    pub label: String,
    pub used_percent: Option<f64>,
    pub remaining_percent: Option<f64>,
    pub window_seconds: Option<i64>,
    pub reset_at: Option<String>,
    pub allowed: Option<bool>,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelQuota {
    pub model_id: String,
    pub display_name: String,
    pub remaining_percent: Option<f64>,
    pub reset_at: Option<String>,
    pub group_label: Option<String>,
    pub source_exact: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalTokens {
    pub schema_version: u32,
    pub origin: String,
    pub range_start: String,
    pub range_end: String,
    pub input: Option<i64>,
    pub output: Option<i64>,
    pub reasoning: Option<i64>,
    pub cache_read: Option<i64>,
    pub cache_write: Option<i64>,
    pub total: Option<i64>,
    pub models: Vec<ModelTokenUsage>,
    pub completeness: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelTokenUsage {
    pub model_id: String,
    pub tokens: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderDiag {
    pub provider_id: ProviderId,
    pub status: Availability,
    pub origin: OriginKind,
    pub last_success: Option<String>,
    pub last_attempt: Option<String>,
    pub error_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SanitizedDiagnostics {
    pub timestamp: String,
    pub app_version: String,
    pub os: String,
    pub providers: Vec<ProviderDiag>,
    pub opencode_db_detected: bool,
    pub antigravity_accounts_detected: bool,
    pub codex_cli_detected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicSettings {
    pub start_with_windows: bool,
    pub theme: String,
    pub google_live_opt_in: bool,
    pub poll_interval_mins: u32,
    pub notifications_enabled: bool,
    pub notify_threshold_25: bool,
    pub notify_threshold_10: bool,
    pub notify_threshold_0: bool,
    pub pinned: bool,
}

impl Default for PublicSettings {
    fn default() -> Self {
        Self {
            start_with_windows: true,
            theme: "carbon-dark".to_string(),
            google_live_opt_in: true,
            poll_interval_mins: 1,
            notifications_enabled: false,
            notify_threshold_25: false,
            notify_threshold_10: false,
            notify_threshold_0: false,
            pinned: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SafeSettingsPatch {
    pub start_with_windows: Option<bool>,
    pub theme: Option<String>,
    pub google_live_opt_in: Option<bool>,
    pub poll_interval_mins: Option<u32>,
    pub notifications_enabled: Option<bool>,
    pub notify_threshold_25: Option<bool>,
    pub notify_threshold_10: Option<bool>,
    pub notify_threshold_0: Option<bool>,
    pub pinned: Option<bool>,
}
