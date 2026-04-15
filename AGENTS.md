# AGENTS.md - AI Agent Development Guide

## 1. Project Overview
**Project Name**: dand-rs (Dand)
**Type**: Static Site Generator (SSG)
**Language**: Rust (2021 Edition)
**Goal**: A fast, minimal, single-binary SSG that converts Markdown (with Front Matter) to HTML using Tera templates.

## 2. Architecture & Mental Model
The data flow is a linear pipeline:
1.  **Discover (`src/discover.rs`)**: Recursively scan the source directory for `.md` files.
2.  **Load & Parse (`src/post/parser.rs`)**: Read file content, parse Front Matter (YAML/TOML), and separate the body.
3.  **Environment Build (`src/env.rs`)**:
    - Build `SharedPost` objects (wrapped in `Rc<RefCell>` for interior mutability).
    - Build `TagSet` and link Posts <-> Tags (Many-to-Many relationship).
4.  **Render (`src/render.rs`, `src/index.rs`, `src/post/render.rs`)**:
    - Convert Markdown to HTML using `pulldown-cmark`.
    - Inject data into `tera` templates (`page/post.html`, `page/index.html`).
5.  **Output**: Write generated HTML and copy static assets to the output directory.

## 3. Directory Structure Map
*   `src/main.rs`: Entry point. **CURRENTLY BROKEN** (contains hardcoded paths). Needs CLI integration.
*   `src/discover.rs`: File system walking logic. Handles `PostReference`.
*   `src/post.rs`: Core data structures (`Post`, `SharedPost`, `Metadata`).
*   `src/post/flag.rs`: Bitmask logic for Post status (Draft, Hidden, Latex).
*   `src/datetime.rs`: Custom Serde logic for loose date parsing.
*   `src/env.rs`: Global state holder (`Environment`) and Tera instance management.
*   `template/`: Default HTML templates and assets (CSS/JS).

## 4. Coding Standards & Constraints

### Rust Idioms
*   **Error Handling**: Use `anyhow::Result` for application-level errors. Use `context()` to provide meaningful error chains.
*   **Path Handling**: Always use `Path` and `PathBuf`. Never use `String` for file paths unless strictly necessary for display.
*   **Serialization**: Use `serde` with `derive`.
*   **String Manipulation**: Prefer references (`&str`) in function arguments where possible.

### Critical Architecture Warnings (DO NOT BREAK)
*   **Memory Model (`Rc<RefCell>`)**: The project uses `Rc<RefCell<Post>>` and `Rc<RefCell<Tag>>` to handle the cyclic/many-to-many relationship between Posts and Tags.
    *   **Rule**: Do NOT attempt to refactor this into lifetimes or `Arc<Mutex>` unless you are solving a specific concurrency bug. The current single-threaded model relies on this for simplicity.
    *   **Serialization**: Be careful when serializing `SharedTag` inside `SharedPost` (and vice versa) to avoid infinite recursion. See `src/post/tag.rs`.

### Configuration
*   **Front Matter**: Must support both YAML (`---`) and TOML (`+++`).
*   **Config File**: `dand.toml` is the source of truth for site-wide settings.

## 5. Current Task & Roadmap
The project is being revived. The immediate priorities for any Agent working on this are:

1.  **Refactor `main.rs` (High Priority)**:
    *   Remove ALL hardcoded paths (e.g., `/home/sunnysab/...`).
    *   Implement a CLI using `clap`.
    *   Required args: `build [SOURCE_DIR] -o [OUTPUT_DIR]`.
2.  **Fix Asset Copying**:
    *   The `copy_assets` function in `main.rs` also has hardcoded paths. It needs to copy from the theme directory relative to the binary or config.
3.  **LaTeX Verification**:
    *   Ensure `pulldown-cmark` is configured to pass through LaTeX syntax or handle it correctly for the frontend `KaTeX` script.

## 6. How to Run & Verify
*   **Build**: `cargo build`
*   **Run**: `cargo run -- [args]` (After CLI implementation)
*   **Test**: `cargo test` (Ensure unit tests in `src/datetime.rs` pass).

## 7. Interaction Style
*   **Be Concise**: Show the code changes directly.
*   **Explain Why**: If changing `src/post.rs` or `src/env.rs`, explain the memory safety implications.
*   **Modernize**: If you see deprecated dependencies in `Cargo.toml`, suggest updates but check for breaking changes.

