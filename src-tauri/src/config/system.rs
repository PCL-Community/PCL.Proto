//! 杂项页的系统偏好：调试日志与随机延迟。

use crate::config::states::AppState;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct SystemPreferences {
    pub debug_mode: bool,
    pub random_delay: bool,
}

impl Default for SystemPreferences {
    fn default() -> Self {
        Self {
            debug_mode: false,
            random_delay: false,
        }
    }
}

impl From<&AppState> for SystemPreferences {
    fn from(state: &AppState) -> Self {
        Self {
            debug_mode: state.debug_mode,
            random_delay: state.random_delay,
        }
    }
}

impl SystemPreferences {
    pub fn apply(&self, state: &mut AppState) {
        state.debug_mode = self.debug_mode;
        state.random_delay = self.random_delay;
    }
}
