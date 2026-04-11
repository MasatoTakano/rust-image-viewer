use std::io::Read;
use std::path::Path;

use crate::models::image_entry::is_supported_image_extension;
use crate::utils::sorter::sort_entries_by_path;

pub fn enumerate_images(archive_path: &Path) -> Result<Vec<(String, String)>, String> {
    let file = std::fs::File::open(archive_path)
        .map_err(|e| format!("ZIP ファイルを開けません: {}", e))?;

    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("ZIP アーカイブの読み取りに失敗: {}", e))?;

    let mut entries: Vec<(String, String)> = Vec::new();

    for i in 0..archive.len() {
        let entry = archive
            .by_index(i)
            .map_err(|e| format!("エントリ読み取りエラー: {}", e))?;
        let entry_path = entry.name().to_string();

        if entry.is_dir() {
            continue;
        }

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
    let file = std::fs::File::open(archive_path)
        .map_err(|e| format!("ZIP ファイルを開けません: {}", e))?;

    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("ZIP アーカイブの読み取りに失敗: {}", e))?;

    let mut entry = archive
        .by_name(entry_path)
        .map_err(|e| format!("エントリが見つかりません: {} ({})", entry_path, e))?;

    let mut data = Vec::new();
    entry
        .read_to_end(&mut data)
        .map_err(|e| format!("画像データ読み取りエラー: {}", e))?;
    Ok(data)
}
