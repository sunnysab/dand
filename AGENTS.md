# AGENTS.md - AI Agent 开发指南

## 1. 项目概述

**项目名**: dand-rs (Dand)  
**类型**: 静态站点生成器 (SSG)  
**语言**: Rust (2021 Edition)  
**目标**: 用最小、单二进制的方式将带 Front Matter 的 Markdown 渲染为 HTML（Tera 模板）。

## 2. 架构与心智模型

数据流是线性管线：

1. **Discover (`src/discover.rs`)**: 递归扫描源目录中的 `.md` 文件。
2. **Load & Parse (`src/post/parser.rs`)**: 读取文件内容，解析 Front Matter（YAML/TOML），拆分正文。
3. **Environment Build (`src/env.rs`)**:
    - 构建 `SharedPost`（`Rc<RefCell>` 包装，便于可变共享）。
    - 构建标签/分类集合并建立多对多关联。
4. **Render (`src/build/*`, `src/index.rs`, `src/post/render.rs`)**:
    - `pulldown-cmark` 将 Markdown 转为 HTML。
    - 使用 `tera` 模板注入数据（`page/post.html`, `page/index.html`）。
5. **Output**: 写出 HTML，并复制模板静态资源到输出目录。

## 3. 目录结构速览

* `src/main.rs`: CLI 入口（build/serve/--watch）与本地预览服务器。
* `src/discover.rs`: 文件系统扫描与 `PostReference` 构建。
* `src/post.rs`: 核心数据结构（`Post`, `SharedPost`, `Metadata`）。
* `src/post/flag.rs`: 文章状态位（Draft/Hidden/Latex）。
* `src/datetime.rs`: 宽松日期解析的自定义 Serde。
* `src/env.rs`: 全局环境与 Tera 管理。
* `template/`: 默认 HTML 模板与静态资源。
* `tests/`: CLI 与构建逻辑测试。

## 4. 编码规范与约束

### Rust 习惯

* **错误处理**: 使用 `anyhow::Result`，并配合 `context()` 提供错误链。
* **路径处理**: 始终使用 `Path`/`PathBuf`，除非仅用于展示。
* **序列化**: 使用 `serde` 的 `derive`。
* **字符串入参**: 尽量使用 `&str` 引用。

### 关键架构警告（勿破坏）

* **内存模型 (`Rc<RefCell>`)**: Posts 与 Tags/Categories 是多对多关系，依赖 `Rc<RefCell>` 做内部可变共享。
    * **规则**: 除非解决明确的并发问题，不要改为生命周期或 `Arc<Mutex>`。
    * **序列化**: 序列化 `SharedTag`/`SharedPost` 时避免无限递归（见 `src/post/tag.rs`）。

### 配置

* **Front Matter**: 同时支持 YAML (`---`) 与 TOML (`+++`)。
* **站点配置**: `dand.toml` 是站点级配置源。

## 5. 当前任务与关注点

* CLI 已支持 `build`/`serve`/`--watch`。模板目录解析顺序为：
  `source_dir/template` → `cwd/template` → `exe_dir/template`。
* 不要引入硬编码路径；保持模板/资源路径解析的可移植性。
* LaTeX 支持依赖 Front Matter 的 `latex` 字段，并由前端 KaTeX 渲染。

## 6. 运行与验证

* **Build**: `cargo build`
* **Run**: `cargo run -- build <SOURCE_DIR> -o <OUTPUT_DIR>`
* **Serve**: `cargo run -- serve <SOURCE_DIR> -o <OUTPUT_DIR> [--watch]`
* **Test**: `cargo test`

## 7. 交互风格

* **简洁**: 直接展示改动。
* **说明原因**: 若改动 `src/post.rs` 或 `src/env.rs`，说明内存安全影响。
* **现代化**: 发现依赖过旧时可建议更新，但需考虑破坏性改动。
