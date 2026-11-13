//! Discovery of the running JustSlide instance and the pairing secret.
//!
//! Trust model (see docs/mcp-and-plugins.md):
//! * The browser writes `<user-data-dir>/JustSlideAutomation/endpoint.json`
//!   (dir 0700, file 0600) naming its socket and instance id.
//! * The bridge refuses discovery files, directories and sockets that are not
//!   owned by the current effective uid or are group/world accessible, and
//!   refuses sockets outside a private (0700, same-uid) directory.
//! * The pairing token comes from the macOS Keychain or a 0600 token file.
//!   It is never read from argv or ordinary environment variables and is
//!   never logged.
//! These are local-process hygiene checks, not a sandbox: any process running
//! as the same user can still read that user's files.

use anyhow::{Context, Result, anyhow, bail, ensure};
use serde::Deserialize;
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::path::{Path, PathBuf};

pub const AUTOMATION_DIR: &str = "JustSlideAutomation";
pub const ENDPOINT_FILE: &str = "endpoint.json";
pub const KEYCHAIN_SERVICE: &str = "app.justslide.JustSlide.dev.pairing";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EndpointRecord {
    pub protocol: String,
    pub protocol_version: String,
    pub instance_id: String,
    pub socket: PathBuf,
    #[serde(default)]
    pub pid: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct Endpoint {
    pub record: EndpointRecord,
    pub endpoint_file: PathBuf,
}

/// Development default user-data directory. Matches the browser's
/// CrProductDirName ("JustSlide Dev") on macOS; distinct from Chrome,
/// Chromium and the separate `Slide` application.
pub fn default_user_data_dir() -> Result<PathBuf> {
    let home = std::env::var_os("HOME").ok_or_else(|| anyhow!("HOME is not set"))?;
    let home = PathBuf::from(home);
    if cfg!(target_os = "macos") {
        Ok(home.join("Library/Application Support/JustSlide Dev"))
    } else {
        let base = std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from).unwrap_or_else(|| home.join(".config"));
        Ok(base.join("justslide-dev"))
    }
}

fn euid() -> u32 {
    // SAFETY: geteuid has no preconditions and cannot fail.
    unsafe { libc::geteuid() }
}

fn require_private(path: &Path, what: &str, max_mode: u32) -> Result<std::fs::Metadata> {
    let md = std::fs::symlink_metadata(path).with_context(|| format!("{what} {} not found", path.display()))?;
    ensure!(!md.file_type().is_symlink(), "{what} {} must not be a symlink", path.display());
    ensure!(md.uid() == euid(), "{what} {} is not owned by the current user", path.display());
    let mode = md.mode() & 0o777;
    ensure!(mode & !max_mode == 0, "{what} {} has mode {:o}; expected at most {:o}", path.display(), mode, max_mode);
    Ok(md)
}

pub fn discover(user_data_dir: &Path, expected_protocol: &str) -> Result<Endpoint> {
    let dir = user_data_dir.join(AUTOMATION_DIR);
    require_private(&dir, "automation directory", 0o700)?;
    let file = dir.join(ENDPOINT_FILE);
    require_private(&file, "endpoint file", 0o600)?;
    let text = std::fs::read_to_string(&file)?;
    ensure!(text.len() < 4096, "endpoint file is unexpectedly large");
    let record: EndpointRecord = serde_json::from_str(&text).context("endpoint file is not valid JSON")?;
    ensure!(record.protocol == expected_protocol, "endpoint speaks {} not {}", record.protocol, expected_protocol);
    ensure!(record.protocol_version.starts_with("1."), "unsupported protocol version {}", record.protocol_version);
    ensure!(record.socket.is_absolute(), "socket path must be absolute");
    let sock_md = require_private(&record.socket, "socket", 0o700)?;
    ensure!(sock_md.file_type().is_socket(), "{} is not a socket", record.socket.display());
    let parent = record.socket.parent().ok_or_else(|| anyhow!("socket has no parent dir"))?;
    require_private(parent, "socket directory", 0o700)?;
    Ok(Endpoint { record, endpoint_file: file })
}

pub enum TokenSource {
    File(PathBuf),
    Keychain { account: String },
}

pub fn read_token(source: &TokenSource) -> Result<String> {
    let token = match source {
        TokenSource::File(p) => {
            require_private(p, "token file", 0o600)?;
            std::fs::read_to_string(p)?.trim().to_string()
        }
        TokenSource::Keychain { account } => {
            if !cfg!(target_os = "macos") {
                bail!("Keychain token storage is only available on macOS; pass --token-file");
            }
            let out = std::process::Command::new("/usr/bin/security")
                .args(["find-generic-password", "-s", KEYCHAIN_SERVICE, "-a", account, "-w"])
                .output()
                .context("running /usr/bin/security")?;
            ensure!(out.status.success(), "no pairing token in Keychain for service {KEYCHAIN_SERVICE} account {account}; pair this client from JustSlide first");
            String::from_utf8(out.stdout)?.trim().to_string()
        }
    };
    ensure!((43..=128).contains(&token.len()), "pairing token has an invalid length");
    ensure!(token.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'), "pairing token has invalid characters");
    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn chmod(p: &Path, m: u32) { std::fs::set_permissions(p, std::fs::Permissions::from_mode(m)).unwrap(); }

    #[test]
    fn rejects_world_readable_token_file() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("t");
        std::fs::write(&p, "a".repeat(43)).unwrap();
        chmod(&p, 0o644);
        assert!(read_token(&TokenSource::File(p.clone())).is_err());
        chmod(&p, 0o600);
        assert_eq!(read_token(&TokenSource::File(p)).unwrap().len(), 43);
    }

    #[test]
    fn rejects_short_or_weird_tokens() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("t");
        std::fs::write(&p, "short").unwrap();
        chmod(&p, 0o600);
        assert!(read_token(&TokenSource::File(p.clone())).is_err());
        std::fs::write(&p, format!("{}$", "a".repeat(50))).unwrap();
        assert!(read_token(&TokenSource::File(p)).is_err());
    }

    #[test]
    fn rejects_open_automation_dir() {
        let d = tempfile::tempdir().unwrap();
        let a = d.path().join(AUTOMATION_DIR);
        std::fs::create_dir(&a).unwrap();
        chmod(&a, 0o755);
        let err = discover(d.path(), "x").unwrap_err().to_string();
        assert!(err.contains("mode"), "{err}");
    }
}
