# Footer Config Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让页脚开始年份和备案号可通过 `dand.toml` 配置，并在渲染时自动使用当前年份生成版权年份文本。

**Architecture:** 在 `SiteConf` 中增加 `footer` 配置分组，并在渲染阶段构造统一的页脚上下文注入所有模板。模板只消费已经格式化好的年份标签和备案号，避免在 Tera 中堆积逻辑。

**Tech Stack:** Rust, serde, toml, tera, cargo test

---

### Task 1: 写页脚配置回归测试

**Files:**
- Modify: `tests/build_cli.rs`
- Test: `tests/build_cli.rs`

- [ ] **Step 1: Write the failing test**

新增一个集成测试，构造包含：

```toml
title = "test"
home = "https://example.com/"
description = "test"

[footer]
start_year = 2020
beian = "沪ICP备20260001号"
```

并断言生成的首页 HTML 包含：

- `sunnysab.cn &copy; 2020 - <当前年份>`
- `沪ICP备20260001号`

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test build_renders_footer_from_config --test build_cli`

Expected: FAIL，因为当前模板和配置结构还不支持该行为。

- [ ] **Step 3: Write minimal implementation**

在配置和渲染层加入最小支持，让模板能读取页脚配置。

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test build_renders_footer_from_config --test build_cli`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add tests/build_cli.rs src/site.rs src/index.rs src/post/render.rs template/layout/copyright.html docs/superpowers/specs/2026-04-15-footer-config-design.md docs/superpowers/plans/2026-04-15-footer-config.md
git commit -m "feat: make footer metadata configurable"
```

### Task 2: 接入配置与模板上下文

**Files:**
- Modify: `src/site.rs`
- Modify: `src/index.rs`
- Modify: `src/post/render.rs`
- Modify: `template/layout/copyright.html`
- Test: `tests/build_cli.rs`

- [ ] **Step 1: Define serializable footer config and render data**

在 `src/site.rs` 中新增：

- `FooterConf { start_year: u16, beian: String }`
- `FooterView { year_label: String, beian: String }`

并提供把当前年份格式化为文案的方法。

- [ ] **Step 2: Run focused test to verify it still fails before template wiring**

Run: `cargo test build_renders_footer_from_config --test build_cli`

Expected: 仍然 FAIL，说明还没有真正打通渲染。

- [ ] **Step 3: Inject footer view into all page renderers**

在 `src/index.rs` 和 `src/post/render.rs` 每个 `tera::Context` 中插入：

```rust
parameters.insert("footer", &site.footer_view());
```

- [ ] **Step 4: Update template to consume footer values**

把模板改成读取：

```tera
<span>sunnysab.cn &copy; {{ footer.year_label }}</span>
<a ...>{{ footer.beian }}</a>
```

- [ ] **Step 5: Run focused test to verify it passes**

Run: `cargo test build_renders_footer_from_config --test build_cli`

Expected: PASS

### Task 3: 全量回归验证

**Files:**
- Modify: `dand.example.toml`
- Test: `tests/build_cli.rs`

- [ ] **Step 1: Update example config**

为 `dand.example.toml` 增加 `[footer]` 示例，避免配置样例过时。

- [ ] **Step 2: Run project test suite**

Run: `cargo test`

Expected: 全部通过

- [ ] **Step 3: Record actual verification status**

若失败，记录失败用例和原因；若通过，带测试命令和结果汇报。
