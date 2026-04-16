# Reading Defaults Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将文章页默认阅读样式更新为用户指定的字号、页宽、行高、段距和对齐设置，并保持首屏样式、设置面板初始值、JS 默认状态一致。

**Architecture:** 只改 3 个现有前端文件：`post.css` 负责首屏默认变量，`post.html` 负责面板滑杆初始值，`reading-settings.js` 负责无本地存储时的默认状态。通过一条集成测试同时约束这三处默认值，避免后续漂移。

**Tech Stack:** Tera templates, CSS custom properties, vanilla JavaScript, Rust integration tests via `cargo test`

---

### Task 1: 锁定默认值回归测试

**Files:**
- Modify: `tests/build_cli.rs`
- Test: `cargo test build_adds_reading_settings_to_post_pages_only --test build_cli`

- [ ] **Step 1: 写失败测试**

在现有 `build_adds_reading_settings_to_post_pages_only` 里补充断言，检查：
- 面板滑杆 `value` 分别为 `16 / 47 / 155 / 60`
- `reading-settings.js` 默认状态为 `fontSize: 16, pageWidth: 47, lineHeight: 155, paragraphGap: 60, useSans: false, justify: true`
- `post.css` 默认变量为 `1rem / 47rem / 1.55 / 0.6em`

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test build_adds_reading_settings_to_post_pages_only --test build_cli`
Expected: FAIL，提示默认值仍是旧值

### Task 2: 更新默认阅读参数

**Files:**
- Modify: `template/asset/css/post.css`
- Modify: `template/asset/js/reading-settings.js`
- Modify: `template/page/post.html`
- Test: `cargo test build_adds_reading_settings_to_post_pages_only --test build_cli`

- [ ] **Step 1: 修改 CSS 默认变量**

将：
- `--reader-font-size` 改为 `1rem`
- `--reader-max-width` 改为 `47rem`
- `--reader-line-height` 改为 `1.55`
- `--reader-paragraph-gap` 改为 `0.6em`

- [ ] **Step 2: 修改模板滑杆初始值**

将 4 个滑杆 `value` 改为：
- 字号 `16`
- 页宽 `47`
- 行高 `155`
- 段距 `60`

- [ ] **Step 3: 修改 JS 默认状态**

将 `defaults` 改为：
- `fontSize: 16`
- `pageWidth: 47`
- `lineHeight: 155`
- `paragraphGap: 60`
- `useSans: false`
- `justify: true`

- [ ] **Step 4: 运行目标测试确认通过**

Run: `cargo test build_adds_reading_settings_to_post_pages_only --test build_cli`
Expected: PASS

### Task 3: 全量验证

**Files:**
- No code changes expected
- Test: `cargo test`

- [ ] **Step 1: 运行完整测试**

Run: `cargo test`
Expected: PASS，全部测试通过
