// Image upload handling. Saves uploaded files under static/uploads, downscales
// large images to cap disk/RAM usage, and returns the public URL path.
use anyhow::{anyhow, Result};
use std::io::Cursor;
use std::path::Path;

const MAX_DIMENSION: u32 = 1400;
const UPLOAD_DIR: &str = "static/uploads";

/// Persist raw image bytes, re-encoding to a reasonable size. Returns URL path.
pub fn save_image(bytes: &[u8], original_name: &str) -> Result<String> {
    if bytes.is_empty() {
        return Err(anyhow!("file kosong"));
    }
    if bytes.len() > 8 * 1024 * 1024 {
        return Err(anyhow!("ukuran file maksimal 8MB"));
    }
    std::fs::create_dir_all(UPLOAD_DIR)?;

    let ext = Path::new(original_name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("jpg")
        .to_lowercase();

    let id = uuid::Uuid::new_v4().to_string();

    // Try to decode & downscale. If decoding fails (e.g. SVG), store as-is.
    match image::load_from_memory(bytes) {
        Ok(img) => {
            let (w, h) = (img.width(), img.height());
            let resized = if w > MAX_DIMENSION || h > MAX_DIMENSION {
                img.resize(MAX_DIMENSION, MAX_DIMENSION, image::imageops::FilterType::Lanczos3)
            } else {
                img
            };
            let out_ext = if ext == "png" { "png" } else { "jpg" };
            let filename = format!("{id}.{out_ext}");
            let path = format!("{UPLOAD_DIR}/{filename}");
            let mut buf = Cursor::new(Vec::new());
            if out_ext == "png" {
                resized.write_to(&mut buf, image::ImageFormat::Png)?;
            } else {
                // convert to RGB8 to guarantee JPEG compatibility
                let rgb = resized.to_rgb8();
                let dynimg = image::DynamicImage::ImageRgb8(rgb);
                dynimg.write_to(&mut buf, image::ImageFormat::Jpeg)?;
            }
            std::fs::write(&path, buf.into_inner())?;
            Ok(format!("/static/uploads/{filename}"))
        }
        Err(_) => {
            // fallback: raw save (covers svg, etc.)
            let safe_ext = if ["svg", "gif", "webp"].contains(&ext.as_str()) { ext.as_str() } else { "bin" };
            let filename = format!("{id}.{safe_ext}");
            let path = format!("{UPLOAD_DIR}/{filename}");
            std::fs::write(&path, bytes)?;
            Ok(format!("/static/uploads/{filename}"))
        }
    }
}
