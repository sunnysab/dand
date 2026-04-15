# Mobile Responsive Styles Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为全站模板补齐保守阅读型的手机端响应式样式，在不改变整体视觉语言的前提下提升窄屏可读性并消除横向溢出。

**Architecture:** 继续使用 `template/asset/css/main.css` 管理全站框架与列表页，使用 `template/asset/css/post.css` 管理文章页与阅读设置。实现只通过新增和整理响应式断点完成，不做 UA 检测，也不默认修改 HTML 结构。

**Tech Stack:** CSS, Tera templates, Rust CLI build, cargo build, cargo test

---

> **Verification note:** 当前仓库没有浏览器端快照测试或视觉回归测试基础设施。本计划用 `cargo build` / `cargo test` 做构建回归保护，再用固定视口手动验证移动端表现。

### Task 1: 整理 `main.css` 的断点结构

**Files:**
- Modify: `template/asset/css/main.css`
- Test: `cargo build`
- Test: `cargo test`

- [ ] **Step 1: Inspect the existing mobile rules in `main.css`**

确认当前已存在的 `@media (max-width: 599px)` 规则覆盖了哪些区域，标记会与新断点冲突的选择器：

- `.header`
- `.nav`
- `.listing-list li`
- `.listing-page`
- `.page-head`
- `.site-title`
- `.copyright`

- [ ] **Step 2: Consolidate mobile breakpoint sections**

把零散的窄屏规则改为集中在文件尾部的三档断点：

```css
@media (max-width: 900px) { ... }
@media (max-width: 640px) { ... }
@media (max-width: 420px) { ... }
```

要求：

- 不改变桌面端默认规则
- 窄屏覆盖顺序清晰
- 保留现有配色、阴影、边框语义

- [ ] **Step 3: Run build to verify CSS changes do not break asset packaging**

Run: `cargo build`

Expected: PASS

- [ ] **Step 4: Run test suite to verify no CLI regressions**

Run: `cargo test`

Expected: PASS

- [ ] **Step 5: Commit the breakpoint refactor**

```bash
git add template/asset/css/main.css docs/superpowers/specs/2026-04-15-mobile-responsive-design.md docs/superpowers/plans/2026-04-15-mobile-responsive.md
git commit -m "style: reorganize responsive breakpoints"
```

### Task 2: 完成全站框架、导航和列表页手机端样式

**Files:**
- Modify: `template/asset/css/main.css`
- Test: `cargo build`
- Test: `cargo test`

- [ ] **Step 1: Adjust page shell spacing for tablet and phone widths**

在 `main.css` 中为以下区域补充断点样式：

- `body`
- `.page-shell-post`
- `.listing-page`
- `.page-head`
- `.page-title`
- `.page-overview`

目标：

- 手机上的左右留白稳定
- 页面标题和页头间距自然收缩
- 不引入新的整体布局模式

- [ ] **Step 2: Optimize header and nav tap targets on narrow screens**

调整：

- `.header`
- `.nav`
- `.nav-links`
- `.nav-links a`
- `.site-title`

目标：

- 小屏下 header 纵向堆叠但保持当前风格
- 导航允许换行
- 链接点击区更大
- 标题与导航不拥挤

- [ ] **Step 3: Reflow listing rows into a narrow-screen friendly layout**

调整：

- `.listing-list li`
- `.listing-list li > a`
- `#post-list .post-date`
- `#archive-list .archive-count`

目标：

- 手机端改为标题在上、日期/数量在下
- 避免标题与日期互相挤压
- 保持 hover / focus 语义与现有视觉基调一致

- [ ] **Step 4: Tighten footer layout for phone screens**

调整：

- `footer`
- `.copyright`

目标：

- 页脚保持居中和简洁
- 超窄屏下不出现拥挤或怪异换行

- [ ] **Step 5: Run build and test suite after global responsive changes**

Run:

- `cargo build`
- `cargo test`

Expected:

- 两个命令都 PASS
- 未引入模板或静态资源路径回归

- [ ] **Step 6: Commit the global mobile layout changes**

```bash
git add template/asset/css/main.css
git commit -m "style: improve mobile layout for global pages"
```

### Task 3: 完成文章页正文与内容元素的手机端样式

**Files:**
- Modify: `template/asset/css/post.css`
- Test: `cargo build`
- Test: `cargo test`

- [ ] **Step 1: Adjust post container, title, and metadata sizing**

为以下选择器添加 `900px / 640px / 420px` 断点覆盖：

- `.post-page`
- `.post-header`
- `.post-title`
- `.post-eyebrow`
- `.article-metadata`
- `.post-taxonomy`

目标：

- 手机端文章头部不显得过大
- 分类与标签允许自然换行
- 文章主体宽度在窄屏下更稳

- [ ] **Step 2: Tune reading typography without changing the visual style**

调整：

- `.post-content`
- `.post-content h2`
- `.post-content h3`
- `.post-content h4`
- `.post-content p`
- `.post-content ul`
- `.post-content ol`

目标：

- 字号、行高、段间距适合手机阅读
- 保持桌面端的“保守阅读型”基调
- 避免正文显得过密或过散

- [ ] **Step 3: Prevent content media from causing page-level overflow**

调整：

- `.post-content p > img`
- `.post-content pre`
- `.post-content table`
- `.post-content th`
- `.post-content td`
- `.post-content blockquote`
- `.post-content code`

目标：

- 代码块和表格只在自身区域横向滚动
- 图片不贴边、不溢出
- 引用块、内联代码和代码块在手机端内边距更紧凑

- [ ] **Step 4: Run build and tests after post-page responsive changes**

Run:

- `cargo build`
- `cargo test`

Expected:

- 两个命令都 PASS
- 文章页样式改动不影响站点生成

- [ ] **Step 5: Commit the post-page mobile typography changes**

```bash
git add template/asset/css/post.css
git commit -m "style: refine mobile reading layout for posts"
```

### Task 4: 调整阅读设置浮层的手机端可达性

**Files:**
- Modify: `template/asset/css/post.css`
- Test: `cargo build`
- Test: `cargo test`

- [ ] **Step 1: Reduce the floating control footprint on mobile**

调整：

- `.reading-settings`
- `.reading-settings-toggle`
- `.reading-settings-toggle-icon`

目标：

- 手机端按钮尺寸更合适
- 右下角偏移更小
- 不明显遮挡正文或系统手势区

- [ ] **Step 2: Constrain the settings panel for narrow viewports**

调整：

- `.reading-settings-panel`
- `.reading-settings-panel-head`
- `.reading-settings-control`
- `.reading-settings-check`

目标：

- 设置面板在手机端始终落在视口安全区内
- 控件间距和文字大小适合触屏操作
- 视觉上继续沿用当前半透明和轻阴影样式

- [ ] **Step 3: Run build and tests after floating-panel adjustments**

Run:

- `cargo build`
- `cargo test`

Expected:

- 两个命令都 PASS
- 前端脚本仍可正常操作现有阅读设置 DOM

- [ ] **Step 4: Commit the mobile reading-settings adjustments**

```bash
git add template/asset/css/post.css
git commit -m "style: optimize mobile reading settings panel"
```

### Task 5: 手动验收移动端视口表现

**Files:**
- Modify: none unless a final polish is needed
- Test: generated site output

- [ ] **Step 1: Build the project in release-like local flow**

Run: `cargo build`

Expected: PASS

- [ ] **Step 2: Generate a local preview site from a representative content source**

Run:

```bash
cargo run -- build <SOURCE_DIR> -o /tmp/dand-mobile-preview
```

Expected:

- PASS
- `/tmp/dand-mobile-preview/index.html` exists
- generated `asset/css/main.css` and `asset/css/post.css` are present

- [ ] **Step 3: Inspect the generated site in browser responsive mode**

Check these viewport sizes:

- `390x844`
- `430x932`
- `768x1024`

Required checks:

- no page-level horizontal scroll
- header and nav remain readable
- listing items are easy to scan
- post typography stays comfortable
- images, code blocks, and tables do not break layout
- reading-settings button and panel remain usable

- [ ] **Step 4: Apply minimal polish if a concrete viewport issue is found**

只允许修复在手动验收中已复现的问题，不做额外视觉扩张。

- [ ] **Step 5: Run final verification**

Run:

- `cargo build`
- `cargo test`

Expected:

- 两个命令都 PASS

- [ ] **Step 6: Commit the final responsive polish**

```bash
git add template/asset/css/main.css template/asset/css/post.css
git commit -m "style: finalize mobile responsive polish"
```
