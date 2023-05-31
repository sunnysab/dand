use std::collections::VecDeque;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};
use anyhow::{anyhow, Result};

fn walk(root: &Path, cb: fn(&DirEntry)) -> Result<()> {
    let root = Path::new(root);
    if !root.is_dir() {
        return Err(anyhow!("The root directory you passed is not an actual directory."));
    }

    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            walk(&path, cb)?;
        } else if path.is_file() {
            cb(&entry);
        } else {
            /* symbol link is ignored. */
        }
    }
    Ok(())
}

fn is_markdown_file(path: &Path) -> bool {
    path.ends_with(".md")
}

fn walk_markdown_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut file_list = Vec::new();

    walk(root, |&entry: &DirEntry| {
        let path = entry.path();
        if is_markdown_file(&path) {
            file_list.push(path);
        }
    })?;
    Ok(file_list)
}