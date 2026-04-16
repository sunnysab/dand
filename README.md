# dand-rs

dand-rs 是一个用 Rust 编写的静态站点生成器，输入带 Front Matter 的 Markdown，输出可部署的 HTML 站点。它内置 Tera
模板、标签/分类页面与本地预览能力，适合快速搭建博客或文档站。

## 特性

* 支持 YAML/TOML Front Matter
* Markdown → HTML（`pulldown-cmark`）
* Tera 模板渲染与静态资源复制
* 标签/分类详情页与总览页
* `build`/`serve` CLI，支持 `--watch` 自动重建

## 快速开始

### 构建站点

```bash
cargo build
cargo run -- build <source_dir> -o <output_dir>
```

### 本地预览

```bash
cargo run -- serve <source_dir> -o <output_dir> -p 8000 --watch --interval 1000
```

### 模板目录查找顺序

1. `<source_dir>/template`
2. `./template`（当前工作目录）
3. `<可执行文件目录>/template`

## 配置

`source_dir` 下需要放置 `dand.toml`（可参考 `dand.example.toml`）：

```toml
title = "@sunnysab"
home = "https://example.com/"
description = "my blog"

[footer]
start_year = 2020
beian = "沪ICP备20002980号"
```

## 文章 Front Matter

支持 YAML (`---`) 或 TOML (`+++`) 格式。常用字段：

* `title`（必需）
* `date` / `updated`（可选，未提供时使用文件创建/修改时间）
* `tags` / `categories`（可选）
* `is_hidden` / `is_draft`（可选）
* `latex`（可选，启用 KaTeX 渲染）

示例（YAML）：

```yaml
---
title: "Hello Dand"
date: 2024-01-01
updated: 2024-01-05
tags: [ "Rust", "SSG" ]
categories: [ "Notes" ]
is_hidden: false
is_draft: false
latex: true
---
```

## 测试

```bash
cargo test
```
