# Rust + Tauri 画像ビューア — 仕様・実装計画書

## 1. 前提環境の確認とセットアップ

### 1.1 現在の環境状況

| ツール | 状態 | 備考 |
|---|---|---|
| **Rust / Cargo** | ❌ 未インストール | rustup からのインストールが必要（手動） |
| **Node.js** | ⚠️ v14.15.4（現在） | **nvm** でバージョン切替可能。Tauri v2 は **Node.js ≥ 18** が必要 |
| **npm** | v6.14.10 | Node.js 切替に伴い最新化 |

### 1.2 セットアップ手順（実装開始前に手動で実施）

#### ステップ 1: Rust のインストール

1. ブラウザで https://rustup.rs/ を開く
2. **rustup-init.exe** をダウンロードして実行
3. インストールオプションはデフォルト（`1`）を選択
4. インストール完了後、**新しいターミナル**を開いて確認：

```powershell
rustc --version
cargo --version
```

> ※ Visual Studio Build Tools が必要です。未インストールの場合は [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) から「C++ビルドツール」ワークロードをインストールしてください。

#### ステップ 2: Node.js のバージョン切替（nvm 使用）

Tauri v2 は Node.js 18 以上が必要です。nvm を使用して切り替えます。

```powershell
nvm install 20        # Node.js 20 LTS をインストール（未インストールの場合）
nvm use 20            # Node.js 20 に切替
node --version        # v20.x であることを確認
npm --version         # v9.x 以上であることを確認
```

#### ステップ 3: Tauri CLI の確認

Tauri v2 プロジェクト作成時に `npm create tauri-app` が自動的に必要な依存をインストールしますが、事前に prerequisites を確認してください：

- **Windows**: WebView2 は Windows 11 では標準搭載済み
- 詳細: https://v2.tauri.app/start/prerequisites/

#### ステップ 4: セットアップの確認

すべてインストール後、以下のコマンドで動作確認：

```powershell
rustc --version    # Rust 1.70 以上
cargo --version    # Cargo 1.70 以上
node --version     # Node.js 18 以上
npm --version      # npm 9 以上
```

---

## 2. プロジェクト概要

Rust（バックエンド）+ Tauri（デスクトップフレームワーク）+ TypeScript/HTML/CSS（フロントエンド）による、軽量・高速な画像ビューアを開発する。

### 2.1 対象プラットフォーム

- Windows 11（一次対応）

---

## 3. 機能仕様

### 3.1 ファイル読み込み（D&D対応）

| 項目 | 仕様 |
|---|---|
| 対応入力 | フォルダ、`.zip`、`.rar` ファイル |
| 操作方法 | メインウィンドウへのドラッグ＆ドロップ |
| 読み込み範囲 | 指定フォルダ／アーカイブ内の **サブフォルダを再帰的に** 探索 |
| 対応画像形式 | JPEG、PNG、GIF、BMP、WebP、AVIF（Tauri/Rust で扱えるもの） |
| 並び順 | **ファイルパスの辞書式（自然順）ソート**（`001.jpg`, `002.jpg` …） |

#### 処理フロー

```
D&D → 入力判定（フォルダ / .zip / .rar）
  → [フォルダ] walkdir で再帰列挙 → 画像拡張子でフィルタ → ソート
  → [.zip]    zip クレートで内部エントリ列挙 → フィルタ → ソート
  → [.rar]    unrar クレートで内部エントリ列挙 → フィルタ → ソート
  → 画像リストをフロントエンドに送信
```

### 3.2 画像表示

| 項目 | 仕様 |
|---|---|
| 表示モード | **単ページ表示** / **左右見開き表示（2ページ）** |
| 画像スケーリング | ウィンドウサイズに合わせて **フィット表示**（アスペクト比維持） |
| 背景色 | 黒（設定で変更可能とする拡張性を残す） |

#### 単ページ表示

- 1画面に1枚の画像を表示
- 画像はウィンドウ/画面中央に配置

#### 左右見開き表示

- 1画面に2枚の画像を **横並び** で表示
- **右ページ（奇数インデックス）が右側**、**左ページ（偶数インデックス）が左側** に表示（漫画・書籍の見開き順）
  - 例：`[左: 2枚目] [右: 1枚目]` → 次→ `[左: 4枚目] [右: 3枚目]`
- 画像の総数が奇数の場合、最後のページは単独表示（中央寄せ）または空白

### 3.3 ページ送り操作

| 操作 | 動作 |
|---|---|
| 次ページ（単ページ） | インデックス + 1 |
| 前ページ（単ページ） | インデックス − 1 |
| 次ページ（見開き） | インデックス + 2 |
| 前ページ（見開き） | インデックス − 2 |
| 最初のページへ | インデックス = 0 |
| 最後のページへ | インデックス = 最終 |

#### マウス操作

| 操作 | 動作 |
|---|---|
| 左クリック（画面右半分） | 次ページ |
| 左クリック（画面左半分） | 前ページ |
| ホイール下回転 | 次ページ |
| ホイール上回転 | 前ページ |

### 3.4 フルスクリーン切り替え

| 項目 | 仕様 |
|---|---|
| トグル操作 | 設定されたショートカットキーで **ウィンドウ ↔ フルスクリーン** を切り替え |
| デフォルトキー | `F11` |
| ウィンドウモードの初期サイズ | 1280 × 900（変更可能） |
| フルスクリーン解除時 | 直前のウィンドウサイズ・位置を復元 |

### 3.5 表示モード切り替え（単ページ ↔ 見開き）

| 項目 | 仕様 |
|---|---|
| トグル操作 | 設定されたショートカットキーで **単ページ ↔ 見開き** を切り替え |
| デフォルトキー | `D` |
| 切り替え時のページ位置 | 現在表示中の **先頭ページインデックス** を維持 |

### 3.6 キーボードショートカット設定

| 項目 | 仕様 |
|---|---|
| 設定対象 | 「フルスクリーン切替」「表示モード切替」「次ページ」「前ページ」の各操作 |
| デフォルトキーマッピング | 下表を参照 |
| 設定の永続化 | アプリデータディレクトリに `settings.json` として保存 |
| 設定UI | メニューまたは設定パネルから変更可能 |

#### デフォルトキーマッピング

| 操作 | デフォルトキー |
|---|---|
| 次ページ | `→` / `Space` / マウスホイール下 / 右クリック |
| 前ページ | `←` / `Backspace` / マウスホイール上 / 左クリック（左半分） |
| フルスクリーン切替 | `F11` |
| 表示モード切替 | `D` |
| 最初のページへ | `Home` |
| 最後のページへ | `End` |
| 設定画面を開く | `Ctrl+,` |

---

## 4. アーキテクチャ

### 4.1 全体構成

```
┌─────────────────────────────────────┐
│           Tauri Application          │
│  ┌───────────────┐ ┌──────────────┐ │
│  │   Frontend     │ │   Backend    │ │
│  │ (TypeScript +  │ │   (Rust)     │ │
│  │  HTML + CSS)   │ │              │ │
│  │                │ │              │ │
│  │ - 画像表示UI   │←→ - ファイル列挙 │ │
│  │ - キー受信     │ │ - アーカイブ展開│ │
│  │ - D&D 受理     │ │ - 画像デコード │ │
│  │ - 設定パネル   │ │ - 設定IO      │ │
│  └───────────────┘ └──────────────┘ │
└─────────────────────────────────────┘
```

### 4.2 技術スタック

| レイヤー | 技術 | 用途 |
|---|---|---|
| フレームワーク | **Tauri v2** | デスクトップアプリ基盤 |
| バックエンド | **Rust** | ファイルシステム操作、アーカイブ展開、画像処理 |
| フロントエンド | **TypeScript** + HTML + CSS | UI描画、イベント処理 |
| フロントエンドビルド | **Vite** | バンドリング・開発サーバー |
| UIフレームワーク | **Vanilla TS**（または軽量ライブラリ） | シンプルな画像表示に不要な依存を避ける |

### 4.3 主要Rustクレート

| クレート | 用途 |
|---|---|
| `tauri` | アプリフレームワーク |
| `walkdir` | ディレクトリ再帰探索 |
| `zip` | ZIP アーカイブ読み取り |
| `unrar` | RAR アーカイブ読み取り |
| `image` | 画像デコード・サムネイル生成 |
| `serde` / `serde_json` | 設定ファイルのシリアライズ・デシリアライズ |
| `base64` | 画像データのフロントエンド転送 |
| `tokio` | 非同期ランタイム（Tauri v2 依存） |

### 4.4 データフロー

```
[D&D] → Frontend (JS) → Tauri Command → Rust Backend
        ↓                                     ↓
   ファイルパス受取                    ファイル列挙・ソート
                                        ↓
                                   画像リスト返却
                                        ↓
   Frontend ← Tauri Event ← 画像リスト受取
        ↓
   画像表示要求 → Tauri Command → Rust: 画像を読み込み
                                    base64エンコードして返却
        ↓
   <img> タグに反映
```

#### 画像転送方式

- **方式**: Rust 側で画像ファイルを読み込み、Base64 エンコードしてフロントエンドに返却
- **最適化**: 画面解像度に合わせてリサイズ後に転送（大きな画像のメモリ・転送コスト削減）
- **キャッシュ**: フロントエンド側で直近数ページ分をキャッシュ

---

## 5. ファイル構成（プロジェクト構造）

```
RustImageViewer/
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── src/
│   │   ├── main.rs                # エントリポイント
│   │   ├── lib.rs                  # Tauri コマンド登録・モジュール統合
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── file_loader.rs      # D&D受取 → ファイル列挙コマンド
│   │   │   ├── image_provider.rs   # 画像読込・Base64エンコード返却
│   │   │   └── settings.rs         # 設定読込・保存コマンド
│   │   ├── archive/
│   │   │   ├── mod.rs
│   │   │   ├── zip_handler.rs      # ZIP 展開処理
│   │   │   └── rar_handler.rs      # RAR 展開処理
│   │   ├── models/
│   │   │   ├── mod.rs
│   │   │   ├── image_entry.rs      # 画像エントリ構造体
│   │   │   └── settings.rs         # 設定構造体
│   │   └── utils/
│   │       ├── mod.rs
│   │       └── sorter.rs           # 自然順ソート
│   └── icons/
├── src/                            # Frontend (Vite)
│   ├── main.ts                     # エントリポイント
│   ├── index.html
│   ├── styles/
│   │   └── main.css
│   ├── viewer/
│   │   ├── image_viewer.ts         # 画像表示ロジック
│   │   ├── page_controller.ts      # ページ送り制御
│   │   └── layout.ts               # 単ページ/見開きレイアウト
│   ├── input/
│   │   ├── keyboard_handler.ts     # キーボードイベント処理
│   │   ├── mouse_handler.ts        # マウスイベント処理
│   │   └── drag_drop_handler.ts    # D&D 処理
│   ├── settings/
│   │   ├── settings_manager.ts     # 設定読込・保存
│   │   └── settings_panel.ts       # 設定UI
│   └── types/
│       └── index.ts                # 型定義
├── package.json
├── tsconfig.json
├── vite.config.ts
└── plan.md                         # 本ファイル
```

---

## 6. 実装計画

### フェーズ 1: プロジェクト雛形の作成

- [ ] Tauri v2 プロジェクトの初期化（`npm create tauri-app`）
- [ ] Rust 側のモジュール構造（`commands/`, `archive/`, `models/`, `utils/`）を作成
- [ ] フロントエンドのディレクトリ構造を作成
- [ ] `Cargo.toml` に必要クレートを追加

### フェーズ 2: バックエンド — ファイル列挙

- [ ] `models/image_entry.rs`: 画像エントリの構造体定義（パス、アーカイブ内パス、ファイル名等）
- [ ] `utils/sorter.rs`: 自然順ソートユーティリティの実装
- [ ] `commands/file_loader.rs`:
  - フォルダパス受け取り → `walkdir` で再帰列挙
  - 画像拡張子フィルタリング
  - 自然順ソート → パスリストを返却
- [ ] `archive/zip_handler.rs`: ZIP 内ファイル列挙 → 画像フィルタ → ソート
- [ ] `archive/rar_handler.rs`: RAR 内ファイル列挙 → 画像フィルタ → ソート
- [ ] `commands/file_loader.rs` にアーカイブ処理を統合（入力パスの拡張子で分岐）

### フェーズ 3: バックエンド — 画像供給

- [ ] `commands/image_provider.rs`:
  - インデックス指定で画像を取得
  - フォルダ画像: ファイルパスから直接読み込み
  - アーカイブ画像: メモリ上のデータから画像を取得
  - 画面解像度に合わせたリサイズ
  - Base64 エンコードしてフロントエンドに返却

### フェーズ 4: フロントエンド — 基本UI

- [ ] メイン画面の HTML 構造（画像表示エリア、ステータスバー）
- [ ] CSS スタイリング（黒背景、中央配置、レスポンシブ）
- [ ] D&D ハンドラ（`drag_drop_handler.ts`）: ファイルドロップ → Rust コマンド呼び出し
- [ ] `image_viewer.ts`: Base64 画像データを `<img>` に反映

### フェーズ 5: フロントエンド — ページ制御・表示モード

- [ ] `page_controller.ts`: 現在のページインデックス管理、次/前ページ移動
- [ ] `layout.ts`: 単ページ表示レイアウト
- [ ] `layout.ts`: 見開き表示レイアウト（右奇数・左偶数の並び）
- [ ] 表示モード切替ロジック（単ページ ↔ 見開き）

### フェーズ 6: フロントエンド — 入力処理

- [ ] `keyboard_handler.ts`: キーボードイベント → アクション マッピング
- [ ] `mouse_handler.ts`: クリック位置判定（左半分/右半分）、ホイールイベント
- [ ] フルスクリーン切替（Tauri API 経由）

### フェーズ 7: 設定機能

- [ ] `models/settings.rs`: 設定構造体（キーマッピング、ウィンドウサイズ等）
- [ ] `commands/settings.rs`: 設定ファイル読込・保存コマンド
- [ ] `settings/settings_panel.ts`: 設定画面UI
- [ ] `settings/settings_manager.ts`: 設定のキャッシュ・変更通知
- [ ] キーマッピング変更のリアルタイム反映

### フェーズ 8: 統合テスト・調整

- [ ] フォルダ読み込みの動作確認
- [ ] ZIP/RAR 読み込みの動作確認
- [ ] サブフォルダ含めた列挙の確認
- [ ] ページ送り（キーボード・マウス）の確認
- [ ] フルスクリーン切替の確認
- [ ] 表示モード切替の確認
- [ ] 設定保存・読込の確認
- [ ] パフォーマンス調整（大きな画像、多数ファイルのフォルダ）

---

## 7. 主要データ構造

### 7.1 画像エントリ（Rust）

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageEntry {
    /// ソート済みインデックス
    pub index: usize,
    /// 表示用ファイル名
    pub display_name: String,
    /// ソースの種別
    pub source: ImageSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageSource {
    /// フォルダ内のファイル
    FileSystem { path: PathBuf },
    /// ZIP アーカイブ内のファイル
    Zip { archive_path: PathBuf, entry_path: String },
    /// RAR アーカイブ内のファイル
    Rar { archive_path: PathBuf, entry_path: String },
}
```

### 7.2 設定（Rust）

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// キーボードショートカット設定
    pub key_bindings: KeyBindings,
    /// ウィンドウモード時のサイズ
    pub window_size: WindowSize,
    /// 背景色
    pub background_color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBindings {
    pub next_page: Vec<String>,
    pub prev_page: Vec<String>,
    pub toggle_fullscreen: Vec<String>,
    pub toggle_spread: Vec<String>,
    pub go_first: Vec<String>,
    pub go_last: Vec<String>,
    pub open_settings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}
```

### 7.3 フロントエンド側キャッシュ

```typescript
interface ImageCache {
  // インデックス → Base64 画像データ
  dataUrl: Map<number, string>;
  // キャッシュ上限（前後 N ページ分）
  capacity: number;
}
```

---

## 8. Tauri コマンド一覧

| コマンド名 | 引数 | 戻り値 | 説明 |
|---|---|---|---|
| `load_source` | `path: String` | `Vec<ImageEntry>` | フォルダ/ZIP/RARから画像リストを取得 |
| `get_image` | `entries: Vec<ImageEntry>, index: number, max_width: number, max_height: number` | `{ data_url: String }` | 指定インデックスの画像をリサイズ・Base64返却 |
| `load_settings` | — | `AppSettings` | 設定を読み込み |
| `save_settings` | `settings: AppSettings` | — | 設定を保存 |
| `toggle_fullscreen` | — | `bool` | フルスクリーン切替、現在の状態を返却 |

---

## 9. エラー処理方針

| ケース | 対応 |
|---|---|
| 対応外ファイルがD&Dされた | 通知メッセージを表示（フロントエンド側） |
| 破損したアーカイブ | エラーダイアログ表示、読み込み中止 |
| 破損した画像ファイル | 該当ページをスキップし「画像を読み込めません」プレースホルダ表示 |
| 設定ファイル破損 | デフォルト設定で起動、ユーザーに通知 |
| アーカイブ展開中のメモリ不足 | ファイルサイズ制限またはストリーミング処理 |

---

## 10. パフォーマンス考慮事項

- **画像遅延読み込み**: 必要なページ分のみを取得（先読みは前後 2〜3 ページ）
- **リサイズ**: 原寸画像をそのまま転送せず、画面解像度に合わせて縮小
- **キャッシュ**: フロントエンドで直近の画像をキャッシュ、重複リクエスト防止
- **自然順ソート**: 大量ファイルでも高速な実装を使用

---

## 11. 今後の拡張候補（本スコープ外）

- サポート画像形式の追加（TIFF、SVG 等）
- ページサムネイル一覧表示
- フォルダ履歴・ブックマーク機能
- スライドショー機能
- 画像の回転・ズーム
- テーマ切り替え
- 多言語対応