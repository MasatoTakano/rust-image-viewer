use std::path::Path;

use crate::archive::{rar_handler, zip_handler};
use crate::models::image_entry::{is_supported_image_extension, ImageEntry, ImageSource};
use crate::utils::sorter::sort_paths_naturally;

/// D&D されたパス（フォルダ/ZIP/RAR）から画像エントリリストを生成する
#[tauri::command]
pub fn load_source(path: String) -> Result<Vec<ImageEntry>, String> {
    let input_path = Path::new(&path);

    if !input_path.exists() {
        return Err(format!("パスが存在しません: {}", path));
    }

    let entries = if input_path.is_dir() {
        enumerate_folder(input_path)?
    } else {
        match input_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .as_deref()
        {
            Some("zip") => enumerate_zip(input_path)?,
            Some("rar") => enumerate_rar(input_path)?,
            _ => {
                return Err(
                    "対応していないファイル形式です。フォルダ、ZIP、RARをサポートしています。"
                        .to_string(),
                )
            }
        }
    };

    Ok(entries)
}

/// フォルダ内の画像を再帰的に列挙し、自然順ソートして返す
fn enumerate_folder(folder_path: &Path) -> Result<Vec<ImageEntry>, String> {
    let mut path_strings: Vec<String> = Vec::new();

    for entry in walkdir::WalkDir::new(folder_path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && is_supported_image_extension(path) {
            if let Some(path_str) = path.to_str() {
                path_strings.push(path_str.to_string());
            }
        }
    }

    sort_paths_naturally(&mut path_strings);

    let entries = path_strings
        .into_iter()
        .enumerate()
        .map(|(index, path_str)| {
            let display_name = Path::new(&path_str)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&path_str)
                .to_string();
            ImageEntry {
                index,
                display_name,
                source: ImageSource::FileSystem {
                    path: Path::new(&path_str).to_path_buf(),
                },
            }
        })
        .collect();

    Ok(entries)
}

/// ZIP アーカイブ内の画像を列挙する
fn enumerate_zip(archive_path: &Path) -> Result<Vec<ImageEntry>, String> {
    let raw_entries = zip_handler::enumerate_images(archive_path)?;

    let entries = raw_entries
        .into_iter()
        .enumerate()
        .map(|(index, (entry_path, display_name))| ImageEntry {
            index,
            display_name,
            source: ImageSource::Zip {
                archive_path: archive_path.to_path_buf(),
                entry_path,
            },
        })
        .collect();

    Ok(entries)
}

/// RAR アーカイブ内の画像を列挙する
fn enumerate_rar(archive_path: &Path) -> Result<Vec<ImageEntry>, String> {
    let raw_entries = rar_handler::enumerate_images(archive_path)?;

    let entries = raw_entries
        .into_iter()
        .enumerate()
        .map(|(index, (entry_path, display_name))| ImageEntry {
            index,
            display_name,
            source: ImageSource::Rar {
                archive_path: archive_path.to_path_buf(),
                entry_path,
            },
        })
        .collect();

    Ok(entries)
}
