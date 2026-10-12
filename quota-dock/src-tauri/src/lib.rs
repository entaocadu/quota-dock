pub mod contracts;
pub mod providers;
pub mod quota {
    pub mod normalization;
}
pub mod security {
    pub mod masking;
}
pub mod storage;
pub mod tray;

use rusqlite::Connection;
use tauri::Manager;

use crate::contracts::{
    Availability, LocalTokens, OriginKind, ProviderDiag, ProviderId, PublicSettings,
    SafeSettingsPatch, SanitizedDiagnostics, QuotaSnapshot,
};
use crate::providers::antigravity::{get_default_accounts_path, read_local_antigravity_cache};
use crate::providers::openai::query_codex_app_server;
use crate::providers::opencode::query_opencode_local_tokens;
use crate::storage::{
    clear_history as db_clear_history, get_app_db_path, get_history as db_get_history,
    init_db, load_settings, save_settings, save_snapshot,
};

fn get_db_conn() -> Result<Connection, String> {
    let path = get_app_db_path();
    let conn = Connection::open(path).map_err(|e| format!("Erro ao abrir banco QuotaDock: {}", e))?;
    let _ = init_db(&conn);
    Ok(conn)
}

#[tauri::command]
async fn get_overview() -> Result<Vec<QuotaSnapshot>, String> {
    let mut snapshots = Vec::new();

    // 1. Google Antigravity local cache
    match read_local_antigravity_cache(None) {
        Ok(snaps) => {
            if let Ok(conn) = get_db_conn() {
                for s in &snaps {
                    let _ = save_snapshot(&conn, s);
                }
            }
            snapshots.extend(snaps);
        }
        Err(e) => {
            snapshots.push(QuotaSnapshot {
                schema_version: 1,
                provider_id: ProviderId::GoogleAntigravity,
                origin: OriginKind::AntigravityPluginCache,
                account_key: "google_read_error".to_string(),
                account_label: "Google Antigravity".to_string(),
                status: Availability::UnknownError,
                observed_at: None,
                attempted_at: Some(chrono::Utc::now().to_rfc3339()),
                expires_at: None,
                source_version: None,
                note: Some(e),
                windows: vec![],
                models: vec![],
                warnings: vec!["Erro ao ler cache local do Google".to_string()],
                raw_error_code: Some("CACHE_READ_ERROR".to_string()),
            });
        }
    }

    // 2. OpenAI Codex app-server live query
    match query_codex_app_server().await {
        Ok(snap) => {
            if let Ok(conn) = get_db_conn() {
                let _ = save_snapshot(&conn, &snap);
            }
            snapshots.push(snap);
        }
        Err(e) => {
            snapshots.push(QuotaSnapshot {
                schema_version: 1,
                provider_id: ProviderId::OpenaiCodex,
                origin: OriginKind::CodexAppServer,
                account_key: "openai_query_error".to_string(),
                account_label: "OpenAI Codex".to_string(),
                status: Availability::UnauthorizedScope,
                observed_at: None,
                attempted_at: Some(chrono::Utc::now().to_rfc3339()),
                expires_at: None,
                source_version: None,
                note: Some(e),
                windows: vec![],
                models: vec![],
                warnings: vec![
                    "Não foi possível obter quotas pelo Codex app-server.".to_string(),
                    "Verifique se o Codex CLI está autenticado ou use o painel oficial.".to_string(),
                ],
                raw_error_code: Some("APP_SERVER_UNAVAILABLE".to_string()),
            });
        }
    }

    Ok(snapshots)
}

#[tauri::command]
async fn get_local_tokens(range: String) -> Result<Option<LocalTokens>, String> {
    match query_opencode_local_tokens(&range).await {
        Ok(tokens) => Ok(Some(tokens)),
        Err(_) => Ok(None),
    }
}

#[tauri::command]
async fn request_refresh(provider_id: Option<String>) -> Result<Vec<QuotaSnapshot>, String> {
    let _ = provider_id;
    get_overview().await
}

#[tauri::command]
async fn get_diagnostics() -> Result<SanitizedDiagnostics, String> {
    let db_path = std::env::var("USERPROFILE")
        .map(|p| std::path::PathBuf::from(p).join(".local/share/opencode/opencode.db"))
        .unwrap_or_default();
    let opencode_db_detected = db_path.exists();

    let antigravity_path = get_default_accounts_path();
    let antigravity_accounts_detected = antigravity_path.exists();

    let codex_cli_detected = which_codex();

    let snapshots = get_overview().await.unwrap_or_default();
    let mut providers = Vec::new();

    for s in snapshots {
        providers.push(ProviderDiag {
            provider_id: s.provider_id,
            status: s.status,
            origin: s.origin,
            last_success: s.observed_at,
            last_attempt: s.attempted_at,
            error_code: s.raw_error_code,
        });
    }

    Ok(SanitizedDiagnostics {
        timestamp: chrono::Utc::now().to_rfc3339(),
        app_version: "0.1.0".to_string(),
        os: "Windows 11 x64".to_string(),
        providers,
        opencode_db_detected,
        antigravity_accounts_detected,
        codex_cli_detected,
    })
}

fn which_codex() -> bool {
    std::process::Command::new("codex")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[tauri::command]
async fn get_settings() -> Result<PublicSettings, String> {
    let conn = get_db_conn()?;
    load_settings(&conn).map_err(|e| format!("Erro ao carregar configurações: {}", e))
}

#[tauri::command]
async fn update_settings(patch: SafeSettingsPatch) -> Result<PublicSettings, String> {
    let conn = get_db_conn()?;
    let mut current = load_settings(&conn).unwrap_or_default();

    if let Some(v) = patch.start_with_windows {
        current.start_with_windows = v;
    }
    if let Some(v) = patch.theme {
        current.theme = v;
    }
    if let Some(v) = patch.google_live_opt_in {
        current.google_live_opt_in = v;
    }
    if let Some(v) = patch.poll_interval_mins {
        current.poll_interval_mins = v;
    }
    if let Some(v) = patch.notifications_enabled {
        current.notifications_enabled = v;
    }
    if let Some(v) = patch.notify_threshold_25 {
        current.notify_threshold_25 = v;
    }
    if let Some(v) = patch.notify_threshold_10 {
        current.notify_threshold_10 = v;
    }
    if let Some(v) = patch.notify_threshold_0 {
        current.notify_threshold_0 = v;
    }
    if let Some(v) = patch.pinned {
        current.pinned = v;
    }

    save_settings(&conn, &current).map_err(|e| format!("Erro ao salvar configurações: {}", e))?;
    Ok(current)
}

#[tauri::command]
async fn get_history(limit: Option<usize>) -> Result<Vec<QuotaSnapshot>, String> {
    let conn = get_db_conn()?;
    let lim = limit.unwrap_or(50);
    db_get_history(&conn, lim).map_err(|e| format!("Erro ao obter histórico: {}", e))
}

#[tauri::command]
async fn clear_history() -> Result<(), String> {
    let conn = get_db_conn()?;
    db_clear_history(&conn).map_err(|e| format!("Erro ao limpar histórico: {}", e))
}

#[tauri::command]
async fn open_external_help(key: String) -> Result<(), String> {
    let url = match key.as_str() {
        "codex-usage" => "https://help.openai.com/en/articles/11369540-using-codex-with-your-chatgpt-plan",
        "google-help" => "https://github.com/pieliesdie/opencode-antigravity-auth",
        "opencode-help" => "https://dev.opencode.ai",
        "codex-settings" => "https://chatgpt.com/codex/cloud/settings/analytics#usage",
        _ => return Err("URL não permitida".to_string()),
    };

    let _ = open::that(url);
    Ok(())
}

#[tauri::command]
async fn hide_window(window: tauri::WebviewWindow) -> Result<(), String> {
    let _ = window.hide();
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let _ = tray::setup_tray(app.handle());

            // Window close event hides rather than quits
            if let Some(window) = app.get_webview_window("main") {
                let win_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = win_clone.hide();
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_overview,
            get_local_tokens,
            request_refresh,
            get_diagnostics,
            get_settings,
            update_settings,
            get_history,
            clear_history,
            open_external_help,
            hide_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
