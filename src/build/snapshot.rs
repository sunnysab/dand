use std::collections::{HashMap, HashSet};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Categories of input files relevant to build invalidation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileKind {
    /// `dand.toml` configuration file.
    Config,
    /// `template/**/*.html` (non-asset HTML template).
    Template,
    /// `template/asset/**` file.
    TemplateAsset,
    /// Source markdown post.
    Post,
    /// Root-level static file in the source directory (non-md, non-config).
    RootStatic,
    /// File inside a post's companion resource directory.
    PostResource {
        /// Absolute path to the companion `.md` file.
        post_source: PathBuf,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotFile {
    pub path: PathBuf,
    /// Milliseconds since UNIX_EPOCH.
    pub mtime: u64,
    pub kind: FileKind,
}

pub struct CurrentSnapshot {
    pub files: Vec<SnapshotFile>,
}

impl CurrentSnapshot {
    /// Build a path -> mtime map for quick lookup.
    pub fn as_mtime_map(&self) -> HashMap<PathBuf, u64> {
        self.files.iter().map(|f| (f.path.clone(), f.mtime)).collect()
    }
}

pub fn mtime_millis(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
}

fn file_mtime(path: &Path) -> Result<u64> {
    let meta = std::fs::metadata(path)
        .with_context(|| format!("failed to stat: {}", path.display()))?;
    let modified = meta
        .modified()
        .with_context(|| format!("mtime unavailable: {}", path.display()))?;
    Ok(mtime_millis(modified))
}

fn walk_dir_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)
        .with_context(|| format!("failed to read dir: {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_dir_files(&path, out)?;
        } else if path.is_file() {
            out.push(path);
        }
    }
    Ok(())
}

/// Scan the source and template trees to produce a current input snapshot.
///
/// - `source_dir`: directory containing posts, `dand.toml`, and root static files.
/// - `template_dir`: directory containing template HTML files and their assets.
/// - `config_path`: path to `dand.toml`.
pub fn scan_source_tree(
    source_dir: &Path,
    template_dir: &Path,
    config_path: &Path,
) -> Result<CurrentSnapshot> {
    let mut files = Vec::new();

    // 1. Config
    if config_path.is_file() {
        files.push(SnapshotFile {
            mtime: file_mtime(config_path)?,
            path: config_path.to_path_buf(),
            kind: FileKind::Config,
        });
    }

    // 2. Template files and assets
    let template_asset_dir = template_dir.join("asset");
    let mut template_all = Vec::new();
    walk_dir_files(template_dir, &mut template_all)?;

    for path in template_all {
        let kind = if path.starts_with(&template_asset_dir) {
            FileKind::TemplateAsset
        } else if path.extension() == Some("html".as_ref()) {
            FileKind::Template
        } else {
            continue;
        };
        files.push(SnapshotFile {
            mtime: file_mtime(&path)?,
            path,
            kind,
        });
    }

    // 3. Source dir: collect markdown posts (and their companion resource files)
    let mut md_files = Vec::new();
    if source_dir.is_dir() {
        collect_md_files(source_dir, &mut md_files)?;
    }

    for md_path in &md_files {
        files.push(SnapshotFile {
            mtime: file_mtime(md_path)?,
            path: md_path.clone(),
            kind: FileKind::Post,
        });

        // Companion resource directory: same name as the .md stem, adjacent to the .md file.
        if let (Some(parent), Some(stem)) = (md_path.parent(), md_path.file_stem()) {
            let resource_dir = parent.join(stem);
            if resource_dir.is_dir() {
                let mut resource_files = Vec::new();
                walk_dir_files(&resource_dir, &mut resource_files)?;
                for res_path in resource_files {
                    files.push(SnapshotFile {
                        mtime: file_mtime(&res_path)?,
                        path: res_path,
                        kind: FileKind::PostResource {
                            post_source: md_path.clone(),
                        },
                    });
                }
            }
        }
    }

    // 4. Root static files (files directly in source_dir, excluding .md and dand.toml)
    if source_dir.is_dir() {
        for entry in std::fs::read_dir(source_dir)
            .with_context(|| format!("failed to read source dir: {}", source_dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if ext == "md" || name == "dand.toml" {
                continue;
            }
            files.push(SnapshotFile {
                mtime: file_mtime(&path)?,
                path,
                kind: FileKind::RootStatic,
            });
        }
    }

    Ok(CurrentSnapshot { files })
}

/// Recursively collect all `.md` files from `dir`, skipping subdirectories that are companion
/// resource directories (i.e., a sibling `.md` file with the same name exists).
fn collect_md_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    let mut md_stems: HashSet<OsString> = HashSet::new();
    let mut subdirs: Vec<PathBuf> = Vec::new();
    let mut md_paths: Vec<PathBuf> = Vec::new();

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension() == Some("md".as_ref()) {
            if let Some(stem) = path.file_stem() {
                md_stems.insert(stem.to_os_string());
            }
            md_paths.push(path);
        } else if path.is_dir() {
            subdirs.push(path);
        }
    }

    out.extend(md_paths);

    for subdir in subdirs {
        let dir_name = subdir
            .file_name()
            .map(|n| n.to_os_string())
            .unwrap_or_default();
        // Skip companion resource directories.
        if md_stems.contains(&dir_name) {
            continue;
        }
        collect_md_files(&subdir, out)?;
    }

    Ok(())
}
