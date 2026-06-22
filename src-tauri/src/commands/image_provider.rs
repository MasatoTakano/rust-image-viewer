use std::io::Cursor;
use std::path::Path;

use crate::archive::{rar_handler, zip_handler};
use crate::models::image_entry::{ImageEntry, ImageSource};
use base64::Engine;
use image::ImageEncoder;

/// 1画像あたりの最大 raw バイト数 (150 MB)
/// アーカイブ展開後・デコード前の生データに対する制限
const MAX_RAW_BYTES: usize = 150 * 1024 * 1024;

#[derive(serde::Serialize)]
pub struct ImageResult {
    pub data_url: String,
}

#[tauri::command]
pub fn get_image(
    entry: ImageEntry,
    max_width: u32,
    max_height: u32,
) -> Result<ImageResult, String> {
    let raw_data = match &entry.source {
        ImageSource::FileSystem { path } => read_filesystem_image(path)?,
        ImageSource::Zip {
            archive_path,
            entry_path,
        } => zip_handler::read_image_data(archive_path, entry_path)?,
        ImageSource::Rar {
            archive_path,
            entry_path,
        } => rar_handler::read_image_data(archive_path, entry_path)?,
    };

    if raw_data.len() > MAX_RAW_BYTES {
        return Err(format!(
            "画像データが大きすぎます ({} bytes、上限 {} bytes)",
            raw_data.len(),
            MAX_RAW_BYTES
        ));
    }

    let display_name = &entry.display_name;
    let ext = Path::new(display_name)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    // ブラウザネイティブ形式は常に元のバイト列をパススルーし、
    // リサイズ/再エンコードによる品質低下を避けて GPU スケーリングに任せる。
    let native_mime = match ext.as_str() {
        "avif" => Some("image/avif"),
        "webp" => Some("image/webp"),
        "gif" => Some("image/gif"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "png" => Some("image/png"),
        _ => None,
    };

    if let Some(mime) = native_mime {
        let b64 = base64::engine::general_purpose::STANDARD.encode(&raw_data);
        return Ok(ImageResult {
            data_url: format!("data:{};base64,{}", mime, b64),
        });
    }

    let data_url = encode_image_to_data_url(&raw_data, max_width, max_height)?;
    Ok(ImageResult { data_url })
}

fn read_filesystem_image(path: &Path) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|e| format!("画像ファイルの読み取りに失敗: {}", e))
}

fn encode_image_to_data_url(
    data: &[u8],
    max_width: u32,
    max_height: u32,
) -> Result<String, String> {
    let mut limits = image::Limits::no_limits();
    limits.max_image_width = Some(40_000);
    limits.max_image_height = Some(40_000);
    limits.max_alloc = Some(512 * 1024 * 1024);

    let mut reader = image::ImageReader::new(Cursor::new(data));
    reader.limits(limits);
    let img = reader
        .with_guessed_format()
        .map_err(|e| format!("画像フォーマット検出エラー: {}", e))?
        .decode()
        .map_err(|e| format!("画像のデコードに失敗: {}", e))?;

    let img = img.to_rgb8();
    let (orig_width, orig_height) = img.dimensions();

    let needs_resize = orig_width > max_width || orig_height > max_height;

    let final_img = if needs_resize && max_width > 0 && max_height > 0 {
        let scale_x = max_width as f64 / orig_width as f64;
        let scale_y = max_height as f64 / orig_height as f64;
        let scale = scale_x.min(scale_y);

        let new_width = ((orig_width as f64) * scale) as u32;
        let new_height = ((orig_height as f64) * scale) as u32;

        image::imageops::resize(
            &img,
            new_width,
            new_height,
            image::imageops::FilterType::CatmullRom,
        )
    } else {
        img
    };

    let base64_str = if needs_resize {
        let mut out = Vec::new();
        let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 92);
        encoder
            .write_image(
                final_img.as_raw(),
                final_img.width(),
                final_img.height(),
                image::ExtendedColorType::Rgb8,
            )
            .map_err(|e| format!("JPEG エンコードに失敗: {}", e))?;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&out);
        format!("data:image/jpeg;base64,{}", b64)
    } else {
        let mut out = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut out);
        encoder
            .write_image(
                final_img.as_raw(),
                final_img.width(),
                final_img.height(),
                image::ExtendedColorType::Rgb8,
            )
            .map_err(|e| format!("PNG エンコードに失敗: {}", e))?;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&out);
        format!("data:image/png;base64,{}", b64)
    };

    Ok(base64_str)
}
