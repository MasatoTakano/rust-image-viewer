use std::io::Cursor;
use std::path::Path;

use crate::archive::{rar_handler, zip_handler};
use crate::models::image_entry::{ImageEntry, ImageSource};
use base64::Engine;
use image::ImageEncoder;

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

    let display_name = &entry.display_name;
    let ext = Path::new(display_name)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    if matches!(ext.as_str(), "avif" | "webp" | "gif") {
        let mime = match ext.as_str() {
            "avif" => "image/avif",
            "webp" => "image/webp",
            "gif" => "image/gif",
            _ => "application/octet-stream",
        };
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
    let img = image::ImageReader::new(Cursor::new(data))
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
            image::imageops::FilterType::Lanczos3,
        )
    } else {
        img
    };

    let mut png_data = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_data);
    encoder
        .write_image(
            final_img.as_raw(),
            final_img.width(),
            final_img.height(),
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|e| format!("PNG エンコードに失敗: {}", e))?;

    let base64_str = base64::engine::general_purpose::STANDARD.encode(&png_data);
    Ok(format!("data:image/png;base64,{}", base64_str))
}
