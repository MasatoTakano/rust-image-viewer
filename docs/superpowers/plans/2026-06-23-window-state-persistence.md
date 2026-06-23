# ウィンドウ状態(サイズ・位置)永続化 実装計画

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Windows 版で終了時のウィンドウサイズ・位置を確実に保存し、次回起動時に復元する。フルスクリーン/最大化状態で終了した場合は、最後に保存した通常ウィンドウ状態を復元する。

**Architecture:** ウィンドウ状態の永続化をフロントエンド(`beforeunload` + 非同期IPC)から Rust 側(`CloseRequested` イベント)へ完全移行。Rust の `outer_size/outer_position` で真のジオメトリを取得し、`fullscreen/is_maximized` で保存可否を判定。`setup` で復元、`CloseRequested` で保存。`display_mode` はトグル時に即時保存へ変更。

**Tech Stack:** Tauri v2, Rust, Vue 3 + TypeScript, serde_json

**参照仕様:** `docs/superpowers/specs/2026-06-23-window-state-persistence-design.md`

**注意:** 本プロジェクトは現在 main ブランチで作業中。コミットメッセージは既存スタイル(`feat:` / `fix:` / `refactor:` / `docs:` プレフィックス、日本語可)に合わせる。

---

## ファイル構成

| ファイル | 責務 | 変更種別 |
|---|---|---|
| `src-tauri/src/models/settings.rs` | `WindowSize`→`WindowState` モデル、serde 後方互換 | 修正 |
| `src-tauri/src/commands/settings.rs` | ファイル IO の `pub(crate)` ヘルパ抽出(DRY) | 修正 |
| `src-tauri/src/commands/window_state.rs` | 復元/保存ロジック + 画面外ガード純粋関数 | 新規 |
| `src-tauri/src/commands/mod.rs` | `window_state` モジュール登録 | 修正 |
| `src-tauri/src/lib.rs` | `setup` で復元、`on_window_event` で保存 | 修正 |
| `src/types/index.ts` | `WindowSize`→`WindowState` 型 | 修正 |
| `src/composables/useSettings.ts` | `createDefaults` の `window_state` 化 | 修正 |
| `src/composables/usePageController.ts` | `toggleDisplayMode` で display_mode 即時保存 | 修正 |
| `src/App.vue` | `onSaveState`/`setSize` 復元を削除 | 修正 |
| `README.md` | 「起動時の状態復元」節へ位置・フルスクリーン挙動を追記 | 修正 |

---

## Task 1: WindowState モデル + serde 後方互換 (TDD)

**Files:**
- Modify: `src-tauri/src/models/settings.rs`

- [ ] **Step 1: 既存テストが通ることを確認(ベースライン)**

Run (in `src-tauri/`): `cargo test`
Expected: 既存テストすべて PASS(`test_supported_extensions` 等)

- [ ] **Step 2: 失敗するテストを追加**

`src-tauri/src/models/settings.rs` の末尾(`impl Default for WindowSize` の後、ファイル終端)に以下を追加。ただし `WindowSize` はこの時点でまだ存在するため、`WindowState` を参照するテストはコンパイルエラーになるのが意図的(RED)。

```rust
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
        let old = r#"{
            "key_bindings": { "next_page": ["ArrowLeft"], "prev_page": ["ArrowRight"], "toggle_fullscreen": ["Enter"], "toggle_spread": ["Space"], "go_first": ["Home"], "go_last": ["End"], "open_settings": ["Escape"] },
            "window_size": { "width": 800, "height": 600 },
            "background_color": "#111111",
            "preload_range": 5,
            "key_throttle_ms": 40,
            "wheel_throttle_ms": 150,
            "display_mode": "spread"
        }"#;
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
```

- [ ] **Step 3: テストが失敗することを確認**

Run (in `src-tauri/`): `cargo test`
Expected: FAIL / コンパイルエラー(`WindowState` 未定義、`window_state` フィールドなし)

- [ ] **Step 4: モデルを実装**

`src-tauri/src/models/settings.rs` で以下の3箇所を変更する。

(1) `AppSettings` 構造体のフィールド `window_size` を `window_state` に置換:

```rust
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
```

(2) `impl Default for AppSettings` の `window_size: WindowSize::default()` を `window_state: WindowState::default()` に置換:

```rust
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
```

(3) `WindowSize` 構造体 + `impl Default for WindowSize` ブロック全体を、以下の `WindowState` 定義で置換(`default_width` / `default_height` 関数はそのまま残す):

```rust
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
```

- [ ] **Step 5: テストが通ることを確認**

Run (in `src-tauri/`): `cargo test`
Expected: 全テスト PASS(追加4件 + 既存分)

- [ ] **Step 6: コミット**

```bash
git add src-tauri/src/models/settings.rs
git commit -m "refactor: WindowSize を位置対応の WindowState へ拡張(serde 後方互換)"
```

---

## Task 2: settings.rs のファイル IO ヘルパ抽出 (DRY)

**Files:**
- Modify: `src-tauri/src/commands/settings.rs`

`window_state.rs`(Task 3〜)から settings.json の読込・書込を再利用できるよう、内部ヘルパを `pub(crate)` で抽出する。コマンド関数はこれらを呼ぶように整理し、振る舞いは不变。

- [ ] **Step 1: ヘルパ関数を追加し、コマンドをリファクタ**

`src-tauri/src/commands/settings.rs` を以下の全体内容で置換:

```rust
use crate::models::settings::AppSettings;

/// 設定ファイルのパスを取得する
pub(crate) fn get_settings_path() -> Result<std::path::PathBuf, String> {
    let config_dir =
        dirs::config_dir().ok_or_else(|| "設定ディレクトリのパスを取得できません".to_string())?;
    let app_dir = config_dir.join("rust-image-viewer");
    std::fs::create_dir_all(&app_dir)
        .map_err(|e| format!("設定ディレクトリの作成に失敗: {}", e))?;
    Ok(app_dir.join("settings.json"))
}

/// 設定をファイルから読み込む。ファイルが存在しない場合はデフォルト設定を返す
pub(crate) fn read_settings_file() -> Result<AppSettings, String> {
    let path = get_settings_path()?;

    if !path.exists() {
        return Ok(AppSettings::default());
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("設定ファイルの読み取りに失敗: {}", e))?;

    let settings: AppSettings = serde_json::from_str(&content).map_err(|e| {
        format!(
            "設定ファイルのパースに失敗: {}。デフォルト設定を使用してください。",
            e
        )
    })?;

    Ok(settings)
}

/// 設定をファイルへ書き込む
pub(crate) fn write_settings_file(settings: &AppSettings) -> Result<(), String> {
    let path = get_settings_path()?;

    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("設定のシリアライズに失敗: {}", e))?;

    std::fs::write(&path, content).map_err(|e| format!("設定ファイルの書き込みに失敗: {}", e))?;

    Ok(())
}

/// 設定を読み込むコマンド(フロントエンド向け)
#[tauri::command]
pub fn load_settings() -> Result<AppSettings, String> {
    read_settings_file()
}

/// 設定を保存するコマンド(フロントエンド向け)
#[tauri::command]
pub fn save_settings(settings: AppSettings) -> Result<(), String> {
    write_settings_file(&settings)
}
```

- [ ] **Step 2: ビルドとテストが通ることを確認**

Run (in `src-tauri/`): `cargo test`
Expected: 全テスト PASS、コンパイル成功

- [ ] **Step 3: コミット**

```bash
git add src-tauri/src/commands/settings.rs
git commit -m "refactor: settings.rs のファイル IO を pub(crate) ヘルパへ分離"
```

---

## Task 3: window_state モジュール + 画面外ガード純粋関数 (TDD)

**Files:**
- Create: `src-tauri/src/commands/window_state.rs`
- Modify: `src-tauri/src/commands/mod.rs`

- [ ] **Step 1: モジュールを登録**

`src-tauri/src/commands/mod.rs` を以下に置換:

```rust
pub mod file_loader;
pub mod image_provider;
pub mod settings;
pub mod window_state;
```

- [ ] **Step 2: 純粋関数の失敗テストを含むモジュールを作成**

`src-tauri/src/commands/window_state.rs` を新規作成(実装未の `is_position_visible` を参照するテスト → RED):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_inside_monitor() {
        let monitors = [MonitorRect { x: 0, y: 0, width: 1920, height: 1080 }];
        assert!(is_position_visible((100, 100), &monitors));
    }

    #[test]
    fn not_visible_outside_any_monitor() {
        let monitors = [MonitorRect { x: 0, y: 0, width: 1920, height: 1080 }];
        assert!(!is_position_visible((5000, 5000), &monitors));
    }

    #[test]
    fn visible_at_top_left_corner() {
        let monitors = [MonitorRect { x: 0, y: 0, width: 100, height: 100 }];
        assert!(is_position_visible((0, 0), &monitors));
    }

    #[test]
    fn not_visible_just_past_edge() {
        let monitors = [MonitorRect { x: 0, y: 0, width: 100, height: 100 }];
        assert!(!is_position_visible((100, 50), &monitors));
    }

    #[test]
    fn visible_in_second_monitor() {
        let monitors = [
            MonitorRect { x: 0, y: 0, width: 1920, height: 1080 },
            MonitorRect { x: 1920, y: 0, width: 1920, height: 1080 },
        ];
        assert!(is_position_visible((2000, 500), &monitors));
        assert!(!is_position_visible((1919, 500), &monitors));
    }

    #[test]
    fn visible_negative_coords_multi_monitor() {
        let monitors = [
            MonitorRect { x: -1920, y: 0, width: 1920, height: 1080 },
            MonitorRect { x: 0, y: 0, width: 1920, height: 1080 },
        ];
        assert!(is_position_visible((-1000, 500), &monitors));
    }

    #[test]
    fn not_visible_when_no_monitors() {
        assert!(!is_position_visible((100, 100), &[]));
    }
}
```

- [ ] **Step 3: テストが失敗することを確認**

Run (in `src-tauri/`): `cargo test`
Expected: FAIL / コンパイルエラー(`MonitorRect`, `is_position_visible` 未定義)

- [ ] **Step 4: 純粋関数を実装**

`src-tauri/src/commands/window_state.rs` の先頭(テストの前)に以下を実装:

```rust
/// モニタの論理座標系矩形
#[derive(Debug, Clone, Copy)]
pub(crate) struct MonitorRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

/// 位置がいずれかのモニタ矩形内にあるか(画面外復元のガード用)
pub(crate) fn is_position_visible(pos: (i32, i32), monitors: &[MonitorRect]) -> bool {
    let (x, y) = pos;
    monitors
        .iter()
        .any(|m| x >= m.x && x < m.x + m.width && y >= m.y && y < m.y + m.height)
}
```

- [ ] **Step 5: テストが通ることを確認**

Run (in `src-tauri/`): `cargo test`
Expected: 追加7件のテストすべて PASS

- [ ] **Step 6: コミット**

```bash
git add src-tauri/src/commands/window_state.rs src-tauri/src/commands/mod.rs
git commit -m "feat: window_state モジュールと画面外ガード純粋関数を追加"
```

---

## Task 4: 復元・保存ロジックの実装(統合関数)

**Files:**
- Modify: `src-tauri/src/commands/window_state.rs`

Tauri ウィンドウ API を直接叩く統合関数は単体テスト困難のため、`cargo check` と最終手動検証(Task 9)で検証する。

- [ ] **Step 1: 復元・保存関数を追加**

`src-tauri/src/commands/window_state.rs` の先頭に import と2関数を追加(`MonitorRect`/`is_position_visible` の前、ファイル先頭):

```rust
use crate::commands::settings::{read_settings_file, write_settings_file};
use crate::models::settings::AppSettings;
use tauri::{LogicalPosition, LogicalSize, Monitor, WebviewWindow};

/// モニタ情報から論理座標系の矩形一覧を構築
fn collect_monitor_rects(window: &WebviewWindow) -> Vec<MonitorRect> {
    window
        .available_monitors()
        .unwrap_or_default()
        .iter()
        .map(|m: &Monitor| {
            let scale = m.scale_factor();
            let pos = m.position().to_logical::<f64>(scale);
            let size = m.size().to_logical::<f64>(scale);
            MonitorRect {
                x: pos.x.round() as i32,
                y: pos.y.round() as i32,
                width: size.width.round() as i32,
                height: size.height.round() as i32,
            }
        })
        .collect()
}

/// 起動時に保存済みウィンドウ状態を復元する。
/// 常にサイズを適用し、位置は有効モニタ内の場合のみ適用する。
/// フルスクリーン・最大化は復元しない(常に通常ウィンドウ)。
pub fn restore_window_state(window: &WebviewWindow, settings: &AppSettings) {
    let ws = &settings.window_state;

    let _ = window.set_size(LogicalSize::new(ws.width as f64, ws.height as f64));

    if let (Some(x), Some(y)) = (ws.x, ws.y) {
        let monitors = collect_monitor_rects(window);
        if is_position_visible((x, y), &monitors) {
            let _ = window.set_position(LogicalPosition::new(x as f64, y as f64));
        }
    }
}

/// 終了時にウィンドウ状態を保存する。
/// フルスクリーン中または最大化中はスキップし、最後の通常ウィンドウ状態を保持する。
pub fn save_window_state(window: &WebviewWindow) {
    if window.fullscreen().is_some() {
        return;
    }
    if window.is_maximized().unwrap_or(false) {
        return;
    }

    let pos = match window.outer_position() {
        Ok(p) => p,
        Err(_) => return,
    };
    let size = match window.outer_size() {
        Ok(s) => s,
        Err(_) => return,
    };

    let scale = window.scale_factor();
    let logical_pos = pos.to_logical::<f64>(scale);
    let logical_size = size.to_logical::<f64>(scale);

    if let Ok(mut settings) = read_settings_file() {
        settings.window_state.x = Some(logical_pos.x.round() as i32);
        settings.window_state.y = Some(logical_pos.y.round() as i32);
        settings.window_state.width = logical_size.width.round() as u32;
        settings.window_state.height = logical_size.height.round() as u32;
        let _ = write_settings_file(&settings);
    }
}
```

- [ ] **Step 2: コンパイルが通ることを確認(API 表面の検証)**

Run (in `src-tauri/`): `cargo check`
Expected: 成功。もし `WebviewWindow` のメソッド(`set_size` / `set_position` / `outer_position` / `outer_size` / `fullscreen` / `is_maximized` / `available_monitors` / `scale_factor`)や `Monitor`、`to_logical` の型不一致でエラーが出た場合は、コンパイラの指示に従い import や型パラメータ(`to_logical::<i32>` 等)を調整する。ロジック本体は変更しない。

- [ ] **Step 3: テストが引き続き通ることを確認**

Run (in `src-tauri/`): `cargo test`
Expected: 全テスト PASS

- [ ] **Step 4: コミット**

```bash
git add src-tauri/src/commands/window_state.rs
git commit -m "feat: ウィンドウ状態の復元・保存関数を実装"
```

---

## Task 5: lib.rs への組み込み(起動時復元 + 終了時保存)

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: import と組み込みを追加**

`src-tauri/src/lib.rs` を以下の全体内容で置換:

```rust
mod archive;
mod commands;
mod models;
mod utils;

use commands::{file_loader, image_provider, settings as settings_cmd, window_state};
use tauri::Emitter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let cli_args: Vec<String> = std::env::args().skip(1).collect();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            file_loader::load_source,
            image_provider::get_image,
            settings_cmd::load_settings,
            settings_cmd::save_settings,
        ])
        .setup(move |app| {
            if !cli_args.is_empty() {
                let path = cli_args[0].clone();
                let handle = app.handle().clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    let _ = handle.emit("cli-file-open", path);
                });
            }

            if let Some(window) = app.get_webview_window("main") {
                if let Ok(s) = settings_cmd::read_settings_file() {
                    window_state::restore_window_state(&window, &s);
                }
                let win = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { .. } = event {
                        window_state::save_window_state(&win);
                    }
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 2: dev ビルドでコンパイル確認**

Run (in `src-tauri/`): `cargo build`
Expected: 成功。`on_window_event` が `WebviewWindow` に無い等のエラーが出た場合はフォールバック参照(末尾「API フォールバック」)へ。

- [ ] **Step 3: コミット**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: 起動時のウィンドウ状態復元と終了時保存を Rust 側へ組み込み"
```

**API フォールバック:** Task 5 Step2 で `WebviewWindow::on_window_event` が解決しない場合、グローバルハンドラへ切替:
- `lib.rs` の `if let Some(window) = ...` ブロックから `on_window_event` 部分を削除
- `.setup(...)` の直後、`.run(...)` の前に以下を追加:
```rust
let app_handle = app.handle().clone();
// setup 内で:
app.on_window_event(move |_window, event| {
    if let tauri::WindowEvent::CloseRequested { .. } = event {
        if let Some(win) = app_handle.get_webview_window("main") {
            window_state::save_window_state(&win);
        }
    }
});
```
※ `_window`(=&Window)は使わず、`app_handle` から `WebviewWindow` を再取得することで関数シグニチャ(`&WebviewWindow`)を維持。

---

## Task 6: フロントエンド一括更新(型 + App.vue + display_mode 即時保存)

**Files:**
- Modify: `src/App.vue`
- Modify: `src/composables/usePageController.ts`
- Modify: `src/composables/useSettings.ts`
- Modify: `src/types/index.ts`

**注意:** 型リネーム(`window_size`→`window_state`)と `App.vue` からの `window_size` 参照削除を **1タスクで完結** させ、最後に1回だけビルド検証して1コミットとする(中間状態でコンパイルが壊れないよう、すべての編集をしてからビルド・コミットする)。

- [ ] **Step 1: App.vue から invoke import を削除**

`src/App.vue` の import セクション(26行目付近):
```typescript
import { invoke } from "@tauri-apps/api/core";
```
この1行を削除。

- [ ] **Step 2: App.vue の usePageController 分割から未使用 displayMode を削除**

43行目付近:
```typescript
const { setDisplayMode, displayMode, resetIndex } = usePageController();
```
を以下に置換(`displayMode` は onSaveState 削除で不要になる):
```typescript
const { setDisplayMode, resetIndex } = usePageController();
```

- [ ] **Step 3: App.vue の onSaveState 関数を削除**

94-102行目の `function onSaveState() { ... }` 全体を削除:
```typescript
function onSaveState() {
  const currentSettings = settings.settings.value;
  const newSettings = {
    ...currentSettings,
    window_size: { width: window.innerWidth, height: window.innerHeight },
    display_mode: displayMode.value,
  };
  invoke("save_settings", { settings: newSettings }).catch(() => {});
}
```

- [ ] **Step 4: App.vue の onMounted から setSize 復元ブロックを削除**

onMounted 内の以下のブロック(112-118行目付近)を削除:
```typescript
  try {
    const { LogicalSize } = await import("@tauri-apps/api/dpi");
    await getCurrentWindow().setSize(new LogicalSize(
      s.window_size.width,
      s.window_size.height,
    ));
  } catch {}
```
※ `s` 変数の読み出し(`const s = await settings.load();` 等)は onMounted の冒頭でそのまま残し、`setKeyBindings(s.key_bindings)` と `setDisplayMode(...)` で引き続き使用する。

- [ ] **Step 5: App.vue の beforeunload リスナ登録/削除を削除**

onMounted 内の `window.addEventListener("beforeunload", onSaveState);`(124行目付近)を1行削除。
onUnmounted 内の `window.removeEventListener("beforeunload", onSaveState);`(135行目付近)を1行削除。

- [ ] **Step 6: usePageController で display_mode を即時保存**

`src/composables/usePageController.ts` を以下の通り変更。

(1) import に useSettings を追加(3行目 `useImageStore` import の後):
```typescript
import { useImageStore } from "./useImageStore";
import { useSettings } from "./useSettings";
```

(2) `usePageController()` 関数内の先頭(`const { entries, preloadAround } = useImageStore();` の次)に useSettings を追加:
```typescript
  const { entries, preloadAround } = useImageStore();
  const { settings, save } = useSettings();
```

(3) `toggleDisplayMode` 関数(61-66行)を以下に置換:
```typescript
  async function persistDisplayMode() {
    try {
      const newSettings = {
        ...settings.value,
        display_mode: displayMode.value,
      };
      await save(newSettings);
    } catch {
      // 保存失敗はユーザー操作に影響させないため無視
    }
  }

  function toggleDisplayMode() {
    displayMode.value =
      displayMode.value === DisplayMode.Single
        ? DisplayMode.Spread
        : DisplayMode.Single;
    void persistDisplayMode();
  }
```

- [ ] **Step 7: types/index.ts の WindowState 型へ置換**

`src/types/index.ts` の `WindowSize` interface(36-39行)を以下に置換:

```typescript
export interface WindowState {
  x: number | null;
  y: number | null;
  width: number;
  height: number;
}
```

同じく `AppSettings` interface(16-24行)の `window_size: WindowSize;` を `window_state: WindowState;` に置換:

```typescript
export interface AppSettings {
  key_bindings: KeyBindings;
  window_state: WindowState;
  background_color: string;
  preload_range: number;
  key_throttle_ms: number;
  wheel_throttle_ms: number;
  display_mode: string;
}
```

- [ ] **Step 8: useSettings のデフォルトを更新**

`src/composables/useSettings.ts` の `createDefaults()` 内 `window_size: { width: 1280, height: 900 }` を以下に置換:

```typescript
    window_state: { x: null, y: null, width: 1280, height: 900 },
```

- [ ] **Step 9: フロントエンドのビルド確認**

Run (repo root): `npm run build`
Expected: 成功(tsc + vite build)。`window_size` 参照残りや未使用変数が無いこと。

- [ ] **Step 10: コミット**

```bash
git add src/App.vue src/composables/usePageController.ts src/composables/useSettings.ts src/types/index.ts
git commit -m "refactor: フロント側ウィンドウサイズ保存を廃止し display_mode を即時保存化、WindowState 型へ追従"
```

---

## Task 7: README の状態復元説明を更新

**Files:**
- Modify: `README.md`

- [ ] **Step 1: 「起動時の状態復元」節を更新**

`README.md` の118-124行目付近:

変更前:
```markdown
### 起動時の状態復元

終了時に以下の状態が自動的に保存され、次回起動時に復元されます。

- ウィンドウサイズ
- 表示モード(単ページ / 見開き)
```

変更後:
```markdown
### 起動時の状態復元

終了時に以下の状態が自動的に保存され、次回起動時に復元されます。

- ウィンドウサイズ・位置
- 表示モード(単ページ / 見開き)

フルスクリーンまたは最大化状態で終了した場合は、それらの状態ではなく最後に保存した通常ウィンドウのサイズ・位置が復元されます。モニタ構成が変わり保存位置がいずれの画面にも存在しない場合は、位置は復元されません(画面外に表示されるのを防ぎます)。
```

- [ ] **Step 2: 設計方針の復元記述を更新**

`README.md` の192行目付近:

変更前:
```markdown
- **起動時の状態復元**: ウィンドウサイズと表示モードを保存・復元
```

変更後:
```markdown
- **起動時の状態復元**: ウィンドウのサイズ・位置と表示モードを保存・復元(フルスクリーン/最大化状態での終了時は通常ウィンドウ状態を保持)
```

- [ ] **Step 3: コミット**

```bash
git add README.md
git commit -m "docs: 起動時の状態復元について位置とフルスクリーン挙動を追記"
```

---

## Task 8: 手動検証(Windows)

**Files:** なし(検証のみ)

- [ ] **Step 1: dev モードで起動**

Run (repo root): `npm run tauri dev`
Expected: アプリが起動する。初回は既定サイズ(1280x900)で表示。

- [ ] **Step 2: 通常ウィンドウのサイズ・位置復元**

1. ウィンドウをリサイズし、画面左上付近に移動。
2. アプリを閉じる(タイトルバーの ×)。
3. `%APPDATA%\rust-image-viewer\settings.json` を開き、`window_state` に `x/y/width/height` が論理ピクセルで保存されていることを確認。
4. 再起動 → 同じサイズ・位置で復元されること。

- [ ] **Step 3: フルスクリーン終了時の復元**

1. 起動 → `Enter` でフルスクリーンへ。
2. フルスクリーン状態のまま × で終了。
3. 再起動 → **フルスクリーン解除済み**の通常ウィンドウ(前回保存したサイズ・位置)で起動すること。`settings.json` の `window_state` はフルスクリーン前の通常値のままであること。

- [ ] **Step 4: 最大化終了時の復元**

1. 起動 → 最大化ボタンで最大化。
2. 最大化状態のまま × で終了。
3. 再起動 → 最大化されず、通常サイズで起動すること。

- [ ] **Step 5: マルチモニタ + 画面外ガード**

1. セカンダリモニタにウィンドウを移動 → 終了 → 再起動 → セカンダリに復元されること。
2. 設定ファイルの `x/y` を大きい値(例: 50000)に書き換え → 起動 → 画面外に復元されず、既定位置に現れること(位置のみスキップ、サイズは適用)。

- [ ] **Step 6: display_mode 即時保存**

1. 起動 → `Space` で見開きへ切替。
2. `settings.json` を開き、`display_mode` が `"spread"` に即時保存されていること(終了を待たず)。
3. 再起動 → 見開きで起動すること。

- [ ] **Step 7: 後方互換(旧 settings.json)**

1. アプリ終了。
2. `%APPDATA%\rust-image-viewer\settings.json` を旧形式(`window_size` を持ち `window_state` 無し)に書き換え:
   ```json
   { "window_size": { "width": 800, "height": 600 }, "background_color": "#000000", "display_mode": "single" }
   ```
3. 起動 → クラッシュせず起動し、終了後に `window_state` 形式へ移行されていること。

- [ ] **Step 8: リリースビルドで最終確認**

Run (repo root): `npm run tauri build`
Expected: ビルド成功。生成された `src-tauri/target/release/rust-image-viewer.exe` で Step 2〜3 を再確認。

---

## 自己レビューメモ(実装者向け)

- `is_position_visible` と serde 後方互換が単体テストで担保されている。ウィンドウ API 統合部分は `cargo check` + 手動検証で担保。
- フロントエンドは `window_state` を読み出さない(Rust 側が復元)。`AppSettings` 型に `window_state` を持つのは IPC レスポンスの型整合のためのみ。
- `display_mode` の即時保存は `settings.value`(読込時のキャッシュ)を元にするため、window_state を上書きしない(セッション中は window_state は不変、最終的な window_state の書込は Rust の close ハンドラが担う)。
