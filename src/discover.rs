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
    type Item = anyhow::Result<PathBuf>;

    /// Get next file in the directory and sub-directories.
    /// Any error will make a None returned.
    fn next(&mut self) -> Option<Self::Item> {
        // Jump to current directory
        while let Some(lowest_dir_iter) = self.stack.last_mut() {
            // If current layer's iterator is available, read next file
            if let Some(next_entry) = lowest_dir_iter.next() {

                // If current entrance is not available, consider that there are still some files to
                // iterate and `lowest_dir_iter` has been moved, return error and wait for a next call.
                if let Err(e) = next_entry {
                    return Some(Err(e.into()));
                }
                let path = next_entry.unwrap().path();
                if path.is_file() {
                    // Once we found a file
                    return Some(Ok(path));
                } else {
                    // Once we found a new directory, push its iterator back to the stack
                    match fs::read_dir(path) {
                        Ok(new_subdirectory) => {
                            self.stack.push(new_subdirectory);
                        },
                        Err(e) => {
                            // Maybe for permission denied the sub-directory can not be accessed,
                            // just return that error and ignore, wait for a next call
                            return Some(Err(e.into()));
                        }
                    }
                }
            } else {
                // Return to up level and try again
                self.stack.pop();
            }
        }
        None
    }
}

