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
