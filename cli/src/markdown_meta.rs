use anyhow::{Context, Result};
use chrono::NaiveDate;
use comrak::{Arena, Options, nodes::NodeValue};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct FrontMatter {
    pub title: String,
    pub author: String,
    pub tags: Vec<String>,
    pub date: Option<NaiveDate>,
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
pub struct Index {
    paragraph_under_certain_topic: HashMap<String, Vec<String>>,
    table_of_content: Vec<TableOfContentItem>,
    #[serde(skip_serializing)]
    markdowns: Vec<Markdown>,
}

#[derive(Debug, Serialize)]
pub struct TableOfContentItem {
    title: String,
    path: String,
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

struct BuiltMarkdown {
    markdown: Markdown,
    out_path: PathBuf,
}

fn build_markdown_and_write_json(path: &Path, dist_dir: &Path) -> anyhow::Result<BuiltMarkdown> {
    // 1) 转成 Markdown
    let one_md: Markdown = path
        .to_path_buf()
        .try_into()
        .with_context(|| format!("convert markdown failed: {}", path.display()))?;

    // 2) 计算输出路径
    let rel = path
        .strip_prefix(std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        .unwrap_or(path);

    let mut out_path = dist_dir.join(rel);
    out_path.set_extension("json");

    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create parent dir failed: {}", parent.display()))?;
    }

    // 3) 写 json
    let json = serde_json::to_string_pretty(&one_md).context("serde_json serialize failed")?;
    fs::write(&out_path, json)
        .with_context(|| format!("write to {} failed", out_path.display()))?;

    Ok(BuiltMarkdown {
        markdown: one_md,
        out_path,
    })
}
impl TryFrom<Vec<PathBuf>> for Index {
    type Error = anyhow::Error;
    fn try_from(paths: Vec<PathBuf>) -> anyhow::Result<Self> {
        let dist_dir = PathBuf::from("dist");
        fs::create_dir_all(&dist_dir).context("failed to create dist/")?;
        let mut paragraph_under_certain_topic: HashMap<String, Vec<String>> = HashMap::new();
        let mut markdowns: Vec<Markdown> = Vec::new();
        let mut table_of_content: Vec<TableOfContentItem> = Vec::new();
        for path in paths {
            if !path.exists() {
                eprintln!("Skip: {} (not exists)", path.display());
                continue;
            }
            if path.is_file() && is_markdown(&path) {
                let built_md = build_markdown_and_write_json(&path, &dist_dir)?;
                let title = built_md.markdown.metadata.title.clone();
                let rel_path = relative_json_path(&built_md.out_path, &dist_dir);
                for tag in &built_md.markdown.metadata.tags {
                    paragraph_under_certain_topic
                        .entry(tag.clone())
                        .or_default()
                        .push(title.clone());
                }
                table_of_content.push(TableOfContentItem {
                    title,
                    path: rel_path,
                });
                markdowns.push(built_md.markdown);
            }
            for entry in walkdir::WalkDir::new(&path).follow_links(false) {
                let entry = match entry {
                    Ok(e) => e,
                    Err(e) => {
                        eprintln!("Walk error under {}: {e}", path.display());
                        continue;
                    }
                };
                if !entry.file_type().is_file() {
                    continue;
                }
                let md_path = entry.path();
                if !is_markdown(md_path) {
                    continue;
                }
                let built_md = build_markdown_and_write_json(md_path, &dist_dir)?;
                let title = built_md.markdown.metadata.title.clone();
                let rel_path = relative_json_path(&built_md.out_path, &dist_dir);
                for tag in &built_md.markdown.metadata.tags {
                    paragraph_under_certain_topic
                        .entry(tag.clone())
                        .or_default()
                        .push(title.clone());
                }
                table_of_content.push(TableOfContentItem {
                    title,
                    path: rel_path,
                });
                markdowns.push(built_md.markdown);
            }
        }
        let index = Self {
            table_of_content,
            paragraph_under_certain_topic,
            markdowns,
        };
        let index_path = dist_dir.join("index.json");
        let index_json = serde_json::to_string_pretty(&index).context("serialize index failed")?;
        fs::write(&index_path, index_json)
            .with_context(|| format!("write to {} failed", index_path.display()))?;
        Ok(index)
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

fn relative_json_path(path: &Path, dist_dir: &Path) -> String {
    let rel = path.strip_prefix(dist_dir).unwrap_or(path);
    rel.to_string_lossy().into_owned()
}
