use std::path::{Path, PathBuf};

/// Validate and canonicalize a path, ensuring it stays within the workspace sandbox.
///
/// # Security properties
/// - Resolves symlinks via `canonicalize()` to prevent symlink traversal.
/// - Rejects `..` components before canonicalization as an early guard.
/// - Confirms the canonical path starts with the canonical workspace prefix.
/// - Rejects absolute paths in the relative component.
pub fn resolve_sandboxed_path(
    workspace: &Path,
    relative_path: &str,
) -> Result<PathBuf, SandboxError> {
    // Reject empty paths
    if relative_path.is_empty() {
        return Err(SandboxError::EmptyPath);
    }

    // Reject absolute paths in the relative component
    if Path::new(relative_path).is_absolute() {
        return Err(SandboxError::AbsolutePath);
    }

    // Early rejection of path traversal components
    for component in Path::new(relative_path).components() {
        if matches!(component, std::path::Component::ParentDir) {
            return Err(SandboxError::TraversalAttempt);
        }
    }

    // Reject null bytes
    if relative_path.contains('\0') {
        return Err(SandboxError::NullByte);
    }

    // Canonicalize workspace to resolve any symlinks in the base
    let canonical_workspace = workspace
        .canonicalize()
        .map_err(|e| SandboxError::IoError(format!("Cannot canonicalize workspace: {e}")))?;

    let target = canonical_workspace.join(relative_path);

    // For existence checks, canonicalize the full path to resolve symlinks.
    // For new files, canonicalize the parent directory.
    let canonical_target = if target.exists() {
        target
            .canonicalize()
            .map_err(|e| SandboxError::IoError(format!("Cannot canonicalize target: {e}")))?
    } else {
        // File doesn't exist yet — walk up to find the nearest existing ancestor,
        // canonicalize it, verify confinement, and append the remaining components.
        let parent = target
            .parent()
            .ok_or_else(|| SandboxError::IoError("No parent directory".to_string()))?;

        // Find the nearest existing ancestor by walking up with Path::ancestors()
        let mut existing_ancestor = None;
        for ancestor in parent.ancestors() {
            if ancestor.exists() {
                existing_ancestor = Some(ancestor.to_path_buf());
                break;
            }
        }
        let existing_ancestor = existing_ancestor.ok_or_else(|| {
            SandboxError::IoError(format!(
                "No existing ancestor directory found for: {}",
                parent.display()
            ))
        })?;

        let canonical_ancestor = existing_ancestor
            .canonicalize()
            .map_err(|e| SandboxError::IoError(format!("Cannot canonicalize ancestor: {e}")))?;

        // Verify the existing ancestor is inside the workspace sandbox
        if !canonical_ancestor.starts_with(&canonical_workspace) {
            return Err(SandboxError::OutsideWorkspace);
        }

        // Rebuild the full path: canonical ancestor + remaining relative components + filename
        let suffix = target
            .strip_prefix(&existing_ancestor)
            .map_err(|e| SandboxError::IoError(format!("Cannot compute path suffix: {e}")))?;
        canonical_ancestor.join(suffix)
    };

    // Prefix check: canonical target must be inside canonical workspace
    if !canonical_target.starts_with(&canonical_workspace) {
        return Err(SandboxError::OutsideWorkspace);
    }

    Ok(canonical_target)
}

#[derive(Debug)]
pub enum SandboxError {
    EmptyPath,
    AbsolutePath,
    TraversalAttempt,
    NullByte,
    OutsideWorkspace,
    IoError(String),
}

impl std::fmt::Display for SandboxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyPath => write!(f, "Path must not be empty"),
            Self::AbsolutePath => write!(
                f,
                "Absolute paths are not allowed; use a relative path within the workspace"
            ),
            Self::TraversalAttempt => write!(f, "Path traversal (.. components) is not allowed"),
            Self::NullByte => write!(f, "Null bytes in path are not allowed"),
            Self::OutsideWorkspace => write!(f, "Resolved path is outside the workspace sandbox"),
            Self::IoError(msg) => write!(f, "{msg}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> TempDir {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("chapter1.md"), "# Chapter 1\n").unwrap();
        std::fs::create_dir_all(dir.path().join("subdir")).unwrap();
        std::fs::write(dir.path().join("subdir/chapter2.md"), "# Chapter 2\n").unwrap();
        dir
    }

    #[test]
    fn test_valid_relative_path() {
        let dir = setup();
        let result = resolve_sandboxed_path(dir.path(), "chapter1.md");
        assert!(result.is_ok());
        assert!(result.unwrap().ends_with("chapter1.md"));
    }

    #[test]
    fn test_valid_subdirectory_path() {
        let dir = setup();
        let result = resolve_sandboxed_path(dir.path(), "subdir/chapter2.md");
        assert!(result.is_ok());
    }

    #[test]
    fn test_reject_parent_traversal() {
        let dir = setup();
        let result = resolve_sandboxed_path(dir.path(), "../etc/passwd");
        assert!(matches!(result, Err(SandboxError::TraversalAttempt)));
    }

    #[test]
    fn test_reject_hidden_traversal() {
        let dir = setup();
        let result = resolve_sandboxed_path(dir.path(), "subdir/../../etc/passwd");
        assert!(matches!(result, Err(SandboxError::TraversalAttempt)));
    }

    #[test]
    fn test_reject_absolute_path() {
        let dir = setup();
        let result = resolve_sandboxed_path(dir.path(), "/etc/passwd");
        assert!(matches!(result, Err(SandboxError::AbsolutePath)));
    }

    #[test]
    fn test_reject_empty_path() {
        let dir = setup();
        let result = resolve_sandboxed_path(dir.path(), "");
        assert!(matches!(result, Err(SandboxError::EmptyPath)));
    }

    #[test]
    fn test_reject_null_byte() {
        let dir = setup();
        let result = resolve_sandboxed_path(dir.path(), "chapter1\0.md");
        assert!(matches!(result, Err(SandboxError::NullByte)));
    }

    #[test]
    fn test_new_file_in_existing_directory() {
        let dir = setup();
        let result = resolve_sandboxed_path(dir.path(), "new_chapter.md");
        assert!(result.is_ok());
    }

    #[test]
    fn test_new_file_in_nonexistent_directory() {
        let dir = setup();
        // With ancestor walking, nested dirs within the workspace are allowed
        let result = resolve_sandboxed_path(dir.path(), "nonexistent/chapter.md");
        assert!(result.is_ok());
        assert!(result.unwrap().ends_with("nonexistent/chapter.md"));
    }

    #[cfg(unix)]
    #[test]
    fn test_symlink_escape_blocked() {
        let dir = setup();
        let escape_target = TempDir::new().unwrap();
        std::fs::write(escape_target.path().join("secret.txt"), "secret").unwrap();

        // Create a symlink inside workspace pointing outside
        std::os::unix::fs::symlink(escape_target.path(), dir.path().join("escape")).unwrap();

        let result = resolve_sandboxed_path(dir.path(), "escape/secret.txt");
        assert!(matches!(result, Err(SandboxError::OutsideWorkspace)));
    }
}
