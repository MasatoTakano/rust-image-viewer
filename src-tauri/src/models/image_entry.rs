use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// ソート済み画像リスト内の1エントリ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageEntry {
    /// ソート済みインデックス
    pub index: usize,
    /// 表示用ファイル名
    pub display_name: String,
    /// ソースの種別
    pub source: ImageSource,
}

/// 画像の読み込み元を表す
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImageSource {
    /// フォルダ内のファイル
    FileSystem { path: PathBuf },
    /// ZIP アーカイブ内のファイル
    Zip {
        archive_path: PathBuf,
        entry_path: String,
    },
    /// RAR アーカイブ内のファイル
    Rar {
        archive_path: PathBuf,
        entry_path: String,
    },
}

/// 対応する画像拡張子かどうかを判定する
pub fn is_supported_image_extension(path: &std::path::Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => matches!(
            ext.to_lowercase().as_str(),
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "avif"
        ),
        None => false,
    }
}
