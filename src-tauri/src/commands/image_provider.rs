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
    filter_type: Option<String>,
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

    if matches!(ext.as_str(), "jpg" | "jpeg") {
        if let Some((w, h)) = get_jpeg_dimensions(&raw_data) {
            if w <= max_width && h <= max_height {
                let b64 = base64::engine::general_purpose::STANDARD.encode(&raw_data);
                return Ok(ImageResult {
                    data_url: format!("data:image/jpeg;base64,{}", b64),
                });
            }
        }
    }

    if ext.as_str() == "png" {
        if let Some((w, h)) = get_png_dimensions(&raw_data) {
            if w <= max_width && h <= max_height {
                let b64 = base64::engine::general_purpose::STANDARD.encode(&raw_data);
                return Ok(ImageResult {
                    data_url: format!("data:image/png;base64,{}", b64),
                });
            }
        }
    }

    let data_url =
        encode_image_to_data_url(&raw_data, max_width, max_height, filter_type.as_deref())?;
    Ok(ImageResult { data_url })
}

fn read_filesystem_image(path: &Path) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|e| format!("画像ファイルの読み取りに失敗: {}", e))
}

fn get_jpeg_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    if data.len() < 4 || data[0] != 0xFF || data[1] != 0xD8 {
        return None;
    }
    let mut pos = 2usize;
    while pos + 9 <= data.len() {
        if data[pos] != 0xFF {
            return None;
        }
        let marker = data[pos + 1];
        pos += 2;
        if marker == 0xDA {
            return None;
        }
        if (0xC0..=0xCF).contains(&marker) && marker != 0xC4 && marker != 0xC8 && marker != 0xCC {
            let height = ((data[pos + 3] as u32) << 8) | data[pos + 4] as u32;
            let width = ((data[pos + 5] as u32) << 8) | data[pos + 6] as u32;
            return Some((width, height));
        }
        let seg_len = ((data[pos] as usize) << 8) | data[pos + 1] as usize;
        pos += seg_len;
    }
    None
}

fn get_png_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    if data.len() < 24 {
        return None;
    }
    let sig: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    if data[0..8] != sig {
        return None;
    }
    if &data[12..16] != b"IHDR" {
        return None;
    }
    let width = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
    let height = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
    Some((width, height))
}

fn encode_image_to_data_url(
    data: &[u8],
    max_width: u32,
    max_height: u32,
    filter_name: Option<&str>,
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

        image::imageops::resize(&img, new_width, new_height, parse_filter(filter_name))
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

fn parse_filter(name: Option<&str>) -> image::imageops::FilterType {
    match name.unwrap_or("catmull_rom") {
        "nearest" => image::imageops::FilterType::Nearest,
        "triangle" => image::imageops::FilterType::Triangle,
        "catmull_rom" => image::imageops::FilterType::CatmullRom,
        "gaussian" => image::imageops::FilterType::Gaussian,
        "lanczos3" => image::imageops::FilterType::Lanczos3,
        _ => image::imageops::FilterType::CatmullRom,
    }
}
