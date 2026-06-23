# ウィンドウ状態(サイズ・位置)永続化 設計

## 背景と問題

Windows 版バイナリで、終了時のウィンドウサイズ・位置が次回起動時に復元されない。

### 現状の実装と不具合

`src/App.vue` が以下の処理を行っている:

- `onMounted`: `getCurrentWindow().setSize(LogicalSize(s.window_size.width, s.window_size.height))` で復元
- `beforeunload` リスナ `onSaveState`: `window.innerWidth/innerHeight` と `display_mode` を `invoke("save_settings")` で保存

これには3つの問題がある:

1. **保存のタイミング破損(主原因)**: `beforeunload` 内の `invoke()` は非同期 IPC。OS の閉じるボタンでウィンドウを閉じると WebView 破棄が先に完了し、保存コマンドが実行されない。失敗は `.catch(() => {})` で握り潰されているため気付かれない。
2. **位置が未保存**: `width/height` のみで `x/y` を保存していないため、位置復元が不可能。
3. **フルスクリーン副作用**: フルスクリーン中は `window.innerWidth/innerHeight` がモニタ全面解像度になる。その値を次回 `setSize`(外側サイズ指定) で復元すると、非フルスクリーンにもかかわらずモニタ全面を覆う(タスクバーを隠す)異常なウィンドウになる。
   - さらに `innerWidth/innerHeight` は WebView のコンテンツ領域(タイトルバー・枠線を含まない)であり、`setSize` は外側サイズを指定するため、セッションごとに実サイズがドリフトする。

## 要件

1. 終了時のウィンドウサイズ・位置を保存し、次回起動時に復元する。
2. フルスクリーン状態で終了した場合は、フルスクリーンを解除した通常ウィンドウ状態で復元する(フルスクリーンそのもので起動しない)。
3. 最大化(最大化ボタン)状態で終了した場合は、最大化は復元せず、最後に保存した通常ウィンドウのサイズ・位置を復元する(ユーザー選定)。

## 設計(アプローチ A: Rust 側永続化)

ウィンドウ状態の保存・復元をフロントエンドから Rust 側へ完全に移行する。Rust の `CloseRequested` イベントはイベントループ上で同期的かつ確実に発火し、ウィンドウ破棄前に完了するため、タイミング問題が解消される。また真のジオメトリ(`outer_size/outer_position`)と状態(`fullscreen/is_maximized`)を Rust API から正確に取得できる。

### データモデル変更

`src-tauri/src/models/settings.rs`:

- 既存 `WindowSize { width: u32, height: u32 }` を **廃止** し、新たに `WindowState { x: Option<i32>, y: Option<i32>, width: u32, height: u32 }` を追加。
  - `x/y` は `Option<i32>`(マルチモニタ環境で負座標があり得るため符号付き。`None` は「位置未保存＝初回起動」を表す)。
- `width/height` に `#[serde(default)]` を付与。古い `settings.json`(`window_size` を持つ)は `window_state` フィールド欠落としてデフォルト補完され、未知フィールド(旧 `window_size`)は `serde_json` 既定で無視される(後方互換)。
- 論理ピクセル(DPI 非依存)で保持する。物理ピクセルではない。

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
```

`AppSettings` の `window_size: WindowSize` を `window_state: WindowState` に置換(`#[serde(default)]`)。

### 起動時 復元 (`lib.rs` の `setup` フック)

1. `AppSettings` を読み込む(`load_settings` 相当を直接呼び出し、またはパスから読込)。
2. **サイズ復元**: 常に `window.set_size(LogicalSize(width, height))` を適用。初回起動時のデフォルト値は `tauri.conf.json` の初期サイズ(1280x900)と一致するため、適用しても無害(実質 no-op)。
3. **位置復元**: `x` と `y` がともに `Some` の場合のみ復元を試みる(`None` = 初回起動で位置未保存なら OS 既定配置に任せる)。
4. **モニタ範囲外ガード**: 保存された `(x, y)` がいずれのモニタの有効領域にも属さない場合、位置の復元をスキップ。画面外に復元され操作不能になるのを防ぐ。
   - 判定: `window.available_monitors()` を走査し、保存位置がいずれかのモニタ矩形内にあるか検証。
5. 位置が有効なら `window.set_position(LogicalPosition(x, y))` を適用(サイズ→位置の順、または位置→サイズの順で重複適用の影響は軽微)。
6. フルスクリーン・最大化は **復元しない**(常に通常ウィンドウで起動)。

### 終了時 保存 (`lib.rs` の `on_window_event` → `CloseRequested`)

`tauri::WindowEvent::CloseRequested` ハンドラで:

```rust
if window.fullscreen().is_none() && !window.is_maximized() {
    let pos = window.outer_position()?;   // PhysicalPosition<i32>
    let size = window.outer_size()?;       // PhysicalSize<u32>
    // 論理ピクセルに変換: window.scale_factor() で割る
    let logical = ...;
    // settings.json を読み → window_state を更新 → 書き戻し
}
```

- **フルスクリーン中または最大化中は保存をスキップ** する。これにより最後に通常ウィンドウで保存されたサイズ・位置が保持され、要件2・3を満たす。
- 物理ピクセル(`outer_size/outer_position` の戻り値)を `scale_factor()` で除算して論理ピクセルに変換して保存(`x/y` は `Some` で格納)。DPI 設定の異なる環境や、DPI 変更後の再起動でも一貫した復元になる。
- 保存処理は settings の読込→部分更新→書き戻しとし、他の設定(キーバインド等)を上書きしない。失敗はログ出力のみでプロセス終了を阻害しない。

### フロントエンド整理 (`src/App.vue`)

- `onSaveState`(`beforeunload` リスナ)を削除。これに伴い `window.innerWidth/innerHeight` に依存する保存処理を完全排除。
- `onMounted` 内の `setSize` 復元処理(112-118行)を削除(Rust 側に一本化)。
- `display_mode` の保存は、閉じる時ではなく **表示モード切替時(`usePageController.toggleDisplayMode`)に即時保存** するよう変更。同じタイミング問題(IPC 不完)を回避し、確実に永続化する。
- `types/index.ts` と `composables/useSettings.ts` の `WindowSize` 型を `WindowState` に更新(フロント側で `window_state` を参照しない場合は型だけ追従)。

### Tauri 権限

`capabilities/default.json` は `core:default` を含むため、ウィンドウ位置・サイズの取得/設定に必要な権限は既に含まれる。追加権限不要(フルスクリーン操作権限は既存)。

## 影響範囲

| ファイル | 変更内容 |
|---|---|
| `src-tauri/src/models/settings.rs` | `WindowSize`→`WindowState` へ置換、`x/y` 追加、serde default |
| `src-tauri/src/lib.rs` | `setup` で復元、`on_window_event` で `CloseRequested` 保存 |
| `src/App.vue` | `onSaveState`・`setSize` 復元を削除、`display_mode` 即時保存化 |
| `src/composables/usePageController.ts` | `toggleDisplayMode` で `display_mode` を即時保存 |
| `src/composables/useSettings.ts` | 型追従 |
| `src/types/index.ts` | `WindowSize`→`WindowState` 型更新 |
| `README.md` | 「起動時の状態復元」節に「位置」を追記、仕様反映 |

## テスト方針

手動検証(Windows ビルド)で以下を確認:

1. 通常ウィンドウでサイズ・位置を変更 → 終了 → 再起動で同じサイズ・位置が復元される。
2. フルスクリーンに切替 → 終了 → 再起動で **フルスクリーン解除済み** の通常ウィンドウ(前回保存したサイズ・位置)で起動する。
3. 最大化 → 終了 → 再起動で最大化されず、通常サイズで起動する。
4. マルチモニタでセカンダリモニタに移動 → 終了 → 再起動でセカンダリに復元される。
5. セカンダリモニタを外した状態で起動 → 画面外に復元されず、既定位置に現れる(ガード動作)。
6. DPI スケーリング環境でサイズがドリフトしない。
7. 古い `settings.json`(`window_size` を持つ)があっても起動し、新形式に移行される(後方互換)。

## 非対象(YAGNI)

- 最大化状態の復元(ユーザー選定により非対応)。
- ウィンドウのリサイズ/移動中の逐次保存(本要件では不要、close 時保存で十分)。
- 複数ウィンドウの状態管理(本アプリは単一ウィンドウ)。
