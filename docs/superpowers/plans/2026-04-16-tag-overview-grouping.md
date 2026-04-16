# Tag Overview Grouping Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让 `/tags/` 总览页自动把公开文章集合完全相同的标签并排展示为一行，同时保持每个标签详情页与文章页标签展示逻辑不变。

**Architecture:** 在 `src/index.rs` 中把现有总览模型拆成“分类专用”和“标签专用”两套。标签总览通过“公开文章集合 -> 稳定签名 -> 分组”的 view model 生成多链接项；分类总览保留单链接项。模板层拆成分类和标签两个页面模板，标签模板只负责循环链接并输出 ` & `。

**Tech Stack:** Rust, Tera templates, existing unit tests in `src/index.rs`, CLI/render tests in `src/main.rs`, CSS in `template/asset/css/main.css`

---

### Task 1: 为标签分组规则补齐失败的单元测试

**Files:**
- Modify: `src/index.rs`
- Test: `cargo test groups_tags_with_identical_public_post_sets`
- Test: `cargo test omits_tag_groups_without_public_posts`

- [ ] **Step 1: 在 `src/index.rs` 增加测试辅助构造**

在 `#[cfg(test)]` 模块里添加最小辅助函数，用临时目录写入 markdown 文件、通过 `PostWalker` + `post::load(...).build(&mut env)` 构造 `Environment`。测试数据至少覆盖：

- 两个标签共享两篇公开文章
- 其中一个标签额外挂一篇 draft 或 hidden 文章
- 一个只关联 draft/hidden 的标签

- [ ] **Step 2: 写 `groups_tags_with_identical_public_post_sets` 失败测试**

断言标签总览构造函数输出：

- 同组只生成 1 个 `TagOverviewItem`
- `links` 中同时包含 `新东方` 与 `新概念英语`
- `post_count` 只统计公开文章数

- [ ] **Step 3: 写 `omits_tag_groups_without_public_posts` 失败测试**

断言仅关联 draft/hidden 文章的标签不会出现在标签总览项中。

- [ ] **Step 4: 运行目标测试确认失败**

Run:

- `cargo test groups_tags_with_identical_public_post_sets`
- `cargo test omits_tag_groups_without_public_posts`

Expected:

- FAIL，原因应为标签分组数据结构或构造逻辑尚未实现

### Task 2: 实现标签总览专用 view model 和分组逻辑

**Files:**
- Modify: `src/index.rs`
- Test: `cargo test groups_tags_with_identical_public_post_sets`
- Test: `cargo test omits_tag_groups_without_public_posts`

- [ ] **Step 1: 将分类总览结构收窄为分类专用**

在 `src/index.rs` 中重命名现有通用结构：

- `ArchiveOverviewItem` -> `CategoryOverviewItem`
- `build_archive_items(...)` -> `build_category_overview_items(...)`
- `render_archive_overview(...)` -> `render_category_overview(...)`

保持分类页当前行为不变。

- [ ] **Step 2: 新增标签总览数据结构**

在 `src/index.rs` 中新增：

- `TagOverviewLink { name: String, href: String }`
- `TagOverviewItem { links: Vec<TagOverviewLink>, post_count: usize }`

并让 `TagOverviewRenderer` 改为使用标签专用 items。

- [ ] **Step 3: 实现公开文章集合签名与分组**

在 `src/index.rs` 中新增标签总览构造函数，使用：

- 现有 `sorted_public_posts(...)` 或等价过滤逻辑取得公开文章
- `post.0.borrow().reference.relative_name.clone()` 作为稳定文章标识
- 排序后的文章标识拼接结果作为 group key

要求：

- 同 key 的标签归为同一组
- 组内链接按标签名升序
- 组间按 `post_count desc` 再按首个标签名升序
- `post_count == 0` 的组直接过滤

- [ ] **Step 4: 运行单元测试确认通过**

Run:

- `cargo test groups_tags_with_identical_public_post_sets`
- `cargo test omits_tag_groups_without_public_posts`

Expected:

- PASS

- [ ] **Step 5: 提交分组逻辑**

```bash
git add src/index.rs
git commit -m "feat: group identical tag overviews"
```

### Task 3: 为最终 HTML 输出补齐失败的渲染回归测试

**Files:**
- Modify: `src/main.rs`
- Test: `cargo test render_groups_identical_tags_in_tags_overview`
- Test: `cargo test render_categories_overview_remains_single_linked_items`

- [ ] **Step 1: 写标签总览渲染失败测试**

在 `src/main.rs` 现有 `#[cfg(test)]` 模块中新增端到端测试：

- 构造一个最小 source 目录
- 写入站点配置、最小静态资源、带多个标签的文章
- 调用 `blog.render()` 或等价构建路径
- 读取输出的 `tags/index.html`

断言输出至少包含：

- `href="/tags/xin-dong-fang/"`
- `href="/tags/xin-gai-nian-ying-yu/"`
- `</a> & <a`
- 正确的 `2 篇` 或对应测试数据篇数
- 不包含只关联 draft/hidden 的标签名

- [ ] **Step 2: 写分类总览未受影响的失败测试**

在同一测试模块新增一个分类页回归测试，断言输出的 `categories/index.html` 仍然是单链接项，并包含正确数量文本。

- [ ] **Step 3: 运行渲染测试确认失败**

Run:

- `cargo test render_groups_identical_tags_in_tags_overview`
- `cargo test render_categories_overview_remains_single_linked_items`

Expected:

- FAIL，原因应为模板仍按单链接渲染或分类模板路径尚未拆分

### Task 4: 拆分模板并补齐列表样式

**Files:**
- Create: `template/page/tag_archive_list.html`
- Create: `template/page/category_archive_list.html`
- Modify: `template/asset/css/main.css`
- Modify: `src/index.rs`
- Test: `cargo test render_groups_identical_tags_in_tags_overview`
- Test: `cargo test render_categories_overview_remains_single_linked_items`

- [ ] **Step 1: 创建分类总览模板**

把当前 `template/page/archive_list.html` 的内容迁移到 `template/page/category_archive_list.html`，保持现有单链接结构和文案不变。

- [ ] **Step 2: 创建标签总览模板**

新增 `template/page/tag_archive_list.html`，每个列表项使用例如下面的结构：

```tera
<span class="archive-links">
  {% for link in item.links %}
  <a href="{{ link.href }}">{{ link.name }}</a>{% if not loop.last %} & {% endif %}
  {% endfor %}
</span>
<span class="archive-count">{{ item.post_count }} 篇</span>
```

- [ ] **Step 3: 更新 `src/index.rs` 的模板渲染入口**

让：

- `TagOverviewRenderer` 渲染 `page/tag_archive_list.html`
- `CategoryOverviewRenderer` 渲染 `page/category_archive_list.html`

不再使用旧的 `page/archive_list.html`。

- [ ] **Step 4: 调整列表页样式以兼容多链接分组**

在 `template/asset/css/main.css` 中补齐标签总览链接组样式，至少处理：

- `.archive-links`
- `.archive-links a`
- 窄屏下 `archive-count` 与多链接分组的换行关系

要求：

- 标签总览的多个链接在视觉上继承原单链接权重
- 分类页和文章列表页原样式不回退
- 手机端多链接时不挤压计数

- [ ] **Step 5: 运行渲染测试确认通过**

Run:

- `cargo test render_groups_identical_tags_in_tags_overview`
- `cargo test render_categories_overview_remains_single_linked_items`

Expected:

- PASS

- [ ] **Step 6: 提交模板与样式改动**

```bash
git add src/index.rs src/main.rs template/page/tag_archive_list.html template/page/category_archive_list.html template/asset/css/main.css
git commit -m "feat: render grouped tags in overview"
```

### Task 5: 全量验证与收尾

**Files:**
- No code changes expected
- Test: `cargo build`
- Test: `cargo test`

- [ ] **Step 1: 运行完整构建验证**

Run: `cargo build`

Expected: PASS

- [ ] **Step 2: 运行完整测试集**

Run: `cargo test`

Expected: PASS

- [ ] **Step 3: 清理无用旧模板**

如果 `template/page/archive_list.html` 已无引用：

- 删除该文件，或
- 明确保留并记录原因（例如临时兼容）

完成后重新运行：

- `cargo build`
- `cargo test`

Expected:

- PASS
