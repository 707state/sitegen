use anyhow::{Context, Result};
use comrak::{Arena, Options, nodes::NodeValue};
use core::fmt;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct FrontMatter {
    pub title: String,
    pub author: String,
    pub tags: Vec<String>,
    pub date: Option<String>,
}
#[derive(Debug, Serialize)]
pub struct Markdown {
    // file meta info
    path: PathBuf,
    modified_at_unix: Option<u64>,
    metadata: FrontMatter,
    // content, think when dumping json, content should be a HTML string
    content: String,
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

pub fn is_markdown(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown"))
        .unwrap_or(false)
}
impl TryFrom<PathBuf> for Markdown {
    type Error = anyhow::Error;
    fn try_from(path: PathBuf) -> Result<Self> {
        if !path.exists() {
            anyhow::bail!("path does not exist: {}", path.display());
        }
        if !path.is_file() {
            anyhow::bail!("not a file: {}", path.display());
        }
        if !is_markdown(&path) {
            anyhow::bail!("not a markdown file: {}", path.display());
        }
        // 2) 文件元信息
        let md =
            fs::metadata(&path).with_context(|| format!("metadata failed: {}", path.display()))?;
        let modified_at_unix = md
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok().map(|d| d.as_secs()));

        // 3) 读文件内容
        let input = fs::read_to_string(&path)
            .with_context(|| format!("failed to read: {}", path.display()))?;
        // comrak options
        let mut options = Options::default();
        options.extension.front_matter_delimiter = Some("---".to_owned());
        let arena = Arena::new();
        let root = comrak::parse_document(&arena, &input, &options);
        let mut front_matter_string = extract_front_matter_from_ast(root)
            .with_context(|| format!("missing front matter in: {}", path.display()))?;
        front_matter_string = front_matter_string
            .trim()
            .trim_start_matches("---")
            .trim()
            .trim_end_matches("---")
            .trim()
            .to_string();
        // serde_yaml解析front matter
        let metadata: FrontMatter =
            serde_yaml::from_str(&front_matter_string).with_context(|| {
                format!(
                    "Invalid YAML front matter in: {}\nInput YAML string is: {}",
                    path.display(),
                    front_matter_string
                )
            })?;
        Ok(Self {
            path,
            modified_at_unix,
            metadata,
            content: comrak::markdown_to_html(&input, &options),
        })
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
            "Process file: {} generated an error: {}",
            self.path.display(),
            self.message
        )
    }
}

fn extract_front_matter_from_ast<'a>(root: &'a comrak::nodes::AstNode<'a>) -> Option<String> {
    for child in root.children() {
        let data = child.data.borrow();
        if let NodeValue::FrontMatter(ref s) = data.value {
            return Some(s.clone());
        }
    }
    None
}
