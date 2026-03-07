use super::audit::AuditLog;
use super::error::{no_workspace_error, sandbox_error};
use super::sandbox::resolve_sandboxed_path;
use rmcp::ErrorData as McpError;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Maximum bytes returned from a single read_file call (1 MB).
const MAX_READ_BYTES: u64 = 1_048_576;

/// Default page size for paginated reads (64 KB).
const DEFAULT_PAGE_SIZE: u64 = 65_536;

/// Maximum file size for write_file (1 MB).
const MAX_WRITE_BYTES: usize = 1_048_576;

/// Read a file within the workspace sandbox with pagination/capping.
pub fn read_file(
    workspace: Option<&Path>,
    relative_path: &str,
    offset: Option<u64>,
    max_bytes: Option<u64>,
) -> Result<ReadFileResult, McpError> {
    let ws = workspace.ok_or_else(no_workspace_error)?;
    let resolved =
        resolve_sandboxed_path(ws, relative_path).map_err(|e| sandbox_error(e.to_string()))?;

    let metadata = std::fs::metadata(&resolved)
        .map_err(|e| McpError::internal_error(format!("Cannot read file metadata: {e}"), None))?;
    let file_size = metadata.len();

    let offset = offset.unwrap_or(0);
    let page_size = max_bytes.unwrap_or(DEFAULT_PAGE_SIZE).min(MAX_READ_BYTES);

    if offset >= file_size {
        return Ok(ReadFileResult {
            content: String::new(),
            file_size,
            offset,
            bytes_read: 0,
            truncated: false,
        });
    }

    use std::io::{Read, Seek, SeekFrom};
    let mut file = std::fs::File::open(&resolved)
        .map_err(|e| McpError::internal_error(format!("Cannot open file: {e}"), None))?;

    if offset > 0 {
        file.seek(SeekFrom::Start(offset))
            .map_err(|e| McpError::internal_error(format!("Cannot seek in file: {e}"), None))?;
    }

    let bytes_to_read = page_size.min(file_size - offset) as usize;
    let mut buffer = vec![0u8; bytes_to_read];
    let bytes_read = file
        .read(&mut buffer)
        .map_err(|e| McpError::internal_error(format!("Cannot read file: {e}"), None))?;
    buffer.truncate(bytes_read);

    let truncated = offset + (bytes_read as u64) < file_size;
    let content = String::from_utf8_lossy(&buffer).into_owned();

    Ok(ReadFileResult {
        content,
        file_size,
        offset,
        bytes_read: bytes_read as u64,
        truncated,
    })
}

/// Write a file within the workspace sandbox. Creates parent directories as needed.
pub fn write_file(
    workspace: Option<&Path>,
    relative_path: &str,
    content: &str,
    audit_log: Option<&AuditLog>,
) -> Result<WriteFileResult, McpError> {
    let ws = workspace.ok_or_else(no_workspace_error)?;
    let resolved =
        resolve_sandboxed_path(ws, relative_path).map_err(|e| sandbox_error(e.to_string()))?;

    if content.len() > MAX_WRITE_BYTES {
        return Err(McpError::invalid_params(
            format!(
                "Content exceeds maximum write size ({MAX_WRITE_BYTES} bytes). \
                 Split large content into multiple writes."
            ),
            None,
        ));
    }

    // Create parent directories if they don't exist (within the sandbox)
    if let Some(parent) = resolved.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| {
                McpError::internal_error(format!("Cannot create parent directories: {e}"), None)
            })?;
            // Verify the created dirs are still inside sandbox
            let canonical_parent = parent.canonicalize().map_err(|e| {
                McpError::internal_error(format!("Cannot canonicalize new parent: {e}"), None)
            })?;
            let canonical_ws = ws.canonicalize().map_err(|e| {
                McpError::internal_error(format!("Cannot canonicalize workspace: {e}"), None)
            })?;
            if !canonical_parent.starts_with(&canonical_ws) {
                return Err(sandbox_error("Created directories escaped the workspace"));
            }
        }
    }

    let bytes_written = content.len();
    std::fs::write(&resolved, content)
        .map_err(|e| McpError::internal_error(format!("Cannot write file: {e}"), None))?;

    if let Some(log) = audit_log {
        log.record_write(relative_path, bytes_written);
    }

    Ok(WriteFileResult {
        success: true,
        bytes_written,
        path: relative_path.to_string(),
    })
}

/// List files in the workspace (non-recursive by default, optional recursive).
pub fn list_files(
    workspace: Option<&Path>,
    subdirectory: Option<&str>,
    recursive: bool,
) -> Result<ListFilesResult, McpError> {
    let ws = workspace.ok_or_else(no_workspace_error)?;

    let target_dir = if let Some(subdir) = subdirectory {
        resolve_sandboxed_path(ws, subdir).map_err(|e| sandbox_error(e.to_string()))?
    } else {
        ws.canonicalize().map_err(|e| {
            McpError::internal_error(format!("Cannot canonicalize workspace: {e}"), None)
        })?
    };

    if !target_dir.is_dir() {
        return Err(McpError::invalid_params(
            format!("{} is not a directory", target_dir.display()),
            None,
        ));
    }

    let canonical_ws = ws.canonicalize().map_err(|e| {
        McpError::internal_error(format!("Cannot canonicalize workspace: {e}"), None)
    })?;

    let mut entries = Vec::new();
    let mut visited = HashSet::new();
    // Seed visited set with the starting directory
    if let Ok(canonical_start) = target_dir.canonicalize() {
        visited.insert(canonical_start);
    }
    collect_entries(
        &target_dir,
        &canonical_ws,
        recursive,
        &mut entries,
        &mut visited,
    )?;

    Ok(ListFilesResult { files: entries })
}

fn collect_entries(
    dir: &Path,
    workspace_root: &Path,
    recursive: bool,
    out: &mut Vec<FileEntry>,
    visited: &mut HashSet<PathBuf>,
) -> Result<(), McpError> {
    let read_dir = std::fs::read_dir(dir)
        .map_err(|e| McpError::internal_error(format!("Cannot read directory: {e}"), None))?;

    for entry in read_dir {
        let entry = entry.map_err(|e| {
            McpError::internal_error(format!("Cannot read directory entry: {e}"), None)
        })?;
        let path = entry.path();

        // Skip hidden files/directories
        if path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with('.'))
        {
            continue;
        }

        // Use symlink_metadata to detect symlinks without following them
        let meta = match std::fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };

        // Skip symlinked directories to prevent symlink loops and escapes
        if meta.file_type().is_symlink() && path.is_dir() {
            continue;
        }

        let is_dir = meta.is_dir();
        let relative = path
            .strip_prefix(workspace_root)
            .unwrap_or(&path)
            .to_string_lossy()
            .into_owned();

        let size = if is_dir { None } else { Some(meta.len()) };

        out.push(FileEntry {
            path: relative,
            is_directory: is_dir,
            size,
        });

        if is_dir && recursive {
            // Canonicalize and verify the directory is within the workspace and not visited
            if let Ok(canonical_dir) = path.canonicalize() {
                if !canonical_dir.starts_with(workspace_root) {
                    continue; // Outside workspace boundary
                }
                if !visited.insert(canonical_dir) {
                    continue; // Already visited — prevent loops
                }
            } else {
                continue; // Cannot canonicalize — skip
            }
            collect_entries(&path, workspace_root, true, out, visited)?;
        }
    }

    Ok(())
}

// ── Result types ──────────────────────────────────────────────

#[derive(serde::Serialize)]
pub struct ReadFileResult {
    pub content: String,
    pub file_size: u64,
    pub offset: u64,
    pub bytes_read: u64,
    pub truncated: bool,
}

#[derive(serde::Serialize)]
pub struct WriteFileResult {
    pub success: bool,
    pub bytes_written: usize,
    pub path: String,
}

#[derive(serde::Serialize)]
pub struct ListFilesResult {
    pub files: Vec<FileEntry>,
}

#[derive(serde::Serialize)]
pub struct FileEntry {
    pub path: String,
    pub is_directory: bool,
    pub size: Option<u64>,
}

// ── Workspace resolution ──────────────────────────────────────

/// Resolve workspace_dir parameter: use explicit if provided, else fall back to configured default.
pub fn resolve_workspace(explicit: Option<&str>, default: Option<&PathBuf>) -> Option<PathBuf> {
    explicit
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .or_else(|| default.cloned())
}
