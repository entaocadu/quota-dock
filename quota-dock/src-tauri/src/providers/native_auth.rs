use super::live::{client, jwt, response_json};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::Utc;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use tauri::Emitter;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
const CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
static LOGIN_PENDING: AtomicBool = AtomicBool::new(false);
fn session_gate(key: &str) -> std::sync::Arc<tokio::sync::Mutex<()>> {
    use std::sync::{Arc, Mutex, OnceLock};
    static GATES: OnceLock<Mutex<std::collections::HashMap<String, Arc<tokio::sync::Mutex<()>>>>> =
        OnceLock::new();
    let mut gates = GATES
        .get_or_init(|| Mutex::new(std::collections::HashMap::new()))
        .lock()
        .expect("session gates");
    gates
        .entry(key.to_owned())
        .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
        .clone()
}
fn path() -> PathBuf {
    crate::storage::get_app_db_path().with_file_name("openai-session.bin")
}
fn account_path(key: &str) -> Result<PathBuf, String> {
    if key.len() != 16 || !key.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("Identidade de conexão inválida".into());
    }
    Ok(path()
        .with_file_name("openai-sessions")
        .join(format!("{}.bin", key)))
}
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectedAccount {
    pub account_key: String,
    pub account_label: String,
}
pub fn session_keys() -> Vec<String> {
    let mut keys: Vec<String> = std::fs::read_dir(path().with_file_name("openai-sessions"))
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let p = entry.path();
            if p.extension().and_then(|s| s.to_str()) != Some("bin") {
                return None;
            }
            let key = p.file_stem()?.to_str()?.to_owned();
            account_path(&key).ok()?;
            Some(key)
        })
        .collect();
    if path().exists() {
        let legacy_key = load("legacy")
            .ok()
            .and_then(|v| token_identity(&v).ok())
            .map(|(id, _)| crate::security::masking::hash_account_key(&id));
        if !legacy_key.is_some_and(|key| keys.contains(&key)) {
            keys.push("legacy".into());
        }
    }
    keys.sort();
    keys.dedup();
    keys
}
pub fn connected_accounts() -> Vec<ConnectedAccount> {
    session_keys()
        .into_iter()
        .map(|key| {
            let label = load(&key)
                .ok()
                .and_then(|v| token_identity(&v).ok())
                .and_then(|(_, email)| email)
                .map(|s| crate::security::masking::mask_email(&s))
                .unwrap_or_else(|| format!("Conexão {}", &key[..key.len().min(6)]));
            ConnectedAccount {
                account_key: key,
                account_label: label,
            }
        })
        .collect()
}
#[cfg(windows)]
fn protect(input: &[u8], decrypt: bool) -> Result<Vec<u8>, String> {
    use windows_sys::Win32::{
        Foundation::LocalFree,
        Security::Cryptography::{
            CryptProtectData, CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
        },
    };
    let data = CRYPT_INTEGER_BLOB {
        cbData: input.len() as u32,
        pbData: input.as_ptr() as *mut u8,
    };
    let entropy = b"QuotaDock/OpenAI/session/v1";
    let extra = CRYPT_INTEGER_BLOB {
        cbData: entropy.len() as u32,
        pbData: entropy.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };
    let ok = unsafe {
        if decrypt {
            CryptUnprotectData(
                &data,
                std::ptr::null_mut(),
                &extra,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        } else {
            CryptProtectData(
                &data,
                std::ptr::null(),
                &extra,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        }
    };
    if ok == 0 {
        return Err("Não foi possível acessar a sessão protegida pelo Windows".into());
    }
    let bytes =
        unsafe { std::slice::from_raw_parts(output.pbData, output.cbData as usize) }.to_vec();
    unsafe {
        LocalFree(output.pbData as _);
    }
    Ok(bytes)
}
#[cfg(not(windows))]
fn protect(_input: &[u8], _decrypt: bool) -> Result<Vec<u8>, String> {
    Err("Proteção de sessão disponível apenas no Windows".into())
}
pub fn session_present() -> bool {
    !session_keys().is_empty()
}
fn load(key: &str) -> Result<Value, String> {
    let p = if key == "legacy" {
        path()
    } else {
        account_path(key)?
    };
    if std::fs::metadata(&p)
        .map_err(|_| "Sem sessão própria do QuotaDock")?
        .len()
        > 1024 * 1024
    {
        return Err("Sessão protegida inválida".into());
    }
    let bytes = std::fs::read(p).map_err(|_| "Falha ao ler sessão protegida")?;
    let auth: Value =
        serde_json::from_slice(&protect(&bytes, true)?).map_err(|_| "Sessão protegida inválida")?;
    if key != "legacy" {
        let (id, _) = token_identity(&auth)?;
        if crate::security::masking::hash_account_key(&id) != key {
            return Err("A identidade da sessão protegida não corresponde à conexão".into());
        }
    }
    Ok(auth)
}
fn save(value: &Value) -> Result<(), String> {
    let (id, _) = token_identity(value)?;
    let p = account_path(&crate::security::masking::hash_account_key(&id))?;
    std::fs::create_dir_all(p.parent().ok_or("Diretório de conexão inválido")?)
        .map_err(|_| "Falha ao preparar sessões protegidas")?;
    let bytes = protect(
        &serde_json::to_vec(value).map_err(|_| "Sessão inválida")?,
        false,
    )?;
    let mut random = [0u8; 16];
    getrandom::fill(&mut random).map_err(|_| "Falha na proteção de sessão")?;
    let temp = p.with_file_name(format!(
        "openai-session-{}.tmp",
        URL_SAFE_NO_PAD.encode(random)
    ));
    use std::io::Write;
    let result = (|| {
        let mut f = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temp)
            .map_err(|_| "Falha ao salvar sessão protegida")?;
        f.write_all(&bytes)
            .and_then(|_| f.sync_all())
            .map_err(|_| "Falha ao salvar sessão protegida")?;
        drop(f);
        #[cfg(windows)]
        if p.exists() {
            use std::os::windows::ffi::OsStrExt;
            let old: Vec<u16> = p.as_os_str().encode_wide().chain(Some(0)).collect();
            let new: Vec<u16> = temp.as_os_str().encode_wide().chain(Some(0)).collect();
            if unsafe {
                windows_sys::Win32::Storage::FileSystem::ReplaceFileW(
                    old.as_ptr(),
                    new.as_ptr(),
                    std::ptr::null(),
                    0,
                    std::ptr::null(),
                    std::ptr::null(),
                )
            } == 0
            {
                return Err("Falha ao salvar sessão protegida".into());
            }
            return Ok(());
        }
        std::fs::rename(&temp, &p).map_err(|_| "Falha ao salvar sessão protegida".into())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result
}
#[derive(Debug)]
pub struct SessionError {
    pub status: crate::contracts::Availability,
    pub note: String,
}
fn session_error(status: crate::contracts::Availability, note: &str) -> SessionError {
    SessionError {
        status,
        note: note.into(),
    }
}
fn auth_error(note: &str) -> SessionError {
    session_error(
        crate::contracts::Availability::UnauthorizedScope,
        &format!("{}. Conecte a quota OpenAI novamente em Ajustes.", note),
    )
}
pub async fn own_session(key: &str, force_refresh: bool) -> Result<Value, SessionError> {
    use crate::contracts::Availability;
    let identity_key = if key == "legacy" {
        load(key)
            .ok()
            .and_then(|v| token_identity(&v).ok())
            .map(|v| crate::security::masking::hash_account_key(&v.0))
            .unwrap_or_else(|| key.to_owned())
    } else {
        key.to_owned()
    };
    let gate = session_gate(&identity_key);
    let _gate = gate.lock().await;
    let mut auth =
        load(key).map_err(|_| auth_error("A sessão protegida não pôde ser carregada"))?;
    let access = auth
        .get("access_token")
        .and_then(Value::as_str)
        .ok_or_else(|| auth_error("Sessão OpenAI inválida"))?;
    let expired = jwt(access)
        .and_then(|v| v.get("exp").and_then(Value::as_i64))
        .is_none_or(|t| t <= Utc::now().timestamp() + 60);
    if force_refresh || expired {
        let refresh = auth
            .get("refresh_token")
            .and_then(Value::as_str)
            .ok_or_else(|| auth_error("Sessão OpenAI sem renovação"))?;
        let r=client().map_err(|_|session_error(Availability::NetworkError,"Falha na conexão segura"))?.post("https://auth.openai.com/oauth/token")
            .json(&json!({"grant_type":"refresh_token","client_id":CLIENT_ID,"refresh_token":refresh}))
            .send().await.map_err(|_|session_error(Availability::NetworkError,"Falha de conexão ao renovar sessão OpenAI"))?;
        let status = r.status();
        if !status.is_success() {
            if [400, 401, 403].contains(&status.as_u16()) {
                return Err(auth_error("A OpenAI solicitou uma nova conexão"));
            }
            return Err(session_error(
                Availability::NetworkError,
                &format!("A renovação OpenAI retornou HTTP {}", status.as_u16()),
            ));
        }
        let fresh = response_json(r).await.map_err(|_| {
            session_error(
                Availability::SourceChanged,
                "A resposta de renovação OpenAI mudou",
            )
        })?;
        if fresh.get("access_token").and_then(Value::as_str).is_none() {
            return Err(session_error(
                Availability::SourceChanged,
                "OpenAI não retornou um token renovado",
            ));
        }
        for field in ["access_token", "refresh_token", "id_token"] {
            if let Some(v) = fresh.get(field).filter(|v| v.is_string()) {
                auth[field] = v.clone();
            }
        }
        let (account, _) = token_identity(&auth)
            .map_err(|_| auth_error("Não foi possível confirmar a conta renovada"))?;
        if auth
            .get("account_id")
            .and_then(Value::as_str)
            .is_some_and(|expected| expected != account)
        {
            return Err(auth_error("A conta retornada não corresponde à conexão"));
        }
        save(&auth).map_err(|_| auth_error("Não foi possível salvar a sessão renovada"))?;
    }
    if key == "legacy" {
        save(&auth).map_err(|_| auth_error("Não foi possível migrar a sessão protegida"))?;
        std::fs::remove_file(path())
            .map_err(|_| auth_error("Não foi possível concluir a migração da sessão"))?;
    }
    Ok(auth)
}
pub(super) fn token_identity(auth: &Value) -> Result<(String, Option<String>), String> {
    let access = auth
        .get("access_token")
        .or_else(|| auth.pointer("/tokens/access_token"))
        .and_then(Value::as_str)
        .and_then(jwt);
    let id = auth
        .get("id_token")
        .or_else(|| auth.pointer("/tokens/id_token"))
        .and_then(Value::as_str)
        .and_then(jwt);
    let account = access
        .as_ref()
        .and_then(|v| v.pointer("/https:~1~1api.openai.com~1auth/chatgpt_account_id"))
        .or_else(|| {
            id.as_ref()
                .and_then(|v| v.pointer("/https:~1~1api.openai.com~1auth/chatgpt_account_id"))
        })
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or("A sessão não informou a identidade da conta")?
        .to_owned();
    let email = id
        .as_ref()
        .and_then(|v| v.get("email"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    Ok((account, email))
}
async fn read_request<R: AsyncReadExt + Unpin>(reader: &mut R) -> Result<String, ()> {
    tokio::time::timeout(Duration::from_secs(3), async {
        let mut request = Vec::new();
        loop {
            let mut buffer = [0u8; 512];
            let count = reader.read(&mut buffer).await.map_err(|_| ())?;
            if count == 0 {
                return Err(());
            }
            request.extend_from_slice(&buffer[..count]);
            if request.len() > 8192 {
                return Err(());
            }
            if request.windows(4).any(|w| w == b"\r\n\r\n") {
                return String::from_utf8(request).map_err(|_| ());
            }
        }
    })
    .await
    .map_err(|_| ())?
}

fn callback_code(target: &str, state: &str) -> Result<Option<String>, String> {
    let url = reqwest::Url::parse(&format!("http://localhost{}", target))
        .map_err(|_| "Retorno de login inválido")?;
    if url.path() != "/auth/callback" {
        return Ok(None);
    }
    let params: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
    if params.get("state").map(String::as_str) != Some(state) {
        return Err("Retorno de login não corresponde à solicitação".into());
    }
    if params.contains_key("error") {
        return Err("A conexão OpenAI foi cancelada".into());
    }
    params
        .get("code")
        .filter(|s| !s.is_empty())
        .cloned()
        .map(Some)
        .ok_or_else(|| "Retorno de login sem código".into())
}
async fn login_flow(
    listener: tokio::net::TcpListener,
    state: String,
    verifier: String,
    redirect: String,
    expected: Option<String>,
) -> Result<(), String> {
    let code=tokio::time::timeout(Duration::from_secs(180),async{
        loop{
            let (mut stream,_)=listener.accept().await.map_err(|_|"Falha ao receber login local")?;
            let Ok(request)=read_request(&mut stream).await else{continue};
            let Some(line)=request.lines().next() else{continue};
            let mut parts=line.split_whitespace();
            if parts.next()!=Some("GET"){continue;}
            let Some(target)=parts.next() else{continue};
            let result=callback_code(target,&state);
            let body=if matches!(result,Ok(Some(_))){"<!doctype html><meta charset=utf-8><title>QuotaDock</title><body style='background:#0c0d0e;color:#ddd;font:16px system-ui;padding:40px'>Conectando sua quota OpenAI. Volte ao QuotaDock para ver o resultado.</body>"}else{"Retorno de login inválido."};
            let response=format!("HTTP/1.1 {}\r\nContent-Type: text/html; charset=utf-8\r\nCache-Control: no-store\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",if matches!(result,Ok(Some(_))){"200 OK"}else{"400 Bad Request"},body.len(),body);
            let _=tokio::time::timeout(Duration::from_secs(2),stream.write_all(response.as_bytes())).await;
            match result{Ok(Some(code))=>break Ok::<String,String>(code),Err(message) if message=="A conexão OpenAI foi cancelada"=>break Err(message),_=>continue}
        }
    }).await.map_err(|_|"O login expirou. Tente conectar novamente.")??;
    let c = client()?;
    let r = c
        .post("https://auth.openai.com/oauth/token")
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", CLIENT_ID),
            ("code", &code),
            ("code_verifier", &verifier),
            ("redirect_uri", &redirect),
        ])
        .send()
        .await
        .map_err(|_| "Falha de conexão ao concluir login OpenAI")?;
    let auth = response_json(r).await?;
    let access = auth
        .get("access_token")
        .and_then(Value::as_str)
        .ok_or("OpenAI não retornou uma sessão válida")?;
    if auth.get("refresh_token").and_then(Value::as_str).is_none() {
        return Err("OpenAI não retornou uma sessão renovável".into());
    }
    let (account, email) = token_identity(&auth)?;
    if expected
        .as_deref()
        .is_some_and(|expected| expected != account)
    {
        return Err("Conecte a mesma conta usada no OpenCode e no login local. A conta escolhida é diferente.".into());
    }
    let usage = c
        .get("https://chatgpt.com/backend-api/wham/usage")
        .bearer_auth(access)
        .header("ChatGPT-Account-Id", &account)
        .send()
        .await
        .map_err(|_| "Falha ao confirmar quota OpenAI")?;
    let mut usage = response_json(usage).await?;
    if usage
        .get("account_id")
        .and_then(Value::as_str)
        .is_some_and(|id| id != account)
    {
        return Err("A quota retornada pertence a outra conta".into());
    }
    usage["account_id"] = Value::String(account.clone());
    if let Some(email) = &email {
        usage["email"] = Value::String(email.clone());
    }
    super::live::parse_openai_usage(&usage)?;
    let mut saved = auth;
    saved["account_id"] = Value::String(account);
    if let Some(email) = email {
        saved["email"] = Value::String(email);
    }
    let key = crate::security::masking::hash_account_key(
        saved
            .get("account_id")
            .and_then(Value::as_str)
            .ok_or("Sessão sem identidade")?,
    );
    let gate = session_gate(&key);
    let _guard = gate.lock().await;
    save(&saved)?;
    // Migrate the previous single connection only after the protected replacement is saved.
    if let Ok(old) = load("legacy") {
        if token_identity(&old).ok().map(|v| v.0) == token_identity(&saved).ok().map(|v| v.0) {
            let _ = std::fs::remove_file(path());
        }
    }
    Ok(())
}

pub async fn begin_login(app: tauri::AppHandle, account_key: Option<String>) -> Result<(), String> {
    if LOGIN_PENDING.swap(true, Ordering::SeqCst) {
        return Err("Já há uma conexão em andamento no navegador".into());
    }
    let expected = if let Some(key) = account_key {
        if key != "legacy" && account_path(&key).is_err() {
            LOGIN_PENDING.store(false, Ordering::SeqCst);
            return Err("Identidade de conexão inválida".into());
        }
        let account = load(&key)
            .ok()
            .and_then(|v| token_identity(&v).ok())
            .map(|v| v.0);
        if account.is_none() {
            LOGIN_PENDING.store(false, Ordering::SeqCst);
            return Err("Não foi possível confirmar a conexão. Adicione a conta novamente.".into());
        }
        account
    } else {
        None
    };
    let result = async {
        let listener = match tokio::net::TcpListener::bind("127.0.0.1:1455").await {
            Ok(l) => l,
            Err(_) => tokio::net::TcpListener::bind("127.0.0.1:1457")
                .await
                .map_err(|_| "As portas de login local estão ocupadas. Tente novamente.")?,
        };
        let port = listener
            .local_addr()
            .map_err(|_| "Falha no login local")?
            .port();
        let redirect = format!("http://127.0.0.1:{}/auth/callback", port);
        let mut random = [0u8; 64];
        getrandom::fill(&mut random).map_err(|_| "Falha ao iniciar login protegido")?;
        let verifier = URL_SAFE_NO_PAD.encode(&random[..32]);
        let state = URL_SAFE_NO_PAD.encode(&random[32..]);
        let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
        let mut url = reqwest::Url::parse("https://auth.openai.com/oauth/authorize")
            .map_err(|_| "URL de autenticação inválida")?;
        url.query_pairs_mut().extend_pairs([
            ("client_id", CLIENT_ID),
            ("response_type", "code"),
            ("redirect_uri", &redirect),
            ("scope", "openid profile email offline_access"),
            ("code_challenge", &challenge),
            ("code_challenge_method", "S256"),
            ("state", &state),
            ("originator", "quotadock"),
            ("prompt", "select_account"),
            ("id_token_add_organizations", "true"),
        ]);
        open::that(url.as_str()).map_err(|_| "Não foi possível abrir o navegador")?;
        tauri::async_runtime::spawn(async move {
            let result = login_flow(listener, state, verifier, redirect, expected).await;
            LOGIN_PENDING.store(false, Ordering::SeqCst);
            match result {
                Ok(()) => {
                    let _ = app.emit("openai-login-state", "Conectado");
                    let _ = crate::refresh(&app, true).await;
                    let _ = crate::tray::show_flyout(&app, None);
                }
                Err(message) => {
                    let _ = app.emit("openai-login-state", message);
                }
            }
        });
        Ok(())
    }
    .await;
    if result.is_err() {
        LOGIN_PENDING.store(false, Ordering::SeqCst);
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sessions_for_many_accounts_do_not_replace_each_other() {
        let mut paths = std::collections::HashSet::new();
        for i in 0..100 {
            let account = format!("account-{}", i);
            let p = account_path(&crate::security::masking::hash_account_key(&account)).unwrap();
            assert!(paths.insert(p.clone()));
            assert!(!p.to_string_lossy().contains(&account));
        }
        assert!(account_path("../outside").is_err());
    }
    #[test]
    fn callback_requires_matching_state_and_code() {
        assert_eq!(
            callback_code("/auth/callback?state=test&code=valid", "test").unwrap(),
            Some("valid".into())
        );
        assert!(callback_code("/auth/callback?state=wrong&code=valid", "test").is_err());
        assert!(callback_code("/auth/callback?state=test", "test").is_err());
        assert_eq!(
            callback_code("/other?state=test&code=valid", "test").unwrap(),
            None
        );
    }
    #[tokio::test]
    async fn callback_accepts_fragmented_headers_and_rejects_eof() {
        let (mut writer, mut reader) = tokio::io::duplex(64);
        tokio::spawn(async move {
            writer.write_all(b"GET /auth/call").await.unwrap();
            tokio::task::yield_now().await;
            writer
                .write_all(b"back?state=test&code=valid HTTP/1.1\r\nHost: localhost\r\n\r\n")
                .await
                .unwrap();
        });
        let request = read_request(&mut reader).await.unwrap();
        assert!(request.contains("/auth/callback?state=test&code=valid"));
        let (writer, mut reader) = tokio::io::duplex(8);
        drop(writer);
        assert!(read_request(&mut reader).await.is_err());
    }
    #[test]
    fn trusted_token_identity_is_not_a_generic_account() {
        let payload = URL_SAFE_NO_PAD.encode(
            br#"{"https://api.openai.com/auth":{"chatgpt_account_id":"specific-account"}}"#,
        );
        let auth = json!({"access_token":format!("header.{}.signature",payload)});
        assert_eq!(token_identity(&auth).unwrap().0, "specific-account");
        assert!(token_identity(&json!({"access_token":"bad"})).is_err());
        assert_eq!(
            callback_code("/auth/callback?state=test&error=access_denied", "test").unwrap_err(),
            "A conexão OpenAI foi cancelada"
        );
    }
    #[cfg(windows)]
    #[test]
    fn windows_protection_roundtrips_and_rejects_tampering() {
        let encrypted = protect(b"test session", false).unwrap();
        assert_ne!(encrypted, b"test session");
        assert_eq!(protect(&encrypted, true).unwrap(), b"test session");
        let mut bad = encrypted;
        bad[0] ^= 1;
        assert!(protect(&bad, true).is_err());
    }
}
