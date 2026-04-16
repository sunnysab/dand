pub mod diff;
pub mod render;
pub mod report;
pub mod snapshot;
pub mod state;

use std::collections::HashMap;
use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::build::diff::{diff_snapshots, plan_dirty_outputs};
use crate::build::render::{execute_dirty_tasks, render_full};
use crate::build::report::BuildReport;
use crate::build::snapshot::{scan_source_tree, CurrentSnapshot};
use crate::build::state::{
    read_state, write_state_atomic, BuildState, CachedPost, GeneratedOutputs,
};
use crate::discover::{PostWalker, WalkDirectory};
use crate::env::Environment;
use crate::site::SiteConf;

/// Primary build entry point.
///
/// Performs a full rebuild if no valid cache exists in `output_dir`, otherwise performs
/// an incremental build based on mtime diffs.
pub fn build_site(
    source_dir: &Path,
    output_dir: &Path,
    config_path: &Path,
    template_dir: &Path,
) -> Result<BuildReport> {
    validate_dirs(source_dir, output_dir)?;

    let old_state = output_dir
        .exists()
        .then(|| read_state(output_dir))
        .flatten();

    if old_state.is_none() {
        return full_build(source_dir, output_dir, config_path, template_dir);
    }

    incremental_build(
        source_dir,
        output_dir,
        config_path,
        template_dir,
        old_state.unwrap(),
    )
}

fn full_build(
    source_dir: &Path,
    output_dir: &Path,
    config_path: &Path,
    template_dir: &Path,
) -> Result<BuildReport> {
    reset_output_dir(output_dir)?;

    let site = SiteConf::load(config_path)?;
    let env = build_environment(source_dir)?;

    let mut report = render_full(&env, &site, source_dir, output_dir, template_dir)?;
    report.is_incremental = false;

    let snapshot = scan_source_tree(source_dir, template_dir, config_path)
        .context("failed to scan source tree after full build")?;
    let new_state = capture_state(&snapshot, &env);
    write_state_atomic(output_dir, &new_state)?;

    Ok(report)
}

fn incremental_build(
    source_dir: &Path,
    output_dir: &Path,
    config_path: &Path,
    template_dir: &Path,
    old_state: BuildState,
) -> Result<BuildReport> {
    let snapshot = scan_source_tree(source_dir, template_dir, config_path)
        .context("failed to scan source tree")?;

    let diff = diff_snapshots(&old_state, &snapshot);

    // Always build a full Environment so relationship data is consistent.
    let site = SiteConf::load(config_path)?;
    let env = build_environment(source_dir)?;

    let dirty = plan_dirty_outputs(&diff, &old_state, &env);

    let mut report =
        execute_dirty_tasks(&dirty, &env, &site, source_dir, output_dir, template_dir)?;
    report.is_incremental = true;

    let new_state = capture_state(&snapshot, &env);
    write_state_atomic(output_dir, &new_state)?;

    Ok(report)
}

/// Build the in-memory `Environment` by loading all markdown posts from `source_dir`.
fn build_environment(source_dir: &Path) -> Result<Environment> {
    let mut env = Environment::new();
    let walker = PostWalker::from(source_dir.to_path_buf());
    for post_ref in walker.walk()? {
        let path_display = post_ref.full_path.display().to_string();
        crate::post::load(post_ref)
            .with_context(|| format!("failed to load post: {}", path_display))?
            .build(&mut env);
    }
    Ok(env)
}

/// Serialise the current build result into a `BuildState` ready to persist.
fn capture_state(snapshot: &CurrentSnapshot, env: &Environment) -> BuildState {
    let input_files = snapshot.as_mtime_map();

    let cached_posts: HashMap<String, CachedPost> = env
        .get_posts()
        .iter()
        .map(|post| {
            let p = post.0.borrow();
            let source_path = p.reference.full_path.clone();
            let has_resource_dir = source_path
                .parent()
                .and_then(|parent| {
                    source_path
                        .file_stem()
                        .map(|stem| parent.join(stem).is_dir())
                })
                .unwrap_or(false);

            let cached = CachedPost {
                source_path: source_path.clone(),
                relative_name: p.reference.relative_name.clone(),
                tags: p.metadata.tags.iter().map(|t| t.slug()).collect(),
                categories: p.metadata.categories.iter().map(|c| c.slug()).collect(),
                hidden: p.metadata.flag.is_hidden(),
                draft: p.metadata.flag.is_draft(),
                has_resource_dir,
            };
            (source_path.to_string_lossy().to_string(), cached)
        })
        .collect();

    let generated_outputs = GeneratedOutputs {
        post_relative_names: env
            .get_posts()
            .iter()
            .map(|p| p.0.borrow().reference.relative_name.clone())
            .collect(),
        rendered_tags: env
            .get_tags()
            .iter()
            .map(|(slug, _)| slug.clone())
            .collect(),
        rendered_categories: env
            .get_categories()
            .iter()
            .map(|(slug, _)| slug.clone())
            .collect(),
        root_static_files: Vec::new(),
    };

    BuildState {
        input_files,
        cached_posts,
        generated_outputs,
        ..Default::default()
    }
}

/// Clear the output directory and recreate it (preserving nothing).
fn reset_output_dir(output_dir: &Path) -> Result<()> {
    if output_dir.exists() {
        std::fs::remove_dir_all(output_dir).with_context(|| {
            format!("failed to clean output directory: {}", output_dir.display())
        })?;
    }
    std::fs::create_dir_all(output_dir).with_context(|| {
        format!(
            "failed to create output directory: {}",
            output_dir.display()
        )
    })?;
    Ok(())
}

/// Verify that source and output directories are not the same path.
fn validate_dirs(source_dir: &Path, output_dir: &Path) -> Result<()> {
    let source_canonical = source_dir.canonicalize().with_context(|| {
        format!(
            "failed to resolve source directory: {}",
            source_dir.display()
        )
    })?;

    if !output_dir.exists() {
        return Ok(());
    }

    let output_canonical = output_dir.canonicalize().with_context(|| {
        format!(
            "failed to resolve output directory: {}",
            output_dir.display()
        )
    })?;

    if source_canonical == output_canonical {
        bail!("output directory must be different from source directory");
    }
    Ok(())
}
