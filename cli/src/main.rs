use anyhow::{Context, Result};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

use crate::markdown_meta::is_markdown;

mod markdown_meta;

fn help() {
    println!("Usage: convert markdown to json in specified paths.")
}
fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => {
            help();
            return Ok(());
        }
        _ => {
            let paths: Vec<PathBuf> = args.iter().skip(1).map(PathBuf::from).collect();
            let dist_dir = PathBuf::from("dist");
            fs::create_dir_all(&dist_dir).context("failed to create dist/")?;
            for path in paths {
                if !path.exists() {
                    eprintln!("Skip: {} (not exists)", path.display());
                    continue;
                }
                if path.is_file() {
                    if is_markdown(&path) {
                        process_one_md(&path, path.parent().unwrap_or(Path::new(".")), &dist_dir)?;
                    }
                    continue;
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
                    if let Err(e) = process_one_md(md_path, &path, &dist_dir) {
                        eprintln!("Process failed: {}: {e:#}", md_path.display());
                    }
                }
            }
            Ok(())
        }
    }
}

fn process_one_md(md_path: &Path, base: &Path, dist_dir: &Path) -> Result<()> {
    let md: markdown_meta::Markdown = md_path
        .to_path_buf()
        .try_into()
        .with_context(|| format!("convert markdown failed: {}", md_path.display()))?;
    let rel = md_path.strip_prefix(base).unwrap_or(md_path);
    let mut out_path = dist_dir.join(rel);
    out_path.set_extension("json");
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create parent dir failed: {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(&md).context("serde_json serialize failed")?;
    fs::write(&out_path, json).with_context(|| format!("write failed: {}", out_path.display()))?;
    eprintln!("Wrote {}", out_path.display());
    Ok(())
}
