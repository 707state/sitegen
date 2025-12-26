use anyhow::Result;
use std::{
    env,
    path::{Path, PathBuf},
};

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

            Ok(())
        }
    }
}
