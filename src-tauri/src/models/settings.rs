use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default)]
    pub key_bindings: KeyBindings,
    #[serde(default)]
    pub window_size: WindowSize,
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
            window_size: WindowSize::default(),
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
pub struct WindowSize {
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
}

fn default_width() -> u32 {
    1280
}
fn default_height() -> u32 {
    900
}

impl Default for WindowSize {
    fn default() -> Self {
        Self {
            width: default_width(),
            height: default_height(),
        }
    }
}
