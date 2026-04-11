use std::path::Path;

use crate::models::image_entry::is_supported_image_extension;
use crate::utils::sorter::sort_entries_by_path;

pub fn enumerate_images(archive_path: &Path) -> Result<Vec<(String, String)>, String> {
    let archive = unrar::Archive::new(archive_path)
        .open_for_listing()
        .map_err(|e| format!("RAR アーカイブを開けません: {}", e))?;

    let mut entries: Vec<(String, String)> = Vec::new();

    for entry_result in archive {
        let entry = entry_result.map_err(|e| format!("RAR エントリ読み取りエラー: {}", e))?;

        if entry.is_directory() {
            continue;
        }

        let entry_path = entry.filename.to_string_lossy().to_string();
        let path = Path::new(&entry_path);

        if is_supported_image_extension(path) {
            let display_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&entry_path)
                .to_string();
            entries.push((entry_path, display_name));
        }
    }

    sort_entries_by_path(&mut entries);

    Ok(entries)
}

pub fn read_image_data(archive_path: &Path, entry_path: &str) -> Result<Vec<u8>, String> {
    let mut archive = unrar::Archive::new(archive_path)
        .open_for_processing()
        .map_err(|e| format!("RAR アーカイブを開けません: {}", e))?;

    while let Some(archive_at_file) = archive
        .read_header()
        .map_err(|e| format!("RAR ヘッダ読み取りエラー: {}", e))?
    {
        let entry = archive_at_file.entry();
        let filename = entry.filename.to_string_lossy().to_string();

        if filename == entry_path {
            let (data, _next) = archive_at_file
                .read()
                .map_err(|e| format!("RAR ファイル読み取りエラー: {}", e))?;
            return Ok(data);
        } else {
            archive = archive_at_file
                .skip()
                .map_err(|e| format!("RAR スキップエラー: {}", e))?;
        }
    }

    Err(format!("エントリが見つかりません: {}", entry_path))
}
