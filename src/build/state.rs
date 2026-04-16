use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub const CACHE_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildState {
    pub cache_format_version: u32,
    pub generator_version: String,
    /// Maps absolute input file path -> mtime in milliseconds since UNIX_EPOCH.
    pub input_files: HashMap<PathBuf, u64>,
    /// Maps source path string -> cached post metadata, keyed by source path as string.
    pub cached_posts: HashMap<String, CachedPost>,
    pub generated_outputs: GeneratedOutputs,
}

impl Default for BuildState {
    fn default() -> Self {
        Self {
            cache_format_version: CACHE_FORMAT_VERSION,
            generator_version: env!("CARGO_PKG_VERSION").to_string(),
            input_files: HashMap::new(),
            cached_posts: HashMap::new(),
            generated_outputs: GeneratedOutputs::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedPost {
    pub source_path: PathBuf,
    /// Relative output name, e.g. "my-post" -> output at `<output>/my-post/index.html`.
    pub relative_name: String,
    /// Tag slugs associated with this post.
    pub tags: Vec<String>,
    /// Category slugs associated with this post.
    pub categories: Vec<String>,
    pub hidden: bool,
    pub draft: bool,
    /// Whether a companion resource directory existed alongside this post.
    pub has_resource_dir: bool,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct GeneratedOutputs {
    /// Relative names of rendered post output directories.
    pub post_relative_names: Vec<String>,
    /// Tag slugs that were rendered.
    pub rendered_tags: Vec<String>,
    /// Category slugs that were rendered.
    pub rendered_categories: Vec<String>,
    /// File names of copied root static files.
    pub root_static_files: Vec<String>,
}

/// Load cached state from `OUTPUT/.dand-cache/state.json`.
/// Returns `None` if the file is absent, corrupt, or has an incompatible version.
pub fn read_state(output_dir: &Path) -> Option<BuildState> {
    let state_path = output_dir.join(".dand-cache").join("state.json");
    let content = std::fs::read_to_string(&state_path).ok()?;
    let state: BuildState = serde_json::from_str(&content).ok()?;
    if state.cache_format_version != CACHE_FORMAT_VERSION {
        return None;
    }
    Some(state)
}

/// Write state atomically: write to `.tmp`, then rename to final path.
pub fn write_state_atomic(output_dir: &Path, state: &BuildState) -> Result<()> {
    let cache_dir = output_dir.join(".dand-cache");
    std::fs::create_dir_all(&cache_dir)
        .with_context(|| format!("failed to create cache dir: {}", cache_dir.display()))?;

    let state_path = cache_dir.join("state.json");
    let tmp_path = cache_dir.join("state.json.tmp");

    let json = serde_json::to_string_pretty(state).context("failed to serialize build state")?;
    std::fs::write(&tmp_path, &json)
        .with_context(|| format!("failed to write tmp state: {}", tmp_path.display()))?;
    std::fs::rename(&tmp_path, &state_path)
        .with_context(|| format!("failed to rename state file: {}", state_path.display()))?;
    Ok(())
}
