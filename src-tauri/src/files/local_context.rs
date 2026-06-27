//! Explicitly selected, bounded local files for AI context.
//!
//! The frontend never passes a filesystem path to the read command. A native
//! picker registers canonical paths in this process and returns opaque IDs;
//! only those IDs can subsequently be read. Registrations are in-memory and
//! disappear when the app exits.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::Mutex,
};
use tokio::io::AsyncReadExt;

pub const MAX_CONTEXT_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const MAX_FILES_PER_SELECTION: usize = 10;
const MAX_REGISTERED_FILES: usize = 64;

/// Kept in one place so the native picker and backend validation cannot drift.
pub const ALLOWED_CONTEXT_EXTENSIONS: &[&str] = &[
    "txt",
    "log",
    "md",
    "markdown",
    "csv",
    "tsv",
    "json",
    "jsonl",
    "ndjson",
    "yaml",
    "yml",
    "toml",
    "xml",
    "html",
    "htm",
    "ini",
    "conf",
    "properties",
    "sql",
    "rs",
    "ts",
    "js",
    "svelte",
    "py",
    "go",
    "java",
    "c",
    "h",
    "cpp",
    "hpp",
    "cs",
    "sh",
    "ps1",
];

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContextFileKind {
    PlainText,
    Markdown,
    DelimitedData,
    Json,
    Yaml,
    Toml,
    Xml,
    Html,
    Configuration,
    Sql,
    SourceCode,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContextFileErrorCode {
    DialogFailed,
    UnsupportedPath,
    UnsupportedType,
    NotAFile,
    FileTooLarge,
    FileUnavailable,
    InvalidEncoding,
    BinaryContent,
    NotSelected,
    SelectionLimit,
    RegistryFull,
    Internal,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextFileError {
    pub code: ContextFileErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
}

impl ContextFileError {
    pub fn new(
        code: ContextFileErrorCode,
        message: impl Into<String>,
        file_name: Option<String>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            file_name,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextFileIssue {
    pub code: ContextFileErrorCode,
    pub message: String,
    pub file_name: String,
}

impl From<ContextFileError> for ContextFileIssue {
    fn from(error: ContextFileError) -> Self {
        Self {
            code: error.code,
            message: error.message,
            file_name: error.file_name.unwrap_or_else(|| "unknown".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextFileMetadata {
    pub selection_id: String,
    pub file_name: String,
    pub extension: String,
    pub kind: ContextFileKind,
    pub size_bytes: u64,
    pub modified_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextFileSelection {
    pub cancelled: bool,
    pub files: Vec<ContextFileMetadata>,
    pub rejected: Vec<ContextFileIssue>,
    pub max_file_bytes: u64,
}

impl ContextFileSelection {
    pub fn cancelled() -> Self {
        Self {
            cancelled: true,
            files: Vec::new(),
            rejected: Vec::new(),
            max_file_bytes: MAX_CONTEXT_FILE_BYTES,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextFileDocument {
    pub file: ContextFileMetadata,
    pub content: String,
    pub content_sha256: String,
}

#[derive(Debug, Clone)]
pub(crate) struct RegisteredContextFile {
    path: PathBuf,
    metadata: ContextFileMetadata,
}

#[derive(Default)]
pub struct ContextFileAccess {
    selected: Mutex<HashMap<String, RegisteredContextFile>>,
}

impl ContextFileAccess {
    pub(crate) fn register_paths(
        &self,
        paths: Vec<PathBuf>,
        mut rejected: Vec<ContextFileIssue>,
    ) -> Result<ContextFileSelection, ContextFileError> {
        let mut selected = self.selected.lock().map_err(|_| {
            ContextFileError::new(
                ContextFileErrorCode::Internal,
                "The selected-file registry is unavailable.",
                None,
            )
        })?;
        let mut accepted = Vec::new();
        let mut seen = HashSet::new();

        for (index, path) in paths.into_iter().enumerate() {
            let visible_name = display_file_name(&path);
            if index >= MAX_FILES_PER_SELECTION {
                rejected.push(
                    ContextFileError::new(
                        ContextFileErrorCode::SelectionLimit,
                        format!("Select at most {MAX_FILES_PER_SELECTION} files at a time."),
                        Some(visible_name),
                    )
                    .into(),
                );
                continue;
            }

            let inspected = match inspect_selected_path(&path) {
                Ok(file) => file,
                Err(error) => {
                    rejected.push(error.into());
                    continue;
                }
            };

            if !seen.insert(inspected.path.clone()) {
                continue;
            }

            if selected.len() >= MAX_REGISTERED_FILES {
                rejected.push(
                    ContextFileError::new(
                        ContextFileErrorCode::RegistryFull,
                        "Too many files are registered. Clear previous context files and try again.",
                        Some(inspected.metadata.file_name),
                    )
                    .into(),
                );
                continue;
            }

            let metadata = inspected.metadata.clone();
            selected.insert(metadata.selection_id.clone(), inspected);
            accepted.push(metadata);
        }

        Ok(ContextFileSelection {
            cancelled: false,
            files: accepted,
            rejected,
            max_file_bytes: MAX_CONTEXT_FILE_BYTES,
        })
    }

    pub(crate) fn resolve(
        &self,
        selection_id: &str,
    ) -> Result<RegisteredContextFile, ContextFileError> {
        if selection_id.len() > 64 {
            return Err(not_selected_error());
        }
        self.selected
            .lock()
            .map_err(|_| {
                ContextFileError::new(
                    ContextFileErrorCode::Internal,
                    "The selected-file registry is unavailable.",
                    None,
                )
            })?
            .get(selection_id)
            .cloned()
            .ok_or_else(not_selected_error)
    }

    pub fn forget(&self, selection_id: &str) -> Result<bool, ContextFileError> {
        if selection_id.len() > 64 {
            return Ok(false);
        }
        Ok(self
            .selected
            .lock()
            .map_err(|_| {
                ContextFileError::new(
                    ContextFileErrorCode::Internal,
                    "The selected-file registry is unavailable.",
                    None,
                )
            })?
            .remove(selection_id)
            .is_some())
    }

    pub fn clear(&self) -> Result<usize, ContextFileError> {
        let mut selected = self.selected.lock().map_err(|_| {
            ContextFileError::new(
                ContextFileErrorCode::Internal,
                "The selected-file registry is unavailable.",
                None,
            )
        })?;
        let removed = selected.len();
        selected.clear();
        Ok(removed)
    }
}

fn not_selected_error() -> ContextFileError {
    ContextFileError::new(
        ContextFileErrorCode::NotSelected,
        "This file was not explicitly selected in the current app session.",
        None,
    )
}

fn inspect_selected_path(path: &Path) -> Result<RegisteredContextFile, ContextFileError> {
    let visible_name = display_file_name(path);
    let canonical = std::fs::canonicalize(path).map_err(|_| {
        ContextFileError::new(
            ContextFileErrorCode::FileUnavailable,
            "The selected file is no longer available.",
            Some(visible_name.clone()),
        )
    })?;
    let fs_metadata = std::fs::metadata(&canonical).map_err(|_| {
        ContextFileError::new(
            ContextFileErrorCode::FileUnavailable,
            "The selected file metadata could not be read.",
            Some(visible_name.clone()),
        )
    })?;

    if !fs_metadata.is_file() {
        return Err(ContextFileError::new(
            ContextFileErrorCode::NotAFile,
            "Only regular files can be attached as AI context.",
            Some(visible_name),
        ));
    }

    let (extension, kind) = classify_path(&canonical).ok_or_else(|| {
        ContextFileError::new(
            ContextFileErrorCode::UnsupportedType,
            "This file type is not allowed for AI text context.",
            Some(visible_name.clone()),
        )
    })?;

    if fs_metadata.len() > MAX_CONTEXT_FILE_BYTES {
        return Err(ContextFileError::new(
            ContextFileErrorCode::FileTooLarge,
            format!(
                "The file exceeds the {} MiB context limit.",
                MAX_CONTEXT_FILE_BYTES / 1024 / 1024
            ),
            Some(visible_name),
        ));
    }

    let modified_at = fs_metadata
        .modified()
        .ok()
        .map(DateTime::<Utc>::from)
        .map(|value| value.to_rfc3339());
    let file_name = display_file_name(&canonical);
    let selection_id = uuid::Uuid::new_v4().to_string();

    Ok(RegisteredContextFile {
        path: canonical,
        metadata: ContextFileMetadata {
            selection_id,
            file_name,
            extension,
            kind,
            size_bytes: fs_metadata.len(),
            modified_at,
        },
    })
}

fn classify_path(path: &Path) -> Option<(String, ContextFileKind)> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    let kind = match extension.as_str() {
        "txt" | "log" => ContextFileKind::PlainText,
        "md" | "markdown" => ContextFileKind::Markdown,
        "csv" | "tsv" => ContextFileKind::DelimitedData,
        "json" | "jsonl" | "ndjson" => ContextFileKind::Json,
        "yaml" | "yml" => ContextFileKind::Yaml,
        "toml" => ContextFileKind::Toml,
        "xml" => ContextFileKind::Xml,
        "html" | "htm" => ContextFileKind::Html,
        "ini" | "conf" | "properties" => ContextFileKind::Configuration,
        "sql" => ContextFileKind::Sql,
        "rs" | "ts" | "js" | "svelte" | "py" | "go" | "java" | "c" | "h" | "cpp" | "hpp" | "cs"
        | "sh" | "ps1" => ContextFileKind::SourceCode,
        _ => return None,
    };
    Some((extension, kind))
}

fn display_file_name(path: &Path) -> String {
    path.file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

pub(crate) async fn read_registered_file(
    registered: RegisteredContextFile,
) -> Result<ContextFileDocument, ContextFileError> {
    let file_name = registered.metadata.file_name.clone();
    let canonical = tokio::fs::canonicalize(&registered.path)
        .await
        .map_err(|_| {
            ContextFileError::new(
                ContextFileErrorCode::FileUnavailable,
                "The selected file is no longer available.",
                Some(file_name.clone()),
            )
        })?;
    if canonical != registered.path {
        return Err(ContextFileError::new(
            ContextFileErrorCode::FileUnavailable,
            "The selected file location changed. Select it again.",
            Some(file_name),
        ));
    }

    let fs_metadata = tokio::fs::metadata(&canonical).await.map_err(|_| {
        ContextFileError::new(
            ContextFileErrorCode::FileUnavailable,
            "The selected file metadata could not be read.",
            Some(file_name.clone()),
        )
    })?;
    if !fs_metadata.is_file() {
        return Err(ContextFileError::new(
            ContextFileErrorCode::NotAFile,
            "Only regular files can be attached as AI context.",
            Some(file_name),
        ));
    }
    if fs_metadata.len() > MAX_CONTEXT_FILE_BYTES {
        return Err(ContextFileError::new(
            ContextFileErrorCode::FileTooLarge,
            "The selected file grew beyond the context limit. Select a smaller file.",
            Some(file_name),
        ));
    }
    if classify_path(&canonical).is_none() {
        return Err(ContextFileError::new(
            ContextFileErrorCode::UnsupportedType,
            "The selected file type is no longer allowed.",
            Some(file_name),
        ));
    }

    // `take(limit + 1)` enforces the limit even if the file grows between the
    // metadata check and read.
    let file = tokio::fs::File::open(&canonical).await.map_err(|_| {
        ContextFileError::new(
            ContextFileErrorCode::FileUnavailable,
            "The selected file could not be opened.",
            Some(file_name.clone()),
        )
    })?;
    let mut bytes = Vec::with_capacity(fs_metadata.len() as usize);
    file.take(MAX_CONTEXT_FILE_BYTES + 1)
        .read_to_end(&mut bytes)
        .await
        .map_err(|_| {
            ContextFileError::new(
                ContextFileErrorCode::FileUnavailable,
                "The selected file could not be read.",
                Some(file_name.clone()),
            )
        })?;
    if bytes.len() as u64 > MAX_CONTEXT_FILE_BYTES {
        return Err(ContextFileError::new(
            ContextFileErrorCode::FileTooLarge,
            "The selected file grew beyond the context limit. Select a smaller file.",
            Some(file_name),
        ));
    }
    if bytes.contains(&0) {
        return Err(ContextFileError::new(
            ContextFileErrorCode::BinaryContent,
            "Binary content is not allowed in AI text context.",
            Some(file_name),
        ));
    }

    let text_bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(&bytes);
    let content = std::str::from_utf8(text_bytes)
        .map_err(|_| {
            ContextFileError::new(
                ContextFileErrorCode::InvalidEncoding,
                "The selected file must use UTF-8 text encoding.",
                Some(file_name),
            )
        })?
        .to_owned();

    let content_sha256 = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    let mut metadata = registered.metadata;
    metadata.size_bytes = bytes.len() as u64;

    Ok(ContextFileDocument {
        file: metadata,
        content,
        content_sha256,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_file(name: &str, bytes: &[u8]) -> (PathBuf, PathBuf) {
        let dir = std::env::temp_dir().join(format!("exsul-context-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        std::fs::write(&path, bytes).unwrap();
        (dir, path)
    }

    #[tokio::test]
    async fn selected_utf8_file_can_be_read_by_opaque_id() {
        let (dir, path) = temp_file("context.md", "ქართული context".as_bytes());
        let access = ContextFileAccess::default();
        let selection = access.register_paths(vec![path], Vec::new()).unwrap();
        assert_eq!(selection.files.len(), 1);
        assert!(!selection.files[0].selection_id.contains("context.md"));

        let registered = access.resolve(&selection.files[0].selection_id).unwrap();
        let document = read_registered_file(registered).await.unwrap();
        assert_eq!(document.content, "ქართული context");
        assert_eq!(document.content_sha256.len(), 64);

        let _ = std::fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn arbitrary_or_binary_files_are_not_read() {
        let access = ContextFileAccess::default();
        let error = access.resolve("C:\\private\\secret.txt").unwrap_err();
        assert_eq!(error.code, ContextFileErrorCode::NotSelected);

        let (dir, path) = temp_file("binary.txt", &[b'a', 0, b'b']);
        let selection = access.register_paths(vec![path], Vec::new()).unwrap();
        let registered = access.resolve(&selection.files[0].selection_id).unwrap();
        let error = read_registered_file(registered).await.unwrap_err();
        assert_eq!(error.code, ContextFileErrorCode::BinaryContent);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn unsupported_and_oversized_files_are_rejected_at_selection() {
        let (bad_dir, bad_path) = temp_file("document.exe", b"not executable");
        let (large_dir, large_path) = temp_file(
            "large.txt",
            &vec![b'x'; MAX_CONTEXT_FILE_BYTES as usize + 1],
        );
        let access = ContextFileAccess::default();
        let selection = access
            .register_paths(vec![bad_path, large_path], Vec::new())
            .unwrap();

        assert!(selection.files.is_empty());
        assert_eq!(selection.rejected.len(), 2);
        assert_eq!(
            selection.rejected[0].code,
            ContextFileErrorCode::UnsupportedType
        );
        assert_eq!(
            selection.rejected[1].code,
            ContextFileErrorCode::FileTooLarge
        );
        let _ = std::fs::remove_dir_all(bad_dir);
        let _ = std::fs::remove_dir_all(large_dir);
    }
}
