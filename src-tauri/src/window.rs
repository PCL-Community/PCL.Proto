//! 启动器窗口的可见性策略：按游戏会话状态隐藏、最小化、恢复或退出。
//!
//! 这是桌面应用的交互行为，不属于可复用的启动核心；决策结果通过 `WindowActions`
//! 落到真实的 Tauri 窗口上。

use std::{
    collections::HashSet,
    sync::{LazyLock, Mutex},
};

use pcl_core::minecraft::launch::session::SessionEvent;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LauncherVisibility {
    HideAndExit,
    HideAndReopen,
    Minimize,
    #[default]
    Keep,
}

/// 窗口动作；真实实现是 Tauri 窗口，测试里换成假窗口。
pub trait WindowActions {
    fn hide(&self) -> Result<(), String>;
    fn show(&self) -> Result<(), String>;
    fn minimize(&self) -> Result<(), String>;
    fn exit(&self);
}

impl WindowActions for tauri::WebviewWindow {
    fn hide(&self) -> Result<(), String> {
        self.hide().map_err(|error| error.to_string())
    }

    fn show(&self) -> Result<(), String> {
        self.show().map_err(|error| error.to_string())
    }

    fn minimize(&self) -> Result<(), String> {
        self.minimize().map_err(|error| error.to_string())
    }

    fn exit(&self) {
        use tauri::Manager;
        self.app_handle().exit(0);
    }
}

#[derive(Default)]
struct Controller {
    active: HashSet<String>,
    hidden_exit: Option<bool>,
    failed: bool,
}

impl Controller {
    fn event(
        &mut self,
        window: &impl WindowActions,
        id: &str,
        mode: LauncherVisibility,
        event: SessionEvent,
    ) {
        match event {
            SessionEvent::Spawned => {
                if self.active.is_empty() {
                    self.failed = false;
                }
                self.active.insert(id.to_owned());
            }
            SessionEvent::Ready if self.active.contains(id) => match mode {
                LauncherVisibility::HideAndExit | LauncherVisibility::HideAndReopen => {
                    let exit = mode == LauncherVisibility::HideAndExit;
                    if let Some(hidden_exit) = &mut self.hidden_exit {
                        *hidden_exit &= exit;
                    } else if let Err(error) = window.hide() {
                        log::warn!("Cannot hide launcher window: {error}");
                    } else {
                        self.hidden_exit = Some(exit);
                    }
                }
                LauncherVisibility::Minimize if self.hidden_exit.is_none() => {
                    if let Err(error) = window.minimize() {
                        log::warn!("Cannot minimize launcher window: {error}");
                    }
                }
                _ => {}
            },
            SessionEvent::Finished { safe_exit, .. } if self.active.remove(id) => {
                self.failed |= !safe_exit;
                if self.active.is_empty() {
                    if let Some(exit) = self.hidden_exit.take() {
                        if exit && !self.failed {
                            window.exit();
                        } else if let Err(error) = window.show() {
                            log::error!("Cannot restore launcher window: {error}");
                        }
                    }
                    self.failed = false;
                }
            }
            _ => {}
        }
    }
}

static CONTROLLER: LazyLock<Mutex<Controller>> =
    LazyLock::new(|| Mutex::new(Controller::default()));

pub fn handle(
    window: &tauri::WebviewWindow,
    id: &str,
    mode: LauncherVisibility,
    event: SessionEvent,
) {
    match CONTROLLER.lock() {
        Ok(mut controller) => controller.event(window, id, mode, event),
        Err(error) => log::error!("Cannot update launcher visibility: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};

    #[derive(Default)]
    struct FakeWindow {
        actions: RefCell<Vec<&'static str>>,
        fail_hide: Cell<bool>,
    }
    impl WindowActions for FakeWindow {
        fn hide(&self) -> Result<(), String> {
            self.actions.borrow_mut().push("hide");
            if self.fail_hide.get() {
                Err("failed".into())
            } else {
                Ok(())
            }
        }
        fn show(&self) -> Result<(), String> {
            self.actions.borrow_mut().push("show");
            Ok(())
        }
        fn minimize(&self) -> Result<(), String> {
            self.actions.borrow_mut().push("minimize");
            Ok(())
        }
        fn exit(&self) {
            self.actions.borrow_mut().push("exit");
        }
    }

    #[test]
    fn waits_for_ready_and_all_games_then_restores_on_failure() {
        let window = FakeWindow::default();
        let mut controller = Controller::default();
        let mode = LauncherVisibility::HideAndExit;
        controller.event(&window, "a", mode, SessionEvent::Spawned);
        controller.event(
            &window,
            "b",
            LauncherVisibility::Keep,
            SessionEvent::Spawned,
        );
        assert!(window.actions.borrow().is_empty());
        controller.event(&window, "a", mode, SessionEvent::Ready);
        controller.event(
            &window,
            "a",
            mode,
            SessionEvent::Finished {
                safe_exit: true,
                failure: None,
            },
        );
        assert_eq!(*window.actions.borrow(), ["hide"]);
        controller.event(
            &window,
            "b",
            LauncherVisibility::Keep,
            SessionEvent::Finished {
                safe_exit: false,
                failure: None,
            },
        );
        assert_eq!(*window.actions.borrow(), ["hide", "show"]);
        controller.event(&window, "c", mode, SessionEvent::Spawned);
        controller.event(&window, "c", mode, SessionEvent::Ready);
        controller.event(
            &window,
            "c",
            mode,
            SessionEvent::Finished {
                safe_exit: true,
                failure: None,
            },
        );
        assert_eq!(*window.actions.borrow(), ["hide", "show", "hide", "exit"]);
    }

    #[test]
    fn failed_hide_never_exits_and_reopen_wins_over_exit() {
        let window = FakeWindow::default();
        let mut controller = Controller::default();
        window.fail_hide.set(true);
        controller.event(
            &window,
            "a",
            LauncherVisibility::HideAndExit,
            SessionEvent::Spawned,
        );
        controller.event(
            &window,
            "a",
            LauncherVisibility::HideAndExit,
            SessionEvent::Ready,
        );
        controller.event(
            &window,
            "a",
            LauncherVisibility::HideAndExit,
            SessionEvent::Finished {
                safe_exit: true,
                failure: None,
            },
        );
        assert_eq!(*window.actions.borrow(), ["hide"]);
        window.fail_hide.set(false);
        controller.event(
            &window,
            "a",
            LauncherVisibility::HideAndExit,
            SessionEvent::Spawned,
        );
        controller.event(
            &window,
            "b",
            LauncherVisibility::HideAndReopen,
            SessionEvent::Spawned,
        );
        controller.event(
            &window,
            "a",
            LauncherVisibility::HideAndExit,
            SessionEvent::Ready,
        );
        controller.event(
            &window,
            "b",
            LauncherVisibility::HideAndReopen,
            SessionEvent::Ready,
        );
        controller.event(
            &window,
            "a",
            LauncherVisibility::HideAndExit,
            SessionEvent::Finished {
                safe_exit: true,
                failure: None,
            },
        );
        controller.event(
            &window,
            "b",
            LauncherVisibility::HideAndReopen,
            SessionEvent::Finished {
                safe_exit: true,
                failure: None,
            },
        );
        assert_eq!(*window.actions.borrow(), ["hide", "hide", "show"]);
    }
}
