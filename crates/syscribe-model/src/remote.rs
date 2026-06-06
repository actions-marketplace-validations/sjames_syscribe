//! Download hook for remote `sourceFile:` URIs.
//!
//! A model may point a `sourceFile:` at a remote location (`https://…`,
//! `git+ssh://…`, …). By default such files are accepted but not verified. When
//! a `[remote]` download hook is configured in `<model_root>/.syscribe.toml`
//! **and** the user opts in with `validate --fetch-remote`, the tool fetches each
//! remote source into a local cache and then applies the normal `W004`/`W009`
//! checks to the downloaded copy.
//!
//! Execution is opt-in for safety: defining a hook never runs it; only the
//! explicit `--fetch-remote` flag does. The hook is a POSIX `sh` command with
//! `{url}` and `{dest}` placeholders (substituted as shell-quoted values) and the
//! `SYSCRIBE_URL` / `SYSCRIBE_DEST` environment variables.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// A configured remote-download hook.
#[derive(Debug, Clone)]
pub struct RemoteHook {
    /// Command template run via `sh -c`; `{url}`/`{dest}` are substituted.
    pub command: String,
    /// Directory where downloaded files are cached.
    pub cache_dir: PathBuf,
}

#[derive(Debug, Default, Deserialize)]
struct SyscribeTomlRemote {
    #[serde(default)]
    remote: Option<RemoteSection>,
}

#[derive(Debug, Deserialize)]
struct RemoteSection {
    download: Option<String>,
    #[serde(default, alias = "cacheDir")]
    cache_dir: Option<String>,
}

impl RemoteHook {
    /// Load the `[remote]` hook from `<model_root>/.syscribe.toml`, if a
    /// `download` command is configured. Returns `None` when absent or malformed.
    pub fn load(model_root: &Path) -> Option<Self> {
        let text = std::fs::read_to_string(model_root.join(".syscribe.toml")).ok()?;
        let parsed: SyscribeTomlRemote = toml::from_str(&text).ok()?;
        let section = parsed.remote?;
        let command = section.download?;
        let cache_dir = match section.cache_dir {
            Some(c) => {
                let p = PathBuf::from(&c);
                if p.is_absolute() { p } else { model_root.join(p) }
            }
            None => model_root.join(".syscribe").join("cache"),
        };
        Some(RemoteHook { command, cache_dir })
    }

    /// Cache path for `url`, preserving the URL's file extension so the language
    /// matcher still resolves by extension.
    fn cache_path(&self, url: &str) -> PathBuf {
        let mut h = DefaultHasher::new();
        url.hash(&mut h);
        let id = format!("{:016x}", h.finish());
        match url_extension(url) {
            Some(ext) => self.cache_dir.join(format!("{id}.{ext}")),
            None => self.cache_dir.join(id),
        }
    }

    /// Fetch `url` into the cache and return the local path. A cache hit skips the
    /// hook entirely. Returns `None` if the hook fails or produces no file.
    pub fn fetch(&self, url: &str) -> Option<PathBuf> {
        let dest = self.cache_path(url);
        if dest.exists() {
            return Some(dest);
        }
        std::fs::create_dir_all(&self.cache_dir).ok()?;
        let cmd = self
            .command
            .replace("{url}", &sh_quote(url))
            .replace("{dest}", &sh_quote(&dest.to_string_lossy()));
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .env("SYSCRIBE_URL", url)
            .env("SYSCRIBE_DEST", &dest)
            .status()
            .ok()?;
        if status.success() && dest.exists() {
            Some(dest)
        } else {
            let _ = std::fs::remove_file(&dest); // drop any partial output
            None
        }
    }
}

/// Single-quote a value for safe inclusion in a `sh -c` command.
fn sh_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// Extract the file extension from a URL (ignoring `?query`/`#fragment`).
fn url_extension(url: &str) -> Option<String> {
    let path = url.split(['?', '#']).next().unwrap_or(url);
    let last = path.rsplit('/').next().unwrap_or(path);
    last.rsplit_once('.').and_then(|(_, ext)| {
        if ext.is_empty() || ext.contains(':') {
            None
        } else {
            Some(ext.to_ascii_lowercase())
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_from_url() {
        assert_eq!(url_extension("https://x/y/lib.rs"), Some("rs".to_string()));
        assert_eq!(url_extension("https://x/y/lib.rs?ref=main"), Some("rs".to_string()));
        assert_eq!(url_extension("https://x/y/Test.java#L10"), Some("java".to_string()));
        assert_eq!(url_extension("https://x/y/noext"), None);
    }

    #[test]
    fn shell_quoting_is_safe() {
        assert_eq!(sh_quote("a b"), "'a b'");
        assert_eq!(sh_quote("it's"), "'it'\\''s'");
    }

    #[test]
    fn fetch_runs_hook_and_caches() {
        let dir = std::env::temp_dir().join(format!("syscribe-remote-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let hook = RemoteHook {
            command: "printf 'fn remote_present(){}' > {dest}".to_string(),
            cache_dir: dir.join("cache"),
        };
        let p = hook.fetch("mock://ok/remote.rs").expect("hook should produce a file");
        assert!(p.exists());
        assert_eq!(p.extension().unwrap(), "rs");
        let body = std::fs::read_to_string(&p).unwrap();
        assert!(body.contains("fn remote_present"));

        // A failing hook yields None and leaves no file.
        let bad = RemoteHook {
            command: "exit 1".to_string(),
            cache_dir: dir.join("cache"),
        };
        assert_eq!(bad.fetch("mock://fail/x.rs"), None);
    }
}
