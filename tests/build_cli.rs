use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

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
fn build_uses_posts_config_by_default() {
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

    let status = Command::new(env!("CARGO_BIN_EXE_dand-rs"))
        .arg("build")
        .arg(&source_dir)
        .arg("-o")
        .arg(&output_dir)
        .status()
        .unwrap();

    assert!(status.success());
    assert!(output_dir.join("_posts/hello/index.html").is_file());

    fs::remove_dir_all(&source_dir).unwrap();
    fs::remove_dir_all(&output_dir).unwrap();
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
    assert!(output_dir.join("_posts/first/index.html").is_file());

    fs::remove_file(source_dir.join("_posts/first.md")).unwrap();

    let second_status = Command::new(env!("CARGO_BIN_EXE_dand-rs"))
        .arg("build")
        .arg(&source_dir)
        .arg("-o")
        .arg(&output_dir)
        .status()
        .unwrap();
    assert!(second_status.success());
    assert!(!output_dir.join("_posts/first/index.html").exists());

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

    let post_html = fs::read_to_string(output_dir.join("_posts/linked-post/index.html")).unwrap();
    assert!(post_html.contains("href=\"/tags/rust/\""));
    assert!(post_html.contains("href=\"/tags/web/\""));
    assert!(post_html.contains("href=\"/categories/数据库/\""));

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

    let categories_overview =
        fs::read_to_string(output_dir.join("categories/index.html")).unwrap();
    assert!(categories_overview.contains("href=\"/categories/数据库/\""));
    assert!(categories_overview.contains("href=\"/categories/网络/\""));
    assert!(categories_overview.contains("数据库"));
    assert!(categories_overview.contains("1 篇"));

    fs::remove_dir_all(&source_dir).unwrap();
    fs::remove_dir_all(&output_dir).unwrap();
}
