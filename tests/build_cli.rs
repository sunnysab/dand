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
