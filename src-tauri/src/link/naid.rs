//! Natayark ID 登录：OAuth 授权码 + 本地回环回调，对齐 PCL CE。

use crate::config::{link::LinkState, manager};
use crate::link::LinkError;
use crate::secrets;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::sync::LazyLock;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, watch};
use tokio::task::AbortHandle;

const AUTHORIZE_URL: &str = "https://account.naids.com/oauth2/authorize";
const TOKEN_URL: &str = "https://account.naids.com/api/oauth2/token";
const USER_URL: &str = "https://account.naids.com/api/api/user/data";
const CALLBACK_TIMEOUT: Duration = Duration::from_secs(300);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);
const CALLBACK_REQUEST_TIMEOUT: Duration = Duration::from_secs(3);

/// 联机页展示的 Natayark 账户状态。
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkAccount {
    pub configured: bool,
    pub logged_in: bool,
    pub pending: bool,
    pub username: Option<String>,
    pub status: Option<i64>,
    pub realname: bool,
    pub expired: bool,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct NaidProfile {
    username: String,
    status: i64,
    realname: bool,
}

struct Task {
    abort: AbortHandle,
    result: watch::Receiver<Option<Result<(), LinkError>>>,
}

#[derive(Default)]
struct Runtime {
    profile: Option<NaidProfile>,
    pending: bool,
    error: Option<String>,
    generation: u64,
    task: Option<Task>,
}

static RUNTIME: LazyLock<Mutex<Runtime>> = LazyLock::new(|| Mutex::new(Runtime::default()));

#[derive(Deserialize)]
struct NaidTokens {
    access_token: String,
    refresh_token: String,
    #[serde(rename = "refresh_token_expires_at")]
    refresh_expires_at: String,
}

impl Runtime {
    fn account(&self, state: &LinkState) -> LinkAccount {
        let logged_in = state.eula_accepted && state.naid_refresh_token.is_some();
        let expired = is_expired(state);
        let profile = self
            .profile
            .as_ref()
            .filter(|_| logged_in && !expired && self.error.is_none());
        LinkAccount {
            configured: secrets::naid_credentials().is_some(),
            logged_in,
            pending: self.pending,
            username: profile.map(|profile| profile.username.clone()),
            status: profile.map(|profile| profile.status),
            realname: profile.is_some_and(|profile| profile.realname),
            expired,
            error: self.error.clone(),
        }
    }

    fn cancel(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        if let Some(task) = self.task.take() {
            task.abort.abort();
        }
        self.pending = false;
        self.profile = None;
        self.error = None;
    }

    fn spawn(
        &mut self,
        pending: bool,
        operation: impl Future<Output = Result<(NaidProfile, NaidTokens), LinkError>> + Send + 'static,
    ) -> watch::Receiver<Option<Result<(), LinkError>>> {
        self.generation = self.generation.wrapping_add(1);
        let generation = self.generation;
        self.pending = pending;
        self.profile = None;
        self.error = None;
        let (sender, receiver) = watch::channel(None);
        let task = tokio::spawn(async move {
            let result = operation.await;
            let result = RUNTIME
                .lock()
                .await
                .complete(generation, result, persist_credentials)
                .await;
            let _ = sender.send(Some(result));
        });
        self.task = Some(Task {
            abort: task.abort_handle(),
            result: receiver.clone(),
        });
        receiver
    }

    // 持锁等待落盘：取消必须排在已开始的提交之后，不能中断 spawn_blocking 写入。
    async fn complete<F, Fut>(
        &mut self,
        generation: u64,
        result: Result<(NaidProfile, NaidTokens), LinkError>,
        persist: F,
    ) -> Result<(), LinkError>
    where
        F: FnOnce(Option<NaidTokens>) -> Fut,
        Fut: Future<Output = Result<(), LinkError>>,
    {
        if self.generation != generation {
            return Err(LinkError::new("naid_cancelled"));
        }
        let refreshing = !self.pending;
        self.task = None;
        self.pending = false;
        self.profile = None;
        let result = match result {
            Ok((profile, tokens)) => persist(Some(tokens)).await.map(|()| {
                self.profile = Some(profile);
            }),
            Err(error) if refreshing && error.code == "naid_expired" => {
                persist(None).await.and(Err(error))
            }
            Err(error) => Err(error),
        };
        self.error = result.as_ref().err().map(|error| error.code.clone());
        result
    }
}

fn credentials() -> Result<(String, String), LinkError> {
    secrets::naid_credentials().ok_or_else(|| LinkError::new("naid_not_configured"))
}

pub async fn account() -> LinkAccount {
    let runtime = RUNTIME.lock().await;
    runtime.account(&manager::get_link_state().await)
}

/// 启动时复用刷新流程；失败通过 account.error 展示，不阻止联机页打开。
pub async fn initialize() -> LinkAccount {
    let _ = refresh().await;
    account().await
}

pub async fn refresh() -> Result<LinkAccount, LinkError> {
    let mut receiver = {
        let mut runtime = RUNTIME.lock().await;
        let state = manager::get_link_state().await;
        if !state.eula_accepted || runtime.pending {
            return Ok(runtime.account(&state));
        }
        if let Some(task) = &runtime.task {
            task.result.clone()
        } else {
            let expired = is_expired(&state);
            let token = state
                .naid_refresh_token
                .ok_or_else(|| LinkError::new("login_required"))?;
            let credentials = if expired { None } else { Some(credentials()?) };
            let client = manager::ConfigManager::instance().api_client.http_client();
            runtime.spawn(false, async move {
                let Some((client_id, client_secret)) = credentials else {
                    return Err(LinkError::new("naid_expired"));
                };
                refresh_profile(&client, &client_id, &client_secret, &token).await
            })
        }
    };
    let result = receiver
        .wait_for(Option::is_some)
        .await
        .map_err(|_| LinkError::new("naid_cancelled"))?
        .as_ref()
        .unwrap()
        .clone();
    result?;
    Ok(account().await)
}

/// 打开浏览器完成授权；结果通过轮询 get_link_account 获取。
pub async fn start_login() -> Result<LinkAccount, LinkError> {
    {
        let mut runtime = RUNTIME.lock().await;
        let state = manager::get_link_state().await;
        if !state.eula_accepted {
            return Err(LinkError::new("eula_required"));
        }
        if runtime.task.is_some() {
            return Ok(runtime.account(&state));
        }
        let (client_id, client_secret) = credentials()?;
        let client = manager::ConfigManager::instance().api_client.http_client();
        runtime.spawn(true, async move {
            authorize(&client, &client_id, &client_secret).await
        });
    }
    Ok(account().await)
}

/// 等待在途提交并取消旧任务；调用方在 CONTROL 锁内继续清理持久状态。
pub async fn reset_runtime() {
    RUNTIME.lock().await.cancel();
}

pub async fn logout() -> Result<LinkAccount, LinkError> {
    {
        let mut runtime = RUNTIME.lock().await;
        runtime.cancel();
        if let Err(error) = persist_credentials(None).await {
            runtime.error = Some(error.code.clone());
            return Err(error);
        }
    }
    Ok(account().await)
}

async fn persist_credentials(tokens: Option<NaidTokens>) -> Result<(), LinkError> {
    manager::update_link_state(move |state| {
        if let Some(tokens) = tokens {
            if !state.eula_accepted {
                return Err("naid_cancelled".into());
            }
            state.naid_refresh_token = Some(tokens.refresh_token);
            state.naid_refresh_expires_at = Some(tokens.refresh_expires_at);
        } else {
            state.clear_credentials();
        }
        Ok(())
    })
    .await
    .map_err(|error| {
        if error == "naid_cancelled" {
            LinkError::new(error)
        } else {
            LinkError {
                code: "naid_persist_failed".into(),
                args: vec![error],
            }
        }
    })
}

async fn authorize(
    client: &reqwest::Client,
    client_id: &str,
    client_secret: &str,
) -> Result<(NaidProfile, NaidTokens), LinkError> {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .await
        .map_err(|_| LinkError::new("naid_failed"))?;
    let port = listener
        .local_addr()
        .map_err(|_| LinkError::new("naid_failed"))?
        .port();
    let redirect_uri = format!("http://localhost:{port}/callback");

    let state = uuid::Uuid::new_v4().simple().to_string();
    let mut url = reqwest::Url::parse(AUTHORIZE_URL).map_err(|_| LinkError::new("naid_failed"))?;
    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", client_id)
        .append_pair("redirect_uri", &redirect_uri)
        .append_pair("state", &state);
    tokio::task::spawn_blocking(move || open::that(url.as_str()))
        .await
        .map_err(|_| LinkError::new("naid_failed"))?
        .map_err(|_| LinkError::new("naid_failed"))?;

    let code = tokio::time::timeout(CALLBACK_TIMEOUT, wait_callback(&listener, &state))
        .await
        .map_err(|_| LinkError::new("naid_timeout"))??;

    let mut form = HashMap::new();
    form.insert("grant_type", "authorization_code".to_string());
    form.insert("client_id", client_id.to_string());
    form.insert("client_secret", client_secret.to_string());
    form.insert("code", code);
    form.insert("redirect_uri", redirect_uri);
    let tokens = request_tokens(client, &form).await?;
    let profile = fetch_profile(client, &tokens.access_token).await?;
    Ok((profile, tokens))
}

async fn refresh_profile(
    client: &reqwest::Client,
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> Result<(NaidProfile, NaidTokens), LinkError> {
    let mut form = HashMap::new();
    form.insert("grant_type", "refresh_token".to_string());
    form.insert("client_id", client_id.to_string());
    form.insert("client_secret", client_secret.to_string());
    form.insert("refresh_token", refresh_token.to_string());
    let tokens = request_tokens(client, &form).await?;
    let profile = fetch_profile(client, &tokens.access_token).await?;
    Ok((profile, tokens))
}

async fn request_tokens(
    client: &reqwest::Client,
    form: &HashMap<&str, String>,
) -> Result<NaidTokens, LinkError> {
    let response = client
        .post(TOKEN_URL)
        .form(form)
        .timeout(REQUEST_TIMEOUT)
        .send()
        .await
        .map_err(|_| LinkError::new("naid_failed"))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|_| LinkError::new("naid_failed"))?;
    if !status.is_success() {
        return Err(LinkError::new(token_error_code(
            &body,
            status.as_u16(),
            form.get("grant_type")
                .is_some_and(|grant| grant == "refresh_token"),
        )));
    }
    let tokens: NaidTokens =
        serde_json::from_str(&body).map_err(|_| LinkError::new("naid_failed"))?;
    if tokens.access_token.trim().is_empty()
        || tokens.refresh_token.trim().is_empty()
        || parse_expiry(&tokens.refresh_expires_at).is_none()
    {
        return Err(LinkError::new("naid_failed"));
    }
    Ok(tokens)
}

async fn fetch_profile(
    client: &reqwest::Client,
    access_token: &str,
) -> Result<NaidProfile, LinkError> {
    let response = client
        .get(USER_URL)
        .bearer_auth(access_token)
        .timeout(REQUEST_TIMEOUT)
        .send()
        .await
        .map_err(|_| LinkError::new("naid_failed"))?;
    if !response.status().is_success() {
        return Err(LinkError::new(if response.status().as_u16() == 401 {
            "naid_expired"
        } else {
            "naid_failed"
        }));
    }
    let value: serde_json::Value = response
        .json()
        .await
        .map_err(|_| LinkError::new("naid_failed"))?;
    parse_profile(value)
}

fn parse_profile(value: serde_json::Value) -> Result<NaidProfile, LinkError> {
    let profile: NaidProfile = serde_json::from_value(
        value
            .get("data")
            .cloned()
            .ok_or_else(|| LinkError::new("naid_failed"))?,
    )
    .map_err(|_| LinkError::new("naid_failed"))?;
    if profile.username.trim().is_empty() {
        return Err(LinkError::new("naid_failed"));
    }
    Ok(profile)
}

fn token_error_code(body: &str, status: u16, refreshing: bool) -> &'static str {
    if body.contains("invalid_grant") {
        if refreshing {
            "naid_expired"
        } else {
            "naid_invalid_code"
        }
    } else if refreshing && (status == 401 || body.contains("invalid access token")) {
        "naid_expired"
    } else {
        "naid_failed"
    }
}

fn parse_expiry(expires_at: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::parse_from_rfc3339(expires_at)
        .map(|time| time.to_utc())
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(expires_at, "%Y-%m-%d %H:%M:%S")
                .map(|time| time.and_utc())
        })
        .or_else(|_| {
            chrono::NaiveDateTime::parse_from_str(expires_at, "%Y-%m-%dT%H:%M:%S")
                .map(|time| time.and_utc())
        })
        .ok()
}

fn is_expired(state: &LinkState) -> bool {
    state.naid_refresh_token.is_some()
        && state
            .naid_refresh_expires_at
            .as_deref()
            .and_then(parse_expiry)
            .is_none_or(|time| time <= chrono::Utc::now())
}

async fn wait_callback(listener: &TcpListener, state: &str) -> Result<String, LinkError> {
    loop {
        let (mut stream, _) = listener
            .accept()
            .await
            .map_err(|_| LinkError::new("naid_failed"))?;
        match tokio::time::timeout(
            CALLBACK_REQUEST_TIMEOUT,
            receive_callback(&mut stream, state),
        )
        .await
        {
            Ok(Ok(Some(code))) => return Ok(code),
            Ok(Err(error)) => return Err(error),
            _ => {}
        }
    }
}

async fn receive_callback(
    stream: &mut TcpStream,
    state: &str,
) -> Result<Option<String>, LinkError> {
    let mut buffer = Vec::with_capacity(1024);
    let mut chunk = [0u8; 1024];
    loop {
        let Ok(length) = stream.read(&mut chunk).await else {
            return Ok(None);
        };
        if length == 0 {
            return Ok(None);
        }
        buffer.extend_from_slice(&chunk[..length]);
        if buffer.len() > 16 * 1024 {
            respond(stream, 431, "请求过长").await;
            return Ok(None);
        }
        if buffer.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
    }
    let target = std::str::from_utf8(&buffer).ok().and_then(|request| {
        let mut line = request.lines().next()?.split_whitespace();
        let method = line.next()?;
        let target = line.next()?;
        if !matches!(method, "GET" | "POST")
            || !matches!(line.next()?, "HTTP/1.0" | "HTTP/1.1")
            || line.next().is_some()
        {
            return None;
        }
        Some(target)
    });
    let Some(target) = target else {
        respond(stream, 400, "无效请求").await;
        return Ok(None);
    };
    if target.split('?').next() != Some("/callback") {
        respond(stream, 404, "未找到").await;
        return Ok(None);
    }
    let Ok(url) = reqwest::Url::parse(&format!("http://localhost{target}")) else {
        respond(stream, 400, "无效请求").await;
        return Ok(None);
    };
    let mut parameters = HashMap::new();
    for (key, value) in url.query_pairs() {
        if parameters
            .insert(key.into_owned(), value.into_owned())
            .is_some()
        {
            respond(stream, 400, "重复参数").await;
            return Ok(None);
        }
    }
    if url.fragment().is_some() || parameters.get("state").map(String::as_str) != Some(state) {
        respond(stream, 400, "授权状态不匹配").await;
        return Ok(None);
    }
    if parameters.contains_key("error") && !parameters.contains_key("code") {
        respond(stream, 200, &callback_page(false)).await;
        return Err(LinkError::new("naid_cancelled"));
    }
    let Some(code) = parameters
        .remove("code")
        .filter(|code| !code.trim().is_empty())
    else {
        respond(stream, 400, "缺少授权码").await;
        return Ok(None);
    };
    if parameters.contains_key("error") {
        respond(stream, 400, "无效授权结果").await;
        return Ok(None);
    }
    respond(stream, 200, &callback_page(true)).await;
    Ok(Some(code))
}

async fn respond(stream: &mut tokio::net::TcpStream, status: u16, body: &str) {
    let response = format!(
        "HTTP/1.1 {status} OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes()).await;
    let _ = stream.flush().await;
}

fn callback_page(success: bool) -> String {
    let (title, message) = if success {
        (
            "授权已收到",
            "已收到 Natayark ID 授权，请返回启动器查看登录结果。",
        )
    } else {
        ("登录失败", "授权未完成，请返回启动器重试。")
    };
    format!(
        "<!doctype html><html lang=\"zh-CN\"><head><meta charset=\"utf-8\"><title>{title}</title></head><body style=\"font-family:sans-serif;text-align:center;padding-top:20vh\"><h1>{title}</h1><p>{message}</p></body></html>"
    )
}
