use std::collections::VecDeque;
use std::fs::{self, DirEntry, ReadDir};
use std::path::{Path, PathBuf};

struct FileWalker {
    /// `stack` stores file scanning iterator for each layer, and `stack.last()` always refers to
    /// current directory iterator.
    stack: Vec<ReadDir>,
}

impl FileWalker {
    fn open(root: &str) -> anyhow::Result<Self> {
        let path = Path::new(root);
        let iter = path.read_dir()?;

        let stack = vec![iter];
        Ok(FileWalker { stack })
    }
}

impl Iterator for FileWalker {
    type Item = PathBuf;

    /// Get next file in the directory and sub-directories.
    /// Any error will occur a panic.
    fn next(&mut self) -> Option<Self::Item> {
        // Jump to current directory
        while let Some(dir) = self.stack.last_mut() {
            // If current layer's iterator is available, read next file
            if let Some(dir) = dir.next() {
                let entry = dir.unwrap();
                let path = entry.path();

                if path.is_file() {
                    return Some(path);
                } else {
                    let new_subdirectory = fs::read_dir(path).unwrap();
                    self.stack.push(new_subdirectory);
                }
            } else {
                // Return to up level and try again
                self.stack.pop();
            }
        }
        None
    }
}

