# 持久化增量构建设计

**目标**

为 `dand` 增加基于文件修改时间（mtime）的持久化增量构建，同时覆盖：

- `build`
- `serve --watch`

并支持：

- markdown 新增 / 修改 / 删除 / 重命名
- tag / category 变更
- 模板变更
- `dand.toml` 变更
- 根静态文件变更
- 文章伴生资源目录变更

## 现状

- 当前构建入口在 `src/main.rs`
- 每次 `build()` 都会先清空输出目录，再重新扫描全部 markdown 并全量渲染
- `serve --watch` 只负责轮询检测变化，检测后仍调用同一套全量构建逻辑
- 当前没有持久化缓存、模板依赖图，也没有页面级失效判断
- `Environment` 使用 `Rc<RefCell>` 维护文章、标签、分类之间的关联关系，当前行为稳定

## 决策

### 总体策略

采用“持久化状态缓存 + mtime 输入快照 + 页面级失效矩阵 + 输出层增量执行”的方案。

具体来说：

- 输入检测是增量的
- 页面失效判断是增量的
- 输出写入 / 删除 / 复制是增量的
- `Environment` 构建允许继续全量，避免触碰现有 `Rc<RefCell>` 关系模型

原因：

- 当前痛点是“每次都删输出并全量重写”，不是内存关系层本身
- 先把正确、稳定、可持久化的增量输出做起来，风险最低
- 对当前代码结构来说，这比同时改造内存模型或引入模板依赖图更可控

### 缓存位置

增量状态写入输出目录：

- `OUTPUT/.dand-cache/state.json`

原因：

- 缓存天然与该输出目录绑定
- 多个输出目录互不影响
- 清理输出目录即可连带清理缓存

### 失效判断依据

本次以文件修改时间 `mtime` 作为唯一输入变化判据。

原因：

- 需求明确指定基于 `mtime`
- 对当前项目已经足够
- 避免在第一版引入内容哈希、模板依赖解析等额外复杂度

### 模板变化策略

本次不实现模板依赖图，模板改动采用保守失效矩阵。

原因：

- 当前模板数量有限
- `tera` 的 `extends` / `include` 依赖解析不是当前实现重点
- 先保证正确性，后续再考虑更细粒度优化

## 状态文件设计

`state.json` 需要记录 4 类信息。

### 1. 输入文件快照

记录上次构建看到的输入文件及其 `mtime`，覆盖：

- `dand.toml`
- `template/**/*.html`
- `template/asset/**`
- 源目录内所有 `.md`
- 根目录静态文件
- 文章伴生资源目录中的文件

用途：

- 与当前扫描结果直接做 diff，得到 `added / modified / removed`

### 2. 文章摘要索引

对每篇 markdown 保存轻量摘要，至少包括：

- `source_path`
- `relative_name`
- `output_dir`
- `publish_date`
- `hidden`
- `draft`
- `latex`
- `tags`
- `categories`
- `resource_dir`
- `resource_output_dir`

用途：

- 在文章被删除或重命名时，仍能知道旧输出路径
- 在 tag / category 变化时，能同时标脏旧页面和新页面

### 3. 已生成输出索引

记录上次成功生成过的输出目标，包括：

- 首页：`index.html`
- 文章页：`<relative_name>/index.html`
- 标签页：`tags/<slug>/index.html`
- 分类页：`categories/<slug>/index.html`
- 标签总览：`tags/index.html`
- 分类总览：`categories/index.html`
- 复制出的根静态文件
- 复制出的模板资源与文章伴生资源

用途：

- 删除失效输出
- 避免文章删除、重命名后残留旧目录或旧资源

### 4. 缓存版本信息

记录：

- `cache_format_version`
- `generator_version`

用途：

- 缓存结构升级或生成器版本不兼容时，自动退回一次全量重建

## 失效矩阵

### markdown 新增 / 修改 / 删除

影响：

- 对应文章页
- 首页 `index.html`
- 该文章所属 tag 页
- 该文章所属 category 页
- `tags/index.html`
- `categories/index.html`

如果 `relative_name` 改变，还需：

- 删除旧文章输出目录
- 删除旧资源输出目录

### 文章伴生资源目录变化

只影响：

- 该文章输出目录下复制出的资源

不要求同时重渲染文章 HTML，除非 markdown 也发生变化。

### 根静态文件变化

只影响：

- 输出目录中的对应根文件

### `template/asset/**` 变化

只影响：

- `output/asset/**`

### 模板变化

保守处理如下：

- 任一 `template/layout/*.html` 改动 => 所有 HTML 页脏
- `template/page/post.html` 改动 => 所有文章页脏
- `template/page/index.html` 改动 => 首页 + tag 页 + category 页脏
- `template/page/tag_archive_list.html` 改动 => `tags/index.html` 脏
- `template/page/category_archive_list.html` 改动 => `categories/index.html` 脏

### `dand.toml` 变化

保守处理：

- 所有 HTML 页脏

因为站点标题、描述、footer、home 等配置可能被多个页面消费。

## 构建流程

### 阶段 A：扫描当前输入

扫描并分类收集当前工作树中的输入：

- `dand.toml`
- `template/**/*.html`
- `template/asset/**`
- 源目录下所有 `.md`
- 根静态文件
- 文章伴生资源目录

产出 `CurrentSnapshot`，只描述：

- 文件路径
- `mtime`
- 文件类别
- markdown 与伴生资源之间的归属关系

### 阶段 B：加载旧缓存并做 diff

读取：

- `OUTPUT/.dand-cache/state.json`

若缓存不存在或版本不兼容：

- 视为首次全量构建

否则对比旧状态和当前快照，产出：

- `added_files`
- `modified_files`
- `removed_files`

并归类成更高层的逻辑变化：

- `changed_posts`
- `removed_posts`
- `changed_post_resources`
- `changed_root_static_files`
- `changed_template_assets`
- `changed_templates`
- `config_changed`

### 阶段 C：建立脏任务集合

将文件变化映射为输出任务集合，而不是立即重建所有页面。

任务集合至少包括：

- `render_posts`
- `delete_post_outputs`
- `render_tag_pages`
- `delete_tag_pages`
- `render_category_pages`
- `delete_category_pages`
- `render_index`
- `render_tags_overview`
- `render_categories_overview`
- `copy_asset_paths`
- `delete_asset_paths`
- `copy_root_static_files`
- `delete_root_static_files`

规则：

- 文章正文或 front matter 变化 => 对应文章页脏
- 标题 / 日期 / hidden / draft / tag / category 变化 => 首页和相关 listing 页脏
- 文章删除 => 删除旧文章输出和旧资源输出，并重建受影响 listing
- 模板 / 配置变化 => 按失效矩阵批量标脏

### 阶段 D：按任务最小执行

本次仍允许全量重建 `Environment`，但只最小化执行输出层更新。

执行顺序建议：

1. 必要时扫描 markdown 并重建 `Environment`
2. 删除失效输出
3. 渲染脏的文章页、tag 页、category 页、总览页
4. 复制变化的模板资源、文章资源、根静态文件
5. 汇总生成产物信息

这样可以避免为了追求内存级增量而重构现有关系模型。

### 阶段 E：原子写回缓存

只有所有任务都成功后，才写回新状态：

1. 先写 `state.json.tmp`
2. 再原子 `rename` 为 `state.json`

原因：

- 避免构建失败时把缓存写成“假成功”

## 模块拆分

建议将构建相关逻辑从 `src/main.rs` 拆出：

### `src/build.rs`

负责构建入口与整体流程编排：

- `build_site(...)`
- `incremental_build(...)`
- `full_build(...)`

### `src/build/state.rs`

负责缓存结构定义与读写：

- `BuildState`
- `CachedPost`
- `GeneratedOutputs`
- `read_state(...)`
- `write_state_atomic(...)`

### `src/build/snapshot.rs`

负责扫描当前输入树：

- `CurrentSnapshot`
- `SnapshotFile`
- `FileKind`
- `scan_source_tree(...)`

### `src/build/diff.rs`

负责 diff 与脏任务规划：

- `BuildDiff`
- `DirtySet`
- `diff_snapshots(...)`
- `plan_dirty_outputs(...)`

### `src/build/render.rs`

负责按任务最小执行：

- `execute_dirty_tasks(...)`
- 删除输出
- 渲染页面
- 复制变化资源

### `src/build/report.rs`

负责构建结果摘要：

- 是否为首次全量 / 增量构建
- 渲染页数
- 删除页数
- 拷贝资源数

## `main.rs` 调整范围

`src/main.rs` 最终只保留：

- CLI 解析
- 调构建入口
- 预览服务器与 watch 外围控制

以下逻辑迁出：

- 输出目录清理
- 缓存读写
- 文件 diff
- 页面级失效判断
- 增量删除 / 复制 / 重渲染

## 范围控制

### 本次处理

- 为 `build` 增加持久化增量缓存
- 让 `serve --watch` 复用同一套增量构建逻辑
- 正确处理文章删除、重命名、taxonomy 变更
- 正确处理模板 / 配置 / 静态资源变更
- 保证无改动二次构建不重写已有输出

### 本次不处理

- 内容哈希判定
- 模板依赖图解析
- 只解析改动 markdown 的内存级增量
- 重构 `Environment` / `Post` / `Tag` / `Category` 关系模型

## 风险与兼容性

- 仅依赖 `mtime`，如果外部工具异常保留旧时间戳，可能漏判变化；这是本次明确接受的约束
- 模板变化按保守矩阵处理，会带来一定范围的额外重建；这是为了保证正确性
- 输出层由“每次清空目录”改为“定向删除 + 定向写入”，需要用测试防止残留旧输出
- 缓存文件位于输出目录中，若用户手工删掉缓存，将退回一次全量构建

## 验证策略

### 单元测试

验证 diff、失效矩阵和删除计划：

- 无变化 -> 无脏任务
- markdown 新增 / 删除 / 修改 -> 正确标脏
- tag / category 变化 -> 旧页与新页都脏
- 模板 / 配置变化 -> 按矩阵标脏

### CLI 集成测试

新增覆盖：

1. 首次构建生成缓存
2. 第二次无改动不重写输出
3. 单篇文章正文修改只更新受影响页面
4. taxonomy 变化更新旧 listing 和新 listing
5. 删除文章后无残留旧输出
6. 伴生资源变化只同步资源
7. 模板变化按矩阵重建
8. `dand.toml` 变化触发全站 HTML 重建

### watch 回归测试

重点验证：

- `serve --watch` 检测到变化后调用的是同一套增量构建入口

## 验收标准

完成后应满足：

- 缓存持久化到输出目录
- 无改动二次构建不重写已有 HTML / 资源
- 单篇文章变更只更新受影响页面
- 删除文章后不残留旧输出目录或旧资源
- tag / category 页正确更新
- 模板 / `dand.toml` 变化按失效矩阵重建
- `build` 与 `serve --watch` 共用同一套增量逻辑
- 现有测试与新增测试全部通过
