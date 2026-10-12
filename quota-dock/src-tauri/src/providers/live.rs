use crate::contracts::{
    Availability, ModelQuota, OriginKind, ProviderId, QuotaSnapshot, QuotaWindow,
};
use crate::providers::antigravity::{get_default_accounts_path, parse_antigravity_accounts};
use crate::quota::normalization::{epoch_seconds_to_iso, to_remaining_from_used};
use crate::security::masking::{hash_account_key, mask_email};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::Utc;

use reqwest::{Client, StatusCode};
use rusqlite::{Connection, OpenFlags};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::Duration,
};

pub fn client() -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(15))
        .connect_timeout(Duration::from_secs(5))
        .redirect(reqwest::redirect::Policy::none())
        .https_only(true)
        .user_agent("QuotaDock/0.2.0")
        .build()
        .map_err(|_| "Falha ao iniciar conexão segura".into())
}
fn read_json(path: &Path) -> Result<Value, String> {
    let size = std::fs::metadata(path)
        .map_err(|_| "Arquivo de login não encontrado")?
        .len();
    if size > 10 * 1024 * 1024 {
        return Err("Arquivo de login excede 10 MB".into());
    }
    serde_json::from_slice(&std::fs::read(path).map_err(|_| "Falha ao ler login local")?)
        .map_err(|_| "Arquivo de login inválido".into())
}
pub fn failure(
    provider: ProviderId,
    origin: OriginKind,
    status: Availability,
    note: &str,
) -> QuotaSnapshot {
    QuotaSnapshot {
        schema_version: 1,
        provider_id: provider,
        origin,
        account_key: "source-error".into(),
        account_label: "Consulta da conta".into(),
        status,
        observed_at: None,
        attempted_at: Some(Utc::now().to_rfc3339()),
        expires_at: None,
        source_version: None,
        note: Some(note.into()),
        windows: vec![],
        models: vec![],
        warnings: vec![],
        raw_error_code: None,
    }
}
pub fn status_for_http(status: StatusCode) -> Availability {
    match status.as_u16() {
        401 => Availability::UnauthorizedScope,
        403 => Availability::Forbidden,
        429 => Availability::RateLimited,
        500..=599 => Availability::NetworkError,
        _ => Availability::SourceChanged,
    }
}
pub(super) async fn response_json(response: reqwest::Response) -> Result<Value, String> {
    if !response.status().is_success() {
        return Err(format!(
            "O provedor retornou HTTP {}",
            response.status().as_u16()
        ));
    }
    // Never include response bodies, headers or credentials in errors.
    if response
        .content_length()
        .is_some_and(|n| n > 5 * 1024 * 1024)
    {
        return Err("Resposta excede o limite de leitura".into());
    }
    response
        .json()
        .await
        .map_err(|_| "O provedor retornou uma resposta inválida".into())
}
fn group(id: &str, name: &str) -> Option<String> {
    let combined = format!("{} {}", id, name).to_lowercase();
    if combined.contains("claude") {
        Some("Claude".into())
    } else if combined.contains("gemini") && combined.contains("flash") {
        Some("Gemini Flash".into())
    } else if combined.contains("gemini") && combined.contains("pro") {
        Some("Gemini Pro".into())
    } else {
        None
    }
}
pub fn parse_google_models(value: &Value) -> Result<Vec<ModelQuota>, String> {
    let raw = value
        .get("models")
        .and_then(Value::as_object)
        .ok_or("Formato de quota Google alterado")?;
    let mut models = Vec::new();
    for (id, m) in raw {
        let name = m.get("displayName").and_then(Value::as_str).unwrap_or(id);
        let Some(family) = group(id, name) else {
            continue;
        };
        let Some(q) = m.get("quotaInfo") else {
            continue;
        };
        let fraction = q
            .get("remainingFraction")
            .and_then(Value::as_f64)
            .filter(|v| v.is_finite() && *v >= 0.0 && *v <= 1.0);
        let reset = q
            .get("resetTime")
            .and_then(Value::as_str)
            .filter(|s| chrono::DateTime::parse_from_rfc3339(s).is_ok());
        models.push(ModelQuota {
            model_id: id.clone(),
            display_name: name.into(),
            remaining_percent: fraction.map(|v| v * 100.0),
            reset_at: reset.map(str::to_owned),
            group_label: Some(family),
            source_exact: true,
        });
    }
    models.sort_by(|a, b| {
        a.group_label
            .cmp(&b.group_label)
            .then(a.model_id.cmp(&b.model_id))
    });
    Ok(models)
}
fn label(seconds: Option<i64>, fallback: &str) -> String {
    match seconds {
        Some(s) if s > 0 && s % 86400 == 0 => format!("{} dias", s / 86400),
        Some(s) if s > 0 && s % 3600 == 0 => format!("{} horas", s / 3600),
        Some(s) if s > 0 && s % 60 == 0 => format!("{} minutos", s / 60),
        Some(s) if s > 0 => format!("{} segundos", s),
        _ => fallback.into(),
    }
}
pub fn parse_openai_usage(value: &Value) -> Result<QuotaSnapshot, String> {
    let base = value
        .get("rate_limit")
        .filter(|v| v.is_object())
        .ok_or("Formato de quota OpenAI alterado")?;
    let mut s = failure(
        ProviderId::OpenaiCodex,
        OriginKind::CodexLocalUsage,
        Availability::NoData,
        "",
    );
    let account = value
        .get("account_id")
        .and_then(Value::as_str)
        .or_else(|| value.get("user_id").and_then(Value::as_str))
        .unwrap_or("local-openai");
    s.account_key = hash_account_key(account);
    s.account_label = value
        .get("email")
        .and_then(Value::as_str)
        .map(mask_email)
        .unwrap_or_else(|| "Conta local".into());
    s.note = None;
    let mut buckets: Vec<(String, String, &Value)> = vec![("codex".into(), "".into(), base)];
    if let Some(extra) = value
        .get("additional_rate_limits")
        .and_then(Value::as_array)
    {
        for (i, b) in extra.iter().enumerate() {
            if let Some(rl) = b.get("rate_limit").filter(|v| v.is_object()) {
                buckets.push((
                    b.get("metered_feature")
                        .and_then(Value::as_str)
                        .map(str::to_owned)
                        .unwrap_or_else(|| format!("extra-{}", i)),
                    b.get("limit_name")
                        .and_then(Value::as_str)
                        .unwrap_or("Limite adicional")
                        .into(),
                    rl,
                ));
            }
        }
    }
    for (scope, title, rl) in buckets {
        for (key, fallback) in [
            ("primary_window", "Janela principal"),
            ("secondary_window", "Janela secundária"),
        ] {
            if let Some(w) = rl.get(key).filter(|v| v.is_object()) {
                let used = w
                    .get("used_percent")
                    .and_then(Value::as_f64)
                    .filter(|v| v.is_finite() && *v >= 0.0);
                let secs = w
                    .get("limit_window_seconds")
                    .and_then(Value::as_i64)
                    .filter(|s| *s > 0);
                let window_label = label(secs, fallback);
                s.windows.push(QuotaWindow {
                    id: format!("{}/{}", scope, key),
                    label: if title.is_empty() {
                        window_label
                    } else {
                        format!("{} · {}", title, window_label)
                    },
                    used_percent: used,
                    remaining_percent: to_remaining_from_used(used),
                    window_seconds: secs,
                    reset_at: w
                        .get("reset_at")
                        .and_then(Value::as_i64)
                        .and_then(epoch_seconds_to_iso),
                    allowed: rl.get("allowed").and_then(Value::as_bool),
                    scope: Some(scope.clone()),
                });
            }
        }
    }
    s.status = if base.get("allowed") == Some(&Value::Bool(false))
        || base.get("limit_reached") == Some(&Value::Bool(true))
    {
        Availability::RateLimited
    } else if !s.windows.iter().any(|w| w.remaining_percent.is_some()) {
        Availability::NoData
    } else if s.windows.iter().any(|w| w.remaining_percent.is_none()) {
        Availability::Partial
    } else {
        Availability::Ok
    };
    let now = Utc::now();
    s.observed_at = Some(now.to_rfc3339());
    s.attempted_at = s.observed_at.clone();
    s.expires_at = epoch_seconds_to_iso(now.timestamp() + 120);
    s.source_version = Some("OpenAI /wham/usage".into());
    Ok(s)
}

// Cached access tokens live only in memory. Account files and refresh tokens never enter the frontend or history.
type TokenCache = Mutex<HashMap<String, (String, i64)>>;
static GOOGLE_TOKENS: OnceLock<TokenCache> = OnceLock::new();
const GOOGLE_CLIENT_ID: &str =
    "1071006060591-tmhssin2h21lcre235vtolojh4g403ep.apps.googleusercontent.com";
// Public OAuth client configuration from the installed OpenCode Antigravity plugin, not a user's secret.
const GOOGLE_CLIENT_SECRET: &str = "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";
async fn google_access(c: &Client, a: &Value) -> Result<String, String> {
    let refresh = a
        .get("refreshToken")
        .and_then(Value::as_str)
        .ok_or("Conta Google sem login salvo")?
        .split('|')
        .next()
        .unwrap_or("");
    let key = hash_account_key(refresh);
    let cache = GOOGLE_TOKENS.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some((token, expires)) = cache
        .lock()
        .map_err(|_| "Falha no cache de autenticação")?
        .get(&key)
    {
        if *expires > Utc::now().timestamp() + 60 {
            return Ok(token.clone());
        }
    }
    let r = c
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("client_id", GOOGLE_CLIENT_ID),
            ("client_secret", GOOGLE_CLIENT_SECRET),
            ("refresh_token", refresh),
            ("grant_type", "refresh_token"),
        ])
        .send()
        .await
        .map_err(|_| "Falha de conexão ao autenticar Google")?;
    let v = response_json(r).await?;
    let token = v
        .get("access_token")
        .and_then(Value::as_str)
        .ok_or("Google não retornou token de acesso")?
        .to_owned();
    let expires =
        Utc::now().timestamp() + v.get("expires_in").and_then(Value::as_i64).unwrap_or(300);
    cache
        .lock()
        .map_err(|_| "Falha no cache de autenticação")?
        .insert(key, (token.clone(), expires));
    Ok(token)
}
pub async fn query_google_live() -> Result<Vec<QuotaSnapshot>, String> {
    let path = get_default_accounts_path();
    if !path.exists() {
        return Ok(vec![]);
    }
    let content = read_json(&path)?;
    let accounts = content
        .get("accounts")
        .and_then(Value::as_array)
        .ok_or("Google sem contas configuradas")?;
    let cached = parse_antigravity_accounts(&content.to_string(), Utc::now().timestamp_millis())?;
    let c = client()?;
    let items = accounts
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, a)| (i, a, cached.get(i).cloned()))
        .collect();
    bounded_queries(items, move |(i, a, cached)| {
        query_google_account(c.clone(), i, a, cached)
    })
    .await
}
async fn bounded_queries<T, F, Fut>(items: Vec<T>, query: F) -> Result<Vec<QuotaSnapshot>, String>
where
    T: Send + 'static,
    F: Fn(T) -> Fut + Clone + Send + 'static,
    Fut: std::future::Future<Output = QuotaSnapshot> + Send + 'static,
{
    let gate = std::sync::Arc::new(tokio::sync::Semaphore::new(6));
    let mut tasks = tokio::task::JoinSet::new();
    for (i, item) in items.into_iter().enumerate() {
        let gate = gate.clone();
        let query = query.clone();
        tasks.spawn(async move {
            let _permit = gate.acquire_owned().await.expect("query gate remains open");
            (i, query(item).await)
        });
    }
    let mut results = Vec::new();
    while let Some(result) = tasks.join_next().await {
        results.push(result.map_err(|_| "Falha ao consultar conta")?);
    }
    results.sort_by_key(|(i, _)| *i);
    Ok(results.into_iter().map(|(_, s)| s).collect())
}
async fn query_google_account(
    c: Client,
    i: usize,
    a: Value,
    cached: Option<QuotaSnapshot>,
) -> QuotaSnapshot {
    let mut s = cached.unwrap_or_else(|| {
        failure(
            ProviderId::GoogleAntigravity,
            OriginKind::AntigravityPrivateOptIn,
            Availability::NoData,
            "",
        )
    });
    s.account_label = format!("{} · {}", i + 1, s.account_label);
    s.attempted_at = Some(Utc::now().to_rfc3339());
    if a.get("enabled") == Some(&Value::Bool(false)) {
        s.status = Availability::Disabled;
        s.models.clear();
        return s;
    }
    let attempt = async {
        let token = google_access(&c, &a).await?;
        let project = a
            .get("projectId")
            .or_else(|| a.get("managedProjectId"))
            .and_then(Value::as_str);
        let body = project
            .map(|p| json!({"project":p}))
            .unwrap_or_else(|| json!({}));
        let r = c
            .post("https://cloudcode-pa.googleapis.com/v1internal:fetchAvailableModels")
            .bearer_auth(token)
            .header("User-Agent", "antigravity/1.18.3 windows/amd64")
            .json(&body)
            .send()
            .await
            .map_err(|_| "Falha de conexão ao consultar Google")?;
        parse_google_models(&response_json(r).await?)
    }
    .await;
    match attempt {
        Ok(models) => {
            s.status = if !models.iter().any(|m| m.remaining_percent.is_some()) {
                Availability::NoData
            } else if models.iter().any(|m| m.remaining_percent.is_none()) {
                Availability::Partial
            } else {
                Availability::Ok
            };
            s.models = models;
            s.origin = OriginKind::AntigravityPrivateOptIn;
            s.observed_at = Some(Utc::now().to_rfc3339());
            s.expires_at = epoch_seconds_to_iso(Utc::now().timestamp() + 120);
            s.warnings.clear();
            s.note = None;
            s.source_version = Some("Google fetchAvailableModels".into());
            s.raw_error_code = None;
        }
        Err(message) => {
            s.status = if s.models.is_empty() {
                Availability::NetworkError
            } else {
                Availability::Stale
            };
            s.warnings = vec![format!(
                "Consulta ao vivo falhou: {}. Exibindo apenas a leitura anterior.",
                message
            )];
        }
    }
    s
}

pub fn opencode_data_path() -> PathBuf {
    let root = std::env::var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(
                std::env::var("USERPROFILE")
                    .or_else(|_| std::env::var("HOME"))
                    .unwrap_or_default(),
            )
            .join(".local/share")
        });
    root.join("opencode")
}
pub fn read_opencode_openai() -> Result<Vec<Value>, String> {
    let root = opencode_data_path();
    let mut accounts = Vec::new();
    let db = root.join("opencode.db");
    if db.exists() {
        let c = Connection::open_with_flags(db, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|_| "Falha ao ler credenciais do OpenCode")?;
        c.busy_timeout(Duration::from_millis(1000))
            .map_err(|_| "OpenCode ocupado")?;
        let exists:bool=c.query_row("SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='credential')",[],|r|r.get(0)).unwrap_or(false);
        if exists {
            let mut stmt=c.prepare("SELECT value FROM credential WHERE integration_id='openai' ORDER BY active DESC, time_updated DESC").map_err(|_|"Formato de credenciais OpenCode alterado")?;
            let rows = stmt
                .query_map([], |r| r.get::<_, String>(0))
                .map_err(|_| "Falha ao consultar credenciais OpenCode")?;
            for raw in rows.flatten() {
                if let Ok(v) = serde_json::from_str::<Value>(&raw) {
                    if v.get("type").and_then(Value::as_str) == Some("oauth") {
                        accounts.push(v);
                    }
                }
            }
        }
    }
    let legacy = root.join("auth.json");
    if accounts.is_empty() && legacy.exists() {
        if let Some(v) = read_json(&legacy)?.get("openai").cloned() {
            accounts.push(v);
        }
    }
    Ok(accounts)
}
pub(super) fn jwt(token: &str) -> Option<Value> {
    serde_json::from_slice(&URL_SAFE_NO_PAD.decode(token.split('.').nth(1)?).ok()?).ok()
}
async fn usage(
    c: &Client,
    access: &str,
    account: Option<&str>,
) -> Result<reqwest::Response, String> {
    let mut r = c
        .get("https://chatgpt.com/backend-api/wham/usage")
        .bearer_auth(access);
    if let Some(a) = account {
        r = r.header("ChatGPT-Account-Id", a);
    }
    r.send()
        .await
        .map_err(|_| "Falha de conexão ao consultar OpenAI".into())
}
fn codex_path() -> PathBuf {
    std::env::var("CODEX_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(
                std::env::var("USERPROFILE")
                    .or_else(|_| std::env::var("HOME"))
                    .unwrap_or_default(),
            )
            .join(".codex")
        })
        .join("auth.json")
}
async fn snapshot_from_response(
    response: reqwest::Response,
    origin: OriginKind,
    identity: Option<&Value>,
) -> Result<QuotaSnapshot, String> {
    if !response.status().is_success() {
        let status = response.status();
        let note = if status == StatusCode::UNAUTHORIZED {
            "A sessão expirou. Conecte a quota OpenAI em Ajustes para renovar automaticamente."
                .to_owned()
        } else {
            format!("OpenAI retornou HTTP {}", status.as_u16())
        };
        let mut s = failure(
            ProviderId::OpenaiCodex,
            origin,
            status_for_http(status),
            &note,
        );
        s.raw_error_code = Some(format!("HTTP_{}", status.as_u16()));
        return Ok(s);
    }
    let mut data = response_json(response).await?;
    if let Some(identity) = identity {
        if let Ok((account, email)) = super::native_auth::token_identity(identity) {
            if data
                .get("account_id")
                .and_then(Value::as_str)
                .is_some_and(|id| id != account)
            {
                return Err("A quota retornada não corresponde à conta conectada".into());
            }
            data["account_id"] = Value::String(account);
            if data.get("email").and_then(Value::as_str).is_none() {
                if let Some(email) = email {
                    data["email"] = Value::String(email);
                }
            }
        }
    }
    let mut s = parse_openai_usage(&data)?;
    s.origin = origin;
    s.note=Some("Limites Codex desta conta. Os limites específicos do compartilhamento com o OpenCode não são informados por esta fonte.".into());
    Ok(s)
}
async fn query_owned_openai(
    c: Client,
    record: super::native_auth::ConnectedAccount,
) -> QuotaSnapshot {
    let attempt = async {
        let mut auth = super::native_auth::own_session(&record.account_key, false)
            .await
            .map_err(|e| (e.status, e.note))?;
        let access = auth.get("access_token").and_then(Value::as_str).ok_or((
            Availability::UnauthorizedScope,
            "Sessão OpenAI inválida".to_owned(),
        ))?;
        let mut response = usage(&c, access, auth.get("account_id").and_then(Value::as_str))
            .await
            .map_err(|e| (Availability::NetworkError, e))?;
        if response.status() == StatusCode::UNAUTHORIZED {
            auth = super::native_auth::own_session(&record.account_key, true)
                .await
                .map_err(|e| (e.status, e.note))?;
            response = usage(
                &c,
                auth.get("access_token").and_then(Value::as_str).ok_or((
                    Availability::UnauthorizedScope,
                    "Sessão OpenAI inválida".to_owned(),
                ))?,
                auth.get("account_id").and_then(Value::as_str),
            )
            .await
            .map_err(|e| (Availability::NetworkError, e))?;
        }
        snapshot_from_response(response, OriginKind::OpenaiQuotaLogin, Some(&auth))
            .await
            .map_err(|e| (Availability::SourceChanged, e))
    }
    .await;
    let mut snapshot = attempt.unwrap_or_else(|(status, message)| {
        failure(
            ProviderId::OpenaiCodex,
            OriginKind::OpenaiQuotaLogin,
            status,
            &message,
        )
    });
    if snapshot.account_key == "source-error" {
        snapshot.account_key = record.account_key;
        snapshot.account_label = record.account_label;
    }
    snapshot
}
fn successful_reading(s: &QuotaSnapshot) -> bool {
    matches!(
        s.status,
        Availability::Ok | Availability::Partial | Availability::RateLimited
    ) && (s.windows.iter().any(|w| w.remaining_percent.is_some())
        || s.models.iter().any(|m| m.remaining_percent.is_some()))
}
fn select_account_sources(snapshots: Vec<QuotaSnapshot>) -> Vec<QuotaSnapshot> {
    let mut result: Vec<QuotaSnapshot> = Vec::new();
    for s in snapshots {
        if let Some(existing) = result
            .iter_mut()
            .find(|old| old.account_key == s.account_key)
        {
            if !successful_reading(existing) && successful_reading(&s) {
                *existing = s;
            }
        } else {
            result.push(s);
        }
    }
    result
}
pub async fn query_openai_live() -> Result<Vec<QuotaSnapshot>, String> {
    let c = client()?;
    let records = super::native_auth::connected_accounts();
    let query_client = c.clone();
    let mut snapshots = bounded_queries(records, move |record| {
        query_owned_openai(query_client.clone(), record)
    })
    .await?;
    let seen: std::collections::HashSet<String> = snapshots
        .iter()
        .filter(|s| successful_reading(s))
        .map(|s| s.account_key.clone())
        .collect();
    let opencode = read_opencode_openai().unwrap_or_default();
    let has_opencode = !opencode.is_empty();
    let mut compatible = Vec::new();
    for mut auth in opencode {
        if let Some(access) = auth
            .get("access")
            .and_then(Value::as_str)
            .map(str::to_owned)
        {
            auth["access_token"] = Value::String(access);
            if let Ok((id, _)) = super::native_auth::token_identity(&auth) {
                if !seen.contains(&hash_account_key(&id)) {
                    compatible.push((auth, OriginKind::OpencodeOpenaiUsage));
                }
            }
        }
    }
    if let Ok(auth) = read_json(&codex_path()) {
        if let Ok((id, _)) = super::native_auth::token_identity(&auth) {
            if !seen.contains(&hash_account_key(&id)) {
                compatible.push((auth, OriginKind::CodexLocalUsage));
            }
        }
    }
    let more = bounded_queries(compatible, move |(auth, origin)| {
        let c = c.clone();
        async move {
            let (id, email) =
                super::native_auth::token_identity(&auth).expect("validated identity");
            let result = async {
                let access = auth
                    .get("access_token")
                    .or_else(|| auth.pointer("/tokens/access_token"))
                    .and_then(Value::as_str)
                    .ok_or("Conecte a quota OpenAI em Ajustes")?;
                snapshot_from_response(
                    usage(&c, access, Some(&id)).await?,
                    origin.clone(),
                    Some(&auth),
                )
                .await
            }
            .await;
            let mut s = result.unwrap_or_else(|message| {
                failure(
                    ProviderId::OpenaiCodex,
                    origin,
                    Availability::NetworkError,
                    &message,
                )
            });
            if s.account_key == "source-error" {
                s.account_key = hash_account_key(&id);
                s.account_label = email
                    .as_deref()
                    .map(mask_email)
                    .unwrap_or_else(|| "Conta OpenAI".into());
            }
            s
        }
    })
    .await?;
    snapshots.extend(more);
    snapshots = select_account_sources(snapshots);
    if snapshots.is_empty() && has_opencode {
        snapshots.push(failure(
            ProviderId::OpenaiCodex,
            OriginKind::OpenaiQuotaLogin,
            Availability::UnauthorizedScope,
            "Conecte as contas OpenAI em Ajustes para medir e renovar suas quotas.",
        ));
    }
    // Stable labels distinguish accounts whose masked emails happen to be identical.
    for (i, s) in snapshots.iter_mut().enumerate() {
        s.account_label = format!("{} · {}", i + 1, s.account_label);
    }
    Ok(snapshots)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::Availability;
    use serde_json::json;
    #[test]
    fn expired_preferred_source_does_not_hide_a_healthy_same_account_fallback() {
        let mut bad = failure(
            ProviderId::OpenaiCodex,
            OriginKind::OpencodeOpenaiUsage,
            Availability::UnauthorizedScope,
            "expired",
        );
        bad.account_key = "account-a".into();
        let mut good =
            parse_openai_usage(&json!({"rate_limit":{"primary_window":{"used_percent":10}}}))
                .unwrap();
        good.account_key = "account-a".into();
        let mut other = good.clone();
        other.account_key = "account-b".into();
        let result = select_account_sources(vec![bad, good, other]);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].account_key, "account-a");
        assert_eq!(result[0].windows[0].remaining_percent, Some(90.0));
        assert_eq!(result[1].account_key, "account-b");
    }
    #[tokio::test]
    async fn queries_a_hundred_accounts_with_bounded_parallelism_and_stable_order() {
        use std::sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        };
        let active = Arc::new(AtomicUsize::new(0));
        let peak = Arc::new(AtomicUsize::new(0));
        let a = active.clone();
        let p = peak.clone();
        let results = bounded_queries((0..100).collect(), move |i| {
            let active = a.clone();
            let peak = p.clone();
            async move {
                let n = active.fetch_add(1, Ordering::SeqCst) + 1;
                peak.fetch_max(n, Ordering::SeqCst);
                tokio::time::sleep(Duration::from_millis((i % 3 + 1) * 2)).await;
                active.fetch_sub(1, Ordering::SeqCst);
                let mut s = failure(
                    ProviderId::OpenaiCodex,
                    OriginKind::OpenaiQuotaLogin,
                    Availability::NoData,
                    "test",
                );
                s.account_key = i.to_string();
                s
            }
        })
        .await
        .unwrap();
        assert_eq!(results.len(), 100);
        assert!(peak.load(Ordering::SeqCst) <= 6);
        assert!(peak.load(Ordering::SeqCst) > 1);
        for (i, s) in results.iter().enumerate() {
            assert_eq!(s.account_key, i.to_string());
        }
    }

    #[test]
    fn google_preserves_exact_models_and_unknown_values() {
        let models = parse_google_models(&json!({"models": {
          "gemini-flash-actual": {"displayName":"Gemini Flash actual", "quotaInfo":{"remainingFraction":0.004}},
          "claude-actual": {"quotaInfo":{"resetTime":"2026-10-12T01:00:00Z"}},
          "gemini-pro-invalid": {"quotaInfo":{"remainingFraction":2.0}}
        }})).unwrap();
        assert_eq!(models.len(), 3);
        assert_eq!(
            models
                .iter()
                .find(|m| m.model_id == "gemini-flash-actual")
                .unwrap()
                .remaining_percent,
            Some(0.4)
        );
        assert_eq!(
            models
                .iter()
                .find(|m| m.model_id == "claude-actual")
                .unwrap()
                .remaining_percent,
            None
        );
        assert_eq!(
            models
                .iter()
                .find(|m| m.model_id == "gemini-pro-invalid")
                .unwrap()
                .remaining_percent,
            None
        );
    }

    #[test]
    fn openai_parses_reported_windows_and_additional_buckets() {
        let s = parse_openai_usage(&json!({"account_id":"test", "email":"a@test.com", "rate_limit": {
            "allowed":true, "primary_window":{"used_percent":13.5,"limit_window_seconds":18000,"reset_at":1791781859},
            "secondary_window":{"used_percent":2,"limit_window_seconds":604800}
        }, "additional_rate_limits":[{"limit_name":"Special", "metered_feature":"special", "rate_limit":{"allowed":false,"limit_reached":true,"primary_window":{"used_percent":100,"limit_window_seconds":3600}}}]})).unwrap();
        assert_eq!(s.windows.len(), 3);
        assert_eq!(s.windows[0].remaining_percent, Some(86.5));
        assert_eq!(s.windows[0].window_seconds, Some(18000));
        assert_eq!(s.windows[2].allowed, Some(false));
        assert_eq!(s.status, Availability::Ok); // The exhausted extra bucket does not exhaust the primary bucket.
        assert!(!s.account_key.contains("test"));
    }

    #[test]
    fn missing_usage_is_not_full_quota() {
        let s = parse_openai_usage(&json!({"rate_limit":{"primary_window":{}}})).unwrap();
        assert_eq!(s.status, Availability::NoData);
        assert_eq!(s.windows[0].remaining_percent, None);
        assert_eq!(s.windows[0].allowed, None);
        assert!(parse_openai_usage(&json!({"unexpected":true})).is_err());
    }
}
