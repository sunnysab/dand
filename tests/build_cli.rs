use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{Datelike, Local};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("dand-cli-{prefix}-{timestamp}-{id}"))
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

#[test]
fn build_requires_source_root_config_by_default() {
    let source_dir = unique_temp_dir("source");
    let output_dir = unique_temp_dir("output");

    write_file(&source_dir.join("template/asset/css/main.css"), "body {}");
    write_file(
        &source_dir.join("_posts/dand.toml"),
        "title = \"test\"\nhome = \"https://example.com/\"\ndescription = \"test\"\n",
    );
    write_file(
        &source_dir.join("_posts/hello.md"),
        "---\n\
title: Hello\n\
date: 2024-01-01 00:00\n\
---\n\
\n\
Hello\n",
    );

    let output = Command::new(env!("CARGO_BIN_EXE_dand-rs"))
        .arg("build")
        .arg(&source_dir)
        .arg("-o")
        .arg(&output_dir)
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(!output_dir.join("hello/index.html").is_file());

    fs::remove_dir_all(&source_dir).unwrap();
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir).unwrap();
    }
}

#[test]
fn build_hides_posts_under_drafts_directory_from_index() {
    let source_dir = unique_temp_dir("draft-source");
    let output_dir = unique_temp_dir("draft-output");

    write_file(
        &source_dir.join("dand.toml"),
        "title = \"test\"\nhome = \"https://example.com/\"\ndescription = \"test\"\n",
    );
    write_file(&source_dir.join("template/asset/css/main.css"), "body {}");
    write_file(
        &source_dir.join("_posts/published.md"),
        "---\n\
title: Published\n\
date: 2024-01-02 00:00\n\
---\n\
\n\
Published\n",
    );
    write_file(
        &source_dir.join("_drafts/draft.md"),
        "---\n\
title: Draft\n\
date: 2024-01-03 00:00\n\
---\n\
\n\
Draft\n",
    );

    let status = Command::new(env!("CARGO_BIN_EXE_dand-rs"))
        .arg("build")
        .arg(&source_dir)
        .arg("-o")
        .arg(&output_dir)
        .status()
        .unwrap();

    assert!(status.success());

    let index_html = fs::read_to_string(output_dir.join("index.html")).unwrap();
    assert!(index_html.contains("Published"));
    assert!(!index_html.contains("Draft"));
    assert!(output_dir.join("_drafts/draft/index.html").is_file());

    fs::remove_dir_all(&source_dir).unwrap();
    fs::remove_dir_all(&output_dir).unwrap();
}

#[test]
fn build_removes_stale_generated_pages() {
    let source_dir = unique_temp_dir("stale-source");
    let output_dir = unique_temp_dir("stale-output");

    write_file(
        &source_dir.join("dand.toml"),
        "title = \"test\"\nhome = \"https://example.com/\"\ndescription = \"test\"\n",
    );
    write_file(&source_dir.join("template/asset/css/main.css"), "body {}");
    write_file(
        &source_dir.join("_posts/first.md"),
        "---\n\
title: First\n\
date: 2024-01-01 00:00\n\
---\n\
\n\
First\n",
    );

    let first_status = Command::new(env!("CARGO_BIN_EXE_dand-rs"))
        .arg("build")
        .arg(&source_dir)
        .arg("-o")
        .arg(&output_dir)
        .status()
        .unwrap();
    assert!(first_status.success());
    assert!(output_dir.join("first/index.html").is_file());

    fs::remove_file(source_dir.join("_posts/first.md")).unwrap();

    let second_status = Command::new(env!("CARGO_BIN_EXE_dand-rs"))
        .arg("build")
        .arg(&source_dir)
        .arg("-o")
        .arg(&output_dir)
        .status()
        .unwrap();
    assert!(second_status.success());
    assert!(!output_dir.join("first/index.html").exists());

    fs::remove_dir_all(&source_dir).unwrap();
    fs::remove_dir_all(&output_dir).unwrap();
}

#[test]
fn build_generates_tag_pages() {
    let source_dir = unique_temp_dir("tag-source");
    let output_dir = unique_temp_dir("tag-output");

    write_file(
        &source_dir.join("dand.toml"),
        "title = \"test\"\nhome = \"https://example.com/\"\ndescription = \"test\"\n",
    );
    write_file(&source_dir.join("template/asset/css/main.css"), "body {}");
    write_file(
        &source_dir.join("_posts/rust-1.md"),
        "---\n\
title: Rust One\n\
date: 2024-01-01 00:00\n\
tags:\n\
  - rust\n\
---\n\
\n\
Rust One\n",
    );
    write_file(
        &source_dir.join("_posts/rust-2.md"),
        "---\n\
title: Rust Two\n\
date: 2024-01-02 00:00\n\
tags:\n\
  - rust\n\
  - web\n\
---\n\
\n\
Rust Two\n",
    );
    write_file(
        &source_dir.join("_posts/db.md"),
        "---\n\
title: Database\n\
date: 2024-01-03 00:00\n\
tags:\n\
  - db\n\
---\n\
\n\
Database\n",
    );
    write_file(
        &source_dir.join("_drafts/rust-draft.md"),
        "---\n\
title: Rust Draft\n\
date: 2024-01-04 00:00\n\
tags:\n\
  - rust\n\
---\n\
\n\
Rust Draft\n",
    );

    let status = Command::new(env!("CARGO_BIN_EXE_dand-rs"))
        .arg("build")
        .arg(&source_dir)
        .arg("-o")
        .arg(&output_dir)
        .status()
        .unwrap();
    assert!(status.success());

    let rust_tag_page = output_dir.join("tags/rust/index.html");
    assert!(rust_tag_page.is_file());

    let rust_tag_html = fs::read_to_string(rust_tag_page).unwrap();
    assert!(rust_tag_html.contains("href=\"/asset/css/main.css\""));
    assert!(rust_tag_html.contains("src=\"/asset/js/display.js\""));
    assert!(rust_tag_html.contains("<h1 class=\"page-title\">rust</h1>"));
    assert!(rust_tag_html.contains("<p class=\"page-eyebrow\">标签</p>"));
    assert!(rust_tag_html.contains("Rust One"));
    assert!(rust_tag_html.contains("Rust Two"));
    assert!(!rust_tag_html.contains("Database"));
    assert!(!rust_tag_html.contains("Rust Draft"));

    fs::remove_dir_all(&source_dir).unwrap();
    fs::remove_dir_all(&output_dir).unwrap();
}

#[test]
fn build_generates_category_pages() {
    let source_dir = unique_temp_dir("category-source");
    let output_dir = unique_temp_dir("category-output");

    write_file(
        &source_dir.join("dand.toml"),
        "title = \"test\"\nhome = \"https://example.com/\"\ndescription = \"test\"\n",
    );
    write_file(&source_dir.join("template/asset/css/main.css"), "body {}");
    write_file(
        &source_dir.join("_posts/rust-db-1.md"),
        "---\n\
title: Rust DB One\n\
date: 2024-01-01 00:00\n\
categories:\n\
  - 数据库\n\
---\n\
\n\
Rust DB One\n",
    );
    write_file(
        &source_dir.join("_posts/rust-db-2.md"),
        "---\n\
title: Rust DB Two\n\
date: 2024-01-02 00:00\n\
categories:\n\
  - 数据库\n\
  - Rust\n\
---\n\
\n\
Rust DB Two\n",
    );
    write_file(
        &source_dir.join("_posts/network.md"),
        "---\n\
title: Network\n\
date: 2024-01-03 00:00\n\
categories:\n\
  - 网络\n\
---\n\
\n\
Network\n",
    );
    write_file(
        &source_dir.join("_drafts/db-draft.md"),
        "---\n\
title: DB Draft\n\
date: 2024-01-04 00:00\n\
categories:\n\
  - 数据库\n\
---\n\
\n\
DB Draft\n",
    );

    let status = Command::new(env!("CARGO_BIN_EXE_dand-rs"))
        .arg("build")
        .arg(&source_dir)
        .arg("-o")
        .arg(&output_dir)
        .status()
        .unwrap();
    assert!(status.success());

    let db_category_page = output_dir.join("categories/数据库/index.html");
    assert!(db_category_page.is_file());

    let db_category_html = fs::read_to_string(db_category_page).unwrap();
    assert!(db_category_html.contains("Rust DB One"));
    assert!(db_category_html.contains("Rust DB Two"));
    assert!(!db_category_html.contains("Network"));
    assert!(!db_category_html.contains("DB Draft"));

    fs::remove_dir_all(&source_dir).unwrap();
    fs::remove_dir_all(&output_dir).unwrap();
}

#[test]
fn build_adds_tag_and_category_links_to_post_pages() {
    let source_dir = unique_temp_dir("post-links-source");
    let output_dir = unique_temp_dir("post-links-output");

    write_file(
        &source_dir.join("dand.toml"),
        "title = \"test\"\nhome = \"https://example.com/\"\ndescription = \"test\"\n",
    );
    write_file(&source_dir.join("template/asset/css/main.css"), "body {}");
    write_file(
        &source_dir.join("_posts/linked-post.md"),
        "---\n\
title: Linked Post\n\
date: 2024-01-01 00:00\n\
tags:\n\
  - rust\n\
  - web\n\
categories:\n\
  - 数据库\n\
---\n\
\n\
Linked Post\n",
    );

    let status = Command::new(env!("CARGO_BIN_EXE_dand-rs"))
        .arg("build")
        .arg(&source_dir)
        .arg("-o")
        .arg(&output_dir)
        .status()
        .unwrap();
    assert!(status.success());

    let post_html = fs::read_to_string(output_dir.join("linked-post/index.html")).unwrap();
    assert!(post_html.contains("href=\"/tags/rust/\""));
    assert!(post_html.contains("href=\"/tags/web/\""));
    assert!(post_html.contains("href=\"/categories/数据库/\""));

    fs::remove_dir_all(&source_dir).unwrap();
    fs::remove_dir_all(&output_dir).unwrap();
}

#[test]
fn build_supports_latex_front_matter() {
    let source_dir = unique_temp_dir("latex-source");
    let output_dir = unique_temp_dir("latex-output");

    write_file(
        &source_dir.join("dand.toml"),
        "title = \"test\"\nhome = \"https://example.com/\"\ndescription = \"test\"\n",
    );
    write_file(
        &source_dir.join("_posts/math.md"),
        "---\n\
title: Math\n\
date: 2024-01-01 00:00\n\
latex: true\n\
---\n\
\n\
函数 $y = f(x)$。\n\
\n\
$$\n\
x^2 + y^2 = z^2\n\
$$\n",
    );

    let status = Command::new(env!("CARGO_BIN_EXE_dand-rs"))
        .arg("build")
        .arg(&source_dir)
        .arg("-o")
        .arg(&output_dir)
        .status()
        .unwrap();
    assert!(status.success());

    let post_html = fs::read_to_string(output_dir.join("math/index.html")).unwrap();
    assert!(post_html.contains("href=\"/asset/css/vendor/katex/katex.min.css\""));
    assert!(post_html.contains("src=\"/asset/js/vendor/katex/katex.min.js\""));
    assert!(!post_html.contains("cdnjs.cloudflare.com"));
    assert!(post_html.contains("class=\"arithmatex\""));
    assert!(post_html.contains("data-display-mode=\"false\""));
    assert!(post_html.contains("data-display-mode=\"true\""));

    fs::remove_dir_all(&source_dir).unwrap();
    fs::remove_dir_all(&output_dir).unwrap();
}

#[test]
fn build_uses_local_post_vendor_assets() {
    let source_dir = unique_temp_dir("local-assets-source");
    let output_dir = unique_temp_dir("local-assets-output");

    write_file(
        &source_dir.join("dand.toml"),
        "title = \"test\"\nhome = \"https://example.com/\"\ndescription = \"test\"\n",
    );
    write_file(
        &source_dir.join("_posts/hello.md"),
        "---\n\
title: Hello\n\
date: 2024-01-01 00:00\n\
latex: true\n\
---\n\
\n\
```rust\n\
fn main() {}\n\
```\n\
\n\
公式 $x + y$。\n",
    );

    let status = Command::new(env!("CARGO_BIN_EXE_dand-rs"))
        .arg("build")
        .arg(&source_dir)
        .arg("-o")
        .arg(&output_dir)
        .status()
        .unwrap();
    assert!(status.success());

    let post_html = fs::read_to_string(output_dir.join("hello/index.html")).unwrap();
    assert!(post_html.contains("href=\"/asset/css/vendor/highlight/default.min.css\""));
    assert!(post_html.contains("src=\"/asset/js/vendor/highlight/highlight.min.js\""));
    assert!(post_html.contains("href=\"/asset/css/vendor/katex/katex.min.css\""));
    assert!(post_html.contains("src=\"/asset/js/vendor/katex/katex.min.js\""));
    assert!(!post_html.contains("cdn.jsdelivr.net"));
    assert!(!post_html.contains("cdnjs.cloudflare.com"));
    assert!(output_dir
        .join("asset/css/vendor/highlight/default.min.css")
        .is_file());
    assert!(output_dir
        .join("asset/js/vendor/highlight/highlight.min.js")
        .is_file());
    assert!(output_dir
        .join("asset/css/vendor/katex/katex.min.css")
        .is_file());
    assert!(output_dir
        .join("asset/js/vendor/katex/katex.min.js")
        .is_file());
    assert!(output_dir
        .join("asset/css/vendor/katex/fonts/KaTeX_Main-Regular.woff2")
        .is_file());

    fs::remove_dir_all(&source_dir).unwrap();
    fs::remove_dir_all(&output_dir).unwrap();
}

#[test]
fn build_generates_tag_and_category_overview_pages() {
    let source_dir = unique_temp_dir("archive-overview-source");
    let output_dir = unique_temp_dir("archive-overview-output");

    write_file(
        &source_dir.join("dand.toml"),
        "title = \"test\"\nhome = \"https://example.com/\"\ndescription = \"test\"\n",
    );
    write_file(&source_dir.join("template/asset/css/main.css"), "body {}");
    write_file(
        &source_dir.join("_posts/rust-db.md"),
        "---\n\
title: Rust DB\n\
date: 2024-01-01 00:00\n\
tags:\n\
  - rust\n\
  - db\n\
categories:\n\
  - 数据库\n\
---\n\
\n\
Rust DB\n",
    );
    write_file(
        &source_dir.join("_posts/web.md"),
        "---\n\
title: Web\n\
date: 2024-01-02 00:00\n\
tags:\n\
  - web\n\
categories:\n\
  - 网络\n\
---\n\
\n\
Web\n",
    );
    write_file(
        &source_dir.join("_posts/rust-hidden.md"),
        "---\n\
title: Rust Hidden\n\
date: 2024-01-03 00:00\n\
is_hidden: true\n\
tags:\n\
  - rust\n\
categories:\n\
  - 数据库\n\
---\n\
\n\
Rust Hidden\n",
    );
    write_file(
        &source_dir.join("_drafts/rust-draft.md"),
        "---\n\
title: Rust Draft\n\
date: 2024-01-04 00:00\n\
tags:\n\
  - rust\n\
categories:\n\
  - 数据库\n\
---\n\
\n\
Rust Draft\n",
    );

    let status = Command::new(env!("CARGO_BIN_EXE_dand-rs"))
        .arg("build")
        .arg(&source_dir)
        .arg("-o")
        .arg(&output_dir)
        .status()
        .unwrap();
    assert!(status.success());

    let tags_overview = fs::read_to_string(output_dir.join("tags/index.html")).unwrap();
    assert!(tags_overview.contains("href=\"/tags/rust/\""));
    assert!(tags_overview.contains("href=\"/tags/db/\""));
    assert!(tags_overview.contains("href=\"/tags/web/\""));
    assert!(tags_overview.contains("rust"));
    assert!(tags_overview.contains("1 篇"));

    let categories_overview = fs::read_to_string(output_dir.join("categories/index.html")).unwrap();
    assert!(categories_overview.contains("href=\"/categories/数据库/\""));
    assert!(categories_overview.contains("href=\"/categories/网络/\""));
    assert!(categories_overview.contains("数据库"));
    assert!(categories_overview.contains("1 篇"));

    fs::remove_dir_all(&source_dir).unwrap();
    fs::remove_dir_all(&output_dir).unwrap();
}

#[test]
fn build_normalizes_taxonomy_slugs() {
    let source_dir = unique_temp_dir("taxonomy-slug-source");
    let output_dir = unique_temp_dir("taxonomy-slug-output");

    write_file(
        &source_dir.join("dand.toml"),
        "title = \"test\"\nhome = \"https://example.com/\"\ndescription = \"test\"\n",
    );
    write_file(&source_dir.join("template/asset/css/main.css"), "body {}");
    write_file(
        &source_dir.join("_posts/slugged.md"),
        "---\n\
title: Slugged\n\
date: 2024-01-01 00:00\n\
tags:\n\
  - C++\n\
  - rust\n\
  - Rust\n\
categories:\n\
  - Windows 程序\n\
---\n\
\n\
Slugged\n",
    );

    let status = Command::new(env!("CARGO_BIN_EXE_dand-rs"))
        .arg("build")
        .arg(&source_dir)
        .arg("-o")
        .arg(&output_dir)
        .status()
        .unwrap();
    assert!(status.success());

    assert!(output_dir.join("tags/c-plus-plus/index.html").is_file());
    assert!(output_dir.join("tags/rust/index.html").is_file());
    assert!(!output_dir.join("tags/rust-2/index.html").exists());
    assert!(output_dir
        .join("categories/windows-程序/index.html")
        .is_file());

    let post_html = fs::read_to_string(output_dir.join("slugged/index.html")).unwrap();
    assert!(post_html.contains("href=\"/tags/c-plus-plus/\""));
    assert!(post_html.contains("href=\"/tags/rust/\""));
    assert!(!post_html.contains("href=\"/tags/rust-2/\""));
    assert!(post_html.contains(">Rust<"));
    assert!(post_html.contains("href=\"/categories/windows-程序/\""));

    fs::remove_dir_all(&source_dir).unwrap();
    fs::remove_dir_all(&output_dir).unwrap();
}

#[test]
fn build_adds_tags_and_categories_to_header_navigation() {
    let source_dir = unique_temp_dir("nav-source");
    let output_dir = unique_temp_dir("nav-output");

    write_file(
        &source_dir.join("dand.toml"),
        "title = \"test\"\nhome = \"https://example.com/\"\ndescription = \"test\"\n",
    );
    write_file(&source_dir.join("template/asset/css/main.css"), "body {}");
    write_file(
        &source_dir.join("_posts/example.md"),
        "---\n\
title: Example\n\
date: 2024-01-01 00:00\n\
tags:\n\
  - rust\n\
categories:\n\
  - 数据库\n\
---\n\
\n\
Example\n",
    );

    let status = Command::new(env!("CARGO_BIN_EXE_dand-rs"))
        .arg("build")
        .arg(&source_dir)
        .arg("-o")
        .arg(&output_dir)
        .status()
        .unwrap();
    assert!(status.success());

    for path in [
        output_dir.join("index.html"),
        output_dir.join("example/index.html"),
        output_dir.join("tags/index.html"),
        output_dir.join("categories/index.html"),
    ] {
        let html = fs::read_to_string(path).unwrap();
        assert!(html.contains("href=\"/tags/\""));
        assert!(html.contains(">标签<"));
        assert!(html.contains("href=\"/categories/\""));
        assert!(html.contains(">分类<"));
    }

    fs::remove_dir_all(&source_dir).unwrap();
    fs::remove_dir_all(&output_dir).unwrap();
}

#[test]
fn build_merges_taxonomy_names_by_slug() {
    let source_dir = unique_temp_dir("taxonomy-merge-source");
    let output_dir = unique_temp_dir("taxonomy-merge-output");

    write_file(
        &source_dir.join("dand.toml"),
        "title = \"test\"\nhome = \"https://example.com/\"\ndescription = \"test\"\n",
    );
    write_file(&source_dir.join("template/asset/css/main.css"), "body {}");
    write_file(
        &source_dir.join("_posts/post-1.md"),
        "---\n\
title: Post One\n\
date: 2024-01-01 00:00\n\
tags:\n\
  - rust\n\
categories:\n\
  - postgresql\n\
---\n\
\n\
Post One\n",
    );
    write_file(
        &source_dir.join("_posts/post-2.md"),
        "---\n\
title: Post Two\n\
date: 2024-01-02 00:00\n\
tags:\n\
  - Rust\n\
categories:\n\
  - PostgreSQL\n\
---\n\
\n\
Post Two\n",
    );

    let status = Command::new(env!("CARGO_BIN_EXE_dand-rs"))
        .arg("build")
        .arg(&source_dir)
        .arg("-o")
        .arg(&output_dir)
        .status()
        .unwrap();
    assert!(status.success());

    assert!(output_dir.join("tags/rust/index.html").is_file());
    assert!(!output_dir.join("tags/rust-2/index.html").exists());
    assert!(output_dir
        .join("categories/postgresql/index.html")
        .is_file());
    assert!(!output_dir
        .join("categories/postgresql-2/index.html")
        .exists());

    let tags_overview = fs::read_to_string(output_dir.join("tags/index.html")).unwrap();
    assert!(tags_overview.contains(">Rust<"));
    assert!(!tags_overview.contains(">rust<"));
    assert!(tags_overview.contains("2 篇"));

    let categories_overview = fs::read_to_string(output_dir.join("categories/index.html")).unwrap();
    assert!(categories_overview.contains(">PostgreSQL<"));
    assert!(!categories_overview.contains(">postgresql<"));
    assert!(categories_overview.contains("2 篇"));

    let post_html = fs::read_to_string(output_dir.join("post-1/index.html")).unwrap();
    assert!(post_html.contains("href=\"/tags/rust/\""));
    assert!(post_html.contains(">Rust<"));
    assert!(post_html.contains("href=\"/categories/postgresql/\""));
    assert!(post_html.contains(">PostgreSQL<"));

    fs::remove_dir_all(&source_dir).unwrap();
    fs::remove_dir_all(&output_dir).unwrap();
}

#[test]
fn build_sorts_taxonomy_overview_by_post_count_desc() {
    let source_dir = unique_temp_dir("taxonomy-sort-source");
    let output_dir = unique_temp_dir("taxonomy-sort-output");

    write_file(
        &source_dir.join("dand.toml"),
        "title = \"test\"\nhome = \"https://example.com/\"\ndescription = \"test\"\n",
    );
    write_file(&source_dir.join("template/asset/css/main.css"), "body {}");
    write_file(
        &source_dir.join("_posts/post-1.md"),
        "---\n\
title: Post One\n\
date: 2024-01-01 00:00\n\
tags:\n\
  - rust\n\
categories:\n\
  - 数据库\n\
---\n\
\n\
Post One\n",
    );
    write_file(
        &source_dir.join("_posts/post-2.md"),
        "---\n\
title: Post Two\n\
date: 2024-01-02 00:00\n\
tags:\n\
  - rust\n\
categories:\n\
  - 数据库\n\
---\n\
\n\
Post Two\n",
    );
    write_file(
        &source_dir.join("_posts/post-3.md"),
        "---\n\
title: Post Three\n\
date: 2024-01-03 00:00\n\
tags:\n\
  - db\n\
categories:\n\
  - 网络\n\
---\n\
\n\
Post Three\n",
    );

    let status = Command::new(env!("CARGO_BIN_EXE_dand-rs"))
        .arg("build")
        .arg(&source_dir)
        .arg("-o")
        .arg(&output_dir)
        .status()
        .unwrap();
    assert!(status.success());

    let tags_overview = fs::read_to_string(output_dir.join("tags/index.html")).unwrap();
    let rust_pos = tags_overview.find("href=\"/tags/rust/\"").unwrap();
    let db_pos = tags_overview.find("href=\"/tags/db/\"").unwrap();
    assert!(rust_pos < db_pos);

    let categories_overview = fs::read_to_string(output_dir.join("categories/index.html")).unwrap();
    let database_pos = categories_overview
        .find("href=\"/categories/数据库/\"")
        .unwrap();
    let network_pos = categories_overview
        .find("href=\"/categories/网络/\"")
        .unwrap();
    assert!(database_pos < network_pos);

    fs::remove_dir_all(&source_dir).unwrap();
    fs::remove_dir_all(&output_dir).unwrap();
}

#[test]
fn build_adds_overview_links_to_taxonomy_detail_pages() {
    let source_dir = unique_temp_dir("taxonomy-detail-nav-source");
    let output_dir = unique_temp_dir("taxonomy-detail-nav-output");

    write_file(
        &source_dir.join("dand.toml"),
        "title = \"test\"\nhome = \"https://example.com/\"\ndescription = \"test\"\n",
    );
    write_file(&source_dir.join("template/asset/css/main.css"), "body {}");
    write_file(
        &source_dir.join("_posts/example.md"),
        "---\n\
title: Example\n\
date: 2024-01-01 00:00\n\
tags:\n\
  - rust\n\
categories:\n\
  - 数据库\n\
---\n\
\n\
Example\n",
    );

    let status = Command::new(env!("CARGO_BIN_EXE_dand-rs"))
        .arg("build")
        .arg(&source_dir)
        .arg("-o")
        .arg(&output_dir)
        .status()
        .unwrap();
    assert!(status.success());

    let tag_html = fs::read_to_string(output_dir.join("tags/rust/index.html")).unwrap();
    assert!(tag_html.contains("href=\"/tags/\""));
    assert!(tag_html.contains(">返回标签总览<"));

    let category_html =
        fs::read_to_string(output_dir.join("categories/数据库/index.html")).unwrap();
    assert!(category_html.contains("href=\"/categories/\""));
    assert!(category_html.contains(">返回分类总览<"));

    fs::remove_dir_all(&source_dir).unwrap();
    fs::remove_dir_all(&output_dir).unwrap();
}

#[test]
fn build_keeps_mobile_reading_controls_effective() {
    let source_dir = unique_temp_dir("mobile-reading-source");
    let output_dir = unique_temp_dir("mobile-reading-output");

    write_file(
        &source_dir.join("dand.toml"),
        "title = \"test\"\nhome = \"https://example.com/\"\ndescription = \"test\"\n",
    );
    write_file(
        &source_dir.join("_posts/reading.md"),
        "---\n\
title: Reading\n\
date: 2024-01-01 00:00\n\
---\n\
\n\
Reading\n",
    );

    let status = Command::new(env!("CARGO_BIN_EXE_dand-rs"))
        .arg("build")
        .arg(&source_dir)
        .arg("-o")
        .arg(&output_dir)
        .status()
        .unwrap();
    assert!(status.success());

    let post_html = fs::read_to_string(output_dir.join("reading/index.html")).unwrap();
    let post_css = fs::read_to_string(output_dir.join("asset/css/post.css")).unwrap();

    assert!(post_html.contains("data-reading-line-height"));
    assert!(post_html.contains("reading-settings-control-page-width"));

    assert!(post_css.contains("@media (max-width: 640px) {\n    .post-page {\n        margin: 28px auto 36px;\n        --reader-max-width: 100%;\n        --reader-font-size: 1rem;\n        --reader-line-height: 1.72;"));
    assert!(post_css.contains("@media (max-width: 420px) {\n    .post-page {\n        margin: 24px auto 32px;\n        --reader-font-size: 0.97rem;\n        --reader-line-height: 1.68;"));
    assert!(!post_css.contains(".post-content {\n        --reader-max-width: 100%;"));
    assert!(!post_css.contains(
        ".post-content {\n        --reader-font-size: 1rem;\n        --reader-line-height: 1.72;",
    ));
    assert!(!post_css.contains(
        ".post-content {\n        --reader-font-size: 0.97rem;\n        --reader-line-height: 1.68;",
    ));
    assert!(post_css.contains(".reading-settings-control-page-width {\n        display: none;"));

    fs::remove_dir_all(&source_dir).unwrap();
    fs::remove_dir_all(&output_dir).unwrap();
}

#[test]
fn build_uses_reader_paragraph_gap_variable() {
    let source_dir = unique_temp_dir("paragraph-gap-source");
    let output_dir = unique_temp_dir("paragraph-gap-output");

    write_file(
        &source_dir.join("dand.toml"),
        "title = \"test\"\nhome = \"https://example.com/\"\ndescription = \"test\"\n",
    );
    write_file(
        &source_dir.join("_posts/paragraphs.md"),
        "---\n\
title: Paragraphs\n\
date: 2024-01-01 00:00\n\
---\n\
\n\
First paragraph.\n\
\n\
Second paragraph.\n",
    );

    let status = Command::new(env!("CARGO_BIN_EXE_dand-rs"))
        .arg("build")
        .arg(&source_dir)
        .arg("-o")
        .arg(&output_dir)
        .status()
        .unwrap();
    assert!(status.success());

    let post_css = fs::read_to_string(output_dir.join("asset/css/post.css")).unwrap();

    assert!(post_css.contains("--reader-paragraph-gap: 0.6em;"));
    assert!(post_css.contains(".post-content p {\n    margin: var(--reader-paragraph-gap) 0;"));

    fs::remove_dir_all(&source_dir).unwrap();
    fs::remove_dir_all(&output_dir).unwrap();
}

#[test]
fn build_renders_footer_from_config() {
    let source_dir = unique_temp_dir("footer-source");
    let output_dir = unique_temp_dir("footer-output");
    let current_year = Local::now().year();

    write_file(
        &source_dir.join("dand.toml"),
        "title = \"test\"\nhome = \"https://example.com/\"\ndescription = \"test\"\n\n[footer]\nstart_year = 2020\nbeian = \"沪ICP备20260001号\"\n",
    );
    write_file(&source_dir.join("template/asset/css/main.css"), "body {}");
    write_file(
        &source_dir.join("_posts/example.md"),
        "---\n\
title: Example\n\
date: 2024-01-01 00:00\n\
---\n\
\n\
Example\n",
    );

    let status = Command::new(env!("CARGO_BIN_EXE_dand-rs"))
        .arg("build")
        .arg(&source_dir)
        .arg("-o")
        .arg(&output_dir)
        .status()
        .unwrap();
    assert!(status.success());

    let index_html = fs::read_to_string(output_dir.join("index.html")).unwrap();
    assert!(index_html.contains(&format!("sunnysab.cn &copy; 2020 - {current_year}")));
    assert!(index_html.contains("沪ICP备20260001号"));

    fs::remove_dir_all(&source_dir).unwrap();
    fs::remove_dir_all(&output_dir).unwrap();
}

#[test]
fn build_renders_category_pages_with_zh_cn_lang() {
    let source_dir = unique_temp_dir("lang-source");
    let output_dir = unique_temp_dir("lang-output");

    write_file(
        &source_dir.join("dand.toml"),
        "title = \"test\"\nhome = \"https://example.com/\"\ndescription = \"test\"\n\n[footer]\nstart_year = 2020\nbeian = \"沪ICP备20260001号\"\n",
    );
    write_file(&source_dir.join("template/asset/css/main.css"), "body {}");
    write_file(
        &source_dir.join("_posts/example.md"),
        "---\n\
title: Example\n\
date: 2024-01-01 00:00\n\
categories:\n\
  - 随笔\n\
---\n\
\n\
Example\n",
    );

    let status = Command::new(env!("CARGO_BIN_EXE_dand-rs"))
        .arg("build")
        .arg(&source_dir)
        .arg("-o")
        .arg(&output_dir)
        .status()
        .unwrap();
    assert!(status.success());

    let category_html = fs::read_to_string(output_dir.join("categories/随笔/index.html")).unwrap();
    assert!(category_html.contains("<html lang=\"zh-CN\">"));

    fs::remove_dir_all(&source_dir).unwrap();
    fs::remove_dir_all(&output_dir).unwrap();
}

#[test]
fn build_uses_root_path_for_site_header_link() {
    let source_dir = unique_temp_dir("site-header-link-source");
    let output_dir = unique_temp_dir("site-header-link-output");

    write_file(
        &source_dir.join("dand.toml"),
        "title = \"test\"\nhome = \"https://example.com/blog/\"\ndescription = \"test\"\n\n[footer]\nstart_year = 2020\nbeian = \"沪ICP备20260001号\"\n",
    );
    write_file(&source_dir.join("template/asset/css/main.css"), "body {}");
    write_file(
        &source_dir.join("_posts/example.md"),
        "---\n\
title: Example\n\
date: 2024-01-01 00:00\n\
categories:\n\
  - 随笔\n\
---\n\
\n\
Example\n",
    );

    let status = Command::new(env!("CARGO_BIN_EXE_dand-rs"))
        .arg("build")
        .arg(&source_dir)
        .arg("-o")
        .arg(&output_dir)
        .status()
        .unwrap();
    assert!(status.success());

    for path in [
        output_dir.join("index.html"),
        output_dir.join("example/index.html"),
        output_dir.join("categories/随笔/index.html"),
    ] {
        let html = fs::read_to_string(path).unwrap();
        assert!(html.contains("class=\"site-title font-sans\" href=\"/\">test</a>"));
    }

    fs::remove_dir_all(&source_dir).unwrap();
    fs::remove_dir_all(&output_dir).unwrap();
}

#[test]
fn build_adds_reading_settings_to_post_pages_only() {
    let source_dir = unique_temp_dir("reading-settings-source");
    let output_dir = unique_temp_dir("reading-settings-output");

    write_file(
        &source_dir.join("dand.toml"),
        "title = \"test\"\nhome = \"https://example.com/blog/\"\ndescription = \"test\"\n\n[footer]\nstart_year = 2020\nbeian = \"沪ICP备20260001号\"\n",
    );
    write_file(&source_dir.join("template/asset/css/main.css"), "body {}");
    write_file(
        &source_dir.join("_posts/example.md"),
        "---\n\
title: Example\n\
date: 2024-01-01 00:00\n\
---\n\
\n\
Example\n",
    );

    let status = Command::new(env!("CARGO_BIN_EXE_dand-rs"))
        .arg("build")
        .arg(&source_dir)
        .arg("-o")
        .arg(&output_dir)
        .status()
        .unwrap();
    assert!(status.success());

    let post_html = fs::read_to_string(output_dir.join("example/index.html")).unwrap();
    let reading_settings_js =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("template/asset/js/reading-settings.js"))
            .unwrap();
    let post_css =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("template/asset/css/post.css"))
            .unwrap();
    assert!(post_html.contains("<body class=\"page-shell page-shell-post\">"));
    assert!(post_html.contains("reading-settings-toggle"));
    assert!(post_html.contains("reading-settings-panel"));
    assert!(post_html.contains("阅读设置"));
    assert!(post_html.contains("行高"));
    assert!(post_html.contains("段距"));
    assert!(post_html.contains("data-reading-line-height"));
    assert!(post_html.contains("data-reading-paragraph-gap"));
    assert!(post_html.contains("data-reading-justify checked"));
    assert!(post_html.contains("data-reading-font-size"));
    assert!(post_html.contains("value=\"16\" data-reading-font-size"));
    assert!(post_html.contains("value=\"47\" data-reading-page-width"));
    assert!(post_html.contains("value=\"155\" data-reading-line-height"));
    assert!(post_html.contains("value=\"60\" data-reading-paragraph-gap"));
    assert!(reading_settings_js.contains("fontSize: 16"));
    assert!(reading_settings_js.contains("pageWidth: 47"));
    assert!(reading_settings_js.contains("lineHeight: 155"));
    assert!(reading_settings_js.contains("paragraphGap: 60"));
    assert!(reading_settings_js.contains("paragraphGap"));
    assert!(reading_settings_js.contains("[data-reading-paragraph-gap]"));
    assert!(reading_settings_js.contains("root.style.setProperty(\"--reader-paragraph-gap\""));
    assert!(post_css.contains("--reader-max-width: 47rem;"));
    assert!(post_css.contains("--reader-font-size: 1rem;"));
    assert!(post_css.contains("--reader-line-height: 1.55;"));
    assert!(post_css.contains("--reader-paragraph-gap: 0.6em;"));

    let index_html = fs::read_to_string(output_dir.join("index.html")).unwrap();
    assert!(!index_html.contains("reading-settings-toggle"));
    assert!(!index_html.contains("page-shell-post"));

    fs::remove_dir_all(&source_dir).unwrap();
    fs::remove_dir_all(&output_dir).unwrap();
}

#[test]
fn build_uses_single_main_heading_for_listing_pages() {
    let source_dir = unique_temp_dir("listing-heading-source");
    let output_dir = unique_temp_dir("listing-heading-output");

    write_file(
        &source_dir.join("dand.toml"),
        "title = \"@sunnysab\"\nhome = \"https://example.com/blog/\"\ndescription = \"test\"\n\n[footer]\nstart_year = 2020\nbeian = \"沪ICP备20260001号\"\n",
    );
    write_file(&source_dir.join("template/asset/css/main.css"), "body {}");
    write_file(
        &source_dir.join("_posts/example.md"),
        "---\n\
title: Example\n\
date: 2024-01-01 00:00\n\
categories:\n\
  - 随笔\n\
---\n\
\n\
Example\n",
    );

    let status = Command::new(env!("CARGO_BIN_EXE_dand-rs"))
        .arg("build")
        .arg(&source_dir)
        .arg("-o")
        .arg(&output_dir)
        .status()
        .unwrap();
    assert!(status.success());

    let index_html = fs::read_to_string(output_dir.join("index.html")).unwrap();
    assert!(index_html.contains("<main class=\"listing-page\""));
    assert!(!index_html.contains("<h1 class=\"page-title\">@sunnysab</h1>"));
    assert!(index_html.contains("<title>@sunnysab</title>"));
    assert!(!index_html.contains("<div class=\"nav-logo\"><h1"));

    let category_html = fs::read_to_string(output_dir.join("categories/随笔/index.html")).unwrap();
    assert!(category_html.contains("<h1 class=\"page-title\">随笔</h1>"));
    assert!(category_html.contains("<title>随笔 - @sunnysab</title>"));
    assert!(!category_html.contains("<div class=\"nav-logo\"><h1"));

    let categories_overview = fs::read_to_string(output_dir.join("categories/index.html")).unwrap();
    assert!(categories_overview.contains("<h1 class=\"page-title\">分类</h1>"));
    assert!(!categories_overview.contains("<div class=\"nav-logo\"><h1"));

    fs::remove_dir_all(&source_dir).unwrap();
    fs::remove_dir_all(&output_dir).unwrap();
}

#[test]
fn build_homepage_supports_year_group_toggle() {
    let source_dir = unique_temp_dir("index-grouping-source");
    let output_dir = unique_temp_dir("index-grouping-output");

    write_file(
        &source_dir.join("dand.toml"),
        "title = \"@sunnysab\"\nhome = \"https://example.com/blog/\"\ndescription = \"test\"\n\n[footer]\nstart_year = 2020\nbeian = \"沪ICP备20260001号\"\n",
    );
    write_file(&source_dir.join("template/asset/css/main.css"), "body {}");
    write_file(
        &source_dir.join("_posts/post-2024.md"),
        "---\n\
title: Newer\n\
date: 2024-01-01 00:00\n\
---\n\
\n\
Newer\n",
    );
    write_file(
        &source_dir.join("_posts/post-2023.md"),
        "---\n\
title: Older\n\
date: 2023-01-01 00:00\n\
---\n\
\n\
Older\n",
    );

    let status = Command::new(env!("CARGO_BIN_EXE_dand-rs"))
        .arg("build")
        .arg(&source_dir)
        .arg("-o")
        .arg(&output_dir)
        .status()
        .unwrap();
    assert!(status.success());

    let index_html = fs::read_to_string(output_dir.join("index.html")).unwrap();
    assert!(index_html.contains("data-index-grouping-root"));
    assert!(index_html.contains("data-group-by-year-default=\"true\""));
    assert!(index_html.contains("data-group-toggle"));
    assert!(index_html.contains("data-grouped-post-list"));
    assert!(index_html.contains("data-flat-post-list"));
    assert!(index_html.contains(">2024<"));
    assert!(index_html.contains(">2023<"));

    let category_html = fs::read_to_string(output_dir.join("categories/index.html")).unwrap();
    assert!(!category_html.contains("data-group-toggle"));
    assert!(!category_html.contains("data-grouped-post-list"));

    assert!(index_html.contains("data-index-grouping-root"));
    assert!(index_html.contains("initIndexGrouping();"));

    fs::remove_dir_all(&source_dir).unwrap();
    fs::remove_dir_all(&output_dir).unwrap();
}
