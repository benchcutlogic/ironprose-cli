use chrono::Utc;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Local-only audit log for write operations in the workspace.
///
/// Appends structured JSONL entries to `.ironprose/audit.jsonl` inside the workspace.
/// This log is purely informational and never leaves the local machine.
pub struct AuditLog {
    log_path: PathBuf,
    file: Mutex<Option<std::fs::File>>,
}

#[derive(serde::Serialize)]
struct AuditEntry<'a> {
    timestamp: String,
    operation: &'a str,
    path: &'a str,
    bytes_written: Option<usize>,
    truncated: bool,
}

impl AuditLog {
    /// Create a new audit log for the given workspace directory.
    /// The log file will be created at `<workspace>/.ironprose/audit.jsonl`.
    pub fn new(workspace: &Path) -> Self {
        let log_dir = workspace.join(".ironprose");
        let log_path = log_dir.join("audit.jsonl");
        Self {
            log_path,
            file: Mutex::new(None),
        }
    }

    /// Record a write operation in the audit log.
    pub fn record_write(&self, relative_path: &str, bytes_written: usize) {
        self.append(&AuditEntry {
            timestamp: Utc::now().to_rfc3339(),
            operation: "write_file",
            path: relative_path,
            bytes_written: Some(bytes_written),
            truncated: false,
        });
    }

    fn append(&self, entry: &AuditEntry<'_>) {
        let Ok(line) = serde_json::to_string(entry) else {
            return;
        };

        let mut guard = self.file.lock().unwrap_or_else(|e| e.into_inner());

        // Lazy-open on first write
        if guard.is_none() {
            if let Some(parent) = self.log_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            match std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.log_path)
            {
                Ok(f) => *guard = Some(f),
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to open audit log");
                    return;
                }
            }
        }

        if let Some(ref mut f) = *guard {
            let _ = writeln!(f, "{line}");
        }
    }
}
