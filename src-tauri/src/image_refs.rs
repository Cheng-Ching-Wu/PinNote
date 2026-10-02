//! Finding which image files a note's Markdown refers to (`images/<file>`).
//! Shared by backups, export / import and orphan cleanup.

use std::collections::HashSet;

const PREFIX: &str = "images/";
const EXTENSIONS: [&str; 5] = ["png", "jpg", "gif", "webp", "bmp"];

/// A bare file name that PinNote itself could have written: `<alphanumeric>.<image ext>`.
/// Rejects anything with separators or `..`, so names are safe to join onto a directory.
pub fn is_safe_file_name(name: &str) -> bool {
    let Some((stem, ext)) = name.rsplit_once('.') else {
        return false;
    };
    !stem.is_empty() && stem.chars().all(|c| c.is_ascii_alphanumeric()) && EXTENSIONS.contains(&ext)
}

/// File names referenced as `images/<file>` anywhere in `markdown`.
pub fn referenced_files(markdown: &str) -> HashSet<String> {
    let mut files = HashSet::new();
    let mut rest = markdown;
    while let Some(index) = rest.find(PREFIX) {
        rest = &rest[index + PREFIX.len()..];
        let end = rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '.'))
            .unwrap_or(rest.len());
        let candidate = &rest[..end];
        if is_safe_file_name(candidate) {
            files.insert(candidate.to_string());
        }
    }
    files
}

/// Union of the references of many notes.
pub fn referenced_by_all<'a>(contents: impl IntoIterator<Item = &'a str>) -> HashSet<String> {
    contents.into_iter().flat_map(referenced_files).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_image_references() {
        let md = "截圖 ![a](images/abc123.png) 和 ![](images/DEF.jpg)\n\
                  網路圖 ![](https://x.com/images/y.png) 重複 ![b](images/abc123.png)";
        let files = referenced_files(md);
        // A web URL that happens to contain "images/" is still matched by name;
        // that's harmless (it only keeps a same-named local file alive).
        assert!(files.contains("abc123.png"));
        assert!(files.contains("DEF.jpg"));
        assert_eq!(files.len(), 3);
    }

    #[test]
    fn ignores_unsafe_or_unknown_names() {
        assert!(referenced_files("images/../secret.png images/a.exe images/.png").is_empty());
        assert!(!is_safe_file_name("a/b.png"));
        assert!(!is_safe_file_name("..png"));
        assert!(is_safe_file_name("0f3a9c.webp"));
    }
}
