// Image upload handling. Saves uploaded files under static/uploads, downscales
// large images to cap disk/RAM usage, and returns the public URL path.
use anyhow::{anyhow, Result};
use std::io::Cursor;

const MAX_DIMENSION: u32 = 1400;
const UPLOAD_DIR: &str = "static/uploads";

/// Persist raw image bytes, re-encoding to a reasonable size. Returns URL path.
pub fn save_image(bytes: &[u8], _original_name: &str) -> Result<String> {
    if bytes.is_empty() {
        return Err(anyhow!("file kosong"));
    }
    if bytes.len() > 8 * 1024 * 1024 {
        return Err(anyhow!("ukuran file maksimal 8MB"));
    }
    std::fs::create_dir_all(UPLOAD_DIR)?;

    let id = uuid::Uuid::new_v4().to_string();

    // Only persist bytes successfully decoded by the image crate. Storing
    // arbitrary fallback files (especially active SVG/HTML) under our own
    // origin would allow stored XSS and unbounded file hosting.
    match image::load_from_memory(bytes) {
        Ok(img) => {
            let (w, h) = (img.width(), img.height());
            let resized = if w > MAX_DIMENSION || h > MAX_DIMENSION {
                img.resize(
                    MAX_DIMENSION,
                    MAX_DIMENSION,
                    image::imageops::FilterType::Lanczos3,
                )
            } else {
                img
            };
            let out_ext = if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
                "png"
            } else {
                "jpg"
            };
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
        Err(_) => Err(anyhow!("format gambar tidak didukung atau file rusak")),
    }
}
