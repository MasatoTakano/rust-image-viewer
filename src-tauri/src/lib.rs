mod archive;
mod commands;
mod models;
mod utils;

use commands::{file_loader, image_provider, settings as settings_cmd, window_state};
use tauri::{Emitter, Manager};

/// 診断メッセージをキャッシュディレクトリの diag.log へ出力する。
/// ショートカットD&Dなどターミナル未接続の起動ケースでも記録を残すため。
pub fn diag_log(msg: &str) {
    use std::io::Write;
    if let Some(cache) = dirs::cache_dir() {
        let log_path = cache.join("rust-image-viewer").join("diag.log");
        if let Some(parent) = log_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&log_path)
        {
            let _ = writeln!(f, "{}", msg);
        }
    }
}

/// ショートカットD&DでシステムANSIコードページ(CP932等)に変換され、
/// 表現できないUnicode文字(♡等)が `?` に置換されてしまう問題を救済する。
/// `?` はWindowsファイル名に使用できない文字なので、1文字ワイルドカードとして
/// 親ディレクトリから実ファイルを解決する。解決できなければ元のパスを返す。
pub fn resolve_ansi_corrupted_path(path_str: &str) -> String {
    use std::path::{Path, PathBuf};

    let path = Path::new(path_str);
    if path.exists() || !path_str.contains('?') {
        return path_str.to_string();
    }

    match resolve_wildcard_path(path) {
        Some(resolved) => {
            diag_log(&format!(
                "[diag] resolved ANSI-corrupted path: {} -> {}",
                path_str,
                resolved.to_string_lossy()
            ));
            resolved.to_string_lossy().to_string()
        }
        None => path_str.to_string(),
    }
}

fn resolve_wildcard_path(path: &std::path::Path) -> Option<std::path::PathBuf> {
    use std::path::{Path, PathBuf};

    let mut current = PathBuf::new();
    let components: Vec<_> = path.components().collect();

    for comp in components {
        let comp_str = comp.as_os_str().to_str()?;
        current.push(comp);

        if current.exists() {
            continue;
        }

        // 直前のコンポーネントが ? を含んでいなければ解決不可
        if !comp_str.contains('?') {
            return None;
        }

        let parent = current.parent()?;
        let entries = std::fs::read_dir(parent).ok()?;

        let matches: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                wildcard_match_single_char(&name, comp_str)
            })
            .collect();

        if matches.len() == 1 {
            // 候補が1件だけなら採用。current を置き換える。
            let mut new_current = current.parent()?.to_path_buf();
            new_current.push(matches[0].file_name());
            current = new_current;
        } else {
            // 0件または複数候補は誤解決防止のため拒否
            return None;
        }
    }

    Some(current)
}

/// `pattern` 中の `?` を任意の1文字として、`name` と一致するか判定する。
/// 文字数(コードポイント数)が一致することを前提とする。
fn wildcard_match_single_char(name: &str, pattern: &str) -> bool {
    let n: Vec<char> = name.chars().collect();
    let p: Vec<char> = pattern.chars().collect();
    if n.len() != p.len() {
        return false;
    }
    n.iter().zip(p.iter()).all(|(nc, pc)| *pc == '?' || nc == pc)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let cli_args: Vec<String> = std::env::args().skip(1).collect();

    // [diag] 起動ごとにログを区切る
    diag_log("------------------------------------------------------------");
    diag_log("[diag] ===== application start =====");

    // [diag] 受け取った生の CLI 引数を記録（ショートカットD&Dの文字化け調査用）
    diag_log(&format!(
        "[diag] raw cli_args (count={}):",
        cli_args.len()
    ));
    for (i, arg) in cli_args.iter().enumerate() {
        diag_log(&format!(
            "[diag]   [{}] len={} bytes={:?} codepoints={:X?}",
            i,
            arg.len(),
            arg.as_bytes(),
            arg.chars().map(|c| c as u32).collect::<Vec<_>>()
        ));
    }

    // ショートカットD&Dで (1) スペース分割、(2) CP932変換による `?` 化け が起こる。
    // 両方を救済するため、まず引数を結合して1つのパス候補を作り、
    // そこに `?` が含まれる場合はワイルドカード解決を試みる。
    // それでも実在しなければ、複数ファイルD&D等の可能性があるため先頭引数にフォールバック。
    let cli_path: String = if cli_args.is_empty() {
        String::new()
    } else {
        let joined = cli_args.join(" ");
        let resolved_joined = resolve_ansi_corrupted_path(&joined);
        if std::path::Path::new(&resolved_joined).exists() {
            resolved_joined
        } else if cli_args.len() > 1 {
            // 結合パスが解決できなければ先頭引数を試す（複数ファイルD&Dのフェールセーフ）
            let resolved_first = resolve_ansi_corrupted_path(&cli_args[0]);
            if std::path::Path::new(&resolved_first).exists() {
                resolved_first
            } else {
                cli_args[0].clone()
            }
        } else {
            cli_args[0].clone()
        }
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            file_loader::load_source,
            image_provider::get_image,
            settings_cmd::load_settings,
            settings_cmd::save_settings,
        ])
        .setup(move |app| {
            if !cli_path.is_empty() {
                let path = cli_path.clone();
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
