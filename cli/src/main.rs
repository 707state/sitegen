use anyhow::Result;
use std::{env, path::PathBuf};

use crate::markdown_meta::Index;

mod markdown_meta;

fn help() {
    println!("Usage: convert markdown to json in specified paths.")
}
fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => {
            help();
            Ok(())
        }
        _ => {
            let paths: Vec<PathBuf> = args.iter().skip(1).map(PathBuf::from).collect();
            let index: Index = paths.try_into()?;
            Ok(())
        }
    }
}
