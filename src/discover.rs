use std::fs::{self, ReadDir};
use std::path::{Path, PathBuf};
use chrono::{DateTime, Local};

/// Iterator over the entries in a directory and its sub-directories.
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
                        }
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

/// Reference to a markdown file
#[derive(Debug)]
pub struct PostReference {
    /// Relative name is a pure file title with the column it belongs to.
    relative_name: String,
    /// Full file path
    full_path: PathBuf,

    /// Creation time
    created: DateTime<Local>,
    /// Last modified time
    modified: DateTime<Local>,
}

/// Parse file path, detect section name from path.
///
/// in **column**:
///   `Column/index.md` -> `Column`
///   `Column/1.md` -> `Column/1`
///
/// for **individual post**:
///   `Post/Post.md` -> `Post`
///   `Post.md` -> `Post`
///
/// In this program, we assume that `full_path` is always starting with prefix, because we just
/// iterate root path (`prefix` here) and then get files which path named `full_path` here.
fn convert_path(prefix: &Path, full_path: &Path) -> String {
    // relative path
    let rp = match full_path.strip_prefix(prefix) {
        Ok(p) => p,
        Err(_) => full_path,
    }.display().to_string();

    let rp = rp.trim_end_matches(".md");
    let rp = rp.trim_end_matches("/index"); // TODO: Compatibility on Windows.

    // Do conversion that `Post/Post.md` -> `Post`
    let rp = if let Some(last_slash) = rp.rfind("/") {
        let article = &rp[(last_slash + 1)..];
        // In case that rp is like `Post/Post`, do trim.
        let postfix = format!("/{}/{}", article, article);
        if rp.ends_with(&postfix) || rp == &postfix[1..] {
            &rp[..last_slash]
        } else {
            // If rp is like `P1/1`, do nothing
            rp
        }
    } else {
        rp
    };

    rp.to_string()
}

/// Open the metadata of a markdown file, returns a `PostReference` structure.
fn open(prefix: &Path, path: PathBuf) -> anyhow::Result<PostReference> {
    let metadata = path.metadata()?;
    let created = DateTime::<Local>::from(metadata.created()?);
    let modified = DateTime::<Local>::from(metadata.modified()?);

    let relative_name = convert_path(prefix, &path);
    let result = PostReference {
        relative_name,
        full_path: path,
        created,
        modified,
    };
    Ok(result)
}

/// Read markdown files in `root` directory and its sub-directories and return a `PostReference` list.
pub fn walk_markdown_files(root: &str) -> anyhow::Result<Vec<PostReference>> {
    FileWalker::open(root)?
        .filter_map(|f| match f {
            // Filter the entrance which could not be accessed.
            Ok(f) => Some(f),
            Err(e) => {
                eprintln!("Could not access entry: {e}");
                None
            }
        })
        .filter(|f| {
            // f.ends_with() returns true only if the whole path components matches, use extension() instead.
            f.extension() == Some("md".as_ref())
        } )
        .map(|f| {
            // Read metadata and generate a PostReference.
            open(root.as_ref(), f)
        })
        .collect()
}