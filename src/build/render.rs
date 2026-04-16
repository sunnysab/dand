use std::path::Path;

use anyhow::{Context, Result};

use crate::build::diff::DirtySet;
use crate::build::report::BuildReport;
use crate::env::Environment;
use crate::index::{
    CategoryOverviewRenderer, CategoryRenderer, IndexRenderer, TagOverviewRenderer, TagRenderer,
};
use crate::post::PostRenderer;
use crate::render::Render;
use crate::site::SiteConf;

/// Execute only the output operations indicated by `dirty`.
/// Returns a report summarising what was done.
pub fn execute_dirty_tasks(
    dirty: &DirtySet,
    env: &Environment,
    site: &SiteConf,
    output_dir: &Path,
    template_dir: &Path,
) -> Result<BuildReport> {
    let mut report = BuildReport::default();

    // 1. Delete removed post output directories.
    for (relative_name, _has_resource_dir) in &dirty.delete_post_outputs {
        let post_dir = output_dir.join(relative_name);
        if post_dir.exists() {
            std::fs::remove_dir_all(&post_dir).with_context(|| {
                format!("failed to delete post output: {}", post_dir.display())
            })?;
            report.deleted_posts += 1;
        }
    }

    // 2. Delete obsolete root static files.
    for file_name in &dirty.delete_root_static {
        let file_path = output_dir.join(file_name);
        if file_path.exists() {
            std::fs::remove_file(&file_path)?;
            report.deleted_static_files += 1;
        }
    }

    // 3. Copy template assets (replaces the whole asset directory).
    if dirty.copy_template_assets {
        let src_asset_dir = template_dir.join("asset");
        let dst_asset_dir = output_dir.join("asset");
        copy_dir(&src_asset_dir, &dst_asset_dir)?;
        report.copied_assets = true;
    }

    // 4. Copy changed root static files.
    for src_path in &dirty.copy_root_static {
        if let Some(file_name) = src_path.file_name() {
            let dst_path = output_dir.join(file_name);
            std::fs::copy(src_path, &dst_path).with_context(|| {
                format!("failed to copy static file: {}", src_path.display())
            })?;
            report.copied_root_statics += 1;
        }
    }

    // 5. Render dirty posts and their companion resources.
    for post in env.get_posts() {
        let (source_path, relative_name) = {
            let p = post.0.borrow();
            (p.reference.full_path.clone(), p.reference.relative_name.clone())
        };

        if dirty.render_posts.contains(&source_path) {
            let renderer = PostRenderer::new(site, post);
            let html = renderer.html()?;
            let post_dir = output_dir.join(&relative_name);
            std::fs::create_dir_all(&post_dir).with_context(|| {
                format!("failed to create post dir: {}", post_dir.display())
            })?;
            std::fs::write(post_dir.join("index.html"), html).with_context(|| {
                format!("failed to write post HTML in: {}", post_dir.display())
            })?;
            report.rendered_posts += 1;
        }

        // Copy resources if either the post HTML was re-rendered or resources changed.
        if dirty.render_posts.contains(&source_path)
            || dirty.copy_post_resources.contains(&source_path)
        {
            copy_post_resources_for(&source_path, &relative_name, output_dir)?;
        }
    }

    // 6. Delete obsolete tag output directories.
    for slug in &dirty.delete_tags {
        let tag_dir = output_dir.join("tags").join(slug);
        if tag_dir.exists() {
            std::fs::remove_dir_all(&tag_dir)?;
        }
    }

    // 7. Render dirty tag pages.
    for (slug, tag) in env.get_tags().iter() {
        if dirty.render_tags.contains(slug) {
            let html = TagRenderer::new(site, tag).html()?;
            let tag_dir = output_dir.join("tags").join(slug);
            std::fs::create_dir_all(&tag_dir)?;
            std::fs::write(tag_dir.join("index.html"), html)?;
            report.rendered_tags += 1;
        }
    }

    // 8. Delete obsolete category output directories.
    for slug in &dirty.delete_categories {
        let cat_dir = output_dir.join("categories").join(slug);
        if cat_dir.exists() {
            std::fs::remove_dir_all(&cat_dir)?;
        }
    }

    // 9. Render dirty category pages.
    for (slug, category) in env.get_categories().iter() {
        if dirty.render_categories.contains(slug) {
            let html = CategoryRenderer::new(site, category).html()?;
            let cat_dir = output_dir.join("categories").join(slug);
            std::fs::create_dir_all(&cat_dir)?;
            std::fs::write(cat_dir.join("index.html"), html)?;
            report.rendered_categories += 1;
        }
    }

    // 10. Render index.
    if dirty.render_index {
        let html = IndexRenderer::new(site, env).html()?;
        std::fs::write(output_dir.join("index.html"), html)?;
        report.rendered_index = true;
    }

    // 11. Render tags overview.
    if dirty.render_tags_overview {
        let html = TagOverviewRenderer::new(site, env).html()?;
        let tags_dir = output_dir.join("tags");
        std::fs::create_dir_all(&tags_dir)?;
        std::fs::write(tags_dir.join("index.html"), html)?;
        report.rendered_tags_overview = true;
    }

    // 12. Render categories overview.
    if dirty.render_categories_overview {
        let html = CategoryOverviewRenderer::new(site, env).html()?;
        let cats_dir = output_dir.join("categories");
        std::fs::create_dir_all(&cats_dir)?;
        std::fs::write(cats_dir.join("index.html"), html)?;
        report.rendered_categories_overview = true;
    }

    Ok(report)
}

/// Render the entire site from scratch with no dirty-set filtering.
/// Used for the first full build when no valid cache exists.
pub fn render_full(
    env: &Environment,
    site: &SiteConf,
    source_dir: &Path,
    output_dir: &Path,
    template_dir: &Path,
) -> Result<BuildReport> {
    let mut report = BuildReport {
        is_incremental: false,
        ..Default::default()
    };

    // Copy template assets.
    let src_asset_dir = template_dir.join("asset");
    let dst_asset_dir = output_dir.join("asset");
    copy_dir(&src_asset_dir, &dst_asset_dir)?;
    report.copied_assets = true;

    // Copy root static files.
    copy_root_static_files(source_dir, output_dir, &mut report)?;

    // Render index.
    let html = IndexRenderer::new(site, env).html()?;
    std::fs::write(output_dir.join("index.html"), html)?;
    report.rendered_index = true;

    // Render each post.
    for post in env.get_posts() {
        let (source_path, relative_name) = {
            let p = post.0.borrow();
            (p.reference.full_path.clone(), p.reference.relative_name.clone())
        };
        let renderer = PostRenderer::new(site, post);
        let html = renderer.html()?;
        let post_dir = output_dir.join(&relative_name);
        std::fs::create_dir_all(&post_dir)?;
        std::fs::write(post_dir.join("index.html"), html)?;
        copy_post_resources_for(&source_path, &relative_name, output_dir)?;
        report.rendered_posts += 1;
    }

    // Render tag pages.
    for (slug, tag) in env.get_tags().iter() {
        let html = TagRenderer::new(site, tag).html()?;
        let tag_dir = output_dir.join("tags").join(slug);
        std::fs::create_dir_all(&tag_dir)?;
        std::fs::write(tag_dir.join("index.html"), html)?;
        report.rendered_tags += 1;
    }

    // Render category pages.
    for (slug, category) in env.get_categories().iter() {
        let html = CategoryRenderer::new(site, category).html()?;
        let cat_dir = output_dir.join("categories").join(slug);
        std::fs::create_dir_all(&cat_dir)?;
        std::fs::write(cat_dir.join("index.html"), html)?;
        report.rendered_categories += 1;
    }

    // Render tags overview.
    let html = TagOverviewRenderer::new(site, env).html()?;
    let tags_dir = output_dir.join("tags");
    std::fs::create_dir_all(&tags_dir)?;
    std::fs::write(tags_dir.join("index.html"), html)?;
    report.rendered_tags_overview = true;

    // Render categories overview.
    let html = CategoryOverviewRenderer::new(site, env).html()?;
    let cats_dir = output_dir.join("categories");
    std::fs::create_dir_all(&cats_dir)?;
    std::fs::write(cats_dir.join("index.html"), html)?;
    report.rendered_categories_overview = true;

    Ok(report)
}

/// Copy a post's companion resource directory into the post's output directory.
///
/// For `source_path = …/_posts/example.md` with `relative_name = "example"`:
/// copies `…/_posts/example/` → `output_root/example/example/`.
pub fn copy_post_resources_for(
    source_path: &Path,
    relative_name: &str,
    output_root: &Path,
) -> Result<()> {
    let Some(parent) = source_path.parent() else {
        return Ok(());
    };
    let Some(stem) = source_path.file_stem() else {
        return Ok(());
    };

    let resource_dir = parent.join(stem);
    if resource_dir.is_dir() {
        let dst_dir = output_root.join(relative_name).join(stem);
        copy_dir(&resource_dir, &dst_dir)?;
    }
    Ok(())
}

/// Recursively copy `src` directory into `dst`.
pub fn copy_dir(src: &Path, dst: &Path) -> Result<()> {
    if !src.is_dir() {
        return Ok(());
    }
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// Copy non-md, non-config files that sit directly in `source_dir` to `output_dir`.
pub fn copy_root_static_files(
    source_dir: &Path,
    output_dir: &Path,
    report: &mut BuildReport,
) -> Result<()> {
    if !source_dir.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(source_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !entry.file_type()?.is_file() {
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if ext == "md" || name == "dand.toml" {
            continue;
        }
        std::fs::copy(&path, output_dir.join(entry.file_name()))?;
        report.copied_root_statics += 1;
    }
    Ok(())
}
