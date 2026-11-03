//! Interactive login sessions.
//!
//! CE 的登录由 `ProfileService.AuthenticateAsync` 驱动，交互部分（设备码弹窗、
//! 角色选择弹窗）通过回调回到 WPF。Proto 把同一条流程放进一个后台任务：任务把
//! 当前状态发布到会话里，前端轮询状态并弹窗，取消与选择通过命令回传。

mod login;

use std::collections::HashMap;
use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

use self::login::login;
use super::oauth::{AuthorizeResult, DeviceCodeData, classify};
use super::{AuthError, AuthResult, Candidate};
use crate::minecraft::profile::{ProfileType, SafeProfile};

pub use self::login::{login_for_launch, refresh_microsoft};

/// 用户取消登录时使用的错误码。
pub const CANCELLED: &str = "login_cancelled";
/// 需要用户介入但当前没有界面（启动流程）。
pub const INTERACTION_REQUIRED: &str = "interaction_required";

static SESSIONS: LazyLock<Mutex<HashMap<String, Arc<Session>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// 会话状态，前端轮询它来决定显示什么。
#[derive(Clone, Debug, Default, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum View {
    /// 正在连接认证服务器。
    #[default]
    Working,
    /// 设备码已就绪：展示验证码并自动打开网页（对照 CE 的 `MyMsgLogin`）。
    #[serde(rename_all = "camelCase")]
    DeviceCode {
        user_code: String,
        verification_uri: String,
        verification_uri_complete: Option<String>,
        interval: u64,
    },
    /// 服务器返回多个角色，等待用户选择。
    SelectProfile {
        candidates: Vec<Candidate>,
    },
    Succeeded {
        profile: SafeProfile,
    },
    Failed {
        code: String,
        message: String,
    },
    Cancelled,
}

impl View {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            View::Succeeded { .. } | View::Failed { .. } | View::Cancelled
        )
    }
}

#[derive(Default)]
struct Session {
    view: Mutex<View>,
    cancelled: AtomicBool,
    selection: Mutex<Option<oneshot::Sender<Option<String>>>>,
}

impl Session {
    fn publish(&self, view: View) {
        *self.view.lock().unwrap() = view;
    }

    fn current(&self) -> View {
        self.view.lock().unwrap().clone()
    }

    fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }
}

/// 前端提交的登录请求，对照 CE 的 `AuthenticationRequest`。
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LoginRequest {
    pub profile_type: ProfileType,
    /// 第三方认证服务器地址（authlib 的 API 根地址或 Yggdrasil Connect 的服务器地址）。
    pub server: Option<String>,
    /// Yggdrasil Connect 的 OpenID 发现地址。
    pub discovery_address: Option<String>,
    pub client_id: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    /// 复用已有档案（启动前刷新）；登录页新建档案时留空。
    pub profile_id: Option<String>,
    pub force_refresh: bool,
    pub force_reselect_profile: bool,
}

/// 交互界面：登录页有会话可以弹窗，启动流程没有。
enum Ui {
    Session(Arc<Session>),
    Headless,
}

impl Ui {
    fn session(&self) -> Option<&Arc<Session>> {
        match self {
            Ui::Session(session) => Some(session),
            Ui::Headless => None,
        }
    }

    fn cancelled(&self) -> bool {
        self.session().is_some_and(|session| session.is_cancelled())
    }

    /// 展示设备码并轮询到拿到令牌，对照 CE `MyMsgLogin.WorkThreadAsync`。
    async fn device_code<F, Fut>(
        &self,
        data: DeviceCodeData,
        poll: F,
    ) -> AuthResult<AuthorizeResult>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = AuthResult<AuthorizeResult>>,
    {
        let Some(session) = self.session() else {
            return Err(AuthError::new(
                INTERACTION_REQUIRED,
                "登录需要用户授权，请回到账号页面重新登录。",
            ));
        };
        session.publish(View::DeviceCode {
            user_code: data.user_code.clone().unwrap_or_default(),
            verification_uri: data.verification_uri.clone().unwrap_or_default(),
            verification_uri_complete: data.verification_uri_complete.clone(),
            interval: data.poll_interval(),
        });
        // CE 在弹窗 2 秒后自动打开网页并开始轮询。
        if !sleep_cancellable(session, Duration::from_secs(2)).await {
            return Err(cancelled());
        }
        let mut interval = Duration::from_secs(data.poll_interval());
        loop {
            if session.is_cancelled() {
                return Err(cancelled());
            }
            match classify(poll().await?) {
                super::oauth::PollOutcome::Authorized(result) => return Ok(result),
                super::oauth::PollOutcome::Rejected(error) => return Err(error),
                super::oauth::PollOutcome::SlowDown => interval += Duration::from_secs(5),
                super::oauth::PollOutcome::Pending => {}
            }
            if !sleep_cancellable(session, interval).await {
                return Err(cancelled());
            }
        }
    }

    /// 让用户在多个角色里选一个；返回 `None` 表示取消。
    async fn select_profile(&self, candidates: Vec<Candidate>) -> AuthResult<Option<Candidate>> {
        let Some(session) = self.session() else {
            return Err(AuthError::new(
                INTERACTION_REQUIRED,
                "该账号有多个角色，请在账号页面重新登录以选择角色。",
            ));
        };
        let (sender, receiver) = oneshot::channel();
        *session.selection.lock().unwrap() = Some(sender);
        session.publish(View::SelectProfile {
            candidates: candidates.clone(),
        });
        let selected = receiver.await.unwrap_or(None);
        *session.selection.lock().unwrap() = None;
        if session.is_cancelled() {
            return Err(cancelled());
        }
        Ok(selected.and_then(|id| candidates.into_iter().find(|candidate| candidate.id == id)))
    }
}

/// 分片睡眠，保证取消能及时生效。
async fn sleep_cancellable(session: &Arc<Session>, duration: Duration) -> bool {
    let mut remaining = duration;
    let step = Duration::from_millis(200);
    while remaining > Duration::ZERO {
        if session.is_cancelled() {
            return false;
        }
        let sleep = step.min(remaining);
        tokio::time::sleep(sleep).await;
        remaining = remaining.saturating_sub(sleep);
    }
    !session.is_cancelled()
}

fn cancelled() -> AuthError {
    AuthError::new(CANCELLED, "用户取消了登录。")
}

/// 新建一个登录会话并返回会话 ID。
pub fn start(request: LoginRequest) -> AuthResult<String> {
    if request.profile_type == ProfileType::Offline {
        return Err(AuthError::configuration("离线档案不需要远程认证"));
    }
    let id = uuid::Uuid::new_v4().simple().to_string();
    let session = Arc::new(Session::default());
    let mut sessions = SESSIONS.lock().unwrap();
    // 顺手清掉没被读取的已结束会话，避免界面提前离开时泄漏。
    sessions.retain(|_, existing| !existing.current().is_terminal());
    sessions.insert(id.clone(), session.clone());
    drop(sessions);
    tauri::async_runtime::spawn(async move {
        let view = match login(request, Ui::Session(session.clone())).await {
            Ok(profile) => View::Succeeded {
                profile: SafeProfile::from(&profile),
            },
            Err(error) if error.code == CANCELLED => View::Cancelled,
            Err(error) => View::Failed {
                code: error.code,
                message: error.message,
            },
        };
        session.publish(view);
    });
    Ok(id)
}

/// 读取会话状态；已结束的会话在读完后清理。
pub fn view(session_id: &str) -> Option<View> {
    let mut sessions = SESSIONS.lock().unwrap();
    let session = sessions.get(session_id)?.clone();
    let view = session.current();
    if view.is_terminal() {
        sessions.remove(session_id);
    }
    Some(view)
}

pub fn cancel(session_id: &str) {
    let Some(session) = SESSIONS.lock().unwrap().get(session_id).cloned() else {
        return;
    };
    session.cancelled.store(true, Ordering::Relaxed);
    // 唤醒正在等待角色选择的任务。
    if let Some(sender) = session.selection.lock().unwrap().take() {
        let _ = sender.send(None);
    }
}

/// 提交角色选择；`candidate_id` 为空表示取消。
pub fn select(session_id: &str, candidate_id: Option<String>) -> AuthResult<()> {
    let session = SESSIONS
        .lock()
        .unwrap()
        .get(session_id)
        .cloned()
        .ok_or_else(|| AuthError::new("session_not_found", "登录会话已结束，请重新登录。"))?;
    let sender = session
        .selection
        .lock()
        .unwrap()
        .take()
        .ok_or_else(|| AuthError::new("session_not_found", "当前登录不需要选择角色。"))?;
    sender
        .send(candidate_id)
        .map_err(|_| AuthError::new("session_not_found", "登录会话已结束，请重新登录。"))
}
