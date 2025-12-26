use anyhow::{Context, Result};
use core::fmt;
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

#[derive(Debug, Serialize)]
struct Markdown {
    // file meta info
    path: PathBuf,
    modified_at_unix: Option<u64>,
    // content meta info
    title: Option<String>,
    word_count_estimate: usize,
    label: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ProcessingError {
    path: PathBuf,
    message: String,
}

#[derive(Debug, Serialize)]
pub struct Index {
    generated_at_unix: u64,
    items: Vec<Markdown>,
}

impl Markdown {
    pub fn new() -> Self {
        Markdown {
            path: PathBuf::new(),
            modified_at_unix: None,
            title: None,
            word_count_estimate: 0,
            label: Vec::new(),
        }
    }
}
fn is_markdown(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown"))
        .unwrap_or(false)
}
impl TryFrom<PathBuf> for Markdown {
    type Error = anyhow::Error;
    fn try_from(path: PathBuf) -> Result<Self> {
        if !path.exists() {
            anyhow::bail!("Path: {} does not exist!", path.display());
        }
        if !path.is_file() {
            anyhow::bail!("not a file: {}", path.display());
        }
        if !is_markdown(&path) {
            anyhow::bail!("file: {} is not a markdown file!", path.display());
        }
        let metadata =
            fs::metadata(&path).with_context(|| format!("metadata failed: {}", path.display()))?;
        let modified_at = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok().map(|d| d.as_secs()));
        let content = fs::read_to_string(&path)
            .with_context(|| format!("read failed: {}", path.display()))?;
    }
}
impl ProcessingError {
    pub fn new() -> Self {
        ProcessingError {
            path: PathBuf::new(),
            message: String::new(),
        }
    }
}
impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Process file: {} generated an error: {}\n",
            self.path.display(),
            self.message
        )
    }
}
