//! Images pasted / dropped into notes. Files live in `<app data>/images/` and
//! notes reference them as `images/<file>` in their Markdown (served to the
//! webview through the asset protocol, scope `$APPDATA/images/**`).

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use tauri::{AppHandle, Manager};
use uuid::Uuid;

use crate::database::{note_repository, Database};
use crate::error::{AppError, AppResult};
use crate::image_refs;

/// Unreferenced images younger than this are never removed automatically.
pub const ORPHAN_GRACE: Duration = Duration::from_secs(24 * 60 * 60);

const IMAGES_DIR: &str = "images";
pub const MAX_IMAGE_BYTES: usize = 20 * 1024 * 1024;

pub fn images_dir(app: &AppHandle) -> AppResult<PathBuf> {
    Ok(app.path().app_data_dir()?.join(IMAGES_DIR))
}

pub fn save(app: &AppHandle, bytes: &[u8], extension: &str) -> AppResult<String> {
    save_into(&images_dir(app)?, bytes, extension)
}

/// Images referenced by any note in any status (trash included — restoring a
/// note must bring its images back).
pub fn referenced_by_notes(app: &AppHandle) -> AppResult<HashSet<String>> {
    let notes = app
        .state::<Database>()
        .with_conn(|conn| note_repository::list(conn))?;
    Ok(image_refs::referenced_by_all(
        notes.iter().map(|note| note.content.as_str()),
    ))
}

/// Removes unreferenced images older than `min_age`.
pub fn cleanup(app: &AppHandle, min_age: Duration) -> AppResult<CleanupResult> {
    delete_unreferenced(&images_dir(app)?, &referenced_by_notes(app)?, min_age)
}

/// Writes the image under a fresh name and returns the Markdown path (`images/<file>`).
pub fn save_into(dir: &Path, bytes: &[u8], extension: &str) -> AppResult<String> {
    let extension = normalize_extension(extension)?;
    if bytes.is_empty() {
        return Err(AppError::Internal("圖片是空的".into()));
    }
    if bytes.len() > MAX_IMAGE_BYTES {
        return Err(AppError::Internal(format!(
            "圖片超過 {} MB 上限",
            MAX_IMAGE_BYTES / 1024 / 1024
        )));
    }
    if !looks_like_image(bytes) {
        return Err(AppError::Internal("不是支援的圖片格式".into()));
    }

    std::fs::create_dir_all(dir)?;
    let file_name = format!("{}.{extension}", Uuid::new_v4().simple());
    std::fs::write(dir.join(&file_name), bytes)?;
    Ok(format!("{IMAGES_DIR}/{file_name}"))
}

/// Writes an image under a given (validated) name, e.g. when importing.
/// An existing file with that name is kept — names are unique and files immutable.
pub fn write_named(dir: &Path, file_name: &str, bytes: &[u8]) -> AppResult<bool> {
    if !image_refs::is_safe_file_name(file_name) || !looks_like_image(bytes) {
        return Err(AppError::Internal(format!("無效的圖片: {file_name}")));
    }
    let path = dir.join(file_name);
    if path.exists() {
        return Ok(false);
    }
    std::fs::create_dir_all(dir)?;
    std::fs::write(path, bytes)?;
    Ok(true)
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupResult {
    pub count: usize,
    pub bytes: u64,
}

/// Deletes image files that no note references and that are older than
/// `min_age` (a fresh paste may not be saved into the note yet, and Ctrl+Z can
/// bring a removed image back, so recent files are always kept).
pub fn delete_unreferenced(
    dir: &Path,
    referenced: &HashSet<String>,
    min_age: Duration,
) -> AppResult<CleanupResult> {
    let mut result = CleanupResult::default();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Ok(result); // no images folder yet
    };
    let now = SystemTime::now();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !image_refs::is_safe_file_name(&name) || referenced.contains(&name) {
            continue;
        }
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        let age = metadata
            .modified()
            .ok()
            .and_then(|modified| now.duration_since(modified).ok())
            .unwrap_or_default();
        if !metadata.is_file() || age < min_age {
            continue;
        }
        std::fs::remove_file(entry.path())?;
        result.count += 1;
        result.bytes += metadata.len();
    }
    Ok(result)
}

/// Deletes the given images unless still referenced (used after a permanent delete).
pub fn delete_if_unreferenced(
    dir: &Path,
    candidates: &HashSet<String>,
    referenced: &HashSet<String>,
) -> AppResult<usize> {
    let mut deleted = 0;
    for name in candidates.difference(referenced) {
        let path = dir.join(name);
        if path.is_file() {
            std::fs::remove_file(path)?;
            deleted += 1;
        }
    }
    Ok(deleted)
}

fn normalize_extension(extension: &str) -> AppResult<&'static str> {
    match extension
        .trim_start_matches('.')
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => Ok("png"),
        "jpg" | "jpeg" => Ok("jpg"),
        "gif" => Ok("gif"),
        "webp" => Ok("webp"),
        "bmp" => Ok("bmp"),
        other => Err(AppError::Internal(format!("不支援的圖片格式: {other}"))),
    }
}

/// Magic-number check so arbitrary files can't be written under an image name.
fn looks_like_image(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0x89, b'P', b'N', b'G'])
        || bytes.starts_with(&[0xFF, 0xD8, 0xFF])
        || bytes.starts_with(b"GIF8")
        || (bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP")
        || bytes.starts_with(b"BM")
}

#[cfg(test)]
mod tests {
    use super::*;

    const PNG: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0];

    fn temp_dir() -> PathBuf {
        std::env::temp_dir().join(format!("pinnote-images-{}", Uuid::new_v4()))
    }

    #[test]
    fn saves_valid_image_with_relative_path() {
        let dir = temp_dir();
        let path = save_into(&dir, PNG, "PNG").unwrap();
        assert!(path.starts_with("images/") && path.ends_with(".png"));
        let file = dir.join(path.trim_start_matches("images/"));
        assert_eq!(std::fs::read(file).unwrap(), PNG);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn rejects_bad_input() {
        let dir = temp_dir();
        assert!(save_into(&dir, PNG, "exe").is_err());
        assert!(save_into(&dir, b"MZ not an image", "png").is_err());
        assert!(save_into(&dir, &[], "png").is_err());
        assert!(!dir.exists(), "nothing should be written");
    }

    #[test]
    fn cleanup_keeps_referenced_and_recent_images() {
        let dir = temp_dir();
        let used = save_into(&dir, PNG, "png").unwrap().replace("images/", "");
        let orphan = save_into(&dir, PNG, "png").unwrap().replace("images/", "");
        std::fs::write(dir.join("notes.txt"), b"not an image").unwrap();
        let referenced = HashSet::from([used.clone()]);

        // Within the grace period nothing is touched.
        let kept = delete_unreferenced(&dir, &referenced, ORPHAN_GRACE).unwrap();
        assert_eq!(kept, CleanupResult::default());

        let removed = delete_unreferenced(&dir, &referenced, Duration::ZERO).unwrap();
        assert_eq!(removed.count, 1);
        assert_eq!(removed.bytes, PNG.len() as u64);
        assert!(dir.join(&used).exists());
        assert!(!dir.join(&orphan).exists());
        assert!(
            dir.join("notes.txt").exists(),
            "foreign files are never deleted"
        );
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn delete_if_unreferenced_spares_shared_images() {
        let dir = temp_dir();
        let a = save_into(&dir, PNG, "png").unwrap().replace("images/", "");
        let b = save_into(&dir, PNG, "png").unwrap().replace("images/", "");
        let candidates = HashSet::from([a.clone(), b.clone()]);
        let still_used = HashSet::from([b.clone()]);

        assert_eq!(
            delete_if_unreferenced(&dir, &candidates, &still_used).unwrap(),
            1
        );
        assert!(!dir.join(&a).exists());
        assert!(dir.join(&b).exists());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn write_named_validates_and_never_overwrites() {
        let dir = temp_dir();
        assert!(write_named(&dir, "abc.png", PNG).unwrap());
        assert!(!write_named(&dir, "abc.png", &[0x89, b'P', b'N', b'G', 1]).unwrap());
        assert_eq!(std::fs::read(dir.join("abc.png")).unwrap(), PNG);
        assert!(write_named(&dir, "../evil.png", PNG).is_err());
        assert!(write_named(&dir, "x.png", b"MZ").is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn normalizes_extensions() {
        assert_eq!(normalize_extension(".JPEG").unwrap(), "jpg");
        assert_eq!(normalize_extension("webp").unwrap(), "webp");
    }
}
