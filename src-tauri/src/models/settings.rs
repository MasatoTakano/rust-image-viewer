use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default)]
    pub key_bindings: KeyBindings,
    #[serde(default)]
    pub window_state: WindowState,
    #[serde(default = "default_background_color")]
    pub background_color: String,
    #[serde(default = "default_preload_range")]
    pub preload_range: u32,
    #[serde(default = "default_key_throttle_ms")]
    pub key_throttle_ms: u32,
    #[serde(default = "default_wheel_throttle_ms")]
    pub wheel_throttle_ms: u32,
    #[serde(default = "default_display_mode")]
    pub display_mode: String,
}

fn default_background_color() -> String {
    "#000000".to_string()
}

fn default_preload_range() -> u32 {
    10
}

fn default_key_throttle_ms() -> u32 {
    40
}

fn default_wheel_throttle_ms() -> u32 {
    150
}

fn default_display_mode() -> String {
    "single".to_string()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            key_bindings: KeyBindings::default(),
            window_state: WindowState::default(),
            background_color: default_background_color(),
            preload_range: default_preload_range(),
            key_throttle_ms: default_key_throttle_ms(),
            wheel_throttle_ms: default_wheel_throttle_ms(),
            display_mode: default_display_mode(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindings {
    #[serde(default = "default_next_page")]
    pub next_page: Vec<String>,
    #[serde(default = "default_prev_page")]
    pub prev_page: Vec<String>,
    #[serde(default = "default_toggle_fullscreen")]
    pub toggle_fullscreen: Vec<String>,
    #[serde(default = "default_toggle_spread")]
    pub toggle_spread: Vec<String>,
    #[serde(default = "default_go_first")]
    pub go_first: Vec<String>,
    #[serde(default = "default_go_last")]
    pub go_last: Vec<String>,
    #[serde(default = "default_open_settings")]
    pub open_settings: Vec<String>,
}

fn default_next_page() -> Vec<String> {
    vec!["ArrowLeft".into()]
}
fn default_prev_page() -> Vec<String> {
    vec!["ArrowRight".into()]
}
fn default_toggle_fullscreen() -> Vec<String> {
    vec!["Enter".into()]
}
fn default_toggle_spread() -> Vec<String> {
    vec!["Space".into()]
}
fn default_go_first() -> Vec<String> {
    vec!["Home".into()]
}
fn default_go_last() -> Vec<String> {
    vec!["End".into()]
}
fn default_open_settings() -> Vec<String> {
    vec!["Escape".into()]
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            next_page: default_next_page(),
            prev_page: default_prev_page(),
            toggle_fullscreen: default_toggle_fullscreen(),
            toggle_spread: default_toggle_spread(),
            go_first: default_go_first(),
            go_last: default_go_last(),
            open_settings: default_open_settings(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    #[serde(default)]
    pub x: Option<i32>,
    #[serde(default)]
    pub y: Option<i32>,
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            x: None,
            y: None,
            width: default_width(),
            height: default_height(),
        }
    }
}

fn default_width() -> u32 {
    1280
}
fn default_height() -> u32 {
    900
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_state_default_position_none() {
        let ws = WindowState::default();
        assert_eq!(ws.x, None);
        assert_eq!(ws.y, None);
        assert_eq!(ws.width, 1280);
        assert_eq!(ws.height, 900);
    }

    #[test]
    fn deserialize_old_format_with_window_size() {
        let old = r##"{
            "key_bindings": { "next_page": ["ArrowLeft"], "prev_page": ["ArrowRight"], "toggle_fullscreen": ["Enter"], "toggle_spread": ["Space"], "go_first": ["Home"], "go_last": ["End"], "open_settings": ["Escape"] },
            "window_size": { "width": 800, "height": 600 },
            "background_color": "#111111",
            "preload_range": 5,
            "key_throttle_ms": 40,
            "wheel_throttle_ms": 150,
            "display_mode": "spread"
        }"##;
        let s: AppSettings = serde_json::from_str(old).expect("旧形式は前方互換で読み込めること");
        assert_eq!(s.window_state.width, 1280);
        assert_eq!(s.window_state.x, None);
        assert_eq!(s.background_color, "#111111");
        assert_eq!(s.display_mode, "spread");
    }

    #[test]
    fn window_state_roundtrip() {
        let ws = WindowState { x: Some(-100), y: Some(200), width: 800, height: 600 };
        let json = serde_json::to_string(&ws).unwrap();
        let back: WindowState = serde_json::from_str(&json).unwrap();
        assert_eq!(back.x, Some(-100));
        assert_eq!(back.y, Some(200));
        assert_eq!(back.width, 800);
        assert_eq!(back.height, 600);
    }

    #[test]
    fn app_settings_roundtrip_preserves_window_state() {
        let s = AppSettings {
            window_state: WindowState { x: Some(10), y: Some(20), width: 1000, height: 700 },
            ..AppSettings::default()
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(back.window_state.x, Some(10));
        assert_eq!(back.window_state.width, 1000);
    }
}
