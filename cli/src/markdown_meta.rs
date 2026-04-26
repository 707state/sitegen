use anyhow::{Context, Result};
use chrono::NaiveDate;
use comrak::{
    Arena, Options, format_html_with_plugins, nodes::NodeValue, options::Plugins,
    plugins::syntect::SyntectAdapter,
};
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
    #[serde(default)]
    pub series: Option<String>,
    pub date: NaiveDate,
}
#[derive(Debug, Serialize)]
pub struct Markdown {
    // file meta info
    path: PathBuf,
    modified_at_unix: Option<u64>,
    metadata: FrontMatter,
    headings: Vec<PostHeading>,
    // content, think when dumping json, content should be a HTML string
    content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PostHeading {
    level: u8,
    text: String,
    id: String,
}

#[derive(Debug, Serialize)]
pub struct Index {
    paragraph_under_certain_topic: HashMap<String, Vec<String>>,
    paragraph_under_certain_series: HashMap<String, Vec<String>>,
    table_of_content: Vec<TableOfContentItem>,
}

#[derive(Debug, Serialize)]
pub struct TableOfContentItem {
    title: String,
    path: String,
    date: NaiveDate,
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
        options.extension.table = true;
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
        let headings = extract_headings(root);
        Ok(Self {
            path,
            modified_at_unix,
            metadata,
            headings: headings.clone(),
            content: {
                let adapter = SyntectAdapter::new(Some("InspiredGitHub"));
                let mut plugins = Plugins::default();
                plugins.render.codefence_syntax_highlighter = Some(&adapter);
                let mut html_output = String::new();
                format_html_with_plugins(root, &options, &mut html_output, &plugins)
                    .context("failed to render markdown to HTML")?;
                let html = inject_heading_ids(html_output, &headings);
                // Make all links open in a new tab
                html.replace(
                    "<a href=",
                    "<a target=\"_blank\" rel=\"noopener noreferrer\" href=",
                )
            },
        })
    }
}

struct BuiltMarkdown {
    markdown: Markdown,
    out_path: PathBuf,
}

#[derive(Debug)]
struct SeriesPostEntry {
    title: String,
    date: NaiveDate,
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
        let mut series_posts: HashMap<String, Vec<SeriesPostEntry>> = HashMap::new();
        let mut table_of_content: Vec<TableOfContentItem> = Vec::new();
        for path in paths {
            if !path.exists() {
                eprintln!("Skip: {} (not exists)", path.display());
                continue;
            }
            if path.is_file() && is_markdown(&path) {
                let built_md = build_markdown_and_write_json(&path, &dist_dir)?;
                append_built_markdown(
                    built_md,
                    &dist_dir,
                    &mut paragraph_under_certain_topic,
                    &mut series_posts,
                    &mut table_of_content,
                );
                continue;
            }
            for entry in walkdir::WalkDir::new(&path).follow_links(true) {
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
                append_built_markdown(
                    built_md,
                    &dist_dir,
                    &mut paragraph_under_certain_topic,
                    &mut series_posts,
                    &mut table_of_content,
                );
            }
        }
        table_of_content.sort_by(|a, b| b.date.cmp(&a.date));
        let paragraph_under_certain_series = finalize_series_map(series_posts);
        let index = Self {
            table_of_content,
            paragraph_under_certain_topic,
            paragraph_under_certain_series,
        };
        let index_path = dist_dir.join("index.json");
        let index_json = serde_json::to_string_pretty(&index).context("serialize index failed")?;
        fs::write(&index_path, index_json)
            .with_context(|| format!("write to {} failed", index_path.display()))?;
        Ok(index)
    }
}

fn append_built_markdown(
    built_md: BuiltMarkdown,
    dist_dir: &Path,
    paragraph_under_certain_topic: &mut HashMap<String, Vec<String>>,
    series_posts: &mut HashMap<String, Vec<SeriesPostEntry>>,
    table_of_content: &mut Vec<TableOfContentItem>,
) {
    let title = built_md.markdown.metadata.title.clone();
    let date = built_md.markdown.metadata.date;
    let rel_path = relative_json_path(&built_md.out_path, dist_dir);

    for tag in &built_md.markdown.metadata.tags {
        paragraph_under_certain_topic
            .entry(tag.clone())
            .or_default()
            .push(title.clone());
    }

    if let Some(series) = built_md.markdown.metadata.series.clone() {
        series_posts
            .entry(series)
            .or_default()
            .push(SeriesPostEntry {
                title: title.clone(),
                date,
            });
    }

    table_of_content.push(TableOfContentItem {
        title,
        path: rel_path,
        date,
    });
}

fn finalize_series_map(
    mut series_posts: HashMap<String, Vec<SeriesPostEntry>>,
) -> HashMap<String, Vec<String>> {
    series_posts
        .drain()
        .map(|(series, mut posts)| {
            posts.sort_by(|a, b| a.date.cmp(&b.date).then_with(|| a.title.cmp(&b.title)));
            let titles = posts.into_iter().map(|post| post.title).collect();
            (series, titles)
        })
        .collect()
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

fn extract_headings<'a>(root: &'a comrak::nodes::AstNode<'a>) -> Vec<PostHeading> {
    let mut headings = Vec::new();
    let mut seen_ids = HashMap::<String, usize>::new();
    collect_headings(root, &mut headings, &mut seen_ids);
    headings
}

fn collect_headings<'a>(
    node: &'a comrak::nodes::AstNode<'a>,
    headings: &mut Vec<PostHeading>,
    seen_ids: &mut HashMap<String, usize>,
) {
    {
        let data = node.data.borrow();
        if let NodeValue::Heading(ref heading) = data.value {
            let text = collect_text(node).trim().to_string();
            if !text.is_empty() {
                let base_id = slugify(&text);
                let seen = seen_ids.entry(base_id.clone()).or_insert(0);
                let id = if *seen == 0 {
                    base_id
                } else {
                    format!("{base_id}-{}", *seen + 1)
                };
                *seen += 1;
                headings.push(PostHeading {
                    level: heading.level,
                    text,
                    id,
                });
            }
        }
    }

    for child in node.children() {
        collect_headings(child, headings, seen_ids);
    }
}

fn collect_text<'a>(node: &'a comrak::nodes::AstNode<'a>) -> String {
    let data = node.data.borrow();
    match &data.value {
        NodeValue::Text(text) => text.to_string(),
        NodeValue::Code(code) => code.literal.clone(),
        NodeValue::LineBreak | NodeValue::SoftBreak => " ".to_string(),
        _ => {
            drop(data);
            let mut text = String::new();
            for child in node.children() {
                text.push_str(&collect_text(child));
            }
            text
        }
    }
}

fn slugify(input: &str) -> String {
    let mut slug = String::new();
    let mut previous_was_dash = false;

    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            previous_was_dash = false;
        } else if ch.is_alphanumeric() {
            slug.push(ch);
            previous_was_dash = false;
        } else if (ch.is_whitespace() || matches!(ch, '-' | '_'))
            && !previous_was_dash
            && !slug.is_empty()
        {
            slug.push('-');
            previous_was_dash = true;
        }
    }

    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        "section".to_string()
    } else {
        slug
    }
}

fn inject_heading_ids(mut html: String, headings: &[PostHeading]) -> String {
    let mut search_from = 0;

    for heading in headings {
        let tag = format!("<h{}>", heading.level);
        if let Some(relative_pos) = html[search_from..].find(&tag) {
            let start = search_from + relative_pos;
            let replacement = format!("<h{} id=\"{}\">", heading.level, heading.id);
            html.replace_range(start..start + tag.len(), &replacement);
            search_from = start + replacement.len();
        }
    }

    html
}
