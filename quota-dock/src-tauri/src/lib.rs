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

use crate::contracts::*;
use crate::providers::live::*;
use crate::storage::*;
use rusqlite::Connection;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};
use tauri::{Emitter, Manager};
use tauri_plugin_autostart::ManagerExt;

#[derive(Default)]
struct QuotaService {
    cache: tokio::sync::Mutex<Option<(Instant, Vec<QuotaSnapshot>)>>,
    gate: tokio::sync::Mutex<()>,
    poll_mins: AtomicU32,
}
fn db() -> Result<Connection, String> {
    let conn = Connection::open(get_app_db_path()).map_err(|_| "Falha ao abrir banco local")?;
    init_db(&conn).map_err(|_| "Falha ao preparar banco local")?;
    Ok(conn)
}
async fn refresh(app: &tauri::AppHandle, force: bool) -> Result<Vec<QuotaSnapshot>, String> {
    let state = app.state::<QuotaService>();
    let _guard = state.gate.lock().await;
    if !force {
        if let Some((time, data)) = &*state.cache.lock().await {
            if time.elapsed() < Duration::from_secs(15) {
                return Ok(data.clone());
            }
        }
    }
    let (google, openai) = tokio::join!(query_google_live(), query_openai_live());
    let mut snapshots = google.unwrap_or_else(|e| {
        vec![failure(
            ProviderId::GoogleAntigravity,
            OriginKind::AntigravityPrivateOptIn,
            Availability::UnknownError,
            &e,
        )]
    });
    snapshots.extend(openai.unwrap_or_else(|e| {
        vec![failure(
            ProviderId::OpenaiCodex,
            OriginKind::CodexLocalUsage,
            if e.contains("Formato") || e.contains("corresponde") {
                Availability::SourceChanged
            } else if e.contains("não encontrado") {
                Availability::NotConfigured
            } else if e.contains("login inválido") || e.contains("Conecte") {
                Availability::UnauthorizedScope
            } else {
                Availability::NetworkError
            },
            &e,
        )]
    }));
    if let Some((_, previous)) = &*state.cache.lock().await {
        snapshots = retain_previous_readings(snapshots, previous);
    }
    if let Ok(conn) = db() {
        for s in &snapshots {
            let _ = save_snapshot(&conn, s);
        }
        let _=conn.execute("DELETE FROM snapshots WHERE id NOT IN (SELECT id FROM snapshots ORDER BY id DESC LIMIT 2000)",[]);
    }
    *state.cache.lock().await = Some((Instant::now(), snapshots.clone()));
    let _ = app.emit("quotas-updated", &snapshots);
    Ok(snapshots)
}
fn retain_previous_readings(
    current: Vec<QuotaSnapshot>,
    previous: &[QuotaSnapshot],
) -> Vec<QuotaSnapshot> {
    let mut result = Vec::new();
    for s in current {
        let failed = matches!(
            s.status,
            Availability::NetworkError | Availability::UnknownError | Availability::Stale
        );
        let matches = previous.iter().filter(|old| {
            old.provider_id == s.provider_id
                && (old.account_key == s.account_key || s.account_key == "source-error")
                && (!old.windows.is_empty() || !old.models.is_empty())
        });
        let mut preserved = false;
        if failed {
            for old in matches {
                let old_time = old
                    .observed_at
                    .as_deref()
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok());
                let new_time = s
                    .observed_at
                    .as_deref()
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok());
                if old_time > new_time || (s.models.is_empty() && s.windows.is_empty()) {
                    let mut cached = old.clone();
                    cached.status = Availability::Stale;
                    cached.attempted_at = s.attempted_at.clone();
                    cached.warnings = s.warnings.clone();
                    if let Some(note) = &s.note {
                        cached.warnings.push(note.clone());
                    }
                    result.push(cached);
                    preserved = true;
                }
            }
        }
        if !preserved {
            result.push(s);
        }
    }
    result
}
#[tauri::command]
async fn get_overview(app: tauri::AppHandle) -> Result<Vec<QuotaSnapshot>, String> {
    refresh(&app, false).await
}
#[tauri::command]
async fn request_refresh(app: tauri::AppHandle) -> Result<Vec<QuotaSnapshot>, String> {
    refresh(&app, true).await
}
#[tauri::command]
fn get_settings() -> Result<PublicSettings, String> {
    load_settings(&db()?).map_err(|_| "Falha ao ler ajustes".into())
}
#[tauri::command]
fn update_settings(
    app: tauri::AppHandle,
    patch: SafeSettingsPatch,
) -> Result<PublicSettings, String> {
    let conn = db()?;
    let mut s = load_settings(&conn).map_err(|_| "Falha ao ler ajustes")?;
    if let Some(start) = patch.start_with_windows {
        if start {
            app.autolaunch().enable()
        } else {
            app.autolaunch().disable()
        }
        .map_err(|_| "Falha ao registrar inicialização no Windows")?;
        s.start_with_windows = start;
    }
    if let Some(interval) = patch.poll_interval_mins {
        if ![1, 2, 5, 10].contains(&interval) {
            return Err("Intervalo inválido".into());
        }
        s.poll_interval_mins = interval;
    }
    save_settings(&conn, &s).map_err(|_| "Falha ao salvar ajustes")?;
    app.state::<QuotaService>()
        .poll_mins
        .store(s.poll_interval_mins, Ordering::Relaxed);
    Ok(s)
}
#[tauri::command]
fn hide_window(window: tauri::WebviewWindow) -> Result<(), String> {
    window.hide().map_err(|_| "Falha ao ocultar janela".into())
}
#[tauri::command]
fn start_dragging(window: tauri::WebviewWindow) -> Result<(), String> {
    tray::start_dragging(&window)
}
#[tauri::command]
async fn connect_openai(app: tauri::AppHandle, account_key: Option<String>) -> Result<(), String> {
    providers::native_auth::begin_login(app, account_key).await
}
#[tauri::command]
fn openai_connections() -> Vec<providers::native_auth::ConnectedAccount> {
    providers::native_auth::connected_accounts()
}
#[tauri::command]
fn resize_flyout(window: tauri::WebviewWindow, height: f64) -> Result<(), String> {
    tray::resize_flyout(&window, height).map_err(|_| "Falha ao ajustar janela".into())
}
#[tauri::command]
fn get_history(limit: Option<usize>) -> Result<Vec<QuotaSnapshot>, String> {
    storage::get_history(&db()?, limit.unwrap_or(50).min(2000))
        .map_err(|_| "Falha ao ler histórico".into())
}
#[tauri::command]
fn clear_history() -> Result<(), String> {
    storage::clear_history(&db()?).map_err(|_| "Falha ao limpar histórico".into())
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, _| {
            if args.iter().any(|a| a == "--show") {
                let _ = tray::show_flyout(app, None);
            }
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--background"]),
        ))
        .manage(QuotaService::default())
        .setup(|app| {
            tray::setup_tray(app.handle())?;
            let conn = db().map_err(std::io::Error::other)?;
            let migrated: bool = conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM settings WHERE key='compact_tray_v2')",
                [],
                |r| r.get(0),
            )?;
            let mut settings = load_settings(&conn)?;
            if !migrated {
                settings.start_with_windows = true;
                settings.poll_interval_mins = 1;
                settings.pinned = false;
                settings.google_live_opt_in = true;
                save_settings(&conn, &settings)?;
                conn.execute(
                    "INSERT INTO settings(key,value) VALUES('compact_tray_v2','1')",
                    [],
                )?;
            }
            if settings.start_with_windows {
                app.autolaunch().enable()?;
            } else {
                app.autolaunch().disable()?;
            }
            app.state::<QuotaService>()
                .poll_mins
                .store(settings.poll_interval_mins, Ordering::Relaxed);
            if let Some(window) = app.get_webview_window("main") {
                let win = window.clone();
                window.on_window_event(move |event| match event {
                    tauri::WindowEvent::CloseRequested { api, .. } => {
                        api.prevent_close();
                        tray::hide_flyout(&win);
                    }
                    tauri::WindowEvent::Focused(false) => tray::hide_flyout(&win),
                    _ => {}
                });
            }
            if std::env::args().any(|a| a == "--show") {
                let _ = tray::show_flyout(app.handle(), None);
            }
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let _ = refresh(&handle, false).await;
                let mut last = Instant::now();
                loop {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    let mins = handle
                        .state::<QuotaService>()
                        .poll_mins
                        .load(Ordering::Relaxed)
                        .clamp(1, 10);
                    if last.elapsed() >= Duration::from_secs(u64::from(mins) * 60) {
                        let _ = refresh(&handle, false).await;
                        last = Instant::now();
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_overview,
            request_refresh,
            get_settings,
            update_settings,
            hide_window,
            start_dragging,
            get_history,
            clear_history,
            connect_openai,
            openai_connections,
            resize_flyout
        ])
        .run(tauri::generate_context!())
        .expect("Falha ao iniciar QuotaDock");
}
