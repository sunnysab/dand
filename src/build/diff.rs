use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::build::snapshot::{CurrentSnapshot, FileKind, SnapshotFile};
use crate::build::state::{BuildState, CachedPost};
use crate::env::Environment;
use crate::post::SharedPost;

/// Low-level file diff between the old persisted state and the current snapshot.
pub struct BuildDiff {
    /// Files present now but absent in the old state.
    pub added: Vec<SnapshotFile>,
    /// Files present in both states but with a newer mtime.
    pub modified: Vec<SnapshotFile>,
    /// Paths that existed in the old state but are now gone, with their inferred kind.
    pub removed: Vec<(PathBuf, RemovedKind)>,
}

#[derive(Debug, Clone)]
pub enum RemovedKind {
    Post,
    RootStatic,
    TemplateAsset,
    Template,
    Config,
    PostResource { post_source: PathBuf },
}

/// The set of output operations to execute in this build.
pub struct DirtySet {
    /// Source paths of posts whose HTML must be re-rendered.
    pub render_posts: HashSet<PathBuf>,
    /// `(relative_name, has_resource_dir)` pairs for posts whose output directories must be deleted.
    pub delete_post_outputs: Vec<(String, bool)>,
    /// Tag slugs whose pages must be re-rendered.
    pub render_tags: HashSet<String>,
    /// Tag slugs whose output directories must be deleted (tags that no longer exist).
    pub delete_tags: HashSet<String>,
    /// Category slugs whose pages must be re-rendered.
    pub render_categories: HashSet<String>,
    /// Category slugs whose output directories must be deleted.
    pub delete_categories: HashSet<String>,
    pub render_index: bool,
    pub render_tags_overview: bool,
    pub render_categories_overview: bool,
    /// Whether the template asset directory must be re-copied.
    pub copy_template_assets: bool,
    /// Source paths of root static files that must be copied.
    pub copy_root_static: HashSet<PathBuf>,
    /// File names of root static files that must be deleted from the output.
    pub delete_root_static: HashSet<String>,
    /// Source paths of posts whose companion resource directories must be re-copied.
    pub copy_post_resources: HashSet<PathBuf>,
}

impl DirtySet {
    fn new() -> Self {
        Self {
            render_posts: HashSet::new(),
            delete_post_outputs: Vec::new(),
            render_tags: HashSet::new(),
            delete_tags: HashSet::new(),
            render_categories: HashSet::new(),
            delete_categories: HashSet::new(),
            render_index: false,
            render_tags_overview: false,
            render_categories_overview: false,
            copy_template_assets: false,
            copy_root_static: HashSet::new(),
            delete_root_static: HashSet::new(),
            copy_post_resources: HashSet::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.render_posts.is_empty()
            && self.delete_post_outputs.is_empty()
            && self.render_tags.is_empty()
            && self.delete_tags.is_empty()
            && self.render_categories.is_empty()
            && self.delete_categories.is_empty()
            && !self.render_index
            && !self.render_tags_overview
            && !self.render_categories_overview
            && !self.copy_template_assets
            && self.copy_root_static.is_empty()
            && self.delete_root_static.is_empty()
            && self.copy_post_resources.is_empty()
    }
}

/// Compare the old persisted state against the current snapshot to find changed files.
pub fn diff_snapshots(old_state: &BuildState, current: &CurrentSnapshot) -> BuildDiff {
    let current_map: HashMap<&PathBuf, &SnapshotFile> =
        current.files.iter().map(|f| (&f.path, f)).collect();
    let old_map = &old_state.input_files;

    let mut added = Vec::new();
    let mut modified = Vec::new();
    let mut removed = Vec::new();

    for file in &current.files {
        match old_map.get(&file.path) {
            None => added.push(file.clone()),
            Some(&old_mtime) if old_mtime != file.mtime => modified.push(file.clone()),
            _ => {}
        }
    }

    for old_path in old_map.keys() {
        if !current_map.contains_key(old_path) {
            let kind = infer_removed_kind(old_path, &old_state.cached_posts);
            removed.push((old_path.clone(), kind));
        }
    }

    BuildDiff {
        added,
        modified,
        removed,
    }
}

fn infer_removed_kind(path: &Path, cached_posts: &HashMap<String, CachedPost>) -> RemovedKind {
    let path_str = path.to_string_lossy().to_string();

    if cached_posts.contains_key(&path_str) {
        return RemovedKind::Post;
    }
    if path.extension() == Some("md".as_ref()) {
        return RemovedKind::Post;
    }

    // Determine if this was a resource file by checking whether its grandparent directory
    // contains a .md file named after the parent directory.
    if let (Some(parent), Some(grandparent)) =
        (path.parent(), path.parent().and_then(|p| p.parent()))
    {
        if let Some(parent_name) = parent.file_name() {
            let sibling_md = grandparent.join(format!("{}.md", parent_name.to_string_lossy()));
            if sibling_md.is_file() {
                return RemovedKind::PostResource {
                    post_source: sibling_md,
                };
            }
        }
    }

    let path_str = path.to_string_lossy();
    if path_str.contains("template") {
        if path_str.contains("asset") {
            return RemovedKind::TemplateAsset;
        }
        if path.extension() == Some("html".as_ref()) {
            return RemovedKind::Template;
        }
    }
    if path.file_name().and_then(|n| n.to_str()) == Some("dand.toml") {
        return RemovedKind::Config;
    }

    RemovedKind::RootStatic
}

/// Translate a file-level diff into the set of output operations to perform.
/// Requires the already-built `Environment` so that current tag/category slugs are available.
pub fn plan_dirty_outputs(diff: &BuildDiff, old_state: &BuildState, env: &Environment) -> DirtySet {
    let mut dirty = DirtySet::new();
    let cached_posts = &old_state.cached_posts;

    for file in &diff.added {
        apply_added_file(file, &mut dirty, env, cached_posts);
    }
    for file in &diff.modified {
        apply_modified_file(file, &mut dirty, env, cached_posts);
    }
    for (path, kind) in &diff.removed {
        apply_removed_file(path, kind, &mut dirty, env, cached_posts);
    }

    // Ensure all current tags and categories for any post being rendered are also rendered.
    propagate_post_taxonomy(&mut dirty, env);

    dirty
}

fn apply_added_file(
    file: &SnapshotFile,
    dirty: &mut DirtySet,
    env: &Environment,
    cached_posts: &HashMap<String, CachedPost>,
) {
    match &file.kind {
        FileKind::Post => {
            dirty.render_posts.insert(file.path.clone());
            dirty.render_index = true;
            dirty.render_tags_overview = true;
            dirty.render_categories_overview = true;
        }
        FileKind::PostResource { post_source } => {
            dirty.copy_post_resources.insert(post_source.clone());
        }
        FileKind::RootStatic => {
            dirty.copy_root_static.insert(file.path.clone());
        }
        FileKind::TemplateAsset => {
            dirty.copy_template_assets = true;
        }
        FileKind::Template => {
            apply_template_invalidation(&file.path, dirty, env, cached_posts);
        }
        FileKind::Config => {
            mark_all_html_dirty(dirty, env);
        }
    }
}

fn apply_modified_file(
    file: &SnapshotFile,
    dirty: &mut DirtySet,
    env: &Environment,
    cached_posts: &HashMap<String, CachedPost>,
) {
    match &file.kind {
        FileKind::Config => {
            mark_all_html_dirty(dirty, env);
        }
        FileKind::Template => {
            apply_template_invalidation(&file.path, dirty, env, cached_posts);
        }
        FileKind::TemplateAsset => {
            dirty.copy_template_assets = true;
        }
        FileKind::Post => {
            dirty.render_posts.insert(file.path.clone());
            // Mark old tags/categories dirty using cached metadata.
            let path_str = file.path.to_string_lossy().to_string();
            if let Some(cached) = cached_posts.get(&path_str) {
                for tag_slug in &cached.tags {
                    dirty.render_tags.insert(tag_slug.clone());
                }
                for cat_slug in &cached.categories {
                    dirty.render_categories.insert(cat_slug.clone());
                }
            }
            // A post change can affect listing pages.
            dirty.render_index = true;
            dirty.render_tags_overview = true;
            dirty.render_categories_overview = true;
        }
        FileKind::PostResource { post_source } => {
            dirty.copy_post_resources.insert(post_source.clone());
        }
        FileKind::RootStatic => {
            dirty.copy_root_static.insert(file.path.clone());
        }
    }
}

fn apply_removed_file(
    path: &Path,
    kind: &RemovedKind,
    dirty: &mut DirtySet,
    env: &Environment,
    cached_posts: &HashMap<String, CachedPost>,
) {
    match kind {
        RemovedKind::Post => {
            let path_str = path.to_string_lossy().to_string();
            if let Some(cached) = cached_posts.get(&path_str) {
                dirty
                    .delete_post_outputs
                    .push((cached.relative_name.clone(), cached.has_resource_dir));
                for tag_slug in &cached.tags {
                    dirty.render_tags.insert(tag_slug.clone());
                }
                for cat_slug in &cached.categories {
                    dirty.render_categories.insert(cat_slug.clone());
                }
            }
            dirty.render_index = true;
            dirty.render_tags_overview = true;
            dirty.render_categories_overview = true;
        }
        RemovedKind::RootStatic => {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                dirty.delete_root_static.insert(name.to_string());
            }
        }
        RemovedKind::TemplateAsset => {
            dirty.copy_template_assets = true;
        }
        RemovedKind::Template => {
            apply_template_invalidation(path, dirty, env, cached_posts);
        }
        RemovedKind::Config => {
            mark_all_html_dirty(dirty, env);
        }
        RemovedKind::PostResource { post_source } => {
            dirty.copy_post_resources.insert(post_source.clone());
        }
    }
}

/// Add current tags and categories for every post that will be re-rendered.
fn propagate_post_taxonomy(dirty: &mut DirtySet, env: &Environment) {
    for post in env.get_posts() {
        let source_path = post.0.borrow().reference.full_path.clone();
        if !dirty.render_posts.contains(&source_path) {
            continue;
        }
        for tag_slug in post_tag_slugs(post) {
            dirty.render_tags.insert(tag_slug);
        }
        for cat_slug in post_category_slugs(post) {
            dirty.render_categories.insert(cat_slug);
        }
    }
}

fn mark_all_html_dirty(dirty: &mut DirtySet, env: &Environment) {
    dirty.render_index = true;
    dirty.render_tags_overview = true;
    dirty.render_categories_overview = true;
    for post in env.get_posts() {
        dirty
            .render_posts
            .insert(post.0.borrow().reference.full_path.clone());
    }
    for (slug, _) in env.get_tags().iter() {
        dirty.render_tags.insert(slug.clone());
    }
    for (slug, _) in env.get_categories().iter() {
        dirty.render_categories.insert(slug.clone());
    }
}

/// Apply the conservative template invalidation matrix from the spec.
fn apply_template_invalidation(
    template_path: &Path,
    dirty: &mut DirtySet,
    env: &Environment,
    _cached_posts: &HashMap<String, CachedPost>,
) {
    let path_str = template_path.to_string_lossy();

    // Any layout template change invalidates all HTML pages.
    if path_str.contains("/layout/") || path_str.contains("\\layout\\") {
        mark_all_html_dirty(dirty, env);
        return;
    }

    let name = template_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    match name {
        "post.html" => {
            for post in env.get_posts() {
                dirty
                    .render_posts
                    .insert(post.0.borrow().reference.full_path.clone());
            }
        }
        "index.html" => {
            dirty.render_index = true;
            for (slug, _) in env.get_tags().iter() {
                dirty.render_tags.insert(slug.clone());
            }
            for (slug, _) in env.get_categories().iter() {
                dirty.render_categories.insert(slug.clone());
            }
        }
        "tag_archive_list.html" => {
            dirty.render_tags_overview = true;
        }
        "category_archive_list.html" => {
            dirty.render_categories_overview = true;
        }
        _ => {
            mark_all_html_dirty(dirty, env);
        }
    }
}

fn post_tag_slugs(post: &SharedPost) -> Vec<String> {
    post.0
        .borrow()
        .metadata
        .tags
        .iter()
        .map(|t| t.slug())
        .collect()
}

fn post_category_slugs(post: &SharedPost) -> Vec<String> {
    post.0
        .borrow()
        .metadata
        .categories
        .iter()
        .map(|c| c.slug())
        .collect()
}
