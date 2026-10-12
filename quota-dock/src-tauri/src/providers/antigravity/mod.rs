use std::path::{Path, PathBuf};
use serde::Deserialize;
use std::collections::HashMap;

use crate::contracts::{
    Availability, ModelQuota, OriginKind, ProviderId, QuotaSnapshot,
};
use crate::quota::normalization::{epoch_ms_to_iso, to_remaining_from_fraction};
use crate::security::masking::{hash_account_key, mask_email};

#[derive(Debug, Deserialize)]
struct AccountsFile {
    accounts: Option<Vec<AccountEntry>>,
}

#[derive(Debug, Deserialize)]
struct AccountEntry {
    email: Option<String>,
    enabled: Option<bool>,
    #[serde(rename = "cachedQuota")]
    cached_quota: Option<HashMap<String, ModelQuotaRaw>>,
    #[serde(rename = "cachedQuotaUpdatedAt")]
    cached_quota_updated_at: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct ModelQuotaRaw {
    #[serde(rename = "remainingFraction")]
    remaining_fraction: Option<f64>,
    #[serde(rename = "resetTime")]
    reset_time: Option<String>,
    #[serde(rename = "modelCount")]
    #[allow(dead_code)]
    model_count: Option<u32>,
}

pub fn get_default_accounts_path() -> PathBuf {
    if let Ok(config_dir) = std::env::var("OPENCODE_CONFIG_DIR") {
        let p = PathBuf::from(config_dir).join("antigravity-accounts.json");
        if p.exists() {
            return p;
        }
    }

    if let Ok(user_profile) = std::env::var("USERPROFILE") {
        let p = PathBuf::from(user_profile)
            .join(".config")
            .join("opencode")
            .join("antigravity-accounts.json");
        if p.exists() {
            return p;
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        let p = PathBuf::from(home)
            .join(".config")
            .join("opencode")
            .join("antigravity-accounts.json");
        if p.exists() {
            return p;
        }
    }

    // Default target even if not yet created
    let user_profile = std::env::var("USERPROFILE").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(user_profile)
        .join(".config")
        .join("opencode")
        .join("antigravity-accounts.json")
}

pub fn parse_antigravity_accounts(
    json_str: &str,
    current_time_ms: i64,
) -> Result<Vec<QuotaSnapshot>, String> {
    let parsed: AccountsFile = match serde_json::from_str(json_str) {
        Ok(f) => f,
        Err(e) => return Err(format!("Falha no parser JSON do cache Google: {}", e)),
    };

    let accounts = parsed.accounts.unwrap_or_default();
    if accounts.is_empty() {
        return Ok(vec![QuotaSnapshot {
            schema_version: 1,
            provider_id: ProviderId::GoogleAntigravity,
            origin: OriginKind::AntigravityPluginCache,
            account_key: "google_no_accounts".to_string(),
            account_label: "Google (Sem contas conectadas)".to_string(),
            status: Availability::NotConfigured,
            observed_at: None,
            attempted_at: Some(epoch_ms_to_iso(current_time_ms).unwrap_or_default()),
            expires_at: None,
            source_version: Some("pieliesdie/opencode-antigravity-auth".to_string()),
            note: Some("Nenhuma conta encontrada em antigravity-accounts.json".to_string()),
            windows: vec![],
            models: vec![],
            warnings: vec!["Nenhuma conta Google configurada no plugin Antigravity".to_string()],
            raw_error_code: None,
        }]);
    }

    let mut snapshots = Vec::new();

    for acc in accounts {
        let email_raw = acc.email.unwrap_or_else(|| "conta@google.com".to_string());
        let label = mask_email(&email_raw);
        let key = hash_account_key(&email_raw);

        let mut warnings = Vec::new();
        let mut models = Vec::new();

        let updated_at_ms = acc.cached_quota_updated_at.unwrap_or(0);
        let observed_at = if updated_at_ms > 0 {
            epoch_ms_to_iso(updated_at_ms)
        } else {
            None
        };

        // Determine staleness
        let status = if updated_at_ms <= 0 {
            warnings.push("Não há snapshot local registrado pelo plugin.".to_string());
            Availability::NoData
        } else {
            let age_ms = (current_time_ms - updated_at_ms).max(0);
            let age_mins = age_ms / (60 * 1000);
            if age_mins > 1440 {
                // > 24 hours
                warnings.push(format!(
                    "Snapshot local expirado ({} horas atrás). Não reflete limites atuais.",
                    age_mins / 60
                ));
                Availability::Stale
            } else if age_mins > 15 {
                warnings.push(format!(
                    "Snapshot local de {} minutos atrás. Pode variar por uso recente.",
                    age_mins
                ));
                Availability::Stale
            } else {
                Availability::Ok
            }
        };

        if let Some(quota_map) = acc.cached_quota {
            let model_keys = ["gemini-flash", "gemini-pro", "claude"];
            for m_key in &model_keys {
                if let Some(q) = quota_map.get(*m_key) {
                    let display_name = match *m_key {
                        "gemini-flash" => "Gemini 3.8 Flash",
                        "gemini-pro" => "Gemini Pro",
                        "claude" => "Claude (Antigravity)",
                        _ => *m_key,
                    };
                    let group_label = match *m_key {
                        "gemini-flash" => "Gemini Flash",
                        "gemini-pro" => "Gemini Pro",
                        "claude" => "Claude",
                        _ => "Outros",
                    };

                    models.push(ModelQuota {
                        model_id: format!("antigravity-{}", m_key),
                        display_name: display_name.to_string(),
                        remaining_percent: to_remaining_from_fraction(q.remaining_fraction),
                        reset_at: q.reset_time.clone(),
                        group_label: Some(group_label.to_string()),
                        source_exact: true,
                    });
                }
            }
        }

        if models.is_empty() && status != Availability::NoData {
            warnings.push("Nenhum modelo com quota encontrado no snapshot.".to_string());
        }

        snapshots.push(QuotaSnapshot {
            schema_version: 1,
            provider_id: ProviderId::GoogleAntigravity,
            origin: OriginKind::AntigravityPluginCache,
            account_key: key,
            account_label: label,
            status,
            observed_at,
            attempted_at: Some(epoch_ms_to_iso(current_time_ms).unwrap_or_default()),
            expires_at: if updated_at_ms > 0 {
                epoch_ms_to_iso(updated_at_ms + 15 * 60 * 1000)
            } else {
                None
            },
            source_version: Some("pieliesdie/opencode-antigravity-auth".to_string()),
            note: if acc.enabled == Some(false) {
                Some("Conta desativada no OpenCode".to_string())
            } else {
                None
            },
            windows: vec![],
            models,
            warnings,
            raw_error_code: None,
        });
    }

    Ok(snapshots)
}

pub fn read_local_antigravity_cache(
    custom_path: Option<&Path>,
) -> Result<Vec<QuotaSnapshot>, String> {
    let path = match custom_path {
        Some(p) => p.to_path_buf(),
        None => get_default_accounts_path(),
    };

    if !path.exists() {
        return Ok(vec![QuotaSnapshot {
            schema_version: 1,
            provider_id: ProviderId::GoogleAntigravity,
            origin: OriginKind::AntigravityPluginCache,
            account_key: "google_file_missing".to_string(),
            account_label: "Google Antigravity".to_string(),
            status: Availability::NotConfigured,
            observed_at: None,
            attempted_at: Some(chrono::Utc::now().to_rfc3339()),
            expires_at: None,
            source_version: Some("pieliesdie/opencode-antigravity-auth".to_string()),
            note: Some(format!(
                "Arquivo antigravity-accounts.json não encontrado no caminho padrão: {:?}",
                path
            )),
            windows: vec![],
            models: vec![],
            warnings: vec!["Arquivo de contas do plugin não encontrado.".to_string()],
            raw_error_code: Some("FILE_NOT_FOUND".to_string()),
        }]);
    }

    // Safety: Cap file size to 10MB
    let metadata = std::fs::metadata(&path)
        .map_err(|e| format!("Erro ao obter metadados do arquivo: {}", e))?;
    if metadata.len() > 10 * 1024 * 1024 {
        return Err("Arquivo antigravity-accounts.json excede o limite seguro de 10MB".to_string());
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Erro de leitura somente-leitura do arquivo: {}", e))?;

    let now_ms = chrono::Utc::now().timestamp_millis();
    parse_antigravity_accounts(&content, now_ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_cache() {
        let fixture = include_str!("../../../../tests/fixtures/antigravity_valid.json");
        // Simulated current time: 5 minutes after cachedQuotaUpdatedAt (1791752000000 + 300_000)
        let now_ms = 1791752300000;
        let snapshots = parse_antigravity_accounts(fixture, now_ms).expect("Should parse");

        assert_eq!(snapshots.len(), 1);
        let s = &snapshots[0];
        assert_eq!(s.provider_id, ProviderId::GoogleAntigravity);
        assert_eq!(s.origin, OriginKind::AntigravityPluginCache);
        assert_eq!(s.status, Availability::Ok);
        assert_eq!(s.account_label, "d***@example.com");
        assert_eq!(s.models.len(), 3);

        // Flash: 0.42 -> 42.0
        let flash = s.models.iter().find(|m| m.model_id == "antigravity-gemini-flash").unwrap();
        assert_eq!(flash.remaining_percent, Some(42.0));
        assert_eq!(flash.display_name, "Gemini 3.8 Flash");
        assert_eq!(flash.reset_at.as_deref(), Some("2026-10-11T23:30:00Z"));

        // Pro: 0.85 -> 85.0
        let pro = s.models.iter().find(|m| m.model_id == "antigravity-gemini-pro").unwrap();
        assert_eq!(pro.remaining_percent, Some(85.0));

        // Claude: 0.15 -> 15.0
        let claude = s.models.iter().find(|m| m.model_id == "antigravity-claude").unwrap();
        assert_eq!(claude.remaining_percent, Some(15.0));
    }

    #[test]
    fn test_parse_stale_cache() {
        let fixture = include_str!("../../../../tests/fixtures/antigravity_stale.json");
        // Simulated current time: 60 minutes after cachedQuotaUpdatedAt (1780000000000 + 3_600_000)
        let now_ms = 1780003600000;
        let snapshots = parse_antigravity_accounts(fixture, now_ms).expect("Should parse");

        assert_eq!(snapshots.len(), 1);
        let s = &snapshots[0];
        assert_eq!(s.status, Availability::Stale);
        assert_eq!(s.account_label, "s***@example.com");
        assert!(!s.warnings.is_empty());
        assert!(s.warnings[0].contains("60 minutos"));
    }

    #[test]
    fn test_parse_no_quota() {
        let fixture = include_str!("../../../../tests/fixtures/antigravity_no_quota.json");
        let now_ms = 1790000000000;
        let snapshots = parse_antigravity_accounts(fixture, now_ms).expect("Should parse");

        assert_eq!(snapshots.len(), 1);
        let s = &snapshots[0];
        assert_eq!(s.status, Availability::NoData);
        assert!(s.models.is_empty());
    }
}
