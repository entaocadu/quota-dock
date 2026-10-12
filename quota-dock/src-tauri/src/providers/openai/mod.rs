use std::process::Stdio;
use std::time::Duration;
use serde::Deserialize;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

use crate::contracts::{
    Availability, OriginKind, ProviderId, QuotaSnapshot, QuotaWindow,
};
use crate::quota::normalization::{epoch_seconds_to_iso, to_remaining_from_used};
use crate::security::masking::hash_account_key;

#[derive(Debug, Deserialize)]
struct RpcResponse {
    id: Option<u64>,
    result: Option<RpcResult>,
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct RpcError {
    #[allow(dead_code)]
    code: Option<i64>,
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RpcResult {
    #[serde(rename = "ordinaryUsageAllowed")]
    ordinary_usage_allowed: Option<bool>,
    #[serde(rename = "rateLimits")]
    rate_limits: Option<RateLimits>,
    #[serde(rename = "accountId")]
    account_id: Option<String>,
    #[serde(rename = "rateLimitResetCredits")]
    rate_limit_reset_credits: Option<RateLimitResetCredits>,
}

#[derive(Debug, Deserialize)]
struct RateLimits {
    #[serde(rename = "limitId")]
    limit_id: Option<String>,
    primary: Option<WindowData>,
    secondary: Option<WindowData>,
    #[serde(rename = "planType")]
    plan_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WindowData {
    #[serde(rename = "usedPercent")]
    used_percent: Option<f64>,
    #[serde(rename = "windowDurationMins")]
    window_duration_mins: Option<i64>,
    #[serde(rename = "resetsAt")]
    resets_at: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct RateLimitResetCredits {
    #[serde(rename = "availableCount")]
    available_count: Option<u32>,
}

fn format_window_label(mins: Option<i64>, default_name: &str) -> String {
    match mins {
        Some(m) if m >= 1440 => {
            let days = m / 1440;
            format!("Janela de {}d", days)
        }
        Some(m) if m >= 60 => {
            let hours = m / 60;
            format!("Janela de {}h", hours)
        }
        Some(m) if m > 0 => format!("Janela de {}m", m),
        _ => default_name.to_string(),
    }
}

pub fn parse_codex_ratelimits_response(json_str: &str) -> Result<QuotaSnapshot, String> {
    let rpc: RpcResponse = serde_json::from_str(json_str)
        .map_err(|e| format!("Falha ao decodificar JSON-RPC do Codex: {}", e))?;

    if let Some(err) = rpc.error {
        let msg = err.message.unwrap_or_else(|| "Erro desconhecido retornado pelo app-server".to_string());
        return Ok(QuotaSnapshot {
            schema_version: 1,
            provider_id: ProviderId::OpenaiCodex,
            origin: OriginKind::CodexAppServer,
            account_key: "openai_error".to_string(),
            account_label: "OpenAI Codex".to_string(),
            status: Availability::UnauthorizedScope,
            observed_at: None,
            attempted_at: Some(chrono::Utc::now().to_rfc3339()),
            expires_at: None,
            source_version: Some("codex-cli/app-server".to_string()),
            note: Some(format!("Codex app-server retornou erro: {}", msg)),
            windows: vec![],
            models: vec![],
            warnings: vec![msg],
            raw_error_code: Some("RPC_ERROR".to_string()),
        });
    }

    let result = match rpc.result {
        Some(r) => r,
        None => return Err("Resposta JSON-RPC sem campo 'result'".to_string()),
    };

    let allowed = result.ordinary_usage_allowed.unwrap_or(true);
    let account_id_raw = result.account_id.unwrap_or_else(|| "codex_account".to_string());
    let account_key = hash_account_key(&account_id_raw);

    let mut windows = Vec::new();
    let mut warnings = Vec::new();

    let plan_name = result
        .rate_limits
        .as_ref()
        .and_then(|r| r.plan_type.clone())
        .unwrap_or_else(|| "chatgpt".to_string());

    let label = format!("Codex ({})", plan_name.to_uppercase());

    if let Some(credits) = result.rate_limit_reset_credits {
        if let Some(count) = credits.available_count {
            if count > 0 {
                warnings.push(format!("Créditos de reset disponíveis: {}", count));
            }
        }
    }

    if !allowed {
        warnings.push("Limite de uso ordinário atingido. Uso atualmente bloqueado.".to_string());
    }

    if let Some(rl) = result.rate_limits {
        let scope_id = rl.limit_id.unwrap_or_else(|| "codex".to_string());

        if let Some(primary) = rl.primary {
            let label = format_window_label(primary.window_duration_mins, "Janela Principal");
            let remaining = to_remaining_from_used(primary.used_percent);
            let window_secs = primary.window_duration_mins.map(|m| m * 60);
            let reset_iso = primary.resets_at.and_then(epoch_seconds_to_iso);

            windows.push(QuotaWindow {
                id: format!("{}/primary", scope_id),
                label,
                used_percent: primary.used_percent,
                remaining_percent: remaining,
                window_seconds: window_secs,
                reset_at: reset_iso,
                allowed: Some(allowed),
                scope: Some(scope_id.clone()),
            });
        }

        if let Some(secondary) = rl.secondary {
            let label = format_window_label(secondary.window_duration_mins, "Janela Secundária");
            let remaining = to_remaining_from_used(secondary.used_percent);
            let window_secs = secondary.window_duration_mins.map(|m| m * 60);
            let reset_iso = secondary.resets_at.and_then(epoch_seconds_to_iso);

            windows.push(QuotaWindow {
                id: format!("{}/secondary", scope_id),
                label,
                used_percent: secondary.used_percent,
                remaining_percent: remaining,
                window_seconds: window_secs,
                reset_at: reset_iso,
                allowed: Some(allowed),
                scope: Some(scope_id),
            });
        }
    }

    let status = if !allowed {
        Availability::RateLimited
    } else if windows.is_empty() {
        Availability::NoData
    } else {
        Availability::Ok
    };

    let now_iso = chrono::Utc::now().to_rfc3339();

    Ok(QuotaSnapshot {
        schema_version: 1,
        provider_id: ProviderId::OpenaiCodex,
        origin: OriginKind::CodexAppServer,
        account_key,
        account_label: label,
        status,
        observed_at: Some(now_iso.clone()),
        attempted_at: Some(now_iso),
        expires_at: Some(epoch_seconds_to_iso(chrono::Utc::now().timestamp() + 600).unwrap_or_default()),
        source_version: Some("codex-cli/app-server".to_string()),
        note: None,
        windows,
        models: vec![],
        warnings,
        raw_error_code: None,
    })
}

pub async fn query_codex_app_server() -> Result<QuotaSnapshot, String> {
    // Spawn `codex app-server`
    let mut child = Command::new("codex")
        .arg("app-server")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| {
            format!("Não foi possível executar 'codex app-server'. Verifique se o Codex CLI está instalado no PATH: {}", e)
        })?;

    let mut stdin = child.stdin.take().ok_or("Falha ao abrir stdin do processo codex")?;
    let stdout = child.stdout.take().ok_or("Falha ao abrir stdout do processo codex")?;
    let mut reader = BufReader::new(stdout).lines();

    // 1. Initialize
    let init_cmd = serde_json::json!({
        "method": "initialize",
        "id": 1,
        "params": {
            "clientInfo": {
                "name": "quota_dock",
                "title": "QuotaDock",
                "version": "0.1.0"
            }
        }
    });

    stdin
        .write_all(format!("{}\n", init_cmd).as_bytes())
        .await
        .map_err(|e| format!("Erro ao escrever initialize: {}", e))?;
    stdin.flush().await.map_err(|e| format!("Erro ao enviar initialize: {}", e))?;

    // Wait for id: 1 with timeout
    let mut got_init = false;
    let timeout = Duration::from_secs(5);

    let start = tokio::time::Instant::now();
    while start.elapsed() < timeout {
        if let Ok(Ok(Some(line))) = tokio::time::timeout(Duration::from_millis(500), reader.next_line()).await {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&line) {
                if val.get("id").and_then(|v| v.as_u64()) == Some(1) {
                    got_init = true;
                    break;
                }
            }
        }
    }

    if !got_init {
        let _ = child.kill().await;
        return Err("Timeout aguardando inicialização do Codex app-server".to_string());
    }

    // 2. Initialized + read rate limits
    let initialized_notify = serde_json::json!({
        "method": "initialized",
        "params": {}
    });
    let read_cmd = serde_json::json!({
        "method": "account/rateLimits/read",
        "id": 2
    });

    stdin
        .write_all(format!("{}\n{}\n", initialized_notify, read_cmd).as_bytes())
        .await
        .map_err(|e| format!("Erro ao enviar query de rate limits: {}", e))?;
    stdin.flush().await.map_err(|e| format!("Erro ao enviar query: {}", e))?;

    // Wait for id: 2
    let mut rate_limit_payload = None;
    let query_start = tokio::time::Instant::now();

    while query_start.elapsed() < timeout {
        if let Ok(Ok(Some(line))) = tokio::time::timeout(Duration::from_millis(500), reader.next_line()).await {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&line) {
                if val.get("id").and_then(|v| v.as_u64()) == Some(2) {
                    rate_limit_payload = Some(line);
                    break;
                }
            }
        }
    }

    // Kill process
    let _ = child.kill().await;

    match rate_limit_payload {
        Some(json) => parse_codex_ratelimits_response(&json),
        None => Err("Codex app-server não respondeu a requisição de rate limits em tempo hábil".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_codex_ratelimits_success() {
        let fixture = include_str!("../../../../tests/fixtures/codex_ratelimits_success.json");
        let snapshot = parse_codex_ratelimits_response(fixture).expect("Should parse");

        assert_eq!(snapshot.provider_id, ProviderId::OpenaiCodex);
        assert_eq!(snapshot.origin, OriginKind::CodexAppServer);
        assert_eq!(snapshot.status, Availability::Ok);
        assert_eq!(snapshot.windows.len(), 2);

        // Primary: 35% used -> 65% remaining, 300 mins -> 5h
        let w1 = &snapshot.windows[0];
        assert_eq!(w1.label, "Janela de 5h");
        assert_eq!(w1.used_percent, Some(35.0));
        assert_eq!(w1.remaining_percent, Some(65.0));
        assert_eq!(w1.window_seconds, Some(18000));
        assert!(w1.reset_at.is_some());

        // Secondary: 20% used -> 80% remaining, 10080 mins -> 7d
        let w2 = &snapshot.windows[1];
        assert_eq!(w2.label, "Janela de 7d");
        assert_eq!(w2.used_percent, Some(20.0));
        assert_eq!(w2.remaining_percent, Some(80.0));
        assert_eq!(w2.window_seconds, Some(604800));

        assert!(snapshot.warnings.iter().any(|w| w.contains("Créditos de reset")));
    }

    #[test]
    fn test_parse_codex_ratelimits_blocked() {
        let fixture = include_str!("../../../../tests/fixtures/codex_ratelimits_blocked.json");
        let snapshot = parse_codex_ratelimits_response(fixture).expect("Should parse");

        assert_eq!(snapshot.status, Availability::RateLimited);
        assert!(snapshot.warnings.iter().any(|w| w.contains("bloqueado")));
    }
}
